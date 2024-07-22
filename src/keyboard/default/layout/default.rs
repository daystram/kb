#![allow(dead_code)]
use defmt::Format;
use enum_map::enum_map;

use crate::{
    key::{
        Action::{Control as C, Key as K, LayerModifier as LM, Pass as ___________},
        Control, Key, LayerIndex,
    },
    keyboard::KeyboardConfiguration,
    processor::mapper::InputMap,
    rotary::Direction,
};

pub const LAYER_COUNT: usize = 2;

#[derive(Clone, Copy, PartialEq, PartialOrd, Format)]
pub enum Layer {
    Base,
    Function1,
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

#[rustfmt::skip]
pub fn get_input_map() -> InputMap<{ <super::super::Keyboard as KeyboardConfiguration>::LAYER_COUNT }, { <super::super::Keyboard as KeyboardConfiguration>::KEY_MATRIX_ROW_COUNT }, { <super::super::Keyboard as KeyboardConfiguration>::KEY_MATRIX_COL_COUNT }, <super::super::Keyboard as KeyboardConfiguration>::Layer> {
    return InputMap::new(
        [
            [
                [K(Key::A),                    K(Key::B)],
                [___________,                  LM(Layer::Function1)],
            ],
            [
                [K(Key::C),                    K(Key::D)],
                [C(Control::RGBAnimationNext), ___________],
            ],
        ],
        [
            enum_map! {
                Direction::Clockwise => C(Control::RGBBrightnessUp),
                Direction::CounterClockwise => C(Control::RGBBrightnessDown),
                _ => ___________,
            },
            enum_map! {
                Direction::Clockwise => C(Control::RGBSpeedUp),
                Direction::CounterClockwise => C(Control::RGBSpeedDown),
                _ => ___________,
            },
        ],
    );
}
