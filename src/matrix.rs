extern crate alloc;
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};

use embedded_hal::digital::v2::{InputPin, OutputPin};
use rp2040_hal::gpio::{self};

use crate::util::halt;

#[derive(Clone, Copy)]
pub struct Bitmap<const ROW_COUNT: usize, const COL_COUNT: usize>(
    pub [[bool; COL_COUNT]; ROW_COUNT],
);

impl<const ROW_COUNT: usize, const COL_COUNT: usize> Deref for Bitmap<ROW_COUNT, COL_COUNT> {
    type Target = [[bool; COL_COUNT]; ROW_COUNT];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> DerefMut for Bitmap<ROW_COUNT, COL_COUNT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> Default for Bitmap<ROW_COUNT, COL_COUNT> {
    fn default() -> Self {
        Bitmap([[false; COL_COUNT]; ROW_COUNT])
    }
}

pub trait Scanner<const ROW_COUNT: usize, const COL_COUNT: usize> {
    async fn scan(&mut self) -> Bitmap<ROW_COUNT, COL_COUNT>;
}

pub struct BasicVerticalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn InputPin<Error = gpio::Error>>; ROW_COUNT],
    pub cols: [Box<dyn OutputPin<Error = gpio::Error>>; COL_COUNT],
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize>
    BasicVerticalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    pub fn new(
        rows: [Box<dyn InputPin<Error = gpio::Error>>; ROW_COUNT],
        cols: [Box<dyn OutputPin<Error = gpio::Error>>; COL_COUNT],
    ) -> Self {
        return BasicVerticalSwitchMatrix { rows, cols };
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
                bitmap[i][j] = row.is_high().unwrap();
            }
            col.set_low().unwrap();
            halt(1).await;
        }
        return bitmap;
    }
}

pub struct BasicHorizontalSwitchMatrix<const ROW_COUNT: usize, const COL_COUNT: usize> {
    pub rows: [Box<dyn OutputPin<Error = gpio::Error>>; ROW_COUNT],
    pub cols: [Box<dyn InputPin<Error = gpio::Error>>; COL_COUNT],
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize>
    BasicHorizontalSwitchMatrix<ROW_COUNT, COL_COUNT>
{
    pub fn new(
        rows: [Box<dyn OutputPin<Error = gpio::Error>>; ROW_COUNT],
        cols: [Box<dyn InputPin<Error = gpio::Error>>; COL_COUNT],
    ) -> Self {
        return BasicHorizontalSwitchMatrix { rows, cols };
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
                bitmap[i][j] = col.is_high().unwrap();
            }
            row.set_low().unwrap();
            halt(1).await;
        }
        return bitmap;
    }
}
