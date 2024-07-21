pub mod animation;

use alloc::{boxed::Box, vec::Vec};
use animation::{BreatheAnimation, NoneAnimation, ScanAnimation, WheelAnimation};
use core::ops::Mul;
use hal::timer::Instant;
use rtic_monotonics::{rp2040::prelude::*, Monotonic};
use rtic_sync::channel::{Receiver, Sender};
use smart_leds::{brightness, SmartLedsWrite, RGB8 as SLRGB8};

use crate::{
    kb::Mono,
    key::Edge,
    key::{Action, Control, LayerIndex},
    processor::{Event, EventsProcessor, Result},
};

// More than 64 pulls too much power, it fries the board
const LED_MAX_BRIGHTNESS: u8 = 28;

const FRAME_TIME_MIN_MICROS: u64 = 1_000;
const FRAME_TIME_DEFAULT_MICROS: u64 = 10_000;
const FRAME_TIME_MAX_MICROS: u64 = 1_000_000;

type RGB8 = SLRGB8;

type Frame<const LED_COUNT: usize> = [RGB8; LED_COUNT];

pub trait FrameIterator = Iterator<Item = RGB8>;

#[derive(Clone, Copy)]
pub struct RGBMatrix<const LED_COUNT: usize, W: SmartLedsWrite>
where
    W::Color: From<RGB8>,
{
    writer: W,
}

#[allow(dead_code)]
impl<const LED_COUNT: usize, W: SmartLedsWrite> RGBMatrix<LED_COUNT, W>
where
    W::Color: From<RGB8>,
{
    pub fn new(writer: W) -> Self {
        RGBMatrix { writer }
    }

    pub async fn render(
        &mut self,
        mut frame_receiver: Receiver<'static, Box<dyn FrameIterator>, 1>,
    ) {
        while let Ok(frame) = frame_receiver.recv().await {
            self.writer
                .write(brightness(frame, LED_MAX_BRIGHTNESS))
                .ok();
        }
    }
}

pub struct AnimationState<const LED_COUNT: usize> {
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
    animations: [Box<dyn AnimationIterator<{ LED_COUNT }, Item = Box<dyn FrameIterator>>>; 4],
    animation_idx: usize,
    frame_sender: Sender<'static, Box<dyn FrameIterator>, 1>,
    last_render: Instant,
    frame_time_micros: u64,
    brightness: u8,
}

#[allow(dead_code)]
impl<const LED_COUNT: usize> RGBProcessor<{ LED_COUNT }> {
    pub fn new(frame_sender: Sender<'static, Box<dyn FrameIterator>, 1>) -> Self {
        RGBProcessor {
            animations: [
                Box::new(BreatheAnimation::new(Default::default())),
                Box::new(WheelAnimation::new(Default::default())),
                Box::new(ScanAnimation::new(Default::default())),
                Box::new(NoneAnimation::new()),
            ],
            animation_idx: 0,
            frame_sender,
            last_render: Mono::now(),
            frame_time_micros: FRAME_TIME_DEFAULT_MICROS,
            brightness: 255,
        }
    }
}

impl<const LED_COUNT: usize, L: LayerIndex> EventsProcessor<L> for RGBProcessor<{ LED_COUNT }> {
    fn process(&mut self, events: &mut Vec<Event<L>>) -> Result {
        events.iter_mut().for_each(|e| {
            if e.edge == Edge::Rising {
                if let Action::Control(k) = e.action {
                    match k {
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
                    }
                }
            }
        });

        match Mono::now().checked_duration_since(self.last_render) {
            Some(d) if d > self.frame_time_micros.micros::<1, 1_000_000>() => {
                self.frame_sender
                    .try_send(Box::new(brightness(
                        self.animations[self.animation_idx].next().unwrap(),
                        self.brightness,
                    )))
                    .ok();
                self.last_render = Mono::now();
            }
            _ => {}
        };

        Ok(())
    }
}

trait AnimationIterator<const LED_COUNT: usize> {
    type Item = Box<dyn FrameIterator>;

    fn next(&mut self) -> Option<Self::Item>;
}
