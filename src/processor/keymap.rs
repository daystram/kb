use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::{
    key::{Action, LayerIndex},
    matrix::{Bitmap, Edge},
};

use super::Event;

#[derive(Clone, Copy)]
pub struct KeyMap<
    const ROW_COUNT: usize,
    const COL_COUNT: usize,
    const LAYER_COUNT: usize,
    L: LayerIndex,
>(pub [[[Action<L>; COL_COUNT]; ROW_COUNT]; LAYER_COUNT]);

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    KeyMap<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    fn flatten(&mut self) {
        // TODO: flatten passthrough
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex> Deref
    for KeyMap<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    type Target = [[[Action<L>; COL_COUNT]; ROW_COUNT]; LAYER_COUNT];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    DerefMut for KeyMap<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    Default for KeyMap<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    fn default() -> Self {
        KeyMap([[[Action::default(); COL_COUNT]; ROW_COUNT]; LAYER_COUNT])
    }
}

pub struct KeyMapper<
    const ROW_COUNT: usize,
    const COL_COUNT: usize,
    const LAYER_COUNT: usize,
    L: LayerIndex,
> {
    previous_bitmap: Bitmap<ROW_COUNT, COL_COUNT>,
    mapping: KeyMap<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>,
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    KeyMapper<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    pub fn new(mapping: KeyMap<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>) -> Self {
        let mut m = mapping.clone();
        m.flatten();
        return KeyMapper {
            previous_bitmap: Bitmap::default(),
            mapping: m,
        };
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    KeyMapper<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    pub fn map(&mut self, bitmap: &Bitmap<ROW_COUNT, COL_COUNT>, events: &mut Vec<Event<L>>) {
        let mut provisional_events = Vec::<Event<L>>::with_capacity(10);
        let mut new_layer = true;
        let mut layer_idx = 0;
        while new_layer {
            provisional_events.clear();
            new_layer = false;
            for (i, row) in bitmap.matrix.iter().enumerate() {
                for (j, bit) in row.iter().enumerate() {
                    let action = self.mapping[layer_idx][i][j];
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
                            time_ticks: bitmap.scan_time_ticks,
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
        self.previous_bitmap = *bitmap;
    }
}
