use serde::{Deserialize, Serialize};

use crate::strategy::{binary::{info::BinaryInfo, martingale::Martingale, statistics::Statistics}, platform::BinaryOptionsPlatform, StrategyNode};

#[derive(Serialize, Deserialize)]
pub struct Trader<P: BinaryOptionsPlatform> {
    /// The trading platform being used, which implements the BinaryOptionsPlatform trait.
    pub platform: P,
    /// Information about the binary options strategy.
    pub info: BinaryInfo,
    /// Statistics about the trader's performance.
    pub statistics: Statistics,
    /// The current balance of the trader.
    pub balance: f64,
    /// The strategy node that defines the trading logic.
    pub strategy: StrategyNode,
    /// The martingale strategy being used for managing trades.
    pub martingale: Option<Martingale>,
    /// The maximum loss allowed.
    pub stop_loss: Option<f64>,
    /// The minimum profit before stopping.
    pub take_profit: Option<f64>,
    /// Run only when the market is open.
    pub market_open_only: bool,
}

/// TODO: Implement methods for the Trader struct to handle trading logic, statistics updates, and martingale strategy management.
/// This function will have multiple modes, single threaded => only one trade at the time, only one asset at the time, no concurrency.
/// Multi-threaded => multiple trades at the same time, multiple assets at the same time, concurrency.
/// Also it will have a silent mode => no output, only errors, and a verbose mode => output all trades, statistics, etc.
/// And a api mode => creates a Rest API to display realtime statistics, trades, etc. and allows to handle the trader for example stop it or reset the martingale
/// Websocket mode => creates a Websocket to display realtime statistics, trades, etc. and allows to handle the trader for example stop it or reset the martingale
/// Web mode => uses the websocket mode to display the statistics, trades, etc. in a web interface with all the logs in another tab
impl<P: BinaryOptionsPlatform> Trader<P> {
    /// Creates a new Trader instance with the given platform and initial balance.
    pub fn new(platform: P, initial_balance: f64) -> Self {
        Self {
            platform,
            info: BinaryInfo::default(),
            statistics: Statistics::default(),
            balance: initial_balance,
            strategy: StrategyNode::default(),
            martingale: None,
            stop_loss: None,
            take_profit: None,
            market_open_only: true,
        }
    }
}