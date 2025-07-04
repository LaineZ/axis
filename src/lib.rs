#![no_std]
#![allow(unused_imports)]
use core::mem::MaybeUninit;
use micromath::F32Ext;
pub mod effects;

/// Max [Effect] size in bytes that can be fit in [DynEffect]
pub const MAX_EFFECT_SIZE: usize = 16;

/// Effect trait. If you want to implement custom [Effect] make sure the struct fits in [MAX_EFFECT_SIZE]
pub trait Effect {
    fn update(&mut self, input: u16) -> u16;
}

/// Dynamic dispatching wrapper for [Effect] trait.
#[derive(Clone)]
pub struct DynEffect {
    data: [MaybeUninit<u8>; MAX_EFFECT_SIZE],
    apply_fn: fn(&mut [MaybeUninit<u8>; MAX_EFFECT_SIZE], u16) -> u16,
}

impl DynEffect {
    pub(crate) fn new<T: Effect>(effect: T) -> Self {
        let real_size = core::mem::size_of::<T>();

        if real_size > MAX_EFFECT_SIZE {
            panic!(
                "Size of effect is {real_size} bytes this is more than {MAX_EFFECT_SIZE} bytes allowed."
            );
        }

        let mut data = [MaybeUninit::uninit(); MAX_EFFECT_SIZE];
        unsafe {
            let ptr = data.as_mut_ptr() as *mut T;
            ptr.write(effect);
        }

        fn call<T: Effect, const N: usize>(data: &mut [MaybeUninit<u8>; N], input: u16) -> u16 {
            let ptr = data.as_mut_ptr() as *mut T;
            unsafe { (*ptr).update(input) }
        }

        Self {
            data,
            apply_fn: call::<T, MAX_EFFECT_SIZE>,
        }
    }

    pub fn update(&mut self, input: u16) -> u16 {
        (self.apply_fn)(&mut self.data, input)
    }
}

pub struct Axis {
    pub min: u16,
    pub max: u16,
    pub reversed: bool,
    pub step_filter_factor: u16,
    old_value: u16,
    value: u16,
}

impl<'a> Axis {
    pub fn new(min: u16, max: u16, reversed: bool) -> Self {
        Self {
            min,
            max,
            reversed,
            step_filter_factor: 0,
            old_value: min,
            value: min,
        }
    }

    fn step_filter(&mut self, value: u16) -> u16 {
        if self.step_filter_factor == 0 {
            return value
        }

        if (value as i16 - self.old_value as i16).abs() >= self.step_filter_factor as i16 {
            self.old_value = value;
            value
        } else {
            self.old_value
        }
    }

    fn output_ranged(&self) -> u16 {
        let mut normalized = self.value;
        if self.reversed {
            normalized = self.max.saturating_sub(self.value.saturating_sub(self.min));
        }
        normalized.clamp(self.min, self.max)
    }

    pub fn update<I: IntoIterator<Item = &'a mut DynEffect>>(&mut self, value: u16, chain: I) {
        self.value = self.step_filter(value);
        for filter in chain {
            self.value = filter.update(self.value);
        }
    }

    pub fn output(&self, range_min: u16, range_max: u16) -> u16 {
        let scale = (range_max - range_min) as f32 / (self.max - self.min) as f32;
        let result = range_min as f32 + (self.output_ranged() - self.min) as f32 * scale;

        result.floor() as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::Lerp;

    #[test]
    fn basic_output() {
        let mut axis = Axis::new(0, 4, false);
        axis.update(5, []);
        assert_eq!(axis.output(0, 4), 4);
    }

    #[test]
    fn step_filter() {
        let mut axis = Axis::new(0, 128, false);
        axis.step_filter_factor = 10;

        for i in 0..9 {
            axis.update(i, []);
        }
        assert_eq!(axis.output(0, 128), 0);
        axis.update(10, []);
        assert_eq!(axis.output(0, 128), 10);
    }

    #[test]
    fn lerp_effect() {
        let mut axis = Axis::new(0, 128, false);
        let mut effects = [Lerp::new(0.5).into()];
        axis.update(0, effects.iter_mut());
        assert_eq!(axis.output(0, 128), 0);

        axis.update(100, effects.iter_mut());
        assert_eq!(axis.output(0, 128), 50);

        axis.update(100, effects.iter_mut());
        assert_eq!(axis.output(0, 128), 75);

        axis.update(100, effects.iter_mut());
        assert_eq!(axis.output(0, 128), 87);

        axis.update(100, effects.iter_mut());
        assert_eq!(axis.output(0, 128), 93);
    }
}
