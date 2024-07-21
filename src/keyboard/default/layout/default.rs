#![allow(dead_code)]
use enum_map::enum_map;

use crate::{
    key::{
        Action::{Key as K, LayerModifier as LM, Pass as _________},
        Key,
    },
    keyboard::{Layer, KEY_MATRIX_COL_COUNT, KEY_MATRIX_ROW_COUNT, LAYER_COUNT},
    processor::mapper::InputMap,
};

#[rustfmt::skip]
pub fn get_input_map(
) -> InputMap<{ LAYER_COUNT }, { KEY_MATRIX_ROW_COUNT }, { KEY_MATRIX_COL_COUNT }, Layer> {
    return InputMap::new(
        [
            [
                [K(Key::A), K(Key::B)],
                [_________, LM(Layer::Function1)],
            ],
            [
                [K(Key::C), K(Key::D)],
                [_________, _________],
            ],
        ],
        [
            enum_map! {
                _ => _________,
            },
            enum_map! {
                _ => _________,
            },
        ],
    );
}
