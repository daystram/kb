extern crate alloc;
use alloc::vec::Vec;

use crate::{
    key::LayerIndex,
    processor::{Event, EventsProcessor, Result},
};

pub struct NoneProcessor {}

#[allow(dead_code)]
impl NoneProcessor {
    pub fn new() -> Self {
        return NoneProcessor {};
    }
}

impl<L: LayerIndex> EventsProcessor<L> for NoneProcessor {
    fn process(&mut self, _: &mut Vec<Event<L>>) -> Result {
        return Ok(());
    }
}
