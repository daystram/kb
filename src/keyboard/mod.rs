use hal::{fugit::HertzU32, gpio, pac, pio, pwm};
use ws2812_pio::Ws2812Direct;

use crate::{
    heartbeat::HeartbeatLED,
    key::LayerIndex,
    matrix::BasicVerticalSwitchMatrix,
    processor::{events::rgb::RGBMatrix, mapper::InputMap},
    rotary::RotaryEncoder,
};

#[cfg(keyboard = "default")]
mod default;
#[cfg(keyboard = "default")]
use default as selected_keyboard;

#[cfg(keyboard = "kb_dev")]
mod kb_dev;
#[cfg(keyboard = "kb_dev")]
use kb_dev as selected_keyboard;

pub trait KeyboardConfiguration {
    const LAYER_COUNT: usize = selected_keyboard::layout::LAYER_COUNT;
    type Layer: LayerIndex = selected_keyboard::layout::Layer;

    const KEY_MATRIX_ROW_COUNT: usize;
    const KEY_MATRIX_COL_COUNT: usize;

    const RGB_MATRIX_LED_COUNT: usize;

    fn init(
        pins: gpio::Pins,
        slices: pwm::Slices,
        pio0: pio::PIO<pac::PIO0>,
        sm0: pio::UninitStateMachine<(pac::PIO0, pio::SM0)>,
        clock_freq: HertzU32,
    ) -> (
        Option<
            BasicVerticalSwitchMatrix<
                { selected_keyboard::Keyboard::KEY_MATRIX_ROW_COUNT },
                { selected_keyboard::Keyboard::KEY_MATRIX_COL_COUNT },
            >,
        >,
        Option<RotaryEncoder>,
        Option<HeartbeatLED>,
        // TODO: configurable RGB matrix pinout
        Option<
            RGBMatrix<
                { selected_keyboard::Keyboard::RGB_MATRIX_LED_COUNT },
                Ws2812Direct<
                    pac::PIO0,
                    pio::SM0,
                    gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionPio0, gpio::PullDown>,
                >,
            >,
        >,
    );

    fn get_input_map() -> InputMap<
        { selected_keyboard::layout::LAYER_COUNT },
        { selected_keyboard::Keyboard::KEY_MATRIX_ROW_COUNT },
        { selected_keyboard::Keyboard::KEY_MATRIX_COL_COUNT },
        selected_keyboard::layout::Layer,
    > {
        selected_keyboard::layout::get_input_map()
    }
}

pub use selected_keyboard::Keyboard;
