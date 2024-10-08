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
    Function2,
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
                [K(Key::Escape), K(Key::Keyboard1), K(Key::Keyboard2), K(Key::Keyboard3), K(Key::Keyboard4), K(Key::Keyboard5), K(Key::Keyboard6), K(Key::Keyboard7), K(Key::Keyboard8), K(Key::Keyboard9), K(Key::Keyboard0), K(Key::Minus), K(Key::Equal), K(Key::DeleteBackspace), K(Key::DeleteForward)],
                [K(Key::Tab), K(Key::Q), K(Key::W), K(Key::E), K(Key::R), K(Key::T), K(Key::Y), K(Key::U), K(Key::I), K(Key::O), K(Key::P), K(Key::LeftBrace), K(Key::RightBrace), K(Key::Backslash), K(Key::Home)],
                [K(Key::CapsLock), K(Key::A), K(Key::S), K(Key::D), K(Key::F), K(Key::G), K(Key::H), K(Key::J), K(Key::K), K(Key::L), K(Key::Semicolon), K(Key::Apostrophe), ___________, K(Key::ReturnEnter), K(Key::PageUp)],
                [K(Key::LeftShift), K(Key::Z), K(Key::X), K(Key::C), K(Key::V), K(Key::B), K(Key::N), K(Key::M), K(Key::Comma), K(Key::Dot), K(Key::ForwardSlash), ___________, K(Key::RightShift), K(Key::UpArrow), K(Key::PageDown)],
                [K(Key::LeftControl), K(Key::LeftAlt), K(Key::LeftGUI), ___________, ___________, ___________, K(Key::Space), ___________, ___________, ___________, LM(Layer::Function1), K(Key::RightAlt), K(Key::LeftArrow), K(Key::DownArrow), K(Key::RightArrow)],
            ],
            Layer::Function1 => [
                [K(Key::Grave), K(Key::F1), K(Key::F2), K(Key::F3), K(Key::F4), K(Key::F5), K(Key::F6), K(Key::F7), K(Key::F8), K(Key::F9), K(Key::F10), K(Key::F11), K(Key::F12), ___________, ___________],
                [___________, C(Control::RGBAnimationNext), C(Control::RGBSpeedUp), C(Control::RGBBrightnessUp), ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________],
                [___________, C(Control::RGBAnimationPrevious), C(Control::RGBSpeedDown), C(Control::RGBBrightnessDown), ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________],
                [K(Key::LeftShift), ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, K(Key::RightShift), ___________, ___________],
                [K(Key::LeftControl), K(Key::LeftAlt), K(Key::LeftGUI), ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, LM(Layer::Function2), ___________, ___________, ___________],
            ],
            Layer::Function2 => [
                [___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________],
                [___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________],
                [___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________],
                [___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________],
                [___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________, ___________],
            ],
        },
        enum_map! {
            Layer::Base => enum_map! {
                Direction::Clockwise => K(Key::VolumeUp),
                Direction::CounterClockwise => K(Key::VolumeDown),
                _ => ___________,
            },
            Layer::Function1 => enum_map! {
                Direction::Clockwise => C(Control::RGBBrightnessUp),
                Direction::CounterClockwise => C(Control::RGBBrightnessDown),
                _ => ___________,
            },
            Layer::Function2 => enum_map! {
                Direction::Clockwise => C(Control::RGBSpeedUp),
                Direction::CounterClockwise => C(Control::RGBSpeedDown),
                _ => ___________,
            },
        },
    )
}
