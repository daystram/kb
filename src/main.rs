#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(associated_type_defaults)]
#![feature(trait_alias)]
#![feature(async_closure)]
#![feature(generic_const_exprs)]
#![feature(future_join)]
#![feature(variant_count)]
#![allow(incomplete_features)]
#![allow(refining_impl_trait)]
#![allow(unused_macros)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::await_holding_refcell_ref)]
mod debug;
mod heartbeat;
#[macro_use]
mod key;
mod keyboard;
mod matrix;
mod oled;
mod processor;
mod remote;
mod rotary;
mod split;
mod status;
mod util;

#[macro_use]
extern crate alloc;
extern crate rp2040_hal as hal;
use {defmt_rtt as _, panic_probe as _};

#[rtic::app(
    device = hal::pac,
    dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3]
)]
mod kb {
    // The linker will place this boot block at the start of our program image.
    // We need this to help the ROM bootloader get our code up and running.
    #[link_section = ".boot2"]
    #[used]
    pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

    #[global_allocator]
    pub static HEAP: embedded_alloc::Heap = embedded_alloc::Heap::empty();
    const HEAP_SIZE_BYTES: usize = 16384; // 16 KB
    static mut HEAP_MEM: [core::mem::MaybeUninit<u8>; HEAP_SIZE_BYTES] =
        [core::mem::MaybeUninit::uninit(); HEAP_SIZE_BYTES];

    use alloc::{boxed::Box, rc::Rc, vec::Vec};
    use core::{cell::RefCell, fmt::Write};
    use hal::{
        clocks::init_clocks_and_plls,
        gpio, pac,
        pio::{self, PIOExt},
        pwm, sio, usb, Clock, Sio, Watchdog,
    };
    use rtic_monotonics::rp2040::prelude::*;
    use rtic_sync::{
        arbiter::Arbiter,
        channel::{Receiver, Sender},
    };
    use usb_device::{class_prelude::*, prelude::*, UsbError};
    use usbd_human_interface_device::{
        device::keyboard::{NKROBootKeyboard, NKROBootKeyboardConfig},
        usb_class::{UsbHidClass, UsbHidClassBuilder},
        UsbHidError,
    };

    use crate::{
        debug,
        heartbeat::HeartbeatLED,
        key::{Action, Edge, Key},
        keyboard::{Configuration, Configurator, Keyboard},
        matrix::{SplitScanner, SplitSwitchMatrix},
        processor::{
            events::rgb::{FrameIterator, RGBMatrix, RGBProcessor},
            input::debounce::KeyMatrixRisingFallingDebounceProcessor,
            mapper::{Input, Mapper},
            Event, EventsProcessor, InputProcessor,
        },
        remote::{
            self,
            transport::{
                uart::{UartReceiver, UartSender},
                Sequence, TransportReceiver,
            },
            Server,
        },
        rotary::RotaryEncoder,
        split,
        status::StatusLED,
    };

    rp2040_timer_monotonic!(Mono);

    const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

    const INPUT_CHANNEL_BUFFER_SIZE: usize = 1;
    const KEYS_CHANNEL_BUFFER_SIZE: usize = 1;

    const INPUT_SCANNER_TARGET_POLL_FREQ: u64 = 1000;
    const HID_REPORTER_TARGET_POLL_FREQ: u64 = 1000;
    const INPUT_SCANNER_TARGET_POLL_PERIOD_MICROS: u64 =
        1_000_000u64 / INPUT_SCANNER_TARGET_POLL_FREQ;
    const HID_REPORTER_TARGET_POLL_PERIOD_MICROS: u64 =
        1_000_000u64 / HID_REPORTER_TARGET_POLL_FREQ;

    #[shared]
    struct Shared {
        is_usb_connected: bool,
        usb_device: UsbDevice<'static, usb::UsbBus>,
        usb_keyboard: UsbHidClass<
            'static,
            usb::UsbBus,
            frunk::HList!(NKROBootKeyboard<'static, usb::UsbBus>),
        >,
        transport_sender: Option<Arbiter<Rc<RefCell<UartSender>>>>,
    }

    #[local]
    struct Local {
        transport_receiver: Option<UartReceiver>,
    }

    #[init(local = [usb_allocator: Option<UsbBusAllocator<usb::UsbBus>> = None])]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        defmt::info!("init()");

        // Soft-reset does not release the hardware spinlocks.
        // Release them now to avoid a deadlock after debug or watchdog reset.
        unsafe { sio::spinlock_reset() };

