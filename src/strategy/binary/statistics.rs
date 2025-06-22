use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Statistics {
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
    // TODO: Add more fields as needed for additional statistics.   
}

impl Default for Statistics {
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
        }
    }
}