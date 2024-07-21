pub mod events;
pub mod input;
pub mod mapper;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{error, fmt, result};
use mapper::Input;

use crate::{
    key::Edge,
    key::{Action, LayerIndex},
};

#[allow(dead_code)]
pub struct Event<L: LayerIndex> {
    pub time_ticks: u64,
    pub i: usize,
    pub j: usize,
    pub edge: Edge,
    pub action: Action<L>,
}

pub trait InputProcessor<const KEY_MATRIX_ROW_COUNT: usize, const KEY_MATRIX_COL_COUNT: usize> {
    fn process(&mut self, input: &mut Input<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>) -> Result;
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
        Error {
            msg: msg.to_string(),
        }
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
