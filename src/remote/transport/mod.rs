pub mod uart;

use alloc::vec::Vec;
use core::future::Future;
use defmt::Format;
use rtic_sync::channel::Receiver;
use serde::{Deserialize, Serialize};

use super::{MethodId, ServiceId};

#[derive(Clone, Copy, Debug, Deserialize, Format, PartialEq, Serialize)]
#[repr(u8)]
enum Kind {
    Request,
    Response,
}

pub type Sequence = u8;

#[derive(Clone, Copy, Debug, Deserialize, Format, Serialize)]
pub struct Packet<'a> {
    kind: Kind,
    pub sequence: Sequence,
    pub service_id: super::ServiceId,
    pub method_id: super::MethodId,
    pub payload: &'a [u8],
}

/*
    Packet bit layout
         0   1   2   3   4   5   6   7
       +---+---+---+---+---+---+---+---+
    0  |            length             |
       +---+---+---+---+---+---+---+---+
    8  |             kind              |
       +---+---+---+---+---+---+---+---+
   16  |           sequence            |
       +---+---+---+---+---+---+---+---+
   24  |          service_id           |
       +---+---+---+---+---+---+---+---+
   32  |          method_id            |
       +---+---+---+---+---+---+---+---+
   40  |                               |
    .  |                               |
    .  |           payload             |
    .  |                               |
    N  |                               |
       +---+---+---+---+---+---+---+---+
*/

pub trait TransportReceiver {
    fn initialize_seq_sender(
        &mut self,
    ) -> Receiver<'static, Sequence, { super::REQUEST_SEQUENCE_QUEUE_SIZE }>;
    fn read_into_buffer(&mut self);
}

pub trait TransportSender {
    fn send_request(
        &mut self,
        service_id: super::ServiceId,
        method_id: super::MethodId,
        payload: &[u8],
    ) -> Sequence;
    fn get_payload(
        &mut self,
        sequence: Sequence,
    ) -> Result<(ServiceId, MethodId, &[u8], impl FnMut(&[u8])), Error>;
    fn receive_response_poll(&mut self, sequence: Sequence)
        -> impl Future<Output = Vec<u8>> + Send;
}

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum Error {
    MissingPacketBuffer,
}
