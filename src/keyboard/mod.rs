use crate::processor::mapper::InputMap;

#[cfg(keyboard = "default")]
mod default;
#[cfg(keyboard = "default")]
use default as selected_keyboard;

#[cfg(keyboard = "kb_dev")]
mod kb_dev;
#[cfg(keyboard = "kb_dev")]
use kb_dev as selected_keyboard;

// =========== Heartbeat LED

pub const ENABLE_HEARTBEAT_LED: bool = selected_keyboard::ENABLE_HEARTBEAT_LED;

// =========== Layer

pub const LAYER_COUNT: usize = selected_keyboard::LAYER_COUNT;

pub use selected_keyboard::Layer;

// =========== Key Matrix

pub const ENABLE_KEY_MATRIX: bool = selected_keyboard::ENABLE_KEY_MATRIX;

pub const KEY_MATRIX_COL_COUNT: usize = selected_keyboard::KEY_MATRIX_COL_COUNT;
pub const KEY_MATRIX_ROW_COUNT: usize = selected_keyboard::KEY_MATRIX_ROW_COUNT;

// =========== Rotary Encoder

pub const ENABLE_ROTARY_ENCODER: bool = selected_keyboard::ENABLE_ROTARY_ENCODER;

// =========== RGB Matrix

pub const ENABLE_RGB_MATRIX: bool = selected_keyboard::ENABLE_RGB_MATRIX;

pub const RGB_MATRIX_LED_COUNT: usize = selected_keyboard::RGB_MATRIX_LED_COUNT;

// =========== Input Map

pub fn get_input_map(
) -> InputMap<{ LAYER_COUNT }, { KEY_MATRIX_ROW_COUNT }, { KEY_MATRIX_COL_COUNT }, Layer> {
    return selected_keyboard::layout::get_input_map();
}
