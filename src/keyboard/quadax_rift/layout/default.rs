#![allow(dead_code)]
use defmt::Format;
use enum_map::enum_map;

use crate::{
    key::{
        Action::{Control as C, Key as K, LayerModifier as LM, Pass as ___________},
        Control, Key, LayerIndex,
    },
    keyboard::Configurator,
    processor::mapper::InputMap,
    rotary::Direction,
};

pub const LAYER_COUNT: usize = 3;

#[derive(Clone, Copy, Default, Format, PartialEq, PartialOrd)]
pub enum Layer {
    #[default]
    Base,
    Down,
    Up,
}

impl LayerIndex for Layer {}

impl From<Layer> for usize {
    fn from(value: Layer) -> usize {
        value as usize
    }
}

#[rustfmt::skip]
pub fn get_input_map() -> InputMap<{ <super::super::Keyboard as Configurator>::LAYER_COUNT }, { <super::super::Keyboard as Configurator>::KEY_MATRIX_ROW_COUNT }, { <super::super::Keyboard as Configurator>::KEY_MATRIX_COL_COUNT }, <super::super::Keyboard as Configurator>::Layer> {
    InputMap::new(
        [
            [
                [K(Key::Escape),        K(Key::Keyboard1),     K(Key::Keyboard2),     K(Key::Keyboard3),     K(Key::Keyboard4),     K(Key::Keyboard5),     ___________,           ___________,           K(Key::Keyboard6),     K(Key::Keyboard7),     K(Key::Keyboard8),     K(Key::Keyboard9),     K(Key::Keyboard0),     K(Key::DeleteBackspace)],
                [K(Key::Tab),           K(Key::Q),             K(Key::W),             K(Key::E),             K(Key::R),             K(Key::T),             ___________,           ___________,           K(Key::Y),             K(Key::U),             K(Key::I),             K(Key::O),             K(Key::P),             K(Key::DeleteForward)],
                [K(Key::CapsLock),      K(Key::A),             K(Key::S),             K(Key::D),             K(Key::F),             K(Key::G),             ___________,           ___________,           K(Key::H),             K(Key::J),             K(Key::K),             K(Key::L),             K(Key::Semicolon),     K(Key::ReturnEnter)],
                [K(Key::LeftShift),     K(Key::Z),             K(Key::X),             K(Key::C),             K(Key::V),             K(Key::B),             ___________,           ___________,           K(Key::N),             K(Key::M),             K(Key::Comma),         K(Key::Dot),           K(Key::ForwardSlash),  K(Key::RightShift)],
                [___________,           K(Key::LeftControl),   K(Key::LeftAlt),       K(Key::LeftGUI),       LM(Layer::Down),       K(Key::Space),         ___________,           ___________,           K(Key::Space),         LM(Layer::Up),         K(Key::RightGUI),      K(Key::RightAlt),      K(Key::RightControl),  ___________],
            ],
            [
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
                [___________,           C(Control::RGBAnimationNext), C(Control::RGBSpeedUp), C(Control::RGBBrightnessUp),           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
                [___________,           C(Control::RGBAnimationPrevious), C(Control::RGBSpeedDown), C(Control::RGBBrightnessDown),           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
            ],
            [
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::LeftArrow),     K(Key::DownArrow),     K(Key::UpArrow),       K(Key::RightArrow),    ___________],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________],
            ],
        ],
        [
            enum_map! {
                Direction::Clockwise => K(Key::VolumeUp),
                Direction::CounterClockwise => K(Key::VolumeDown),
                _ => ___________,
            },
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
