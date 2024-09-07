use alloc::vec::Vec;
use hal::rom_data;

use crate::{
    key::{Action, Control, Edge, LayerIndex},
    processor::{Event, EventsProcessor, Result},
};

pub struct SystemProcessor {
    u2f_activity_pin: u8,
}

#[allow(dead_code)]
impl SystemProcessor {
    pub fn new(u2f_activity_pin: u8) -> Self {
        SystemProcessor { u2f_activity_pin }
    }
}

impl<L: LayerIndex> EventsProcessor<L> for SystemProcessor {
    fn process(&mut self, events: &mut Vec<Event<L>>) -> Result {
        events.iter_mut().for_each(|e| {
            if e.edge == Edge::Rising {
                if let Action::Control(c) = e.action {
                    if c == Control::U2FBootloaderJump {
                        rom_data::reset_to_usb_boot(1 << self.u2f_activity_pin, 0)
                    }
                }
            }
        });
        Ok(())
    }
}
