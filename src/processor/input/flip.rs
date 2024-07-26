use defmt::Format;

use crate::{
    key::Edge,
    matrix::Bit,
    processor::{mapper::Input, InputProcessor, Result},
};

pub struct ConcurrentFlipProcessor {
    key1_pos: Pos,
    key2_pos: Pos,
    previous_pressed_state: (bool, bool),
}

#[derive(Clone, Copy, Debug, Format)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

#[allow(dead_code)]
impl ConcurrentFlipProcessor {
    pub fn new(key1_pos: Pos, key2_pos: Pos) -> Self {
        ConcurrentFlipProcessor {
            key1_pos,
            key2_pos,
            previous_pressed_state: (false, false),
        }
    }
}

impl<const KEY_MATRIX_ROW_COUNT: usize, const KEY_MATRIX_COL_COUNT: usize>
    InputProcessor<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT> for ConcurrentFlipProcessor
{
    fn process(&mut self, input: &mut Input<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>) -> Result {
        let key1_bit = input.key_matrix_result.matrix[self.key1_pos.row][self.key1_pos.col];
        let key2_bit = input.key_matrix_result.matrix[self.key2_pos.row][self.key2_pos.col];

        let pressed_state = if key1_bit.edge == Edge::Rising {
            (true, false)
        } else if key1_bit.edge == Edge::Falling {
            if key2_bit.pressed {
                (false, true)
            } else {
                (false, false)
            }
        } else if key2_bit.edge == Edge::Rising {
            (false, true)
        } else if key2_bit.edge == Edge::Falling {
            if key1_bit.pressed {
                (true, false)
            } else {
                (false, false)
            }
        } else if !key1_bit.pressed && !key2_bit.pressed {
            (false, false) // reset to idle
        } else {
            self.previous_pressed_state // no edge changes, maintain pressed state
        };

        input.key_matrix_result.matrix[self.key1_pos.row][self.key1_pos.col] = Bit {
            edge: Edge::from((self.previous_pressed_state.0, pressed_state.0)),
            pressed: pressed_state.0,
        };
        input.key_matrix_result.matrix[self.key2_pos.row][self.key2_pos.col] = Bit {
            edge: Edge::from((self.previous_pressed_state.1, pressed_state.1)),
            pressed: pressed_state.1,
        };

        self.previous_pressed_state = pressed_state;
        Ok(())
    }
}
