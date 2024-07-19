pub mod bitmap;
pub mod events;
pub mod keymap;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{error, fmt, result};

use crate::{
    key::{Action, LayerIndex},
    matrix::{Bitmap, Edge},
};

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

pub trait EventsProcessor<L: LayerIndex> {
    fn process(&mut self, events: &mut Vec<Event<L>>) -> Result;
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
