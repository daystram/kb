extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use core::ops::Mul;
use rtic_monotonics::{rp2040::*, Monotonic};
use rtic_sync::channel::{Receiver, Sender};
use smart_leds::{brightness, SmartLedsWrite, RGB8};

use crate::{
    key::{Action, Control, LayerIndex},
    matrix::Edge,
    stream::{Event, EventsProcessor, Result as StreamResult},
};

// More than 64 pulls too much power, it fries the board
const LED_MAX_BRIGHTNESS: u8 = 28;

const FRAME_TIME_MIN_MICROS: u64 = 1_000;
const FRAME_TIME_DEFAULT_MICROS: u64 = 20_000;
const FRAME_TIME_MAX_MICROS: u64 = 1_000_000;

type Frame<const LED_COUNT: usize> = [RGB8; LED_COUNT];

#[derive(Clone, Copy)]
pub struct RGBMatrix<const LED_COUNT: usize, W: SmartLedsWrite>
where
    W::Color: From<RGB8>,
{
    writer: W,
}

impl<const LED_COUNT: usize, W: SmartLedsWrite> RGBMatrix<LED_COUNT, W>
where
    W::Color: From<RGB8>,
{
    pub fn new(writer: W) -> Self {
        return RGBMatrix { writer };
    }

    pub async fn render(
        &mut self,
        mut frame_receiver: Receiver<'static, Box<dyn Iterator<Item = RGB8>>, 1>,
    ) {
        while let Ok(frame) = frame_receiver.recv().await {
            self.writer
                .write(brightness(frame, LED_MAX_BRIGHTNESS))
                .ok();
        }
    }
}

struct AnimationState<const LED_COUNT: usize> {
    t: u64,
    n: u8,
    frame: Frame<{ LED_COUNT }>,
}

impl<const LED_COUNT: usize> AnimationState<LED_COUNT> {
    fn step(&mut self) {
        self.t = self.t.wrapping_add(1);
        self.n = self.n.wrapping_add(1);
    }
}

impl<const LED_COUNT: usize> Default for AnimationState<LED_COUNT> {
    fn default() -> Self {
        Self {
            t: 0,
            n: 0,
            frame: [Default::default(); LED_COUNT],
        }
    }
}

pub struct RGBProcessor<const LED_COUNT: usize> {
    animations:
        [Box<dyn AnimationIterator<{ LED_COUNT }, Item = Box<dyn Iterator<Item = RGB8>>>>; 4],
    animation_idx: usize,
    frame_sender: Sender<'static, Box<dyn Iterator<Item = RGB8>>, 1>,
    last_render: <rtic_monotonics::rp2040::Timer as Monotonic>::Instant,
    frame_time_micros: u64,
    brightness: u8,
}

impl<const LED_COUNT: usize> RGBProcessor<{ LED_COUNT }> {
    pub fn new(frame_sender: Sender<'static, Box<dyn Iterator<Item = RGB8>>, 1>) -> Self {
        return RGBProcessor {
            animations: [
                Box::new(WheelAnimation::new(Default::default())),
                Box::new(BreatheAnimation::new(Default::default())),
                Box::new(ScanAnimation::new(Default::default())),
                Box::new(NoneAnimation::new()),
            ],
            animation_idx: 0,
            frame_sender,
            last_render: Timer::now(),
            frame_time_micros: FRAME_TIME_DEFAULT_MICROS,
            brightness: 255,
        };
    }
}

impl<const LED_COUNT: usize, L: LayerIndex> EventsProcessor<L> for RGBProcessor<{ LED_COUNT }> {
    async fn process(&mut self, events: &mut Vec<Event<L>>) -> StreamResult {
        events.into_iter().for_each(|e| {
            if e.edge == Edge::Rising {
                match e.action {
                    Action::Control(k) => match k {
                        Control::RGBAnimationPrevious => {
                            self.animation_idx = if self.animation_idx == 0 {
                                self.animations.len() - 1
                            } else {
                                self.animation_idx - 1
                            };
                        }
                        Control::RGBAnimationNext => {
                            self.animation_idx = if self.animation_idx == self.animations.len() - 1
                            {
                                0
                            } else {
                                self.animation_idx + 1
                            };
                        }

                        Control::RGBSpeedDown => {
                            if self.frame_time_micros < FRAME_TIME_MAX_MICROS {
                                self.frame_time_micros = self.frame_time_micros.mul(2)
                            }
                        }
                        Control::RGBSpeedUp => {
                            if self.frame_time_micros > FRAME_TIME_MIN_MICROS {
                                self.frame_time_micros = self.frame_time_micros.div_ceil(2)
                            }
                        }

                        Control::RGBBrightnessDown => {
                            self.brightness = self.brightness.saturating_sub(16)
                        }
                        Control::RGBBrightnessUp => {
                            self.brightness = self.brightness.saturating_add(16)
                        }

                        _ => {}
                    },
                    _ => {}
                }
            }
        });

        match Timer::now().checked_duration_since(self.last_render) {
            Some(d) if d > self.frame_time_micros.micros::<1, 1_000_000>() => {
                self.frame_sender
                    .try_send(Box::new(brightness(
                        self.animations[self.animation_idx].next().unwrap(),
                        self.brightness,
                    )))
                    .ok();
                self.last_render = Timer::now();
            }
            _ => {}
        };

        Ok(())
    }
}

