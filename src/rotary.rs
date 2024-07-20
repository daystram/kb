use alloc::boxed::Box;
use defmt::Format;
use embedded_hal::digital::{InputPin, OutputPin};
use enum_map::Enum;
use hal::gpio;
use rtic_monotonics::Monotonic;

use crate::{kb::Mono, key::Edge};

#[derive(Clone, Copy, Debug, Format)]
pub struct Result {
    pub scan_time_ticks: u64,
    pub direction: Direction,
    pub edge: Edge,
}

impl Default for Result {
    fn default() -> Self {
        return Result {
            scan_time_ticks: 0,
            direction: Direction::None,
            edge: Edge::None,
        };
    }
}

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum Mode {
    Edge,
    DentLowPrecision,
    DentHighPrecision,
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Enum)]
pub enum Direction {
    None,
    Clockwise,
    CounterClockwise,
}

pub struct RotaryEncoder {
    pin_a: Box<dyn InputPin<Error = gpio::Error>>,
    pin_b: Box<dyn InputPin<Error = gpio::Error>>,

    mode: Mode,
    phase_history: u16,
    previous_result: Result,
}

impl RotaryEncoder {
    pub fn new(
        pin_a: Box<dyn InputPin<Error = gpio::Error>>,
        pin_b: Box<dyn InputPin<Error = gpio::Error>>,
        mut pin_c: Box<dyn OutputPin<Error = gpio::Error>>,
        mode: Mode,
    ) -> Self {
        pin_c.set_low().unwrap();
        return RotaryEncoder {
            pin_a,
            pin_b,
            phase_history: 0b0001, // start on one of the valid phases
            mode,
            previous_result: Result::default(),
        };
    }

    pub fn scan(&mut self) -> Result {
        let mut result = Result::default();
        let mut new_phase = (self.phase_history << 2) & 0b0000_1100;
        if self.pin_a.is_high().unwrap() {
            new_phase |= 0b10
        }
        if self.pin_b.is_high().unwrap() {
            new_phase |= 0b01
        }

        result.direction = match new_phase & 0b0000_1111 {
            0b0001 | 0b0111 | 0b1110 | 0b1000 => Direction::Clockwise,
            0b0010 | 0b1011 | 0b1101 | 0b0100 => Direction::CounterClockwise,
            _ => Direction::None,
        };
        if result.direction != Direction::None {
            self.phase_history <<= 4;
            self.phase_history |= new_phase;
            result.direction = match self.mode {
                Mode::Edge => result.direction,
                Mode::DentLowPrecision => match self.phase_history & 0b0000_0000_1111_1111 {
                    0b0001_0111 => Direction::Clockwise,
                    0b0010_1011 => Direction::CounterClockwise,
                    _ => Direction::None,
                },
                Mode::DentHighPrecision => match self.phase_history & 0b1111_1111_1111_1111 {
                    0b1110_1000_0001_0111 => Direction::Clockwise,
                    0b1101_0100_0010_1011 => Direction::CounterClockwise,
                    _ => Direction::None,
                },
            };
        }
        result.edge = Edge::from((
            self.previous_result.direction != Direction::None,
            result.direction != Direction::None,
        ));
        result.scan_time_ticks = Mono::now().ticks();
        self.previous_result = result;
        return result;
    }
}
