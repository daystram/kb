use alloc::{boxed::Box, rc::Rc, vec::Vec};
use async_trait::async_trait;
use core::{cell::RefCell, future};
use defmt::Format;
use embedded_hal::digital::{InputPin, OutputPin};
use rp2040_hal::gpio;
use rtic_monotonics::rp2040::prelude::*;
use rtic_sync::arbiter::Arbiter;
use serde::{de, ser::SerializeStruct, Deserialize, Serialize};

use crate::{
    debug,
    kb::Mono,
    key::Edge,
    remote::{self, MethodId, RemoteInvoker, Service, ServiceId},
    split,
};

#[derive(Clone, Copy, Debug, Format)]
pub struct Result<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub scan_time_ticks: u64,
    pub matrix: [[Bit; COL_COUNT]; ROW_COUNT],
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> Serialize for Result<ROW_COUNT, COL_COUNT> {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Result", 2)?;
        state.serialize_field("scan_time_ticks", &self.scan_time_ticks)?;

        let mut packed_matrix = Vec::with_capacity((ROW_COUNT * COL_COUNT * 3) / 8);
        let mut bit_accumulator: u32 = 0; // use u32 to avoid overflow
        let mut bit_count = 0;

        for row in &self.matrix {
            for bit in row {
                bit_accumulator = (bit_accumulator << 3) | bit.pack() as u32;
                bit_count += 3;

                if bit_count >= 8 {
                    packed_matrix.push((bit_accumulator >> (bit_count - 8)) as u8);
                    bit_count -= 8;
                }
            }
        }

        if bit_count > 0 {
            packed_matrix.push((bit_accumulator << (8 - bit_count)) as u8);
        }

        state.serialize_field("matrix", &packed_matrix)?;
        state.end()
    }
}

