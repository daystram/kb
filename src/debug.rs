use alloc::string::String;
use defmt::Format;
use rtic_monotonics::Monotonic;

use crate::kb::{Mono, HEAP};

pub const ENABLE_LOG_INPUT_SCANNER_ENABLE_TIMING: bool = false;
pub const ENABLE_LOG_PROCESSOR_ENABLE_TIMING: bool = false;
pub const ENABLE_LOG_EVENTS: bool = true;
pub const ENABLE_LOG_SENT_KEYS: bool = false;

const ENABLE_LOG_HEAP: bool = true;
const ENABLE_LOG_DURATION: bool = true;
const ENABLE_LOG_STRING: bool = true;

const LOG_HEAP_SAMPLING_RATE: u32 = 5000;
pub const LOG_INPUT_SCANNER_SAMPLING_RATE: u64 = 50;
pub const LOG_PROCESSOR_SAMPLING_RATE: u64 = 50;
const LOG_DURATION_SAMPLING_RATE: u32 = 1000;
const LOG_STRING_SAMPLING_RATE: u32 = 500;

static mut LOG_HEAP_COUNTER: u32 = 0;

pub fn log_heap() {
    if !ENABLE_LOG_HEAP {
        return;
    }
    let counter = unsafe {
        LOG_HEAP_COUNTER = LOG_HEAP_COUNTER.wrapping_add(1);
        LOG_HEAP_COUNTER
    };
    if counter % LOG_HEAP_SAMPLING_RATE == 0 {
        defmt::trace!(
            "[{}] ========= heap stat: free={}B used={}B",
            counter,
            HEAP.free(),
            HEAP.used()
        );
    }
}

#[derive(Clone, Copy, Debug, Format)]
#[repr(u8)]
pub enum LogDurationTag {
    KeyMatrixScan,

    ClientInputScan,

    ServerListenRetrieve,
    ServerListenDispatch,
    ServerListenRespond,
    ServerListenFull,

    UARTSenderSendSerialize,
    UARTSenderSendTransform,
    UARTSenderSendWrite,

    UARTReceiverReadRead,
    UARTReceiverReadDeserialize,
    UARTReceiverReadBuffer,

    UARTIRQRecieveBuffer,
}

static mut LOG_DURATION_COUNTER: [u32; u8::MAX as usize] = [0; u8::MAX as usize];

pub fn log_duration(
    tag: LogDurationTag,
    start_time: <Mono as Monotonic>::Instant,
    end_time: <Mono as Monotonic>::Instant,
) {
    if !ENABLE_LOG_DURATION {
        return;
    }
    let counter = unsafe {
        LOG_DURATION_COUNTER[tag as usize] = LOG_DURATION_COUNTER[tag as usize].wrapping_add(1);
        LOG_DURATION_COUNTER[tag as usize]
    };
    if counter % LOG_DURATION_SAMPLING_RATE == 0 {
        defmt::trace!(
            "[{}] ========= {}: {}us",
            counter,
            tag,
            (end_time - start_time).to_micros()
        );
    }
}

#[derive(Clone, Copy, Debug, Format)]
#[repr(u8)]
pub enum LogStringTag {
    PacketLength,
}

static mut LOG_STRING_COUNTER: u32 = 0;

pub fn log_string(tag: LogStringTag, str: String) {
    if !ENABLE_LOG_STRING {
        return;
    }
    let counter = unsafe {
        LOG_STRING_COUNTER = LOG_STRING_COUNTER.wrapping_add(1);
        LOG_STRING_COUNTER
    };
    if counter % LOG_STRING_SAMPLING_RATE == 0 {
        defmt::trace!("[{}] ========= {}: {}", counter, tag, str.as_str());
    }
}
