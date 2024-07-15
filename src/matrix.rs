extern crate alloc;
use alloc::boxed::Box;
use defmt::Format;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use rp2040_hal::gpio;
use rtic_monotonics::{rp2040::*, Monotonic};

use crate::util::halt;

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum Edge {
    None,
    Rising,
    Falling,
}

impl From<(bool, bool)> for Edge {
    fn from((from, to): (bool, bool)) -> Self {
        if !from && to {
            Edge::Rising
        } else if from && !to {
            Edge::Falling
        } else {
            Edge::None
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct Bitmap<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub scan_time_ticks: u64,
    pub matrix: [[Bit; COL_COUNT]; ROW_COUNT],
}

#[derive(Clone, Copy, Debug, Format)]
pub struct Bit {
    pub edge: Edge,
    pub pressed: bool,
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> Default for Bitmap<ROW_COUNT, COL_COUNT> {
    fn default() -> Self {
        Bitmap {
            scan_time_ticks: 0,
            matrix: [[Bit {
                edge: Edge::None,
                pressed: false,
            }; COL_COUNT]; ROW_COUNT],
        }
    }
}

pub trait Scanner<const ROW_COUNT: usize, const COL_COUNT: usize> {
    async fn scan(&mut self) -> Bitmap<ROW_COUNT, COL_COUNT>;
}

pub struct BasicVerticalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn InputPin<Error = gpio::Error>>; ROW_COUNT],
    pub cols: [Box<dyn OutputPin<Error = gpio::Error>>; COL_COUNT],
    previous_bitmap: Bitmap<{ ROW_COUNT }, { COL_COUNT }>,
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
            previous_bitmap: Bitmap::default(),
        };
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> Scanner<ROW_COUNT, COL_COUNT>
    for BasicVerticalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    async fn scan(&mut self) -> Bitmap<ROW_COUNT, COL_COUNT> {
        let mut bitmap = Bitmap::default();
        for (j, col) in self.cols.iter_mut().enumerate() {
            col.set_high().unwrap();
            for (i, row) in self.rows.iter().enumerate() {
                let pressed = row.is_high().unwrap();
                bitmap.matrix[i][j] = Bit {
                    edge: Edge::from((self.previous_bitmap.matrix[i][j].pressed, pressed)),
                    pressed,
                }
            }
            col.set_low().unwrap();
            halt(1).await;
        }
        bitmap.scan_time_ticks = Timer::now().ticks();
        self.previous_bitmap = bitmap;
        return bitmap;
    }
}

pub struct BasicHorizontalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn OutputPin<Error = gpio::Error>>; ROW_COUNT],
    pub cols: [Box<dyn InputPin<Error = gpio::Error>>; COL_COUNT],
    previous_bitmap: Bitmap<{ ROW_COUNT }, { COL_COUNT }>,
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
            previous_bitmap: Bitmap::default(),
        };
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> Scanner<ROW_COUNT, COL_COUNT>
    for BasicHorizontalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    async fn scan(&mut self) -> Bitmap<ROW_COUNT, COL_COUNT> {
        let mut bitmap = Bitmap::default();
        for (i, row) in self.rows.iter_mut().enumerate() {
            row.set_high().unwrap();
            for (j, col) in self.cols.iter().enumerate() {
                let pressed = col.is_high().unwrap();
                bitmap.matrix[i][j] = Bit {
                    edge: Edge::from((self.previous_bitmap.matrix[i][j].pressed, pressed)),
                    pressed,
                }
            }
            row.set_low().unwrap();
            halt(1).await;
        }
        self.previous_bitmap = bitmap;
        return bitmap;
    }
}
