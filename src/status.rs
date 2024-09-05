use alloc::boxed::Box;
use embedded_hal::digital::OutputPin;
use hal::gpio;
use rtic_monotonics::rp2040::prelude::*;

use crate::kb::Mono;

const ACTIVITY_UPDATE_PERIOD_TICK: u64 = 5_000;
const ACTIVITY_DELAY_PERIOD_TICK: u64 = 300_000;

pub struct StatusLED {
    link_led: Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>,
    activity_led: Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>,
    activity_led_state: gpio::PinState,
    activity_last_update_tick: u64,
    activity_last_active_tick: u64,
    counter: u16,
}

impl StatusLED {
    pub fn new(
        mut link_led: Box<dyn OutputPin<Error = gpio::Error> + Send + Sync>,
        mut activity_led: Box<dyn OutputPin<Error = gpio::Error> + Send + Sync>,
    ) -> Self {
        link_led.set_low().unwrap();
        activity_led.set_low().unwrap();

        StatusLED {
            link_led,
            activity_led,
            activity_led_state: gpio::PinState::High,
            activity_last_update_tick: 0,
            activity_last_active_tick: 0,
            counter: 0,
        }
    }

    pub fn set_link(&mut self, enable: bool) {
        self.link_led
            .set_state(if enable {
                gpio::PinState::High
            } else {
                gpio::PinState::Low
            })
            .unwrap();
    }

    pub fn update_activity(&mut self, active: bool) {
        let now_tick = Mono::now().ticks();
        if now_tick - self.activity_last_update_tick < ACTIVITY_UPDATE_PERIOD_TICK {
            return;
        }
        self.activity_last_update_tick = now_tick;
        self.counter = self.counter.wrapping_add(1);
        if active {
            self.activity_last_active_tick = now_tick;
        }

        self.activity_led_state =
            if active || now_tick - self.activity_last_active_tick < ACTIVITY_DELAY_PERIOD_TICK {
                if self.counter % 12 < 6 {
                    gpio::PinState::Low
                } else {
                    gpio::PinState::High
                }
            } else {
                gpio::PinState::High
            };
        self.activity_led
            .set_state(self.activity_led_state)
            .unwrap();
    }
}
