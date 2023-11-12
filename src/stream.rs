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

use usbd_human_interface_device::page::Keyboard;

use crate::{
    hid::Keys,
    key::{Action, LayerIndex},
    matrix::Bitmap,
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

pub trait BitmapProcessor<const ROW_COUNT: usize, const COL_COUNT: usize> {
    fn process(&mut self, bitmap: &mut Bitmap<ROW_COUNT, COL_COUNT>) -> Result;
}

pub trait Mapper<const ROW_COUNT: usize, const COL_COUNT: usize> {
    fn map(&self, bitmap: &Bitmap<ROW_COUNT, COL_COUNT>, keys: &mut Keys);
}

pub struct KeysMapper<
    const ROW_COUNT: usize,
    const COL_COUNT: usize,
    const LAYER_COUNT: usize,
    L: LayerIndex,
> {
    mapping: Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>,
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    KeysMapper<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    pub fn new(mapping: Mapping<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>) -> Self {
        let mut m = mapping.clone();
        m.flatten();
        return KeysMapper { mapping: m };
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize, const LAYER_COUNT: usize, L: LayerIndex>
    Mapper<ROW_COUNT, COL_COUNT> for KeysMapper<ROW_COUNT, COL_COUNT, LAYER_COUNT, L>
{
    fn map(&self, bitmap: &Bitmap<ROW_COUNT, COL_COUNT>, keys: &mut Keys) {
        // collect positions
        let mut positions = Vec::new();
        for (i, row) in bitmap.iter().enumerate() {
            for (j, pressed) in row.iter().enumerate() {
                if *pressed {
                    positions.push((i, j));
                };
            }
        }

        // resolve keys
        let mut provisional_keys = Keys::with_capacity(positions.len());
        let mut new_layer = true;
        let mut layer_idx = 0;
        while new_layer {
            provisional_keys.clear();
            new_layer = false;
            for (i, j) in &positions {
                match self.mapping[layer_idx][*i][*j] {
                    Action::Key(k) => provisional_keys.push(k.into()),
                    Action::LayerModifier(l) => {
                        if layer_idx < l.into() {
                            new_layer = true;
                            layer_idx = l.into();
                        }
                    }
                    _ => {}
                }
            }
        }
        keys.append(&mut provisional_keys);
    }
}

pub trait KeysProcessor {
    fn process(&mut self, keys: &mut Keys) -> Result;
}

pub struct KeyReplacer {
    from: Keyboard,
    to: Keyboard,
}

#[allow(dead_code)]
impl KeyReplacer {
    pub fn new(from: Keyboard, to: Keyboard) -> Self {
        return KeyReplacer { from, to };
    }
}

impl KeysProcessor for KeyReplacer {
    fn process(&mut self, keys: &mut Keys) -> Result {
        keys.iter_mut().for_each(|k| {
            if *k == self.from {
                *k = self.to
            }
        });
        return Ok(());
    }
}

type Result = result::Result<(), Error>;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

#[allow(dead_code)]
impl Error {
    fn new(msg: &str) -> Self {
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
