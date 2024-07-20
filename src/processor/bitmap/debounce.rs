use defmt::Format;
use rtic_monotonics::Monotonic;

use crate::{
    kb::Mono,
    matrix::{Bitmap, Edge},
    processor::{BitmapProcessor, Result},
};

pub struct KeyRisingFallingDebounceProcessor<const ROW_COUNT: usize, const COL_COUNT: usize> {
    delay: <Mono as Monotonic>::Duration,
    previous_states: [[State; COL_COUNT]; ROW_COUNT],
}

#[derive(Clone, Copy, Debug, Format)]
struct State {
    pressed_ticks: u64,
    pressed: bool,
}

#[allow(dead_code)]
impl<const ROW_COUNT: usize, const COL_COUNT: usize>
    KeyRisingFallingDebounceProcessor<ROW_COUNT, COL_COUNT>
{
    pub fn new(delay: <Mono as Monotonic>::Duration) -> Self {
        return KeyRisingFallingDebounceProcessor {
            delay,
            previous_states: [[State {
                pressed_ticks: 0,
                pressed: false,
            }; COL_COUNT]; ROW_COUNT],
        };
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> BitmapProcessor<ROW_COUNT, COL_COUNT>
    for KeyRisingFallingDebounceProcessor<ROW_COUNT, COL_COUNT>
{
    fn process(&mut self, bitmap: &mut Bitmap<ROW_COUNT, COL_COUNT>) -> Result {
        for (i, row) in bitmap.matrix.iter_mut().enumerate() {
            for (j, bit) in row.iter_mut().enumerate() {
                let previous_state = &mut self.previous_states[i][j];
                if bitmap.scan_time_ticks - previous_state.pressed_ticks <= self.delay.ticks() {
                    // ignore change
                    if bit.pressed != previous_state.pressed {
                        bit.edge = Edge::None;
                        bit.pressed = previous_state.pressed;
                    }
                } else {
                    if bit.edge == Edge::Rising || bit.edge == Edge::Falling {
                        // update previous_state
                        *previous_state = State {
                            pressed_ticks: bitmap.scan_time_ticks,
                            pressed: bit.pressed,
                        };
                    }
                }
            }
        }
        return Ok(());
    }
}
