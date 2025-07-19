use crate::{error::TaResult, helper_types::Bar};
use core::{f64, fmt::Debug};

pub use crate::indicators::indicator::{IndicatorTrait, Period, Reset};

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


pub trait Output {}
