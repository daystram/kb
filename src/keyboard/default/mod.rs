pub mod layout;

use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc};
use hal::{fugit::HertzU32, gpio, pac, pio, pwm};
use rtic_sync::arbiter::Arbiter;
use ws2812_pio::Ws2812Direct;

use crate::{
    heartbeat::HeartbeatLED,
    keyboard::{Configuration, Configurator},
    matrix::BasicVerticalSwitchMatrix,
    processor::events::rgb::RGBMatrix,
    remote::transport::uart::{UartReceiver, UartSender},
    rotary::{Mode, RotaryEncoder},
};

const ENABLE_HEARTBEAT_LED: bool = true;
const ENABLE_KEY_MATRIX: bool = true;
const ENABLE_ROTARY_ENCODER: bool = true;
const ENABLE_RGB_MATRIX: bool = true;

pub struct Keyboard {}

impl Configurator for Keyboard {
    const KEY_MATRIX_ROW_COUNT: usize = 2;
    const KEY_MATRIX_COL_COUNT: usize = 2;

    const RGB_MATRIX_LED_COUNT: usize = 4;

    fn init(
        pins: gpio::Pins,
        mut slices: pwm::Slices,
        mut pio0: pio::PIO<pac::PIO0>,
        sm0: pio::UninitStateMachine<(pac::PIO0, pio::SM0)>,
        _uart0: pac::UART0,
        _resets: &mut pac::RESETS,
        clock_freq: HertzU32,
    ) -> (
        Configuration,
        Option<(Arbiter<Rc<RefCell<UartSender>>>, UartReceiver)>,
    ) {
        #[rustfmt::skip]
        let key_matrix = if ENABLE_KEY_MATRIX {
            Some(BasicVerticalSwitchMatrix::new(
                [
                    Box::new(pins.gpio21.into_pull_down_input()),
                    Box::new(pins.gpio20.into_pull_down_input()),
                ],
                [
                    Box::new(pins.gpio0.into_push_pull_output()),
                    Box::new(pins.gpio1.into_push_pull_output()),
                ],
            ))
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

        (
            Configuration {
                key_matrix,
                key_matrix_split: None,
                rotary_encoder,
                heartbeat_led,
                rgb_matrix,
            },
            None,
        )
    }
}