impl<'de, const ROW_COUNT: usize, const COL_COUNT: usize> Deserialize<'de>
    for Result<ROW_COUNT, COL_COUNT>
{
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ResultOwned {
            scan_time_ticks: u64,
            matrix: Vec<u8>,
        }

        let temp = ResultOwned::deserialize(deserializer)?;
        let expected_len = (ROW_COUNT * COL_COUNT * 3 + 7) / 8; // add 7 to round up to next byte
        if temp.matrix.len() != expected_len {
            return Err(de::Error::custom(
                "matrix length does not match expected length",
            ));
        }

        let mut matrix = [[Bit {
            edge: Edge::None,
            pressed: false,
        }; COL_COUNT]; ROW_COUNT];
        let mut bit_accumulator: u32 = 0; // use u32 to avoid overflow
        let mut bit_count = 0;
        let mut byte_index = 0;

        #[allow(clippy::needless_range_loop)]
        for i in 0..ROW_COUNT {
            for j in 0..COL_COUNT {
                while bit_count < 3 {
                    bit_accumulator = (bit_accumulator << 8) | temp.matrix[byte_index] as u32;
                    byte_index += 1;
                    bit_count += 8;
                }

                matrix[i][j] = Bit::unpack((bit_accumulator >> (bit_count - 3)) as u8);
                bit_count -= 3;
            }
        }

        Ok(Result {
            scan_time_ticks: temp.scan_time_ticks,
            matrix,
        })
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> Default for Result<ROW_COUNT, COL_COUNT> {
    fn default() -> Self {
        Result {
            scan_time_ticks: 0,
            matrix: [[Bit {
                edge: Edge::None,
                pressed: false,
            }; COL_COUNT]; ROW_COUNT],
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct Bit {
    pub edge: Edge,
    pub pressed: bool,
}

impl Bit {
    fn pack(&self) -> u8 {
        (self.edge as u8) << 1 | (self.pressed as u8)
    }

    fn unpack(packed: u8) -> Self {
        let edge = match packed >> 1 {
            0 => Edge::None,
            1 => Edge::Rising,
            2 => Edge::Falling,
            _ => Edge::None, // invalid
        };
        let pressed = packed & 1 != 0;
        Bit { edge, pressed }
    }
}

#[async_trait]
pub trait Scanner<const ROW_COUNT: usize, const COL_COUNT: usize> {
    async fn scan(&mut self) -> Result<ROW_COUNT, COL_COUNT>;
}

#[async_trait(?Send)]
pub trait SplitScanner<const ROW_COUNT: usize, const COL_COUNT: usize>:
    Scanner<ROW_COUNT, { COL_COUNT / 2 }>
where
    [(); COL_COUNT / 2]:,
{
    async fn scan<I>(&mut self, client: &Arbiter<Rc<RefCell<I>>>) -> Result<ROW_COUNT, COL_COUNT>
    where
        I: RemoteInvoker;
}

pub struct SplitSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize>
where
    [(); COL_COUNT / 2]:,
{
    local_matrix: BasicVerticalSwitchMatrix<{ ROW_COUNT }, { COL_COUNT / 2 }>, // TODO: use boxed scanner
}

#[allow(dead_code)]
impl<const ROW_COUNT: usize, const COL_COUNT: usize> SplitSwitchMatrix<ROW_COUNT, COL_COUNT>
where
    [(); COL_COUNT / 2]:,
{
    pub fn new(local_matrix: BasicVerticalSwitchMatrix<{ ROW_COUNT }, { COL_COUNT / 2 }>) -> Self {
        SplitSwitchMatrix { local_matrix }
    }
}

const SERVICE_ID_KEY_MATRIX: ServiceId = 0x10;
const METHOD_ID_KEY_MATRIX_SCAN: MethodId = 0x11;

#[async_trait]
impl<const ROW_COUNT: usize, const COL_COUNT: usize> Scanner<ROW_COUNT, { COL_COUNT / 2 }>
    for SplitSwitchMatrix<ROW_COUNT, COL_COUNT>
where
    [(); COL_COUNT / 2]:,
{
    async fn scan(&mut self) -> Result<ROW_COUNT, { COL_COUNT / 2 }> {
        let start_time = Mono::now();
        let result = self.local_matrix.scan().await;
        let end_time = Mono::now();
        debug::log_duration(debug::LogDurationTag::KeyMatrixScan, start_time, end_time);
        result
    }
}

#[async_trait(?Send)]
impl<const ROW_COUNT: usize, const COL_COUNT: usize> SplitScanner<ROW_COUNT, COL_COUNT>
    for SplitSwitchMatrix<ROW_COUNT, COL_COUNT>
where
    [(); COL_COUNT / 2]:,
{
    async fn scan<I>(&mut self, client: &Arbiter<Rc<RefCell<I>>>) -> Result<ROW_COUNT, COL_COUNT>
    where
        I: RemoteInvoker,
    {
        let (remote_response, local_result) = future::join!(client
            .access()
            .await
            .borrow_mut()
            .invoke::<SwitchMatrixScanRequest, SwitchMatrixScanResponse<{ROW_COUNT}, {COL_COUNT/2}>>(
                SERVICE_ID_KEY_MATRIX,
                METHOD_ID_KEY_MATRIX_SCAN,
                SwitchMatrixScanRequest {},
            ), Scanner::scan(self)).await;

        // merge
        let mut merged_matrix = [[Bit {
            edge: Edge::None,
            pressed: false,
        }; COL_COUNT]; ROW_COUNT];

        let (left_matrix, right_matrix) = match split::get_self_side() {
            split::Side::Left => (local_result.matrix, remote_response.result.matrix),
            split::Side::Right => (remote_response.result.matrix, local_result.matrix),
        };
        #[allow(clippy::needless_range_loop)]
        for i in 0..ROW_COUNT {
            for j in 0..(COL_COUNT / 2) {
                merged_matrix[i][j] = left_matrix[i][j];
                merged_matrix[i][COL_COUNT - j - 1] = right_matrix[i][j]; // flip
            }
        }

        Result {
            scan_time_ticks: local_result.scan_time_ticks,
            matrix: merged_matrix,
        }
    }
}

#[async_trait(?Send)]
impl<const ROW_COUNT: usize, const COL_COUNT: usize> Service
    for SplitSwitchMatrix<ROW_COUNT, COL_COUNT>
where
    [(); COL_COUNT / 2]:,
{
    fn get_service_id(&self) -> ServiceId {
        SERVICE_ID_KEY_MATRIX
    }

    async fn dispatch(
        &mut self,
        method_id: MethodId,
        _request_buffer: &[u8],
    ) -> core::result::Result<Vec<u8>, remote::Error> {
        match method_id {
            METHOD_ID_KEY_MATRIX_SCAN => {
                let result = Scanner::scan(self).await;
                match postcard::to_allocvec(&SwitchMatrixScanResponse { result }) {
                    Ok(res) => core::result::Result::Ok(res),
                    Err(_) => core::result::Result::Err(remote::Error::ResponseSerializationFailed),
                }
            }
            _ => core::result::Result::Err(remote::Error::MethodUnimplemented),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Format, Serialize)]
pub struct SwitchMatrixScanRequest {}

#[derive(Clone, Copy, Debug, Deserialize, Format, Serialize)]
pub struct SwitchMatrixScanResponse<const ROW_COUNT: usize, const COL_COUNT: usize> {
    result: Result<{ ROW_COUNT }, { COL_COUNT }>,
}

pub struct BasicVerticalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn InputPin<Error = gpio::Error> + Sync + Send>; ROW_COUNT],
    pub cols: [Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>; COL_COUNT],
    previous_result: Result<{ ROW_COUNT }, { COL_COUNT }>,
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize>
    BasicVerticalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    pub fn new(
        rows: [Box<dyn InputPin<Error = gpio::Error> + Sync + Send>; ROW_COUNT],
        cols: [Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>; COL_COUNT],
    ) -> Self {
        BasicVerticalSwitchMatrix {
            rows,
            cols,
            previous_result: Result::default(),
        }
    }
}

#[async_trait]
impl<const ROW_COUNT: usize, const COL_COUNT: usize> Scanner<ROW_COUNT, COL_COUNT>
    for BasicVerticalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    async fn scan(&mut self) -> Result<ROW_COUNT, COL_COUNT> {
        let mut result = Result::default();
        for (j, col) in self.cols.iter_mut().enumerate() {
            col.set_high().unwrap();
            for (i, row) in self.rows.iter_mut().enumerate() {
                let pressed = row.is_high().unwrap();
                result.matrix[i][j] = Bit {
                    edge: Edge::from((self.previous_result.matrix[i][j].pressed, pressed)),
                    pressed,
                }
            }
            col.set_low().unwrap();
            Mono::delay(1.micros()).await;
        }
        result.scan_time_ticks = Mono::now().ticks();
        self.previous_result = result;
        result
    }
}

pub struct BasicHorizontalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>; ROW_COUNT],
    pub cols: [Box<dyn InputPin<Error = gpio::Error> + Sync + Send>; COL_COUNT],
    previous_result: Result<{ ROW_COUNT }, { COL_COUNT }>,
}

#[allow(dead_code)]
impl<const ROW_COUNT: usize, const COL_COUNT: usize>
    BasicHorizontalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    pub fn new(
        rows: [Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>; ROW_COUNT],
        cols: [Box<dyn InputPin<Error = gpio::Error> + Sync + Send>; COL_COUNT],
    ) -> Self {
        BasicHorizontalSwitchMatrix {
            rows,
            cols,
            previous_result: Result::default(),
        }
    }
}

#[async_trait]
impl<const ROW_COUNT: usize, const COL_COUNT: usize> Scanner<ROW_COUNT, COL_COUNT>
    for BasicHorizontalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    async fn scan(&mut self) -> Result<ROW_COUNT, COL_COUNT> {
        let mut result = Result::default();
        for (i, row) in self.rows.iter_mut().enumerate() {
            row.set_high().unwrap();
            for (j, col) in self.cols.iter_mut().enumerate() {
                let pressed = col.is_high().unwrap();
                result.matrix[i][j] = Bit {
                    edge: Edge::from((self.previous_result.matrix[i][j].pressed, pressed)),
                    pressed,
                }
            }
            row.set_low().unwrap();
            Mono::delay(1.micros()).await;
        }
        self.previous_result = result;
        result
    }
}
