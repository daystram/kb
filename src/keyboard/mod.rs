use alloc::rc::Rc;
use core::{cell::RefCell, mem};
use hal::{fugit::HertzU32, gpio, pac, pio, pwm};
use rtic_sync::arbiter::Arbiter;
use ssd1306::prelude::I2CInterface;
use ws2812_pio::Ws2812Direct;

use crate::{
    heartbeat::HeartbeatLED,
    key::LayerIndex,
    matrix::{BasicVerticalSwitchMatrix, SplitSwitchMatrix},
    oled::OLEDDisplay,
    processor::{events::rgb::RGBMatrix, mapper::InputMap},
    remote::transport::uart::{UartReceiver, UartSender},
    rotary::RotaryEncoder,
    status::StatusLED,
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

#[cfg(keyboard = "quadax_rift")]
mod quadax_rift;
#[cfg(keyboard = "quadax_rift")]
use quadax_rift as selected_keyboard;

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
    // TODO: configurable OLED display pinout
    pub oled_display: Option<
        OLEDDisplay<
            I2CInterface<
                hal::I2C<
                    pac::I2C1,
                    (
                        gpio::Pin<gpio::bank0::Gpio26, gpio::FunctionI2c, gpio::PullUp>,
                        gpio::Pin<gpio::bank0::Gpio27, gpio::FunctionI2c, gpio::PullUp>,
                    ),
                >,
            >,
        >,
    >,
    pub status_led: Option<StatusLED>,
}

impl Configuration {
    pub fn is_split(&self) -> bool {
        self.key_matrix_split.is_some()
    }
}

pub trait Configurator {
    const NAME: &str;

    type Layer: LayerIndex = selected_keyboard::layout::Layer;
    const LAYER_COUNT: usize = mem::variant_count::<Self::Layer>();

    const KEY_MATRIX_ROW_COUNT: usize;
    const KEY_MATRIX_COL_COUNT: usize;

    const RGB_MATRIX_LED_COUNT: usize;

    fn init(
        pins: gpio::Pins,
        slices: pwm::Slices,
        pio0: pio::PIO<pac::PIO0>,
        sm0: pio::UninitStateMachine<(pac::PIO0, pio::SM0)>,
        i2c1: pac::I2C1,
        uart0: pac::UART0,
        resets: &mut pac::RESETS,
        clock_freq: HertzU32,
        system_clock: &hal::clocks::SystemClock,
    ) -> (
        Configuration,
        Option<(Arbiter<Rc<RefCell<UartSender>>>, UartReceiver)>,
    );

    fn get_input_map() -> InputMap<
        { <selected_keyboard::Keyboard as Configurator>::LAYER_COUNT },
        { <selected_keyboard::Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
        { <selected_keyboard::Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
        selected_keyboard::layout::Layer,
    > {
        selected_keyboard::layout::get_input_map()
    }
}