trait DirectionControl {
    fn set_direction(&mut self, left: bool);
}

trait AnimationIterator<const LED_COUNT: usize> {
    type Item = Box<dyn Iterator<Item = RGB8>>;

    fn next(&mut self) -> Option<Self::Item>;
}

struct NoneAnimation {}

impl NoneAnimation {
    fn new() -> Self {
        Self {}
    }
}

impl<const LED_COUNT: usize> AnimationIterator<LED_COUNT> for NoneAnimation {
    fn next(&mut self) -> Option<Self::Item> {
        Some(Box::new([(0, 0, 0).into(); LED_COUNT].into_iter()))
    }
}

struct ScanAnimation<const LED_COUNT: usize> {
    animation_state: AnimationState<{ LED_COUNT }>,
    direction_left: bool,
}

impl<const LED_COUNT: usize> ScanAnimation<LED_COUNT> {
    fn new(animation_state: AnimationState<LED_COUNT>) -> Self
    where
        Self: Sized,
    {
        return Self {
            animation_state,
            direction_left: false,
        };
    }
}

impl<const LED_COUNT: usize> AnimationIterator<LED_COUNT> for ScanAnimation<LED_COUNT> {
    fn next(&mut self) -> Option<Box<dyn Iterator<Item = RGB8>>> {
        self.animation_state.step();
        for (i, d) in self.animation_state.frame.iter_mut().enumerate() {
            *d = if self.animation_state.t as usize % LED_COUNT == i {
                (255, 255, 255).into()
            } else {
                (0, 0, 0).into()
            };
        }
        Some(Box::new(self.animation_state.frame.into_iter()))
    }
}

impl<const LED_COUNT: usize> DirectionControl for ScanAnimation<LED_COUNT> {
    fn set_direction(&mut self, left: bool) {
        self.direction_left = left;
    }
}

struct BreatheAnimation<const LED_COUNT: usize> {
    animation_state: AnimationState<{ LED_COUNT }>,
}

impl<const LED_COUNT: usize> BreatheAnimation<LED_COUNT> {
    fn new(animation_state: AnimationState<LED_COUNT>) -> Self
    where
        Self: Sized,
    {
        return Self { animation_state };
    }

    fn breathe(mut t: u8) -> RGB8 {
        if t < 128 {
            t = t * 2;
        } else {
            t = (255 - t) * 2;
        }
        return (t, t, t).into();
    }
}

impl<const LED_COUNT: usize> AnimationIterator<LED_COUNT> for BreatheAnimation<LED_COUNT> {
    fn next(&mut self) -> Option<Box<dyn Iterator<Item = RGB8>>> {
        self.animation_state.step();
        for (i, d) in self.animation_state.frame.iter_mut().enumerate() {
            *d = Self::breathe(
                self.animation_state
                    .n
                    .wrapping_add((i * 128 / LED_COUNT) as u8),
            );
        }
        Some(Box::new(self.animation_state.frame.into_iter()))
    }
}

struct WheelAnimation<const LED_COUNT: usize> {
    animation_state: AnimationState<{ LED_COUNT }>,
}

impl<const LED_COUNT: usize> WheelAnimation<LED_COUNT> {
    fn new(animation_state: AnimationState<LED_COUNT>) -> Self
    where
        Self: Sized,
    {
        return Self { animation_state };
    }

    fn wheel(mut rot: u8) -> RGB8 {
        if rot < 85 {
            return (0, 255 - (rot * 3), rot * 3).into();
        } else if rot < 170 {
            rot -= 85;
            return (rot * 3, 0, 255 - (rot * 3)).into();
        } else {
            rot -= 170;
            return (255 - (rot * 3), rot * 3, 0).into();
        }
    }
}
impl<const LED_COUNT: usize> AnimationIterator<LED_COUNT> for WheelAnimation<LED_COUNT> {
    fn next(&mut self) -> Option<Box<dyn Iterator<Item = RGB8>>> {
        self.animation_state.step();
        for (i, d) in self.animation_state.frame.iter_mut().enumerate() {
            *d = Self::wheel(
                self.animation_state
                    .n
                    .wrapping_add((i * 255 / LED_COUNT) as u8),
            );
        }
        Some(Box::new(self.animation_state.frame.into_iter()))
    }
}
