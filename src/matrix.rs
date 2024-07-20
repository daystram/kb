use alloc::boxed::Box;
use defmt::Format;
use embedded_hal::digital::{InputPin, OutputPin};
use rp2040_hal::gpio;
use rtic_monotonics::rp2040::prelude::*;

use crate::{kb::Mono, key::Edge, util::halt};

#[derive(Clone, Copy, Debug, Format)]
pub struct Result<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub scan_time_ticks: u64,
    pub matrix: [[Bit; COL_COUNT]; ROW_COUNT],
}

#[derive(Clone, Copy, Debug, Format)]
pub struct Bit {
    pub edge: Edge,
    pub pressed: bool,
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

pub trait Scanner<const ROW_COUNT: usize, const COL_COUNT: usize> {
    async fn scan(&mut self) -> Result<ROW_COUNT, COL_COUNT>;
}

pub struct BasicVerticalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn InputPin<Error = gpio::Error>>; ROW_COUNT],
    pub cols: [Box<dyn OutputPin<Error = gpio::Error>>; COL_COUNT],
    previous_result: Result<{ ROW_COUNT }, { COL_COUNT }>,
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize>
    BasicVerticalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    pub fn new(
        rows: [Box<dyn InputPin<Error = gpio::Error>>; ROW_COUNT],
        cols: [Box<dyn OutputPin<Error = gpio::Error>>; COL_COUNT],
    ) -> Self {
        return BasicVerticalSwitchMatrix {
            rows,
            cols,
            previous_result: Result::default(),
        };
    }
}

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
            halt(1).await;
        }
        result.scan_time_ticks = Mono::now().ticks();
        self.previous_result = result;
        return result;
    }
}

pub struct BasicHorizontalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn OutputPin<Error = gpio::Error>>; ROW_COUNT],
    pub cols: [Box<dyn InputPin<Error = gpio::Error>>; COL_COUNT],
    previous_result: Result<{ ROW_COUNT }, { COL_COUNT }>,
}

#[allow(dead_code)]
impl<const ROW_COUNT: usize, const COL_COUNT: usize>
    BasicHorizontalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    pub fn new(
        rows: [Box<dyn OutputPin<Error = gpio::Error>>; ROW_COUNT],
        cols: [Box<dyn InputPin<Error = gpio::Error>>; COL_COUNT],
    ) -> Self {
        return BasicHorizontalSwitchMatrix {
            rows,
            cols,
            previous_result: Result::default(),
        };
    }
}

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
            halt(1).await;
        }
        self.previous_result = result;
        return result;
    }
}
