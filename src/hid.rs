extern crate alloc;
use alloc::vec::Vec;

use usbd_human_interface_device::page::Keyboard;

pub type Keys = Vec<Keyboard>;
