#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(associated_type_defaults)]
#![feature(trait_alias)]
#![allow(clippy::type_complexity)]
mod heartbeat;
mod key;
mod keyboard;
mod matrix;
mod processor;
mod rotary;
mod util;

extern crate alloc;
extern crate rp2040_hal as hal;
use {defmt_rtt as _, panic_probe as _};

#[rtic::app(
    device = hal::pac,
    dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3]
)]
mod kb {
    use alloc::{boxed::Box, vec::Vec};
    #[global_allocator]
    static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();

    use defmt::{debug, info};

    // The linker will place this boot block at the start of our program image.
    // We need this to help the ROM bootloader get our code up and running.
    #[link_section = ".boot2"]
    #[used]
    pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

    use hal::{
        clocks::init_clocks_and_plls,
        gpio, pac,
        pio::{self, PIOExt},
        pwm, sio, usb, Clock, Sio, Watchdog,
    };

    use rtic_monotonics::rp2040::prelude::*;
    rp2040_timer_monotonic!(Mono);

    use rtic_sync::channel::{Receiver, Sender};
    use usb_device::{class_prelude::*, prelude::*, UsbError};
    use usbd_human_interface_device::{
        device::keyboard::{NKROBootKeyboard, NKROBootKeyboardConfig},
        usb_class::{UsbHidClass, UsbHidClassBuilder},
        UsbHidError,
    };

    use crate::{
        heartbeat::HeartbeatLED,
        key::{Action, Edge, Key},
        keyboard::{Keyboard, KeyboardConfiguration},
        matrix::{BasicVerticalSwitchMatrix, Scanner},
        processor::{
            events::rgb::{FrameIterator, RGBMatrix, RGBProcessor},
            input::{
                debounce::KeyMatrixRisingFallingDebounceProcessor,
                flip::{ConcurrentFlipProcessor, Pos},
            },
            mapper::{Input, Mapper},
            Event, EventsProcessor, InputProcessor,
        },
        rotary::RotaryEncoder,
    };

    const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

    const INPUT_CHANNEL_BUFFER_SIZE: usize = 1;
    const KEYS_CHANNEL_BUFFER_SIZE: usize = 1;

    const INPUT_SCANNER_TARGET_POLL_FREQ: u64 = 1000;
    const HID_REPORTER_TARGET_POLL_FREQ: u64 = 1000;
    const INPUT_SCANNER_TARGET_POLL_PERIOD_MICROS: u64 =
        1_000_000u64 / INPUT_SCANNER_TARGET_POLL_FREQ;
    const HID_REPORTER_TARGET_POLL_PERIOD_MICROS: u64 =
        1_000_000u64 / HID_REPORTER_TARGET_POLL_FREQ;

    const DEBUG_LOG_INPUT_SCANNER_ENABLE_TIMING: bool = false;
    const DEBUG_LOG_INPUT_SCANNER_INTERVAL: u64 = 50;
    const DEBUG_LOG_PROCESSOR_ENABLE_TIMING: bool = false;
    const DEBUG_LOG_PROCESSOR_INTERVAL: u64 = 50;
    const DEBUG_LOG_EVENTS: bool = true;
    const DEBUG_LOG_SENT_KEYS: bool = false;

    #[shared]
    struct Shared {
        usb_device: UsbDevice<'static, usb::UsbBus>,
        usb_keyboard: UsbHidClass<
            'static,
            usb::UsbBus,
            frunk::HList!(NKROBootKeyboard<'static, usb::UsbBus>),
        >,
    }

    #[local]
    struct Local {
        key_matrix: Option<
            BasicVerticalSwitchMatrix<
                { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_COL_COUNT },
            >,
        >,
        rotary_encoder: Option<RotaryEncoder>,
        heartbeat_led: Option<HeartbeatLED>,
        rgb_matrix: Option<
            RGBMatrix<
                { <Keyboard as KeyboardConfiguration>::RGB_MATRIX_LED_COUNT },
                ws2812_pio::Ws2812Direct<
                    pac::PIO0,
                    pio::SM0,
                    gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionPio0, gpio::PullDown>,
                >,
            >,
        >,
    }

