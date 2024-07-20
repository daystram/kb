use alloc::vec::Vec;

use crate::{
    key::{Action, Edge, LayerIndex},
    matrix::Result as MatrixResult,
};

use super::Event;

pub struct Input<const KEY_MATRIX_ROW_COUNT: usize, const KEY_MATRIX_COL_COUNT: usize> {
    pub key_matrix_result: MatrixResult<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>,
}

pub struct InputMap<
    const LAYER_COUNT: usize,
    const KEY_MATRIX_ROW_COUNT: usize,
    const KEY_MATRIX_COL_COUNT: usize,
    L: LayerIndex,
> {
    key_matrix: [[[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]; LAYER_COUNT],
}

impl<
        const LAYER_COUNT: usize,
        const KEY_MATRIX_ROW_COUNT: usize,
        const KEY_MATRIX_COL_COUNT: usize,
        L: LayerIndex,
    > InputMap<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>
{
    pub const fn new(
        key_matrix: [[[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]; LAYER_COUNT],
    ) -> Self {
        return InputMap { key_matrix };
    }
}

pub struct Mapper<
    const LAYER_COUNT: usize,
    const KEY_MATRIX_ROW_COUNT: usize,
    const KEY_MATRIX_COL_COUNT: usize,
    L: LayerIndex,
> {
    previous_key_matrix_result: MatrixResult<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>,
    mapping: InputMap<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>,
}

impl<
        const LAYER_COUNT: usize,
        const KEY_MATRIX_ROW_COUNT: usize,
        const KEY_MATRIX_COL_COUNT: usize,
        L: LayerIndex,
    > Mapper<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>
{
    pub fn new(
        mapping: InputMap<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>,
    ) -> Self {
        return Mapper {
            previous_key_matrix_result: MatrixResult::default(),
            mapping,
        };
    }
}

impl<
        const LAYER_COUNT: usize,
        const KEY_MATRIX_ROW_COUNT: usize,
        const KEY_MATRIX_COL_COUNT: usize,
        L: LayerIndex,
    > Mapper<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>
{
    pub fn map(
        &mut self,
        input: &Input<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>,
        events: &mut Vec<Event<L>>,
    ) {
        // map key matrix
        let mut provisional_events = Vec::<Event<L>>::with_capacity(10);
        let mut new_layer = true;
        let mut layer_idx = 0;
        while new_layer {
            provisional_events.clear();
            new_layer = false;
            for (i, row) in input.key_matrix_result.matrix.iter().enumerate() {
                for (j, bit) in row.iter().enumerate() {
                    let action = self.mapping.key_matrix[layer_idx][i][j];
                    if bit.pressed {
                        if let Action::LayerModifier(l) = action {
                            if layer_idx < l.into() {
                                new_layer = true;
                                layer_idx = l.into();
                                break; // repeat resolving on the next layer
                            }
                        }
                    }
                    // push non-idling event
                    if !(bit.edge == Edge::None && !bit.pressed) {
                        provisional_events.push(Event {
                            time_ticks: input.key_matrix_result.scan_time_ticks,
                            i,
                            j,
                            edge: bit.edge,
                            action,
                        })
                    }
                }
            }
        }

        *events = provisional_events;
        self.previous_key_matrix_result = input.key_matrix_result;
    }
}
