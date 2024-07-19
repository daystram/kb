use embedded_hal::digital::OutputPin;
use rtic_monotonics::rp2040::prelude::*;

use crate::kb::Mono;

#[allow(dead_code)]
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

#[allow(dead_code)]
pub async fn halt(us: u64) {
    Mono::delay(us.micros()).await;
}
