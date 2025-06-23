use serde::{Deserialize, Serialize};

use crate::{error::TaResult, strategy::{binary::martingale::{Martingale, MartingaleResetCondition}, platform::TradeResult}};

/// This file is part of the `binary` module in the `strategy` package.
/// It defines the `AmountModifier` trait, which is used to modify the trade amount based on the last trade result.
/// 
/// The `AmountModifier` trait is used to implement different strategies for modifying the trade amount
/// based on the outcome of the last trade. This can be useful for implementing strategies like Martingale,
/// where the trade amount is adjusted based on whether the last trade was a win, loss, or draw.
pub trait AmountModifier {
    fn modify(&mut self, amount: f64, last_result: &Option<TradeResult>) -> f64;
}

#[derive(Serialize, Deserialize)]
pub enum Modifier {
    /// No modification, returns the original amount.
    None,
    /// Martingale strategy, which doubles the amount after a loss.
    Martingale(Martingale)
}

impl Modifier {
    /// Creates a new `None` modifier, which does not modify the trade amount.
    pub fn none() -> Self {
        Self::None
    }

    /// Creates a new `Martingale` modifier with the given parameters.
    pub fn martingale(multiplier: f64, max_doublings: u32, reset_condition: MartingaleResetCondition) -> TaResult<Self> {
        Ok(Self::Martingale(Martingale::new(multiplier, max_doublings, reset_condition)?))
    }
}

impl Default for Modifier {
    fn default() -> Self {
        Self::None
    }
}

impl AmountModifier for Modifier {
    fn modify(&mut self, amount: f64, last_result: &Option<TradeResult>) -> f64 {
        match self {
            Self::None => amount,
            Self::Martingale(martingale) => martingale.modify(amount, last_result),
        }
    }
}
