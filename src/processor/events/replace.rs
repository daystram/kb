use alloc::vec::Vec;

use crate::{
    key::{Action, Key, LayerIndex},
    processor::{Event, EventsProcessor, Result},
};

pub struct KeyReplaceProcessor {
    from: Key,
    to: Key,
}

#[allow(dead_code)]
impl KeyReplaceProcessor {
    pub fn new(from: Key, to: Key) -> Self {
        return KeyReplaceProcessor { from, to };
    }
}

impl<L: LayerIndex> EventsProcessor<L> for KeyReplaceProcessor {
    fn process(&mut self, events: &mut Vec<Event<L>>) -> Result {
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
