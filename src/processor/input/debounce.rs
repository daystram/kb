use defmt::Format;
use rtic_monotonics::Monotonic;

use crate::{
    kb::Mono,
    key::Edge,
    processor::{mapper::Input, InputProcessor, Result},
};

pub struct KeyMatrixRisingFallingDebounceProcessor<
    const KEY_MATRIX_ROW_COUNT: usize,
    const KEY_MATRIX_COL_COUNT: usize,
> {
    delay: <Mono as Monotonic>::Duration,
    previous_states: [[State; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT],
}

#[derive(Clone, Copy, Debug, Format)]
struct State {
    pressed_ticks: u64,
    pressed: bool,
}

#[allow(dead_code)]
impl<const KEY_MATRIX_ROW_COUNT: usize, const KEY_MATRIX_COL_COUNT: usize>
    KeyMatrixRisingFallingDebounceProcessor<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>
{
    pub fn new(delay: <Mono as Monotonic>::Duration) -> Self {
        return KeyMatrixRisingFallingDebounceProcessor {
            delay,
            previous_states: [[State {
                pressed_ticks: 0,
                pressed: false,
            }; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT],
        };
    }
}

impl<const KEY_MATRIX_ROW_COUNT: usize, const KEY_MATRIX_COL_COUNT: usize>
    InputProcessor<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>
    for KeyMatrixRisingFallingDebounceProcessor<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>
{
    fn process(&mut self, input: &mut Input<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>) -> Result {
        for (i, row) in input.key_matrix_result.matrix.iter_mut().enumerate() {
            for (j, bit) in row.iter_mut().enumerate() {
                let previous_state = &mut self.previous_states[i][j];
                if input.key_matrix_result.scan_time_ticks - previous_state.pressed_ticks
                    <= self.delay.ticks()
                {
                    // ignore change
                    if bit.pressed != previous_state.pressed {
                        bit.edge = Edge::None;
                        bit.pressed = previous_state.pressed;
                    }
                } else {
                    if bit.edge == Edge::Rising || bit.edge == Edge::Falling {
                        // update previous_state
                        *previous_state = State {
                            pressed_ticks: input.key_matrix_result.scan_time_ticks,
                            pressed: bit.pressed,
                        };
                    }
                }
            }
        }
        return Ok(());
    }
}
