use alloc::vec::Vec;
use enum_map::{EnumArray, EnumMap};

use crate::{
    key::{Action, Edge, LayerIndex, Modifier},
    matrix::Result as MatrixResult,
    rotary::{Direction, Result as RotaryResult},
};

use super::Event;

pub struct Input<const KEY_MATRIX_ROW_COUNT: usize, const KEY_MATRIX_COL_COUNT: usize> {
    pub key_matrix_result: MatrixResult<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>,
    pub rotary_encoder_result: RotaryResult,
}

pub struct InputMap<
    const LAYER_COUNT: usize,
    const KEY_MATRIX_ROW_COUNT: usize,
    const KEY_MATRIX_COL_COUNT: usize,
    L: LayerIndex
        + EnumArray<[[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]>
        + EnumArray<EnumMap<Direction, Action<L>>>,
> {
    key_matrix: EnumMap<L, [[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]>,
    rotary_encoder: EnumMap<L, EnumMap<Direction, Action<L>>>,
}

impl<
        const LAYER_COUNT: usize,
        const KEY_MATRIX_ROW_COUNT: usize,
        const KEY_MATRIX_COL_COUNT: usize,
        L: LayerIndex
            + EnumArray<[[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]>
            + EnumArray<EnumMap<Direction, Action<L>>>,
    > InputMap<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>
{
    pub const fn new(
        key_matrix: EnumMap<L, [[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]>,
        rotary_encoder: EnumMap<L, EnumMap<Direction, Action<L>>>,
    ) -> Self {
        InputMap {
            key_matrix,
            rotary_encoder,
        }
    }
}

pub struct Mapper<
    const LAYER_COUNT: usize,
    const KEY_MATRIX_ROW_COUNT: usize,
    const KEY_MATRIX_COL_COUNT: usize,
    L: LayerIndex
        + EnumArray<[[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]>
        + EnumArray<EnumMap<Direction, Action<L>>>,
> {
    previous_key_matrix_result: MatrixResult<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>,
    mapping: InputMap<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>,
}

impl<
        const LAYER_COUNT: usize,
        const KEY_MATRIX_ROW_COUNT: usize,
        const KEY_MATRIX_COL_COUNT: usize,
        L: LayerIndex
            + EnumArray<[[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]>
            + EnumArray<EnumMap<Direction, Action<L>>>,
    > Mapper<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>
{
    pub fn new(
        mapping: InputMap<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>,
    ) -> Self {
        Mapper {
            previous_key_matrix_result: MatrixResult::default(),
            mapping,
        }
    }
}

impl<
        const LAYER_COUNT: usize,
        const KEY_MATRIX_ROW_COUNT: usize,
        const KEY_MATRIX_COL_COUNT: usize,
        L: LayerIndex
            + EnumArray<[[Action<L>; KEY_MATRIX_COL_COUNT]; KEY_MATRIX_ROW_COUNT]>
            + EnumArray<EnumMap<Direction, Action<L>>>,
    > Mapper<LAYER_COUNT, KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT, L>
{
    pub fn map(
        &mut self,
        input: &Input<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>,
        events: &mut Vec<Event<L>>,
    ) {
        // map key matrix
        let result = input.key_matrix_result;
        let mut provisional_events = Vec::<Event<L>>::with_capacity(10);
        let mut new_layer = true;
        let mut layer = L::default();
        while new_layer {
            provisional_events.clear();
            new_layer = false;
            for (i, row) in result.matrix.iter().enumerate() {
                for (j, bit) in row.iter().enumerate() {
                    let action = self.mapping.key_matrix[layer][i][j];
                    if bit.pressed {
                        if let Action::LayerModifier(l) = action {
                            if layer < l {
                                new_layer = true;
                                layer = l;
                                break; // repeat resolving on the next layer
                            }
                        }
                    }
                    // push non-idling event
                    #[allow(clippy::nonminimal_bool)]
                    if !(bit.edge == Edge::None && !bit.pressed) {
                        // resolve modified key modifiers
                        if let Action::ModifiedKey(mk) = action {
                            mk.get_modifiers()
                                .iter()
                                .filter(|&&m: &&Modifier| m != Default::default())
                                .for_each(|&m| {
                                    provisional_events.push(Event {
                                        time_ticks: result.scan_time_ticks,
                                        i,
                                        j,
                                        edge: bit.edge,
                                        action: m.into(),
                                    })
                                });
                            provisional_events.push(Event {
                                time_ticks: result.scan_time_ticks,
                                i,
                                j,
                                edge: bit.edge,
                                action: mk.get_key().into(),
                            });
                            continue;
                        }

                        provisional_events.push(Event {
                            time_ticks: result.scan_time_ticks,
                            i,
                            j,
                            edge: bit.edge,
                            action,
                        })
                    }
                }
            }
        }

        // map rotary encoder
        let result = input.rotary_encoder_result;
        if !(result.edge == Edge::None && result.direction == Direction::None) {
            provisional_events.push(Event {
                time_ticks: result.scan_time_ticks,
                i: 0,
                j: 0,
                edge: result.edge,
                action: self.mapping.rotary_encoder[layer][result.direction],
            });
        }

        *events = provisional_events;
        self.previous_key_matrix_result = input.key_matrix_result;
    }
}
