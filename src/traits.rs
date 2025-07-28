use crate::{error::TaResult, helper_types::Bar};
use core::{f64, fmt::Debug};

pub use crate::indicators::indicator::{IndicatorTrait, Period, Reset};

use chipa_ta_utils::TaUtilsResult;
pub use chipa_ta_utils::{Candle, Next};

pub trait NextBatched<T> {
    type Output;

    fn next_batched<A>(&mut self, input: A) -> TaResult<Vec<Self::Output>>
    where
        A: Iterator<Item = T>;
}

impl<'a, T: Next<&'a dyn Candle>> NextBatched<&'a dyn Candle> for T {
    type Output = T::Output;

    fn next_batched<A>(&mut self, input: A) -> TaResult<Vec<Self::Output>>
    where
        A: Iterator<Item = &'a dyn Candle>,
    {
        input
            .map(|e| self.next(e))
            .collect::<TaUtilsResult<Vec<Self::Output>>>()
            .map_err(|e| e.into())
    }
}
