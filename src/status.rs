use alloc::boxed::Box;
use embedded_hal::digital::OutputPin;
use hal::gpio;
use rtic_monotonics::rp2040::prelude::*;

use crate::kb::Mono;

const REMOTE_ACTIVITY_UPDATE_PERIOD_TICK: u64 = 5_000;
const REMOTE_ACTIVITY_DELAY_PERIOD_TICK: u64 = 300_000;

pub struct StatusLED {
    remote_link_led: Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>,
    remote_activity_led: Box<dyn OutputPin<Error = gpio::Error> + Sync + Send>,
    remote_activity_led_state: gpio::PinState,
    remote_activity_last_update_tick: u64,
    remote_activity_last_active_tick: u64,
    u2f_activity_led_pin: u8,
    counter: u16,
}

impl StatusLED {
    pub fn new(
        mut remote_link_led: Box<dyn OutputPin<Error = gpio::Error> + Send + Sync>,
        mut remote_activity_led: Box<dyn OutputPin<Error = gpio::Error> + Send + Sync>,
        u2f_activity_led_pin: u8,
    ) -> Self {
        remote_link_led.set_low().unwrap();
        remote_activity_led.set_low().unwrap();

        StatusLED {
            remote_link_led,
            remote_activity_led,
            remote_activity_led_state: gpio::PinState::High,
            remote_activity_last_update_tick: 0,
            remote_activity_last_active_tick: 0,
            u2f_activity_led_pin,
            counter: 0,
        }
    }

    pub fn set_remote_link(&mut self, enable: bool) {
        self.remote_link_led
            .set_state(if enable {
                gpio::PinState::High
            } else {
                gpio::PinState::Low
            })
            .unwrap();
    }

    pub fn update_remote_activity(&mut self, active: bool) {
        let now_tick = Mono::now().ticks();
        if now_tick - self.remote_activity_last_update_tick < REMOTE_ACTIVITY_UPDATE_PERIOD_TICK {
            return;
        }
        self.remote_activity_last_update_tick = now_tick;
        self.counter = self.counter.wrapping_add(1);
        if active {
            self.remote_activity_last_active_tick = now_tick;
        }

        self.remote_activity_led_state = if active
            || now_tick - self.remote_activity_last_active_tick < REMOTE_ACTIVITY_DELAY_PERIOD_TICK
        {
            if self.counter % 12 < 6 {
                gpio::PinState::Low
            } else {
                gpio::PinState::High
            }
        } else {
            gpio::PinState::High
        };
        self.remote_activity_led
            .set_state(self.remote_activity_led_state)
            .unwrap();
    }

    pub fn get_u2f_activity_led_pin(&mut self) -> u8 {
        self.u2f_activity_led_pin
    }
}
