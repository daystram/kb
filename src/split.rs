use alloc::boxed::Box;
use core::fmt::Display;
use defmt::Format;
use embedded_hal::digital::InputPin;
use hal::gpio;

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum Mode {
    Master,
    Slave,
}

impl Display for Mode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Mode::Master => write!(f, "Master"),
            Mode::Slave => write!(f, "Slave"),
        }
    }
}

pub struct SideDetector {
    pin: Box<dyn InputPin<Error = gpio::Error>>,
}

impl SideDetector {
    pub fn new(pin: Box<dyn InputPin<Error = gpio::Error>>) -> Self {
        SideDetector { pin }
    }

    pub fn detect(&mut self) -> Side {
        let side = if self.pin.is_high().unwrap() {
            Side::Left
        } else {
            Side::Right
        };
        set_self_side(side);
        side
    }
}

static mut SELF_SIDE: Option<Side> = None;

fn set_self_side(side: Side) {
    unsafe {
        if SELF_SIDE.is_some() {
            panic!("self side already set")
        }
        SELF_SIDE = Some(side);
    };
}

pub fn get_self_side() -> Side {
    unsafe { SELF_SIDE.expect("self side not set") }
}

static mut SELF_MODE: Option<Mode> = None;

pub fn set_self_mode(mode: Mode) {
    unsafe {
        if SELF_MODE.is_some() {
            panic!("self mode already set")
        }
        SELF_MODE = Some(mode);
    };
}

pub fn get_self_mode() -> Mode {
    unsafe { SELF_MODE.expect("self mode not set") }
}