        // Initialize global memory allocator
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE_BYTES) };

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
        let (input_sender, input_receiver) = rtic_sync::make_channel!(Input<{<Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT}, {<Keyboard as Configurator>::KEY_MATRIX_COL_COUNT}>, INPUT_CHANNEL_BUFFER_SIZE);
        let (keys_sender, keys_receiver) =
            rtic_sync::make_channel!(Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE);
        let (frame_sender, frame_receiver) = rtic_sync::make_channel!(Box<dyn FrameIterator>, 1);

        // Init HID device
        defmt::info!("init usb allocator");
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

        defmt::info!("init usb keyboard");
        let usb_keyboard = UsbHidClassBuilder::new()
            .add_device(NKROBootKeyboardConfig::default())
            .build(usb_allocator);

        defmt::info!("init usb device");
        let usb_device = UsbDeviceBuilder::new(usb_allocator, UsbVidPid(0x1111, 0x1111))
            .strings(&[StringDescriptors::default()
                .manufacturer("daystram")
                .product("kb")
                .serial_number("8888")])
            .unwrap()
            .build();

        // Init keyboard
        let (pio0, sm0, _, _, _) = ctx.device.PIO0.split(&mut ctx.device.RESETS);
        let (config, transport) = Keyboard::init(
            gpio::Pins::new(
                ctx.device.IO_BANK0,
                ctx.device.PADS_BANK0,
                Sio::new(ctx.device.SIO).gpio_bank0,
                &mut ctx.device.RESETS,
            ),
            pwm::Slices::new(ctx.device.PWM, &mut ctx.device.RESETS),
            pio0,
            sm0,
            ctx.device.I2C1,
            ctx.device.UART0,
            &mut ctx.device.RESETS,
            clocks.peripheral_clock.freq(),
            &clocks.system_clock,
        );
        assert!(
            !config.is_split() || transport.is_some(),
            "keyboard is configured as split but remote transport is not configured"
        );

        let (transport_sender, mut transport_receiver) = match transport {
            Some((transport_sender, transport_receiver)) => {
                (Some(transport_sender), Some(transport_receiver))
            }
            None => (None, None),
        };

        // Start
        start_wait_usb::spawn(
            1.secs(),
            input_sender,
            input_receiver,
            keys_sender,
            keys_receiver,
            frame_sender,
            frame_receiver,
            transport_receiver
                .as_mut()
                .map(|transport_receiver| transport_receiver.initialize_seq_sender()),
            config,
        )
        .ok();

        defmt::info!("init() done");
        (
            Shared {
                is_usb_connected: false,
                usb_device,
                usb_keyboard,
                transport_sender,
            },
            Local { transport_receiver },
        )
    }

    #[idle()]
    fn idle(_: idle::Context) -> ! {
        defmt::info!("idle()");
        loop {
            // https://developer.arm.com/documentation/ddi0406/c/Application-Level-Architecture/Instruction-Details/Alphabetical-list-of-instructions/WFI
            rtic::export::wfi()
        }
    }

    // ============================= Master and Slave
    #[task(shared=[is_usb_connected], priority = 1)]
    async fn start_wait_usb(
        mut ctx: start_wait_usb::Context,
        timeout: <Mono as Monotonic>::Duration,
        input_sender: Sender<
            'static,
            Input<
                { <Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
            >,
            INPUT_CHANNEL_BUFFER_SIZE,
        >,
        input_receiver: Receiver<
            'static,
            Input<
                { <Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
            >,
            INPUT_CHANNEL_BUFFER_SIZE,
        >,
        keys_sender: Sender<'static, Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE>,
        keys_receiver: Receiver<'static, Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE>,
        frame_sender: Sender<'static, Box<dyn FrameIterator>, 1>,
        frame_receiver: Receiver<'static, Box<dyn FrameIterator>, 1>,
        seq_sender: Option<Receiver<'static, Sequence, { remote::REQUEST_SEQUENCE_QUEUE_SIZE }>>,
        mut config: Configuration,
    ) {
        defmt::info!("start_wait_usb()");
        if let Some(ref mut display) = config.oled_display {
            display.write_str("kb").unwrap();
        }

        // Start USB tasks
        hid_usb_tick::spawn().ok();
        hid_reporter::spawn(keys_receiver).ok();
        unsafe { hal::pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ) }

        Mono::delay(timeout).await;

        split::set_self_mode(ctx.shared.is_usb_connected.lock(|is_usb_connected| {
            if *is_usb_connected {
                split::Mode::Master
            } else {
                split::Mode::Slave
            }
        }));
        defmt::warn!(
            "detected as {} {}",
            split::get_self_mode(),
            split::get_self_side()
        );

        if let Some(ref mut status_led) = config.status_led {
            status_led.set_link(true);
        }

        if let Some(ref mut display) = config.oled_display {
            display.clear();
            display
                .write_fmt(format_args!(
                    "{}\n{}",
                    <Keyboard as Configurator>::NAME,
                    split::get_self_mode()
                ))
                .unwrap();
        }
        match split::get_self_mode() {
            split::Mode::Master => {
                heartbeat::spawn(config.heartbeat_led, 500.millis()).ok();
                master_input_scanner::spawn(
                    config.key_matrix_split,
                    config.rotary_encoder,
                    input_sender,
                )
                .ok();
                master_processor::spawn(
                    input_receiver,
                    keys_sender,
                    frame_sender,
                    config.status_led,
                )
                .ok();
                rgb_matrix_renderer::spawn(config.rgb_matrix, frame_receiver).ok();
            }
            split::Mode::Slave => {
                assert!(
                    config.is_split(),
                    "keyboard is not configured to operate as split"
                );

                // Initialize server and register services
                let mut server = Server::new(seq_sender.unwrap());
                if let Some(key_matrix_split) = config.key_matrix_split {
                    server.register_service(Box::new(key_matrix_split)).await;
                }

                heartbeat::spawn(config.heartbeat_led, 2000.millis()).ok();
                slave_server::spawn(server).ok();
            }
        }
        unsafe { hal::pac::NVIC::unmask(hal::pac::Interrupt::UART0_IRQ) }
    }

    #[task(priority = 2)]
    async fn heartbeat(
        _: heartbeat::Context,
        mut heartbeat_led: Option<HeartbeatLED>,
        period: <Mono as Monotonic>::Duration,
    ) {
        defmt::info!("heartbeat()");
        if let Some(ref mut heartbeat_led) = heartbeat_led {
            heartbeat_led.cycle(period).await
        };
    }

    #[task(binds = UART0_IRQ, local = [transport_receiver], priority = 1)]
    fn receive_uart(ctx: receive_uart::Context) {
        match ctx.local.transport_receiver {
            Some(transport_receiver) => {
                let start_time = Mono::now();
                transport_receiver.read_into_buffer();
                let end_time = Mono::now();
                debug::log_duration(
                    debug::LogDurationTag::UARTIRQRecieveBuffer,
                    start_time,
                    end_time,
                );
            }
            None => {}
        }
    }
    // ============================= Master and Slave

    // ============================= Slave
    #[task (shared=[&transport_sender], priority = 1)]
    async fn slave_server(ctx: slave_server::Context, mut server: Server) {
        defmt::info!("slave_server()");
        server
            .listen(ctx.shared.transport_sender.as_ref().unwrap())
            .await;
    }
    // ============================= Slave

    // ============================= Master
    #[task (shared=[&transport_sender], priority = 1)]
    async fn master_input_scanner(
        ctx: master_input_scanner::Context,
        mut key_matrix_split: Option<
            SplitSwitchMatrix<
                { <Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
            >,
        >,
        mut rotary_encoder: Option<RotaryEncoder>,
        mut input_sender: Sender<
            'static,
            Input<
                { <Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
            >,
            INPUT_CHANNEL_BUFFER_SIZE,
        >,
    ) {
        defmt::info!("master_input_scanner()");
        let mut poll_end_time = Mono::now();
        let mut n: u64 = 0;
        loop {
            let scan_start_time = Mono::now();

            let transport_sender = ctx.shared.transport_sender.as_ref();
            let key_matrix_result = match key_matrix_split {
                Some(ref mut key_matrix_split) => {
                    key_matrix_split.scan(transport_sender.unwrap()).await
                }
                None => Default::default(),
            };
            let rotary_encoder_result = match rotary_encoder {
                Some(ref mut rotary_encoder) => rotary_encoder.scan(),
                None => Default::default(),
            };
            input_sender
                .try_send(Input {
                    key_matrix_result,
                    rotary_encoder_result,
                })
                .ok(); // drop data if buffer is full

            if debug::ENABLE_LOG_INPUT_SCANNER_ENABLE_TIMING
                && n % debug::LOG_INPUT_SCANNER_SAMPLING_RATE == 0
            {
                let scan_end_time = Mono::now();
                defmt::debug!(
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
            debug::log_duration(
                debug::LogDurationTag::ClientInputScan,
                scan_start_time,
                poll_end_time,
            );
            debug::log_heap();

            n = n.wrapping_add(1);
            Mono::delay_until(scan_start_time + INPUT_SCANNER_TARGET_POLL_PERIOD_MICROS.micros())
                .await;
        }
    }

    #[task(priority = 2)]
    async fn master_processor(
        _: master_processor::Context,
        mut input_receiver: Receiver<
            'static,
            Input<
                { <Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
                { <Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
            >,
            INPUT_CHANNEL_BUFFER_SIZE,
        >,
        mut keys_sender: Sender<'static, Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE>,
        frame_sender: Sender<'static, Box<dyn FrameIterator>, 1>,
        mut status_led: Option<StatusLED>,
    ) {
        defmt::info!("master_processor()");
        let input_processors: &mut [&mut dyn InputProcessor<
            { <Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
            { <Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
        >] = &mut [&mut KeyMatrixRisingFallingDebounceProcessor::new(
            10.millis(),
        )];
        let mut mapper = Mapper::new(<Keyboard as Configurator>::get_input_map());
        let events_processors: &mut [&mut dyn EventsProcessor<
            <Keyboard as Configurator>::Layer,
        >] = &mut [&mut RGBProcessor::<
            { <Keyboard as Configurator>::RGB_MATRIX_LED_COUNT },
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

            let mut events = Vec::<Event<<Keyboard as Configurator>::Layer>>::with_capacity(10);
            mapper.map(&input, &mut events);

            if debug::ENABLE_LOG_EVENTS {
                events
                    .iter()
                    .filter(|e| e.edge != Edge::None)
                    .for_each(|e| {
                        defmt::debug!("[{}] event: action: {} edge: {}", n, e.action, e.edge)
                    });
            }

            if let Some(ref mut status_led) = status_led {
                status_led.update_activity(!events.is_empty());
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

            if debug::ENABLE_LOG_PROCESSOR_ENABLE_TIMING
                && (n % debug::LOG_PROCESSOR_SAMPLING_RATE == 0)
            {
                let scan_end_time = Mono::now();
                defmt::debug!(
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

    #[task(priority = 3)]
    async fn rgb_matrix_renderer(
        _: rgb_matrix_renderer::Context,
        mut rgb_matrix: Option<
            RGBMatrix<
                { <Keyboard as Configurator>::RGB_MATRIX_LED_COUNT },
                ws2812_pio::Ws2812Direct<
                    pac::PIO0,
                    pio::SM0,
                    gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionPio0, gpio::PullDown>,
                >,
            >,
        >,
        frame_receiver: Receiver<'static, Box<dyn FrameIterator>, 1>,
    ) {
        defmt::info!("rgb_matrix_renderer()");
        if let Some(ref mut rgb_matrix) = rgb_matrix {
            rgb_matrix.render(frame_receiver).await;
        }
    }

    #[task(shared=[usb_keyboard], priority = 2)]
    async fn hid_reporter(
        mut ctx: hid_reporter::Context,
        mut keys_receiver: Receiver<'static, Vec<Key>, KEYS_CHANNEL_BUFFER_SIZE>,
    ) {
        defmt::info!("hid_reporter()");
        while let Ok(keys) = keys_receiver.recv().await {
            let start_time = Mono::now();
            if debug::ENABLE_LOG_SENT_KEYS {
                defmt::debug!("keys: {:?}", keys.as_slice());
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

    #[task(binds = USBCTRL_IRQ, shared = [usb_device, usb_keyboard, is_usb_connected], priority = 2)]
    fn hid_reader(ctx: hid_reader::Context) {
        (ctx.shared.usb_device, ctx.shared.usb_keyboard, ctx.shared.is_usb_connected).lock(|usb_device, usb_keyboard, is_usb_connected| {
            if usb_device.poll(&mut [usb_keyboard]) {
                *is_usb_connected = true; // usb connection detected
                match usb_keyboard.device().read_report() {
                    Ok(leds) => {
                        defmt::debug!(
                            "\nnum_lock: {}\ncaps_lock: {}\nscroll_lock: {}\ncompose: {}\nkana: {}\n",
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
        priority = 2,
    )]
    async fn hid_usb_tick(mut ctx: hid_usb_tick::Context) {
        defmt::info!("hid_usb_tick()");
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
    // ============================= Master
}
