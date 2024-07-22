pub mod layout;

use alloc::boxed::Box;
use hal::{fugit::HertzU32, gpio, pac, pio, pwm};
use ws2812_pio::Ws2812Direct;

use crate::{
    heartbeat::HeartbeatLED,
    keyboard::KeyboardConfiguration,
    matrix::BasicVerticalSwitchMatrix,
    processor::events::rgb::RGBMatrix,
    rotary::{Mode, RotaryEncoder},
};

const ENABLE_HEARTBEAT_LED: bool = true;
const ENABLE_KEY_MATRIX: bool = true;
const ENABLE_ROTARY_ENCODER: bool = true;
const ENABLE_RGB_MATRIX: bool = true;

pub struct Keyboard {}

impl KeyboardConfiguration for Keyboard {
    const KEY_MATRIX_ROW_COUNT: usize = 5;
    const KEY_MATRIX_COL_COUNT: usize = 15;

    const RGB_MATRIX_LED_COUNT: usize = 67;

    fn init(
        pins: gpio::Pins,
        mut slices: pwm::Slices,
        mut pio0: pio::PIO<pac::PIO0>,
        sm0: pio::UninitStateMachine<(pac::PIO0, pio::SM0)>,
        clock_freq: HertzU32,
    ) -> (
        Option<
            BasicVerticalSwitchMatrix<
                { Self::KEY_MATRIX_ROW_COUNT },
                { Self::KEY_MATRIX_COL_COUNT },
            >,
        >,
        Option<RotaryEncoder>,
        Option<HeartbeatLED>,
        Option<
            RGBMatrix<
                { Self::RGB_MATRIX_LED_COUNT },
                Ws2812Direct<
                    pac::PIO0,
                    pio::SM0,
                    gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionPio0, gpio::PullDown>,
                >,
            >,
        >,
    ) {
        #[rustfmt::skip]
        let key_matrix = if ENABLE_KEY_MATRIX {
            Some(BasicVerticalSwitchMatrix::new(
                [
                    Box::new(pins.gpio24.into_pull_down_input()),
                    Box::new(pins.gpio23.into_pull_down_input()),
                    Box::new(pins.gpio22.into_pull_down_input()),
                    Box::new(pins.gpio21.into_pull_down_input()),
                    Box::new(pins.gpio20.into_pull_down_input()),
                ],
                [
                    Box::new(pins.gpio0.into_push_pull_output()),
                    Box::new(pins.gpio1.into_push_pull_output()),
                    Box::new(pins.gpio2.into_push_pull_output()),
                    Box::new(pins.gpio3.into_push_pull_output()),
                    Box::new(pins.gpio4.into_push_pull_output()),
                    Box::new(pins.gpio5.into_push_pull_output()),
                    Box::new(pins.gpio6.into_push_pull_output()),
                    Box::new(pins.gpio7.into_push_pull_output()),
                    Box::new(pins.gpio8.into_push_pull_output()),
                    Box::new(pins.gpio9.into_push_pull_output()),
                    Box::new(pins.gpio10.into_push_pull_output()),
                    Box::new(pins.gpio11.into_push_pull_output()),
                    Box::new(pins.gpio12.into_push_pull_output()),
                    Box::new(pins.gpio13.into_push_pull_output()),
                    Box::new(pins.gpio14.into_push_pull_output()),
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
            let ws = ws2812_pio::Ws2812Direct::new(
                pins.gpio28.into_function(),
                &mut pio0,
                sm0,
                clock_freq,
            );
            Some(RGBMatrix::<{ Keyboard::RGB_MATRIX_LED_COUNT }, _>::new(ws))
        } else {
            None
        };

        (key_matrix, rotary_encoder, heartbeat_led, rgb_matrix)
    }
}
