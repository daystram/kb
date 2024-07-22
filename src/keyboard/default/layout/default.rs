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

#[derive(Clone, Copy, Default, Format, PartialEq, PartialOrd)]
pub enum Layer {
    #[default]
    Base,
    Function1,
}

impl LayerIndex for Layer {}

impl From<Layer> for usize {
    fn from(value: Layer) -> usize {
        value as usize
    }
}

#[rustfmt::skip]
pub fn get_input_map() -> InputMap<{ <super::super::Keyboard as KeyboardConfiguration>::LAYER_COUNT }, { <super::super::Keyboard as KeyboardConfiguration>::KEY_MATRIX_ROW_COUNT }, { <super::super::Keyboard as KeyboardConfiguration>::KEY_MATRIX_COL_COUNT }, <super::super::Keyboard as KeyboardConfiguration>::Layer> {
    InputMap::new(
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
    )
}