    #[init(local = [usb_allocator: Option<UsbBusAllocator<usb::UsbBus>> = None])]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        info!("init()");

        // Soft-reset does not release the hardware spinlocks.
        // Release them now to avoid a deadlock after debug or watchdog reset.
        unsafe {
            sio::spinlock_reset();
        }

        // Initialize global memory allocator
        {
            use core::mem::MaybeUninit;
            const HEAP_COUNT: usize = 2048;
            static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_COUNT] =
                [MaybeUninit::uninit(); HEAP_COUNT];
            unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_COUNT) }
        }

        // Set the ARM SLEEPONEXIT bit to go to sleep after handling interrupts
        // See https://developer.arm.com/docs/100737/0100/power-management/sleep-mode/sleep-on-exit-bit
        ctx.core.SCB.set_sleepdeep();

        // Configure watchdog, monotonics, and clock - The default is to generate a 125 MHz system clock
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        Mono::start(ctx.device.TIMER, &ctx.device.RESETS);
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut ctx.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // Init channels
        let (input_sender, input_receiver) = rtic_sync::make_channel!(Input<{<Keyboard as KeyboardConfiguration>::KEY_MATRIX_ROW_COUNT}, {<Keyboard as KeyboardConfiguration>::KEY_MATRIX_COL_COUNT}>, INPUT_CHANNEL_BUFFER_SIZE);
        let (keys_sender, keys_receiver) =
            rtic_sync::make_channel!(Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE);
        let (frame_sender, frame_receiver) = rtic_sync::make_channel!(Box<dyn FrameIterator>, 1);

        // Init HID device
        info!("init usb allocator");
        let usb_allocator = ctx
            .local
            .usb_allocator
            .insert(UsbBusAllocator::new(usb::UsbBus::new(
                ctx.device.USBCTRL_REGS,
                ctx.device.USBCTRL_DPRAM,
                clocks.usb_clock,
                true,
                &mut ctx.device.RESETS,
            )));

        info!("init usb keyboard");
        let usb_keyboard = UsbHidClassBuilder::new()
            .add_device(NKROBootKeyboardConfig::default())
            .build(usb_allocator);

        info!("init usb device");
        let usb_device = UsbDeviceBuilder::new(usb_allocator, UsbVidPid(0x1111, 0x1111))
            .strings(&[StringDescriptors::default()
                .manufacturer("daystram")
                .product("kb")
                .serial_number("8888")])
            .unwrap()
            .build();

        // Init keyboard
        let (pio0, sm0, _, _, _) = ctx.device.PIO0.split(&mut ctx.device.RESETS);
        let (key_matrix, rotary_encoder, heartbeat_led, rgb_matrix) = Keyboard::init(
            gpio::Pins::new(
                ctx.device.IO_BANK0,
                ctx.device.PADS_BANK0,
                Sio::new(ctx.device.SIO).gpio_bank0,
                &mut ctx.device.RESETS,
            ),
            pwm::Slices::new(ctx.device.PWM, &mut ctx.device.RESETS),
            pio0,
            sm0,
            clocks.peripheral_clock.freq(),
        );

        heartbeat::spawn().ok();
        input_scanner::spawn(input_sender).ok();
        processor::spawn(input_receiver, keys_sender, frame_sender).ok();
        rgb_matrix_renderer::spawn(frame_receiver).ok();
        hid_usb_tick::spawn().ok();
        hid_reporter::spawn(keys_receiver).ok();

        info!("enable interrupts");
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
        };

        info!("init() done");
        (
            Shared {
                usb_device,
                usb_keyboard,
            },
            Local {
                key_matrix,
                rotary_encoder,
                heartbeat_led,
                rgb_matrix,
            },
        )
    }

    #[idle()]
    fn idle(_ctx: idle::Context) -> ! {
        info!("idle()");
        loop {
            // https://developer.arm.com/documentation/ddi0406/c/Application-Level-Architecture/Instruction-Details/Alphabetical-list-of-instructions/WFI
            rtic::export::wfi()
        }
    }

    #[task (local=[key_matrix, rotary_encoder], priority = 1)]
    async fn input_scanner(
        ctx: input_scanner::Context,
        mut input_sender: Sender<
            'static,
            Input<
                { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_COL_COUNT },
            >,
            INPUT_CHANNEL_BUFFER_SIZE,
        >,
    ) {
        info!("input_scanner()");
        let mut poll_end_time = Mono::now();
        let mut n: u64 = 0;
        loop {
            let scan_start_time = Mono::now();
            let key_matrix_result = match ctx.local.key_matrix {
                Some(key_matrix) => key_matrix.scan().await,
                None => Default::default(),
            };
            let rotary_encoder_result = match ctx.local.rotary_encoder {
                Some(rotary_encoder) => rotary_encoder.scan(),
                None => Default::default(),
            };

            input_sender
                .try_send(Input {
                    key_matrix_result,
                    rotary_encoder_result,
                })
                .ok(); // drop data if buffer is full

            if DEBUG_LOG_INPUT_SCANNER_ENABLE_TIMING && n % DEBUG_LOG_INPUT_SCANNER_INTERVAL == 0 {
                let scan_end_time = Mono::now();
                debug!(
                    "[{}] input_scanner: {} us\tpoll: {} us\trate: {} Hz\t budget: {} %",
                    n,
                    (scan_end_time - scan_start_time).to_micros(),
                    (scan_end_time - poll_end_time).to_micros(),
                    1_000_000u64 / (scan_end_time - poll_end_time).to_micros(),
                    (scan_end_time - scan_start_time).to_micros() * 100
                        / INPUT_SCANNER_TARGET_POLL_PERIOD_MICROS
                );
            }

            poll_end_time = Mono::now();
            n = n.wrapping_add(1);
            Mono::delay_until(scan_start_time + INPUT_SCANNER_TARGET_POLL_PERIOD_MICROS.micros())
                .await;
        }
    }

    #[task(priority = 2)]
    async fn processor(
        _: processor::Context,
        mut input_receiver: Receiver<
            'static,
            Input<
                { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_COL_COUNT },
            >,
            INPUT_CHANNEL_BUFFER_SIZE,
        >,
        mut keys_sender: Sender<'static, Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE>,
        frame_sender: Sender<'static, Box<dyn FrameIterator>, 1>,
    ) {
        info!("processor()");
        let input_processors: &mut [&mut dyn InputProcessor<
            { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_ROW_COUNT },
            { <Keyboard as KeyboardConfiguration>::KEY_MATRIX_COL_COUNT },
        >] = &mut [
            &mut KeyMatrixRisingFallingDebounceProcessor::new(10.millis()),
            &mut ConcurrentFlipProcessor::new(Pos { row: 2, col: 1 }, Pos { row: 2, col: 3 }),
        ];
        let mut mapper = Mapper::new(<Keyboard as KeyboardConfiguration>::get_input_map());
        let events_processors: &mut [&mut dyn EventsProcessor<
            <Keyboard as KeyboardConfiguration>::Layer,
        >] = &mut [&mut RGBProcessor::<
            { <Keyboard as KeyboardConfiguration>::RGB_MATRIX_LED_COUNT },
        >::new(frame_sender)];

        let mut poll_end_time = Mono::now();
        let mut n: u64 = 0;
        while let Ok(mut input) = input_receiver.recv().await {
            let process_start_time = Mono::now();
            if input_processors
                .iter_mut()
                .try_for_each(|p| p.process(&mut input))
                .is_err()
            {
                continue;
            }

            let mut events =
                Vec::<Event<<Keyboard as KeyboardConfiguration>::Layer>>::with_capacity(10);
            mapper.map(&input, &mut events);

            if DEBUG_LOG_EVENTS {
                events
                    .iter()
                    .filter(|e| e.edge != Edge::None)
                    .for_each(|e| debug!("[{}] event: action: {} edge: {}", n, e.action, e.edge));
            }

            if events_processors
                .iter_mut()
                .try_for_each(|p| p.process(&mut events))
                .is_err()
            {
                continue;
            }

            keys_sender
                .try_send(
                    events
                        .into_iter()
                        .filter_map(|e| match e.action {
                            Action::Key(k) => Some(k),
                            _ => None,
                        })
                        .collect(),
                )
                .ok(); // drop data if buffer is full

            if DEBUG_LOG_PROCESSOR_ENABLE_TIMING && (n % DEBUG_LOG_PROCESSOR_INTERVAL == 0) {
                let scan_end_time = Mono::now();
                debug!(
                    "[{}] processor: {} us\tpoll: {} us\trate: {} Hz\t budget: {} %",
                    n,
                    (scan_end_time - process_start_time).to_micros(),
                    (scan_end_time - poll_end_time).to_micros(),
                    1_000_000u64 / (scan_end_time - poll_end_time).to_micros(),
                    (scan_end_time - process_start_time).to_micros() * 100
                        / INPUT_SCANNER_TARGET_POLL_PERIOD_MICROS
                );
            }

            poll_end_time = Mono::now();
            n = n.wrapping_add(1);
        }
    }

    #[task(shared=[usb_keyboard], priority = 1)]
    async fn hid_reporter(
        mut ctx: hid_reporter::Context,
        mut keys_receiver: Receiver<'static, Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE>,
    ) {
        info!("hid_reporter()");
        while let Ok(keys) = keys_receiver.recv().await {
            let start_time = Mono::now();
            if DEBUG_LOG_SENT_KEYS {
                debug!("keys: {:?}", keys.as_slice());
            }

            ctx.shared.usb_keyboard.lock(|k| {
                match k.device().write_report(keys.into_iter().map(|k| k.into())) {
                    Ok(_) => {}
                    Err(UsbHidError::WouldBlock) => {}
                    Err(UsbHidError::Duplicate) => {}
                    Err(e) => {
                        core::panic!("Failed to write keyboard report: {:?}", e);
                    }
                }
            });

            Mono::delay_until(start_time + HID_REPORTER_TARGET_POLL_PERIOD_MICROS.micros()).await;
        }
    }

    #[task(binds = USBCTRL_IRQ, shared = [usb_device, usb_keyboard], priority = 1)]
    fn hid_reader(ctx: hid_reader::Context) {
        (ctx.shared.usb_device, ctx.shared.usb_keyboard).lock(|usb_device, usb_keyboard| {
            if usb_device.poll(&mut [usb_keyboard]) {
                match usb_keyboard.device().read_report() {
                    Ok(leds) => {
                        debug!(
                            "num_lock: {}\ncaps_lock: {}\nscroll_lock: {}\ncompose: {}\nkana: {}\n",
                            leds.num_lock,
                            leds.caps_lock,
                            leds.scroll_lock,
                            leds.compose,
                            leds.kana,
                        );
                    }
                    Err(UsbError::WouldBlock) => {}
                    Err(e) => {
                        core::panic!("Failed to read keyboard report: {:?}", e)
                    }
                }
            }
        });
    }

    #[task(
        shared = [usb_keyboard],
        priority = 1,
    )]
    async fn hid_usb_tick(mut ctx: hid_usb_tick::Context) {
        info!("hid_usb_tick()");
        loop {
            ctx.shared.usb_keyboard.lock(|k| match k.tick() {
                Ok(_) => {}
                Err(UsbHidError::WouldBlock) => {}
                Err(e) => {
                    core::panic!("Failed to process keyboard tick: {:?}", e)
                }
            });
            Mono::delay(1.millis()).await;
        }
    }

    #[task(local=[heartbeat_led], priority = 1)]
    async fn heartbeat(ctx: heartbeat::Context) {
        info!("heartbeat()");
        match ctx.local.heartbeat_led {
            Some(heartbeat_led) => heartbeat_led.cycle().await,
            None => {}
        }
    }
    #[task(local=[rgb_matrix], priority = 3)]
    async fn rgb_matrix_renderer(
        ctx: rgb_matrix_renderer::Context,
        frame_receiver: Receiver<'static, Box<dyn FrameIterator>, 1>,
    ) {
        info!("rgb_matrix_renderer()");
        match ctx.local.rgb_matrix {
            Some(rgb_matrix) => {
                rgb_matrix.render(frame_receiver).await;
            }
            None => {}
        }
    }
}
