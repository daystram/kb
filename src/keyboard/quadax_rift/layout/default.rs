use defmt::Format;
use enum_map::{enum_map, Enum};

use crate::{
    key::{
        Action::{
            Control as C, Key as K, LayerModifier as LM, ModifiedKey as MK, Pass as ___________,
        },
        Control, Key, LayerIndex, ModifiedKey, Modifier,
    },
    keyboard::Configurator,
    processor::mapper::InputMap,
    rotary::Direction,
};

#[derive(Clone, Copy, Default, Enum, Format, PartialEq, PartialOrd)]
pub enum Layer {
    #[default]
    Base,
    Symbol,
    Number,
    Navigation,
    System,
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
                [K(Key::Escape),        K(Key::Keyboard1),     K(Key::Keyboard2),     K(Key::Keyboard3),     K(Key::Keyboard4),     K(Key::Keyboard5),     ___________,           ___________,           K(Key::Keyboard6),     K(Key::Keyboard7),     K(Key::Keyboard8),     K(Key::Keyboard9),     K(Key::Keyboard0),     K(Key::DeleteBackspace)],
                [K(Key::Tab),           K(Key::Q),             K(Key::W),             K(Key::E),             K(Key::R),             K(Key::T),             ___________,           ___________,           K(Key::Y),             K(Key::U),             K(Key::I),             K(Key::O),             K(Key::P),             K(Key::DeleteForward)],
                [K(Key::LeftControl),   K(Key::A),             K(Key::S),             K(Key::D),             K(Key::F),             K(Key::G),             ___________,           ___________,           K(Key::H),             K(Key::J),             K(Key::K),             K(Key::L),             K(Key::Semicolon),     K(Key::ReturnEnter)],
                [K(Key::LeftShift),     K(Key::Z),             K(Key::X),             K(Key::C),             K(Key::V),             K(Key::B),             LM(Layer::Number),     ___________,           K(Key::N),             K(Key::M),             K(Key::Comma),         K(Key::Dot),           MK(LS!(Key::ForwardSlash)),K(Key::RightShift)],
                [LM(Layer::System),     K(Key::LeftControl),   K(Key::LeftAlt),       K(Key::LeftGUI),       LM(Layer::Symbol),     K(Key::Space),         ___________,           ___________,           K(Key::Space),         LM(Layer::Navigation), K(Key::RightGUI),      K(Key::RightAlt),      K(Key::RightControl),  ___________],
            ],
            Layer::Symbol => [
                [K(Key::Escape),        MK(LS!(Key::Keyboard1)),MK(LS!(Key::Keyboard2)),MK(LS!(Key::Keyboard3)),MK(LS!(Key::Keyboard4)),___________,       ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::DeleteBackspace)],
                [___________,           MK(LS!(Key::LeftBrace)),K(Key::LeftBrace),     K(Key::Apostrophe),   K(Key::RightBrace),    MK(LS!(Key::RightBrace)),___________,         ___________,           MK(LS!(Key::Comma)),   MK(LS!(Key::Dot)),     ___________,           ___________,           ___________,           ___________],
                [K(Key::LeftControl),   K(Key::Backslash),     MK(LS!(Key::Keyboard9)),MK(LS!(Key::Apostrophe)),MK(LS!(Key::Keyboard0)),K(Key::ForwardSlash),___________,         ___________,           MK(LS!(Key::Minus)),   MK(LS!(Key::Backslash)),MK(LS!(Key::Keyboard7)),MK(LS!(Key::Keyboard6)),K(Key::Equal),   ___________],
                [K(Key::LeftShift),     ___________,           MK(LS!(Key::Comma)),    K(Key::Grave),        MK(LS!(Key::Dot)),     ___________,           LM(Layer::Number),     ___________,           MK(LS!(Key::Equal)),   K(Key::Minus),         MK(LS!(Key::Keyboard8)),MK(LS!(Key::Grave)),  MK(LS!(Key::Keyboard5)),K(Key::RightShift)],
                [___________,           K(Key::LeftControl),   K(Key::LeftAlt),       K(Key::LeftGUI),       ___________,           K(Key::Space),         ___________,           ___________,           K(Key::Space),         ___________,           K(Key::RightGUI),      K(Key::RightAlt),      K(Key::RightControl),  ___________],
            ],
            Layer::Number => [
                [K(Key::Escape),        ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::DeleteBackspace)],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::Keyboard0),     K(Key::Keyboard1),     K(Key::Keyboard2),     K(Key::Keyboard3),     ___________,           ___________],
                [K(Key::LeftControl),   ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::Keyboard4),     K(Key::Keyboard5),     K(Key::Keyboard6),     ___________,           ___________],
                [K(Key::LeftShift),     ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::Keyboard7),     K(Key::Keyboard8),     K(Key::Keyboard9),     ___________,           K(Key::RightShift)],
                [___________,           K(Key::LeftControl),   K(Key::LeftAlt),       K(Key::LeftGUI),       ___________,           K(Key::Space),         ___________,           ___________,           K(Key::Space),         ___________,           K(Key::RightGUI),      K(Key::RightAlt),      K(Key::RightControl),  ___________],
            ],
            Layer::Navigation => [
                [K(Key::Escape),        ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::DeleteBackspace)],
                [___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::Home),          K(Key::PageDown),      K(Key::PageUp),        K(Key::End),           ___________,           ___________],
                [K(Key::LeftControl),   ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::LeftArrow),     K(Key::DownArrow),     K(Key::UpArrow),       K(Key::RightArrow),    ___________,           ___________],
                [K(Key::LeftShift),     ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::RightShift)],
                [___________,           K(Key::LeftControl),   K(Key::LeftAlt),       K(Key::LeftGUI),       ___________,           K(Key::Space),         ___________,           ___________,           K(Key::Space),         ___________,           K(Key::RightGUI),      K(Key::RightAlt),      K(Key::RightControl),  ___________],
            ],
            Layer::System => [
                [K(Key::Escape),        ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::DeleteBackspace)],
                [___________,           C(Control::RGBAnimationNext),C(Control::RGBSpeedUp),C(Control::RGBBrightnessUp),___________,___________,           ___________,           ___________,           K(Key::F10),           K(Key::F1),            K(Key::F2),            K(Key::F3),            ___________,           ___________],
                [K(Key::LeftControl),   C(Control::RGBAnimationPrevious),C(Control::RGBSpeedDown),C(Control::RGBBrightnessDown),___________,___________,   ___________,           ___________,           K(Key::F11),           K(Key::F4),            K(Key::F5),            K(Key::F6),            ___________,           ___________],
                [K(Key::LeftShift),     C(Control::U2FBootloaderJump),___________,    ___________,           ___________,           ___________,           ___________,           ___________,           K(Key::F12),           K(Key::F7),            K(Key::F8),            K(Key::F9),            ___________,           K(Key::RightShift)],
                [___________,           K(Key::LeftControl),   K(Key::LeftAlt),       K(Key::LeftGUI),       ___________,           K(Key::Space),         ___________,           ___________,           K(Key::Space),         ___________,           K(Key::RightGUI),      K(Key::RightAlt),      K(Key::RightControl),  ___________],
            ],
        },
        enum_map! {
            Layer::Base => enum_map! {
                Direction::Clockwise => K(Key::VolumeUp),
                Direction::CounterClockwise => K(Key::VolumeDown),
                _ => ___________,
            },
            Layer::Symbol => enum_map! {
                Direction::Clockwise => C(Control::RGBBrightnessUp),
                Direction::CounterClockwise => C(Control::RGBBrightnessDown),
                _ => ___________,
            },
            Layer::Number => enum_map! {
                Direction::Clockwise => C(Control::RGBBrightnessUp),
                Direction::CounterClockwise => C(Control::RGBBrightnessDown),
                _ => ___________,
            },
            Layer::Navigation => enum_map! {
                Direction::Clockwise => C(Control::RGBSpeedUp),
                Direction::CounterClockwise => C(Control::RGBSpeedDown),
                _ => ___________,
            },
            Layer::System => enum_map! {
                _ => ___________,
            },
        },
    )
}
