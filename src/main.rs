#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![feature(error_in_core)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(associated_type_defaults)]
#![feature(trait_alias)]
mod config;
mod key;
mod matrix;
mod processor;
mod util;

extern crate rp2040_hal as hal;
use {defmt_rtt as _, panic_probe as _};

#[rtic::app(
    device = hal::pac,
    dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3]
)]
mod kb {
    extern crate alloc;
    use alloc::{boxed::Box, vec::Vec};
    #[global_allocator]
    static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();

    use defmt::{debug, info};

    // The linker will place this boot block at the start of our program image.
    // We need this to help the ROM bootloader get our code up and running.
    #[link_section = ".boot2"]
    #[used]
    pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

    use embedded_hal::PwmPin;
    use hal::{
        clocks::init_clocks_and_plls, gpio, pac, pio, prelude::*, pwm, sio, usb, Sio, Watchdog,
    };
    use rtic_monotonics::{rp2040::*, Monotonic};
    use rtic_sync::channel::{Receiver, Sender};
    use usb_device::{
        class_prelude::UsbBusAllocator,
        prelude::{UsbDevice, UsbDeviceBuilder},
        UsbError,
    };
    use usbd_human_interface_device::{
        device::{self, keyboard::NKROBootKeyboard},
        usb_class::{UsbHidClass, UsbHidClassBuilder},
        UsbHidError,
    };

    use crate::{
        config::{self, Layer, KEY_MAP},
        key::{Action, Key},
        matrix::{BasicVerticalSwitchMatrix, Bitmap, Scanner},
        processor::{
            events::rgb::{FrameIterator, RGBMatrix, RGBProcessor},
            keymap::KeyMapper,
            BitmapProcessor, Event, EventsProcessor,
        },
    };

    const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
    const MAX_PWM_POWER: u16 = 0x6000; // max: 0x8000

    const BITMAP_CHANNEL_BUFFER_SIZE: usize = 1;
    const KEYS_CHANNEL_BUFFER_SIZE: usize = 1;

    const MATRIX_SCANNER_TARGET_POLL_FREQ: u64 = 1000;
    const HID_REPORTER_TARGET_POLL_FREQ: u64 = 1000;
    const MATRIX_SCANNER_TARGET_POLL_PERIOD_MICROS: u64 =
        1_000_000u64 / MATRIX_SCANNER_TARGET_POLL_FREQ;
    const HID_REPORTER_TARGET_POLL_PERIOD_MICROS: u64 =
        1_000_000u64 / HID_REPORTER_TARGET_POLL_FREQ;

    const DEBUG_LOG_MATRIX_SCANNER_ENABLE_TIMING: bool = true;
    const DEBUG_LOG_MATRIX_SCANNER_INTERVAL: u64 = 50;
    const DEBUG_LOG_STREAM_PROCESSOR_ENABLE_TIMING: bool = true;
    const DEBUG_LOG_STREAM_PROCESSOR_INTERVAL: u64 = 50;
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
        switch_matrix: BasicVerticalSwitchMatrix<{ config::ROW_COUNT }, { config::COL_COUNT }>,
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

        // Initialize the interrupt for the RP2040 timer and obtain the token proving that we have
        let rp2040_timer_token = rtic_monotonics::create_rp2040_monotonic_token!();

        // Configure the clocks, watchdog - The default is to generate a 125 MHz system clock
        Timer::start(ctx.device.TIMER, &mut ctx.device.RESETS, rp2040_timer_token);
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
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

