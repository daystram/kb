use crate::{
    key::{
        Action::{Key as K, LayerModifier as LM, Pass as ___},
        Key, LayerIndex,
    },
    stream::Mapping,
};

pub const ROW_COUNT: usize = 5;
pub const COL_COUNT: usize = 15;
pub const LAYER_COUNT: usize = 3;

pub const KEY_MAP: Mapping<{ ROW_COUNT }, { COL_COUNT }, { LAYER_COUNT }, Layer> = Mapping([
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
            LM(Layer::Function1), // K(Key::PageUp),
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
            ___,
            ___,
            ___,
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
            ___,
            ___,
            K(Key::LeftGUI),
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

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
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
