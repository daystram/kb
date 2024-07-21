use alloc::boxed::Box;
use embedded_hal::pwm::SetDutyCycle;
use hal::gpio;

use crate::util;

const MAX_PWM_POWER: u16 = 0x6000;

pub struct HeartbeatLED {
    pin: Box<dyn SetDutyCycle<Error = gpio::Error>>,
}

impl HeartbeatLED {
    pub fn new(pin: Box<dyn SetDutyCycle<Error = gpio::Error>>) -> Self {
        HeartbeatLED { pin }
    }

    pub async fn cycle(&mut self) {
        loop {
            util::lerp(&mut self.pin, 0, MAX_PWM_POWER, 200, 10).await;
            util::lerp(&mut self.pin, MAX_PWM_POWER, 0, 200, 10).await;
        }
    }
}