        // Init GPIO pins
        let pins = gpio::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            Sio::new(ctx.device.SIO).gpio_bank0,
            &mut ctx.device.RESETS,
        );

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
            .add_device(device::keyboard::NKROBootKeyboardConfig::default())
            .build(usb_allocator);

        info!("init usb device");
        let usb_device = UsbDeviceBuilder::new(
            usb_allocator,
            usb_device::prelude::UsbVidPid(0x1111, 0x1111),
        )
        .manufacturer("daystram")
        .product("kb")
        .serial_number("8888")
        .build();

        debug!("spawn hid_usb_tick");
        hid_usb_tick::spawn().ok();

        // Init heartbeat LED
        let mut pwm_slices = pwm::Slices::new(ctx.device.PWM, &mut ctx.device.RESETS);
        pwm_slices.pwm6.set_ph_correct();
        pwm_slices.pwm6.enable();

        let mut pwm_channel = pwm_slices.pwm6.channel_b;
        pwm_channel.output_to(
            pins.gpio29
                .into_push_pull_output_in_state(gpio::PinState::Low),
        );

        debug!("spawn heartbeat");
        heartbeat::spawn(pwm_channel).ok();

        // Init channels
        let (bitmap_sender, bitmap_receiver) = rtic_sync::make_channel!(Bitmap<{config::ROW_COUNT}, {config::COL_COUNT}>, BITMAP_CHANNEL_BUFFER_SIZE);
        let (keys_sender, keys_receiver) =
            rtic_sync::make_channel!(Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE);

        let (frame_sender, frame_receiver) = rtic_sync::make_channel!(Box<dyn FrameIterator>, 1);

        // Init switch matrix
        #[rustfmt::skip]
        let switch_matrix = BasicVerticalSwitchMatrix::new(
            [
                Box::new(pins.gpio24.into_pull_down_input()),
                Box::new(pins.gpio23.into_pull_down_input()),
                Box::new(pins.gpio22.into_pull_down_input()),
                Box::new(pins.gpio21.into_pull_down_input()),
                Box::new(pins.gpio20.into_pull_down_input()),
            ],
            [
                Box::new(pins.gpio0.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio1.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio2.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio3.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio4.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio5.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio6.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio7.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio8.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio9.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio10.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio11.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio12.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio13.into_push_pull_output_in_state(gpio::PinState::Low)),
                Box::new(pins.gpio14.into_push_pull_output_in_state(gpio::PinState::Low)),
            ],
        );

        // Init LED matrix
        let (mut pio0, sm0, _, _, _) = ctx.device.PIO0.split(&mut ctx.device.RESETS);
        let ws = ws2812_pio::Ws2812Direct::new(
            pins.gpio28.into_function(),
            &mut pio0,
            sm0,
            clocks.peripheral_clock.freq(),
        );
        debug!("spawn rgb_matrix_renderer");
        rgb_matrix_renderer::spawn(ws, frame_receiver).ok();

        // Init matrix scanner
        debug!("spawn matrix_scanner");
        matrix_scanner::spawn(bitmap_sender).ok();

        // Init stream processor
        debug!("spawn stream_processor");
        stream_processor::spawn(bitmap_receiver, keys_sender, frame_sender).ok();

        // Init HID reporter
        debug!("spawn hid_reporter");
        hid_reporter::spawn(keys_receiver).ok();

        // Enable interrupts
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
            Local { switch_matrix },
        )
    }

    #[idle()]
    fn idle(_ctx: idle::Context) -> ! {
        debug!("idle()");
        loop {
            // https://developer.arm.com/documentation/ddi0406/c/Application-Level-Architecture/Instruction-Details/Alphabetical-list-of-instructions/WFI
            rtic::export::wfi()
        }
    }

    #[task (local=[switch_matrix], priority = 1)]
    async fn matrix_scanner(
        ctx: matrix_scanner::Context,
        mut bitmap_sender: Sender<
            'static,
            Bitmap<{ config::ROW_COUNT }, { config::COL_COUNT }>,
            BITMAP_CHANNEL_BUFFER_SIZE,
        >,
    ) {
        info!("matrix_scanner()");
        let mut poll_end_time = Timer::now();
        let mut n: u64 = 0;
        loop {
            let scan_start_time = Timer::now();
            let bitmap = ctx.local.switch_matrix.scan().await;
            bitmap_sender.try_send(bitmap).ok(); // drop data if buffer is full

            if DEBUG_LOG_MATRIX_SCANNER_ENABLE_TIMING && n % DEBUG_LOG_MATRIX_SCANNER_INTERVAL == 0
            {
                let scan_end_time = Timer::now();
                debug!(
                    "[{}] matrix_scanner: {} us\tpoll: {} us\trate: {} Hz\t budget: {} %",
                    n,
                    (scan_end_time - scan_start_time).to_micros(),
                    (scan_end_time - poll_end_time).to_micros(),
                    1_000_000u64 / (scan_end_time - poll_end_time).to_micros(),
                    (scan_end_time - scan_start_time).to_micros() * 100
                        / MATRIX_SCANNER_TARGET_POLL_PERIOD_MICROS
                );
            }

            poll_end_time = Timer::now();
            n = n.wrapping_add(1);
            Timer::delay_until(scan_start_time + MATRIX_SCANNER_TARGET_POLL_PERIOD_MICROS.micros())
                .await;
        }
    }

    #[task(priority = 2)]
    async fn stream_processor(
        _: stream_processor::Context,
        mut bitmap_receiver: Receiver<
            'static,
            Bitmap<{ config::ROW_COUNT }, { config::COL_COUNT }>,
            BITMAP_CHANNEL_BUFFER_SIZE,
        >,
        mut keys_sender: Sender<'static, Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE>,
        frame_sender: Sender<'static, Box<dyn FrameIterator>, 1>,
    ) {
        info!("stream_processor()");
        let bitmap_processors: &mut [&mut dyn BitmapProcessor<
            { config::ROW_COUNT },
            { config::COL_COUNT },
        >] = &mut [];
        let mut mapper = KeyMapper::new(KEY_MAP);
        let events_processors: &mut [&mut dyn EventsProcessor<Layer>] =
            &mut [&mut RGBProcessor::<{ config::LED_COUNT }>::new(
                frame_sender,
            )];

        let mut poll_end_time = Timer::now();
        let mut n: u64 = 0;
        while let Ok(mut bitmap) = bitmap_receiver.recv().await {
            let process_start_time = Timer::now();
            match bitmap_processors
                .iter_mut()
                .try_for_each(|p| p.process(&mut bitmap))
            {
                Err(_) => continue,
                _ => {}
            }

            let mut events = Vec::<Event<Layer>>::with_capacity(10);
            mapper.map(&bitmap, &mut events);

            match events_processors
                .iter_mut()
                .try_for_each(|p| p.process(&mut events))
            {
                Err(_) => continue,
                _ => {}
            }

            keys_sender
                .try_send(
                    events
                        .into_iter()
                        .filter_map(|e| match e.action {
                            Action::Key(k) => Some(k.into()),
                            _ => None,
                        })
                        .collect(),
                )
                .ok(); // drop data if buffer is full

            if DEBUG_LOG_STREAM_PROCESSOR_ENABLE_TIMING
                && (n % DEBUG_LOG_STREAM_PROCESSOR_INTERVAL == 0)
            {
                let scan_end_time = Timer::now();
                debug!(
                    "[{}] stream_processor: {} us\tpoll: {} us\trate: {} Hz\t budget: {} %",
                    n,
                    (scan_end_time - process_start_time).to_micros(),
                    (scan_end_time - poll_end_time).to_micros(),
                    1_000_000u64 / (scan_end_time - poll_end_time).to_micros(),
                    (scan_end_time - process_start_time).to_micros() * 100
                        / MATRIX_SCANNER_TARGET_POLL_PERIOD_MICROS
                );
            }

            poll_end_time = Timer::now();
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
            let start_time = Timer::now();
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

            Timer::delay_until(start_time + HID_REPORTER_TARGET_POLL_PERIOD_MICROS.micros()).await;
        }
    }

    #[task(binds = USBCTRL_IRQ, shared = [usb_device, usb_keyboard], local=[], priority = 1)]
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
            Timer::delay(1.millis()).await;
        }
    }

    #[task(priority = 1)]
    async fn heartbeat(
        _: heartbeat::Context,
        mut channel: pwm::Channel<pwm::Slice<pwm::Pwm6, pwm::FreeRunning>, pwm::B>,
    ) {
        info!("heartbeat()");
        loop {
            lerp(&mut channel, 0, MAX_PWM_POWER, 200, 10).await;

            lerp(&mut channel, MAX_PWM_POWER, 0, 200, 10).await;
        }
    }
    #[task(priority = 3)]
    async fn rgb_matrix_renderer(
        _ctx: rgb_matrix_renderer::Context,
        ws: ws2812_pio::Ws2812Direct<
            pac::PIO0,
            pio::SM0,
            gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionPio0, gpio::PullDown>,
        >,
        frame_receiver: Receiver<'static, Box<dyn FrameIterator>, 1>,
    ) {
        debug!("rgb_matrix_renderer()");
        let mut rgb_matrix = RGBMatrix::<{ config::LED_COUNT }, _>::new(ws);
        rgb_matrix.render(frame_receiver).await;
    }

    async fn lerp<S: pwm::AnySlice>(
        channel: &mut pwm::Channel<S, pwm::B>,
        from: u16,
        to: u16,
        step: u16,
        delay_ms: u64,
    ) {
        let diff = if from < to {
            (to - from) / step
        } else {
            (from - to) / step
        };

        for d in (0..step)
            .map(|x| x * diff)
            .map(|x| if from < to { from + x } else { from - x })
        {
            Timer::delay(delay_ms.millis()).await;
            channel.set_duty(d);
        }
    }
}
