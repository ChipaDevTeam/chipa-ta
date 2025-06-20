use crate::{error::TaResult, helper_types::Bar, types::OutputShape};
use core::{f64, fmt::Debug};
use std::fmt::Display;

pub trait Candle: Clone + Debug + Sized {
    fn open(&self) -> f64 {
        self.price()
    }

    fn close(&self) -> f64 {
        self.price()
    }

    fn high(&self) -> f64 {
        self.price()
    }

    fn low(&self) -> f64 {
        self.price()
    }

    fn price(&self) -> f64;

    fn volume(&self) -> f64 {
        f64::NAN
    }

    fn to_bar(&self) -> Bar {
        Bar::new()
            .set_open(self.open())
            .set_high(self.high())
            .set_low(self.low())
            .set_close(self.close())
            .set_price(self.price())
            .set_volume(self.volume())
    }
}

pub trait Indicator: Clone + Debug + Reset + Default + PartialEq + Period + Display {
    fn output_shape(&self) -> OutputShape;
    fn name(&self) -> String {
        self.to_string()
    }
}

pub trait Next<T> {
    type Output;

    fn next(&mut self, input: T) -> TaResult<Self::Output>;

    fn next_batched<A>(&mut self, input: A) -> TaResult<Vec<Self::Output>>
    where
        A: Iterator<Item = T>,
    {
        input.map(|e| self.next(e)).collect()
    }
}

/// Resets an indicator to the initial state.
pub trait Reset {
    fn reset(&mut self);
}

pub trait Period {
    fn period(&self) -> usize;
}

pub trait Output {}
