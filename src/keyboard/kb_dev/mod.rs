pub mod layout;

use defmt::Format;

use crate::key::LayerIndex;

// =========== Heartbeat LED

pub const ENABLE_HEARTBEAT_LED: bool = true;

// =========== Layer

pub const LAYER_COUNT: usize = 3;

#[derive(Clone, Copy, PartialEq, PartialOrd, Format)]
pub enum Layer {
    Base,
    Function1,
    Function2,
}

impl LayerIndex for Layer {}

impl Into<usize> for Layer {
    fn into(self) -> usize {
        self as usize
    }
}

impl Default for Layer {
    fn default() -> Self {
        return Layer::Base;
    }
}

// =========== Key Matrix

pub const ENABLE_KEY_MATRIX: bool = true;

pub const KEY_MATRIX_ROW_COUNT: usize = 5;
pub const KEY_MATRIX_COL_COUNT: usize = 15;

// =========== Rotary Encoder

pub const ENABLE_ROTARY_ENCODER: bool = true;

// =========== RGB Matrix

pub const ENABLE_RGB_MATRIX: bool = true;

pub const RGB_MATRIX_LED_COUNT: usize = 67;
