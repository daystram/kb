#![allow(dead_code)]
use alloc::boxed::Box;
use embedded_hal::{digital::OutputPin, pwm::SetDutyCycle};
use hal::gpio;
use rtic_monotonics::rp2040::prelude::*;

use crate::kb::Mono;

pub async fn blink<P: OutputPin>(
    pin: &mut P,
    duty: u8,
    delay: <Mono as Monotonic>::Duration,
) -> Result<(), P::Error> {
    match pin.set_high() {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    Mono::delay((delay.to_nanos() * duty as u64 / u8::MAX as u64).nanos()).await;
    match pin.set_low() {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    Mono::delay((delay.to_nanos() * (u8::MAX - duty) as u64 / u8::MAX as u64).nanos()).await;
    Ok(())
}

pub async fn lerp(
    pin: &mut Box<dyn SetDutyCycle<Error = gpio::Error> + Sync + Send>,
    from: u16,
    to: u16,
    step: u16,
    delay: <Mono as Monotonic>::Duration,
) {
    let diff = if from < to {
        (to - from) / step
    } else {
        (from - to) / step
    };
    let step_delay = (delay.to_nanos() / step as u64).nanos();

    for d in (0..step)
        .map(|x| x * diff)
        .map(|x| if from < to { from + x } else { from - x })
    {
        pin.set_duty_cycle(d).unwrap();
        Mono::delay(step_delay).await;
    }
}
