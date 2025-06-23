use serde::{Deserialize, Serialize};

use crate::{error::{TaError, TaResult}, strategy::{binary::modifier::AmountModifier, platform::TradeResult}};

#[derive(Serialize, Deserialize)]
pub enum MartingaleResetCondition {
    /// Reset the strategy after a win.
    Win,
    /// Reset the strategy after a loss.
    WinOrDraw,
}

/// Implementation of the Martingale strategy for binary options trading.
/// This strategy involves doubling the stake after each loss to recover losses and gain profit.
#[derive(Serialize, Deserialize)]
pub struct Martingale {
    /// Multiplier for the stake after a loss.
    pub multiplier: f64,
    /// Maximum number of times to double the stake.
    pub max_doublings: u32,
    /// Current number of doublings that have been made.
    pub current_doublings: u32,
    /// Reset condition for the strategy.
    pub reset_condition: MartingaleResetCondition,
}

impl Martingale {
    /// Creates a new Martingale strategy with the given parameters.
    pub fn new(multiplier: f64, max_doublings: u32, reset_condition: MartingaleResetCondition) -> TaResult<Self> {
        if multiplier <= 1.0 {
            return Err(TaError::InvalidParameter("Multiplier must be greater than 1.0".to_string()));
        }
        if max_doublings == 0 {
            return Err(TaError::InvalidParameter("Max doublings must be greater than 0".to_string()));
        }
        
        Ok(Self {
                    multiplier,
                    max_doublings,
                    current_doublings: 0,
                    reset_condition,
                })
    }

    /// Resets the strategy based on the reset condition.
    pub fn calculate_multiplier(&mut self, result: &TradeResult) -> f64 {
        match result {
            TradeResult::Win(_) => {
                match self.reset_condition {
                    MartingaleResetCondition::Win => {
                        self.current_doublings = 0; // Reset after a win
                    },
                    MartingaleResetCondition::WinOrDraw => {
                        self.current_doublings = 0; // Reset after a win or draw
                    },
                }
                1.0 // No multiplier on win
            },
            TradeResult::Lose => {
                if self.current_doublings < self.max_doublings {
                    self.current_doublings += 1;
                    self.multiplier.powf(self.current_doublings as f64) // Calculate the new stake multiplier
                } else {
                    1.0 // Reset to initial stake if max doublings reached
                }
            },
            TradeResult::Draw => {
                if let MartingaleResetCondition::WinOrDraw = self.reset_condition {
                    self.current_doublings = 0; // Reset after a draw
                    1.0 
                } else {
                    self.multiplier.powf(self.current_doublings as f64) // Continue with the current multiplier
                }
            },
        }

    }
}

impl AmountModifier for Martingale {
    fn modify(&mut self, amount: f64, last_result: &Option<TradeResult>) -> f64 {
        if let Some(last_result) = last_result {
            // Calculate the multiplier based on the last trade result
            let multiplier = self.calculate_multiplier(last_result);
            amount * multiplier
        } else {
            amount
        }
    }
}