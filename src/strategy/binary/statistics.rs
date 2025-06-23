use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::strategy::platform::{BinaryOptionsPlatform, TradeResult};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Statistics<P: BinaryOptionsPlatform> {
    /// The total number of trades executed.
    pub total_trades: u32,
    /// The number of winning trades.
    pub winning_trades: u32,
    /// The number of losing trades.
    pub losing_trades: u32,
    /// The total profit from all trades.
    pub total_profit: f64,
    /// The total loss from all trades.
    pub total_loss: f64,
    /// Profit factor (total profit / total loss).
    pub profit_factor: f64,
    /// Payoff percentage (average profit / average loss).
    pub payoff_percentage: f64,
    /// Start time of the trading session.
    pub start_time: DateTime<Utc>,
    /// Runtime of the trading session.
    pub runtime: Duration,
    /// Total time running the strategy.
    pub total_time: Duration,
    /// The last trade result.
    pub last_trade_result: Option<TradeResult>,
    /// The last trade time.
    pub last_trade_time: Option<DateTime<Utc>>,
    /// The number of open trades.
    pub open_trades: HashMap<P::TradeId, DateTime<Utc>>,
}

impl<P: BinaryOptionsPlatform + Default> Default for Statistics<P> {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_profit: 0.0,
            total_loss: 0.0,
            profit_factor: 0.0,
            payoff_percentage: 0.0,
            start_time: Utc::now(),
            runtime: Duration::zero(),
            total_time: Duration::zero(),
            last_trade_result: None,
            last_trade_time: None,
            open_trades: HashMap::new(),
        }
    }
}

impl<P: BinaryOptionsPlatform> Statistics<P> {
    pub fn update_on_trade(
        &mut self,
        result: &crate::strategy::platform::TradeResult,
        profit: f64,
        time: DateTime<Utc>,
    ) {
        self.total_trades += 1;
        self.last_trade_result = Some(result.clone());
        self.last_trade_time = Some(time);
        match result {
            crate::strategy::platform::TradeResult::Win(p) => {
                self.winning_trades += 1;
                self.total_profit += *p;
            }
            crate::strategy::platform::TradeResult::Lose => {
                self.losing_trades += 1;
                self.total_loss += profit.abs();
            }
            crate::strategy::platform::TradeResult::Draw => {}
        }
        self.profit_factor = if self.total_loss > 0.0 {
            self.total_profit / self.total_loss
        } else {
            0.0
        };
        self.payoff_percentage = if self.losing_trades > 0 {
            (self.total_profit / self.winning_trades as f64)
                / (self.total_loss / self.losing_trades as f64)
        } else {
            0.0
        };
    }

    pub fn add_opened_trade(&mut self, trade_id: P::TradeId, open_time: DateTime<Utc>) {
        self.open_trades.insert(trade_id, open_time);
    }

    pub fn remove_opened_trade(&mut self, trade_id: &P::TradeId) {
        self.open_trades.remove(trade_id);
    }
}
