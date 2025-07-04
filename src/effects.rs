#![allow(unused_imports)]

use crate::{DynEffect, Effect};
use micromath::F32Ext;

macro_rules! impl_into_dyn_effect {
    ($type:ty) => {
        impl Into<DynEffect> for $type {
            fn into(self) -> DynEffect {
                DynEffect::new(self)
            }
        }
    };
}

/// Linear interpolation filter
#[derive(Default)]
pub struct Lerp {
    smoothed_value: Option<f32>,
    lerp_factor: f32,
}

impl Lerp {
    pub fn new(factor: f32) -> Self {
        Self {
            smoothed_value: None,
            lerp_factor: factor,
        }
    }
}

impl Effect for Lerp {
    fn update(&mut self, value: u16) -> u16 {
        let value_f32 = value as f32;
        let sv = self.smoothed_value.unwrap_or(value_f32);
        let new_value = sv + (value_f32 - sv) * self.lerp_factor;

        self.smoothed_value = Some(new_value);
        new_value.floor() as u16
    }
}

/// Laggy-smooth effect
#[derive(Default)]
pub struct Smooth {
    current: u16,
    target: u16,
    speed: u16
}

impl Smooth {
    pub fn new(speed: u16) -> Self {
        Self {
            current: 0,
            target: 0,
            speed
        }
    }
}

impl Effect for Smooth {
    fn update(&mut self, input: u16) -> u16 {
        self.target = input;
        self.current = self.current.saturating_add(self.speed);
        self.current.clamp(u16::MIN, self.target)
    }
}

impl_into_dyn_effect!(Smooth);
