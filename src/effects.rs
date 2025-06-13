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

impl_into_dyn_effect!(Lerp);

pub struct Step {
    sensivity: u16,
    old_value: u16,
}

impl Effect for Step {
    fn update(&mut self, value: u16) -> u16 {
        if (value as i16 - self.old_value as i16).abs() >= self.sensivity as i16 {
            self.old_value = value;
            value
        } else {
            self.old_value
        }
    }
}

impl Step {
    pub fn new(sensivity: u16) -> Step {
        Self {
            sensivity,
            old_value: 0,
        }
    }
}

impl_into_dyn_effect!(Step);
