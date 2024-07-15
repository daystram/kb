extern crate alloc;
use alloc::boxed::Box;

use super::{AnimationIterator, AnimationState, RGB8};

pub struct NoneAnimation {}

impl NoneAnimation {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const LED_COUNT: usize> AnimationIterator<LED_COUNT> for NoneAnimation {
    fn next(&mut self) -> Option<Self::Item> {
        Some(Box::new([(0, 0, 0).into(); LED_COUNT].into_iter()))
    }
}

pub struct ScanAnimation<const LED_COUNT: usize> {
    animation_state: AnimationState<{ LED_COUNT }>,
}

impl<const LED_COUNT: usize> ScanAnimation<LED_COUNT> {
    pub fn new(animation_state: AnimationState<LED_COUNT>) -> Self {
        return Self { animation_state };
    }
}

impl<const LED_COUNT: usize> AnimationIterator<LED_COUNT> for ScanAnimation<LED_COUNT> {
    fn next(&mut self) -> Option<Self::Item> {
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

pub struct BreatheAnimation<const LED_COUNT: usize> {
    animation_state: AnimationState<{ LED_COUNT }>,
}

impl<const LED_COUNT: usize> BreatheAnimation<LED_COUNT> {
    pub fn new(animation_state: AnimationState<LED_COUNT>) -> Self {
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
    fn next(&mut self) -> Option<Self::Item> {
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

pub struct WheelAnimation<const LED_COUNT: usize> {
    animation_state: AnimationState<{ LED_COUNT }>,
}

impl<const LED_COUNT: usize> WheelAnimation<LED_COUNT> {
    pub fn new(animation_state: AnimationState<LED_COUNT>) -> Self {
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
    fn next(&mut self) -> Option<Self::Item> {
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
