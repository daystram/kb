use defmt::Format;

use crate::{
    key::{
        Action::{Control as C, Key as K, LayerModifier as LM, Pass as ___},
        Control, Key, LayerIndex,
    },
    processor::mapper::InputMap,
};

pub const LAYER_COUNT: usize = 3;
pub const KEY_MATRIX_ROW_COUNT: usize = 5;
pub const KEY_MATRIX_COL_COUNT: usize = 15;

pub const LED_COUNT: usize = 67;

pub fn build_input_map(
) -> InputMap<{ LAYER_COUNT }, { KEY_MATRIX_ROW_COUNT }, { KEY_MATRIX_COL_COUNT }, Layer> {
    return InputMap::new([
        [
            [
                K(Key::Escape),
                K(Key::Keyboard1),
                K(Key::Keyboard2),
                K(Key::Keyboard3),
                K(Key::Keyboard4),
                K(Key::Keyboard5),
                K(Key::Keyboard6),
                K(Key::Keyboard7),
                K(Key::Keyboard8),
                K(Key::Keyboard9),
                K(Key::Keyboard0),
                K(Key::Minus),
                K(Key::Equal),
                K(Key::DeleteBackspace),
                K(Key::DeleteForward),
            ],
            [
                K(Key::Tab),
                K(Key::Q),
                K(Key::W),
                K(Key::E),
                K(Key::R),
                K(Key::T),
                K(Key::Y),
                K(Key::U),
                K(Key::I),
                K(Key::O),
                K(Key::P),
                K(Key::LeftBrace),
                K(Key::RightBrace),
                K(Key::Backslash),
                K(Key::Home),
            ],
            [
                K(Key::CapsLock),
                K(Key::A),
                K(Key::S),
                K(Key::D),
                K(Key::F),
                K(Key::G),
                K(Key::H),
                K(Key::J),
                K(Key::K),
                K(Key::L),
                K(Key::Semicolon),
                K(Key::Apostrophe),
                ___,
                K(Key::ReturnEnter),
                K(Key::PageUp),
            ],
            [
                K(Key::LeftShift),
                K(Key::Z),
                K(Key::X),
                K(Key::C),
                K(Key::V),
                K(Key::B),
                K(Key::N),
                K(Key::M),
                K(Key::Comma),
                K(Key::Dot),
                K(Key::ForwardSlash),
                ___,
                K(Key::RightShift),
                K(Key::UpArrow),
                K(Key::PageDown),
            ],
            [
                K(Key::LeftControl),
                K(Key::LeftAlt),
                K(Key::LeftGUI),
                ___,
                ___,
                ___,
                K(Key::Space),
                ___,
                ___,
                ___,
                LM(Layer::Function1),
                K(Key::RightAlt),
                K(Key::LeftArrow),
                K(Key::DownArrow),
                K(Key::RightArrow),
            ],
        ],
        [
            [
                K(Key::Grave),
                K(Key::F1),
                K(Key::F2),
                K(Key::F3),
                K(Key::F4),
                K(Key::F5),
                K(Key::F6),
                K(Key::F7),
                K(Key::F8),
                K(Key::F9),
                K(Key::F10),
                K(Key::F11),
                K(Key::F12),
                ___,
                ___,
            ],
            [
                ___,
                C(Control::RGBAnimationNext),
                C(Control::RGBSpeedUp),
                C(Control::RGBBrightnessUp),
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
            ],
            [
                ___,
                C(Control::RGBAnimationPrevious),
                C(Control::RGBSpeedDown),
                C(Control::RGBBrightnessDown),
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
            ],
            [
                K(Key::LeftShift),
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                K(Key::RightShift),
                ___,
                ___,
            ],
            [
                K(Key::LeftControl),
                K(Key::LeftAlt),
                K(Key::LeftGUI),
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                ___,
                LM(Layer::Function2),
                ___,
                ___,
                ___,
            ],
        ],
        [
            [
                ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___,
            ],
            [
                ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___,
            ],
            [
                ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___,
            ],
            [
                ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___,
            ],
            [
                ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___, ___,
            ],
        ],
    ]);
}

#[allow(dead_code)]
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
