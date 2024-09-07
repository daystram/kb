use defmt::Format;
use enum_map::{enum_map, Enum};

use crate::{
    key::{
        Action::{Control as C, Key as K, LayerModifier as LM, Pass as ___________},
        Control, Key, LayerIndex,
    },
    keyboard::Configurator,
    processor::mapper::InputMap,
    rotary::Direction,
};

#[derive(Clone, Copy, Default, Enum, Format, PartialEq, PartialOrd)]
pub enum Layer {
    #[default]
    Base,
    Function1,
}

impl LayerIndex for Layer {}

pub fn get_input_map() -> InputMap<
    { <super::super::Keyboard as Configurator>::LAYER_COUNT },
    { <super::super::Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT },
    { <super::super::Keyboard as Configurator>::KEY_MATRIX_COL_COUNT },
    <super::super::Keyboard as Configurator>::Layer,
> {
    #[rustfmt::skip]
    InputMap::new(
        enum_map! {
            Layer::Base => [
                [K(Key::A),                    K(Key::B)],
                [___________,                  LM(Layer::Function1)],
            ],
            Layer::Function1 => [
                [K(Key::C),                    K(Key::D)],
                [C(Control::RGBAnimationNext), ___________],
            ],
        },
        enum_map! {
            Layer::Base => enum_map! {
                Direction::Clockwise => C(Control::RGBBrightnessUp),
                Direction::CounterClockwise => C(Control::RGBBrightnessDown),
                _ => ___________,
            },
            Layer::Function1 => enum_map! {
                Direction::Clockwise => C(Control::RGBSpeedUp),
                Direction::CounterClockwise => C(Control::RGBSpeedDown),
                _ => ___________,
            },
        },
    )
}
