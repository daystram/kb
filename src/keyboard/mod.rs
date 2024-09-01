use core::cell::RefCell;

use alloc::rc::Rc;
use hal::{fugit::HertzU32, gpio, pac, pio, pwm};
use rtic_sync::arbiter::Arbiter;
use ws2812_pio::Ws2812Direct;

use crate::{
    heartbeat::HeartbeatLED,
    key::LayerIndex,
    matrix::{BasicVerticalSwitchMatrix, SplitSwitchMatrix},
    processor::{events::rgb::RGBMatrix, mapper::InputMap},
    remote::transport::uart::{UartReceiver, UartSender},
    rotary::RotaryEncoder,
};

pub use selected_keyboard::Keyboard;

#[cfg(keyboard = "default")]
mod default;
#[cfg(keyboard = "default")]
use default as selected_keyboard;

#[cfg(keyboard = "kb_dev")]
mod kb_dev;
#[cfg(keyboard = "kb_dev")]
use kb_dev as selected_keyboard;

pub trait KeyboardConfiguration {
#[derive(Default)]
pub struct Configuration {
    pub key_matrix: Option<
        BasicVerticalSwitchMatrix<
            { selected_keyboard::Keyboard::KEY_MATRIX_ROW_COUNT },
            { selected_keyboard::Keyboard::KEY_MATRIX_COL_COUNT },
        >,
    >,
    pub key_matrix_split: Option<
        SplitSwitchMatrix<
            { selected_keyboard::Keyboard::KEY_MATRIX_ROW_COUNT },
            { selected_keyboard::Keyboard::KEY_MATRIX_COL_COUNT },
        >,
    >,
    pub rotary_encoder: Option<RotaryEncoder>,
    pub heartbeat_led: Option<HeartbeatLED>,
    // TODO: configurable RGB matrix pinout
    pub rgb_matrix: Option<
        RGBMatrix<
            { selected_keyboard::Keyboard::RGB_MATRIX_LED_COUNT },
            Ws2812Direct<
                pac::PIO0,
                pio::SM0,
                gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionPio0, gpio::PullDown>,
            >,
        >,
    >,
}

impl Configuration {
    pub fn is_split(&self) -> bool {
        self.key_matrix_split.is_some()
    }
}

pub trait Configurator {
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
        uart0: pac::UART0,
        resets: &mut pac::RESETS,
        clock_freq: HertzU32,
    ) -> (
        Configuration,
        Option<(Arbiter<Rc<RefCell<UartSender>>>, UartReceiver)>,
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
