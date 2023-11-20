extern crate alloc;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{
    error, fmt,
    ops::{Deref, DerefMut},
    result,
};

use crate::{
    key::{Action, Key, LayerIndex},
    matrix::{Bitmap, Edge},
};

#[derive(Clone, Copy)]
pub struct Mapping<
    const ROW_COUNT: usize,
    const COL_COUNT: usize,
    const LAYER_COUNT: usize,
    L: LayerIndex,
>(pub [[[Action<L>; COL_COUNT]; ROW_COUNT]; LAYER_COUNT]);

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    fn flatten(&mut self) {
        // TODO: flatten passthrough
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex> Deref
    for Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    type Target = [[[Action<L>; COL_COUNT]; ROW_COUNT]; LAYER_COUNT];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    DerefMut for Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    Default for Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    fn default() -> Self {
        Mapping([[[Action::default(); COL_COUNT]; ROW_COUNT]; LAYER_COUNT])
    }
}

pub struct Event<L: LayerIndex> {
    pub time_ticks: u64,
    pub i: usize,
    pub j: usize,
    pub edge: Edge,
    pub action: Action<L>,
}

pub trait BitmapProcessor<const ROW_COUNT: usize, const COL_COUNT: usize> {
    fn process(&mut self, bitmap: &mut Bitmap<ROW_COUNT, COL_COUNT>) -> Result;
}

pub struct EventsMapper<
    const ROW_COUNT: usize,
    const COL_COUNT: usize,
    const LAYER_COUNT: usize,
    L: LayerIndex,
> {
    previous_bitmap: Bitmap<ROW_COUNT, COL_COUNT>,
    mapping: Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>,
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    EventsMapper<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    pub fn new(mapping: Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>) -> Self {
        let mut m = mapping.clone();
        m.flatten();
        return EventsMapper {
            previous_bitmap: Bitmap::default(),
            mapping: m,
        };
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    EventsMapper<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    pub fn map(&mut self, bitmap: &Bitmap<ROW_COUNT, COL_COUNT>, events: &mut Vec<Event<L>>) {
        let mut provisional_events = Vec::<Event<L>>::with_capacity(10);
        let mut new_layer = true;
        let mut layer_idx = 0;
        while new_layer {
            provisional_events.clear();
            new_layer = false;
            for (i, row) in bitmap.matrix.iter().enumerate() {
                for (j, (edge, pressed)) in row.iter().enumerate() {
                    let action = self.mapping[layer_idx][i][j];
                    if *pressed {
                        match action {
                            Action::LayerModifier(l) => {
                                if layer_idx < l.into() {
                                    new_layer = true;
                                    layer_idx = l.into();
                                    break; // repeat resolving on the next layer
                                }
                            }
                            _ => {}
                        }
                    }
                    if !(*edge == Edge::None && !*pressed) {
                        provisional_events.push(Event {
                            time_ticks: bitmap.sample_time_ticks,
                            i,
                            j,
                            edge: *edge,
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

pub trait EventsProcessor<L: LayerIndex> {
    async fn process(&mut self, events: &mut Vec<Event<L>>) -> Result;
}

pub struct KeyReplacer {
    from: Key,
    to: Key,
}

#[allow(dead_code)]
impl KeyReplacer {
    pub fn new(from: Key, to: Key) -> Self {
        return KeyReplacer { from, to };
    }
}

impl<L: LayerIndex> EventsProcessor<L> for KeyReplacer {
    async fn process(&mut self, events: &mut Vec<Event<L>>) -> Result {
        events.iter_mut().for_each(|e| match &mut e.action {
            Action::Key(k) => {
                if *k == self.from {
                    *k = self.to;
                }
            }
            _ => {}
        });
        return Ok(());
    }
}

pub type Result = result::Result<(), Error>;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

#[allow(dead_code)]
impl Error {
    pub fn new(msg: &str) -> Self {
        return Error {
            msg: msg.to_string(),
        };
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}
