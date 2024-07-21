#![allow(dead_code)]
use embedded_hal::{digital::OutputPin, pwm::SetDutyCycle};
use rtic_monotonics::rp2040::prelude::*;

use crate::kb::Mono;

pub async fn halt_blink<P: OutputPin + ?Sized>(led: &mut P, us: u32) -> Result<(), P::Error> {
    match led.set_high() {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    halt((us / 2) as u64).await;
    match led.set_low() {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    halt((us / 2) as u64).await;
    return Ok(());
}

pub async fn halt(us: u64) {
    Mono::delay(us.micros()).await;
}

pub async fn lerp(channel: &mut impl SetDutyCycle, from: u16, to: u16, step: u16, delay_ms: u64) {
    let diff = if from < to {
        (to - from) / step
    } else {
        (from - to) / step
    };

    for d in (0..step)
        .map(|x| x * diff)
        .map(|x| if from < to { from + x } else { from - x })
    {
        Mono::delay(delay_ms.millis()).await;
        channel.set_duty_cycle(d).unwrap();
    }
}
