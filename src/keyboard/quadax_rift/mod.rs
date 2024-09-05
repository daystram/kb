pub mod layout;

use alloc::{boxed::Box, rc::Rc};
use core::cell::RefCell;
use hal::{
    fugit::{HertzU32, RateExtU32},
    gpio, pac, pio, pwm, uart,
};
use rtic_sync::arbiter::Arbiter;
use ssd1306::{prelude::*, size::DisplaySize128x32, I2CDisplayInterface, Ssd1306};
use ws2812_pio::Ws2812Direct;

use crate::{
    heartbeat::HeartbeatLED,
    keyboard::{Configuration, Configurator},
    matrix::{BasicVerticalSwitchMatrix, SplitSwitchMatrix},
    oled::OLEDDisplay,
    processor::events::rgb::RGBMatrix,
    remote::transport::uart::{UartReceiver, UartSender},
    rotary::{Mode, RotaryEncoder},
    split::{self, SideDetector},
    status::StatusLED,
};

const ENABLE_HEARTBEAT_LED: bool = true;
const ENABLE_KEY_MATRIX: bool = true;
const ENABLE_ROTARY_ENCODER: bool = true;
const ENABLE_RGB_MATRIX: bool = true;
const ENABLE_OLED_SCREEN: bool = true;
const ENABLE_STATUS_LED: bool = true;

pub struct Keyboard {}

impl Configurator for Keyboard {
    const NAME: &str = "quadax-rift";

    const KEY_MATRIX_ROW_COUNT: usize = 5;
    const KEY_MATRIX_COL_COUNT: usize = 14;

    const RGB_MATRIX_LED_COUNT: usize = 67;

    fn init(
        pins: gpio::Pins,
        mut slices: pwm::Slices,
        mut pio0: pio::PIO<pac::PIO0>,
        sm0: pio::UninitStateMachine<(pac::PIO0, pio::SM0)>,
        i2c1: pac::I2C1,
        uart0: pac::UART0,
        resets: &mut pac::RESETS,
        clock_freq: HertzU32,
        system_clock: &hal::clocks::SystemClock,
    ) -> (
        Configuration,
        Option<(Arbiter<Rc<RefCell<UartSender>>>, UartReceiver)>,
    ) {
        SideDetector::new(Box::new(pins.gpio2.into_pull_down_input())).detect();

        #[rustfmt::skip]
        let key_matrix_split = if ENABLE_KEY_MATRIX {
            Some(SplitSwitchMatrix::new(BasicVerticalSwitchMatrix::new(
                [
                    Box::new(pins.gpio10.into_pull_down_input()),
                    Box::new(pins.gpio11.into_pull_down_input()),
                    Box::new(pins.gpio12.into_pull_down_input()),
                    Box::new(pins.gpio13.into_pull_down_input()),
                    Box::new(pins.gpio14.into_pull_down_input()),
                ],
                [
                    Box::new(pins.gpio3.into_push_pull_output()),
                    Box::new(pins.gpio4.into_push_pull_output()),
                    Box::new(pins.gpio5.into_push_pull_output()),
                    Box::new(pins.gpio6.into_push_pull_output()),
                    Box::new(pins.gpio7.into_push_pull_output()),
                    Box::new(pins.gpio8.into_push_pull_output()),
                    Box::new(pins.gpio9.into_push_pull_output()),
                ],
            )))
        } else {
            None
        };

        let rotary_encoder = if ENABLE_ROTARY_ENCODER {
            Some(RotaryEncoder::new(
                Box::new(pins.gpio15.into_pull_up_input()),
                Box::new(pins.gpio17.into_pull_up_input()),
                Box::new(pins.gpio16.into_push_pull_output()),
                Mode::DentHighPrecision,
            ))
        } else {
            None
        };

        let heartbeat_led = if ENABLE_HEARTBEAT_LED {
            slices.pwm6.set_ph_correct();
            slices.pwm6.enable();
            slices.pwm6.channel_b.output_to(
                pins.gpio29
                    .into_push_pull_output_in_state(gpio::PinState::Low),
            );
            Some(HeartbeatLED::new(Box::new(slices.pwm6.channel_b)))
        } else {
            None
        };

        let rgb_matrix = if ENABLE_RGB_MATRIX {
            let ws = Ws2812Direct::new(pins.gpio28.into_function(), &mut pio0, sm0, clock_freq);
            Some(RGBMatrix::<{ Keyboard::RGB_MATRIX_LED_COUNT }, _>::new(ws))
        } else {
            None
        };

        let oled_display = if ENABLE_OLED_SCREEN {
            let sda_pin: gpio::Pin<_, gpio::FunctionI2C, _> = pins.gpio26.reconfigure();
            let scl_pin: gpio::Pin<_, gpio::FunctionI2C, _> = pins.gpio27.reconfigure();
            let i2c = hal::I2C::i2c1(i2c1, sda_pin, scl_pin, 400.kHz(), resets, system_clock);

            Some(OLEDDisplay::new(Ssd1306::new(
                I2CDisplayInterface::new(i2c),
                DisplaySize128x32,
                match split::get_self_side() {
                    split::Side::Left => DisplayRotation::Rotate180,
                    split::Side::Right => DisplayRotation::Rotate0,
                },
            )))
        } else {
            None
        };

        let status_led = if ENABLE_STATUS_LED {
            Some(StatusLED::new(
                Box::new(pins.gpio24.into_push_pull_output()),
                Box::new(pins.gpio25.into_push_pull_output()),
            ))
        } else {
            None
        };

        let mut uart_peripheral = uart::UartPeripheral::new(
            uart0,
            (pins.gpio0.into_function(), pins.gpio1.into_function()),
            resets,
        )
        .enable(
            uart::UartConfig::new(
                // 115_200.Hz(),
                230_400.Hz(),
                uart::DataBits::Eight,
                None,
                uart::StopBits::One,
            ),
            clock_freq,
        )
        .unwrap();
        uart_peripheral.set_fifos(true);
        let (uart_reader, uart_writer) = uart_peripheral.split();
        let uart_sender = Arbiter::new(Rc::new(RefCell::new(UartSender::new(uart_writer))));
        let uart_receiver = UartReceiver::new(uart_reader);

        (
            Configuration {
                key_matrix: None,
                key_matrix_split,
                rotary_encoder,
                heartbeat_led,
                rgb_matrix,
                oled_display,
                status_led,
            },
            Some((uart_sender, uart_receiver)),
        )
    }
}
