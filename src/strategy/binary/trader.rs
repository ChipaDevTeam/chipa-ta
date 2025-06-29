use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tracing::{info, warn};

use crate::{
    error::{TaError, TaResult},
    strategy::{
        binary::{
            info::BinaryInfo,
            modifier::Modifier,
            trader_mut::{TraderMut, TraderMutBuilder},
        },
        platform::{BinaryOptionsPlatform, TradeResult},
        strat::Strategy,
        Action, MarketData, StrategyError,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeDirection {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTrade<P: BinaryOptionsPlatform> {
    pub asset: P::Asset,
    pub trade_id: P::TradeId,
    pub amount: f64,
    pub direction: TradeDirection,
    pub open_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum TraderMode {
    Multithreaded,
    #[default]
    Singlethreaded,
}

#[derive(Serialize, Deserialize)]
pub struct Trader<P: BinaryOptionsPlatform + Default> {
    pub platform: P,
    pub info: BinaryInfo,
    pub trade_amount: f64,
    pub max_trade_amount: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub market_open_only: bool,
    pub mode: TraderMode,
    pub strategy: Strategy,
    pub trader: TraderMut<P>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct TraderConfig {
    pub trade_amount: f64,
    pub mode: TraderMode,
    pub strategy: Strategy,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub max_trade_amount: Option<f64>,
    pub market_open_only: bool,
}

impl TraderConfig {
    /// Create a new TraderConfig with default values
    pub fn new() -> Self {
        Self {
            trade_amount: 10.0, // Default $10 trade amount
            mode: TraderMode::Singlethreaded,
            strategy: Strategy::default(),
            stop_loss: None,
            take_profit: None,
            max_trade_amount: None,
            market_open_only: true,
        }
    }

    /// Set the trade amount
    pub fn with_trade_amount(mut self, amount: f64) -> Self {
        self.trade_amount = amount;
        self
    }

    /// Set the trading mode
    pub fn with_mode(mut self, mode: TraderMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the strategy
    pub fn with_strategy(mut self, strategy: Strategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the stop loss
    pub fn with_stop_loss(mut self, stop_loss: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    /// Set the take profit
    pub fn with_take_profit(mut self, take_profit: f64) -> Self {
        self.take_profit = Some(take_profit);
        self
    }

    /// Set the maximum trade amount
    pub fn with_max_trade_amount(mut self, max_amount: f64) -> Self {
        self.max_trade_amount = Some(max_amount);
        self
    }

    /// Set whether to trade only when market is open
    pub fn with_market_open_only(mut self, market_open_only: bool) -> Self {
        self.market_open_only = market_open_only;
        self
    }

    /// Build a trader with the given platform and trader mut
    pub fn build<P: BinaryOptionsPlatform + Default + Serialize + for<'de> Deserialize<'de>>(
        self,
        platform: P,
        trader: TraderMut<P>,
    ) -> Trader<P> {
        Trader::new(platform, self, trader)
    }
}

impl<P: BinaryOptionsPlatform + Default + Serialize + for<'de> Deserialize<'de>> Trader<P> {
    /// Create a new trader with the given platform and configuration
    pub fn new(platform: P, config: TraderConfig, trader: TraderMut<P>) -> Self {
        Self {
            platform,
            info: BinaryInfo::default(),
            trade_amount: config.trade_amount,
            strategy: config.strategy,
            max_trade_amount: config.max_trade_amount,
            stop_loss: config.stop_loss,
            take_profit: config.take_profit,
            market_open_only: config.market_open_only,
            mode: config.mode,
            trader,
        }
    }

    /// Setup the platform with credentials and return a configured trader
    pub async fn setup(config: TraderConfig, credentials: P::Creds) -> TaResult<Self> {
        let platform = P::default();
        let initialized_platform = platform.initialize(credentials).await?;
        let trader_mut = TraderMut::default();
        Ok(Self::new(initialized_platform, config, trader_mut))
    }

    /// Create a builder pattern for easy trader configuration
    pub fn builder() -> TraderBuilder<P> {
        TraderBuilder::new()
    }

    /// Save the trader state to a JSON file
    pub async fn save<Q: AsRef<Path>>(&self, path: Q) -> TaResult<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| TaError::from(StrategyError::Serialization(e.to_string())))?;

        fs::write(&path, json).map_err(|e| TaError::from(StrategyError::IO(e.to_string())))?;

        info!("Trader state saved to: {}", path.as_ref().display());
        Ok(())
    }

    /// Load trader state from a JSON file
    pub async fn load<Q: AsRef<Path>>(path: Q) -> TaResult<Self> {
        let content = fs::read_to_string(&path)
            .map_err(|e| TaError::from(StrategyError::IO(e.to_string())))?;

        let trader: Self = serde_json::from_str(&content)
            .map_err(|e| TaError::from(StrategyError::Serialization(e.to_string())))?;

        info!("Trader state loaded from: {}", path.as_ref().display());
        Ok(trader)
    }

    /// Load trader state from a JSON file and initialize platform with credentials
    pub async fn load_with_credentials<Q: AsRef<Path>>(
        path: Q,
        credentials: P::Creds,
    ) -> TaResult<Self> {
        let mut trader = Self::load(path).await?;
        trader.initialize_platform(credentials).await?;
        Ok(trader)
    }

    /// Initialize the platform with new credentials
    pub async fn initialize_platform(&mut self, credentials: P::Creds) -> TaResult<()> {
        let default_platform = P::default();
        self.platform = default_platform.initialize(credentials).await?;

        // Re-initialize the trader state with the new platform
        self.trader.init(&self.platform).await?;

        info!("Platform initialized with new credentials");
        Ok(())
    }

    /// Setup platform credentials for an existing trader instance
    pub async fn setup_platform_credentials(&mut self, credentials: P::Creds) -> TaResult<()> {
        self.initialize_platform(credentials).await
    }

    /// Get current balance from the trader
    pub async fn current_balance(&self) -> TaResult<f64> {
        let balance = self
            .trader
            .balance
            .read()
            .map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))?;
        Ok(*balance)
    }

    /// Check if stop loss or take profit conditions are met
    fn should_stop_trading(&self, current_balance: f64) -> Option<String> {
        if let Some(stop_loss) = self.stop_loss {
            if current_balance <= stop_loss {
                return Some(format!(
                    "Stop loss reached: {} <= {}",
                    current_balance, stop_loss
                ));
            }
        }

        if let Some(take_profit) = self.take_profit {
            if current_balance >= take_profit {
                return Some(format!(
                    "Take profit reached: {} >= {}",
                    current_balance, take_profit
                ));
            }
        }

        None
    }

    pub async fn run(
        &mut self,
        assets: impl IntoIterator<Item = impl Into<P::Asset>>,
        candle_length: P::CandleLength,
    ) -> crate::error::TaResult<()> {
        // Initialize the trader
        let assets: Vec<_> = assets.into_iter().map(|a| a.into()).collect();
        if assets.is_empty() {
            return Err(TaError::from(StrategyError::EmptyIterator(
                "No assets provided".to_string(),
            )));
        }
        self.init(&assets).await?;
        // Start the trading loop
        match self.mode {
            TraderMode::Multithreaded => {
                info!("Starting multithreaded trading loop");
                // Here you would implement the logic for the multithreaded loop
                // For example, you could spawn tasks for each asset
            }
            TraderMode::Singlethreaded => {
                info!("Starting single-threaded trading loop");
                self.start_single_threaded_loop(
                    assets
                        .first()
                        .cloned()
                        .ok_or(TaError::from(StrategyError::EmptyIterator(
                            "Expected at least 1 asset".to_string(),
                        )))?,
                    candle_length,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn init(&mut self, assets: &[P::Asset]) -> crate::error::TaResult<()> {
        // Initialize the trader
        info!(
            "Initializing trader with assets: {:?}",
            assets.iter().map(|a| a.to_string()).collect::<Vec<_>>()
        );
        self.trader.init(&self.platform).await?;
        Ok(())
    }

    async fn start_single_threaded_loop(
        &mut self,
        asset: P::Asset,
        candle_length: P::CandleLength,
    ) -> TaResult<()> {
        // Start the single-threaded trading loop
        info!("Starting single-threaded trading loop");
        let mut strategy = self.strategy.clone();
        let mut last_result: Option<TradeResult> = None;

        loop {
            // Check if we should stop trading due to stop loss or take profit
            let current_balance = self.current_balance().await?;
            if let Some(reason) = self.should_stop_trading(current_balance) {
                warn!("Stopping trading: {}", reason);
                break;
            }

            // Check if market is open if required
            if self.market_open_only && !self.platform.is_market_open().await? {
                warn!("Market is closed, skipping trading for this candle.");
                std::thread::sleep(std::time::Duration::from_secs(60)); // Wait 1 minute
                continue;
            }

            // Fetch market data for the asset
            info!(
                "Created realtime subscription for asset: {:?}",
                asset.to_string()
            );
            let data = self
                .platform
                .subscribe_candles(&asset, candle_length)
                .await?;
            ::futures_util::pin_mut!(data);

            while let Some(candle) = data.next().await {
                // Check stop conditions again within the loop
                let current_balance = self.current_balance().await?;
                if let Some(reason) = self.should_stop_trading(current_balance) {
                    warn!("Stopping trading: {}", reason);
                    return Ok(());
                }

                // Step through the strategy
                info!("Received new candle data: {:?}", candle);
                if let Some((trade_id, result)) = self
                    .step(&asset, candle_length, candle, &mut strategy, &last_result)
                    .await?
                {
                    info!("Trade executed with ID: {}, Result: {:?}", trade_id, result);

                    // Update statistics
                    self.update_statistics(&trade_id, &result).await?;
                    last_result = Some(result);
                } else {
                    info!("No trade executed this step.");
                }
            }
        }

        Ok(())
    }

    async fn step(
        &self,
        asset: &P::Asset,
        expiry: P::CandleLength,
        data: MarketData,
        strategy: &mut Strategy,
        last_result: &Option<TradeResult>,
    ) -> crate::error::TaResult<Option<(P::TradeId, TradeResult)>> {
        if let Some(action) = strategy.evaluate(&data)? {
            info!("Evaluating action: {:?}", action);
            match action {
                Action::Buy | Action::StrongBuy => {
                    // Place a buy trade
                    let trade_amount = self.trader.modify(self.trade_amount, last_result)?;
                    let id = self.platform.buy(asset, trade_amount, expiry).await?;
                    info!(
                        "Placed buy trade for asset: {}, amount: {}, expiry: {:?}",
                        asset, trade_amount, expiry
                    );

                    {
                        // Add to statistics as opened trade
                        let mut statistics = self
                            .trader
                            .statistics
                            .write()
                            .map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))?;
                        statistics.add_opened_trade(id.clone(), Utc::now());
                    }

                    let result = self.platform.result(&id).await?;
                    info!(
                        "Trade result for buy action for asset '{}' of id '{}': {:?}",
                        asset, id, result
                    );
                    return Ok(Some((id, result)));
                }
                Action::Sell | Action::StrongSell => {
                    // Place a sell trade
                    let trade_amount = self.trader.modify(self.trade_amount, last_result)?;
                    let id = self.platform.sell(asset, trade_amount, expiry).await?;
                    info!(
                        "Placed sell trade for asset: {}, amount: {}, expiry: {:?}",
                        asset, trade_amount, expiry
                    );

                    {
                        // Add to statistics as opened trade
                        let mut statistics = self
                            .trader
                            .statistics
                            .write()
                            .map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))?;
                        statistics.add_opened_trade(id.clone(), Utc::now());
                    }

                    let result = self.platform.result(&id).await?;
                    info!(
                        "Trade result for sell action for asset '{}' of id '{}': {:?}",
                        asset, id, result
                    );
                    return Ok(Some((id, result)));
                }
                Action::Hold => {
                    info!("Holding position, no trade action taken.");
                }
            }
        } else {
            info!("No action determined by strategy, skipping trade.");
        }
        Ok(None)
    }
    // /// Main single-threaded trading loop. Call this for each new candle.
    // pub async fn on_new_candle(
    //     &self,
    //     data: &crate::strategy::MarketData,
    //     asset: &P::Asset,
    //     expiry: P::CandleLength,
    // ) -> crate::error::TaResult<()> {
    //     // Check if market is open if required
    //     if self.market_open_only && !self.platform.is_market_open().await? {
    //         warn!("Market is closed, skipping trading for this candle.");
    //         return Ok(());
    //     }
    //     // Check stop loss/take profit
    //     if let Some(sl) = self.stop_loss {
    //         if self.balance <= sl {
    //             warn!("Stop loss reached. Trading stopped.");
    //             return Ok(());
    //         }
    //     }
    //     if let Some(tp) = self.take_profit {
    //         if self.balance >= tp {
    //             info!("Take profit reached. Trading stopped.");
    //             return Ok(());
    //         }
    //     }
    //     // If there is an open trade, check result
    //     // if let Some(open_trade) = &self.open_trade {
    //     //     let trade_result = self.platform.result(&open_trade.trade_id).await?;

    //     //     // Handle result
    //     //     self.handle_trade_result(&trade_result, open_trade.amount)
    //     //         .await;
    //     //     self.open_trade = None;
    //     // }
    //     // // Only one trade at a time
    //     // if self.open_trade.is_some() {
    //     //     return Ok(());
    //     // }
    //     // Evaluate strategy
    //     let action = self.strategy.evaluate(data)?;
    //     match action {
    //         Action::Buy | Action::StrongBuy => {
    //             self.place_trade(asset, expiry, TradeDirection::Buy).await?;
    //         }
    //         Action::Sell | Action::StrongSell => {
    //             self.place_trade(asset, expiry, TradeDirection::Sell)
    //                 .await?;
    //         }
    //         Action::Hold => {
    //             info!("No trade action taken (Hold)");
    //         }
    //     }
    //     Ok(())
    // }

    // async fn place_trade(
    //     &mut self,
    //     asset: &P::Asset,
    //     expiry: P::CandleLength,
    //     direction: TradeDirection,
    // ) -> crate::error::TaResult<()> {
    //     let mut amount = self.trade_amount;
    //     // Apply modifier (e.g., Martingale)
    //     if let Some(last_result) = &self.statistics.last_trade_result {
    //         amount = self.modifier.modify(amount, last_result.clone());
    //     }
    //     if let Some(max_amt) = self.max_trade_amount {
    //         if amount > max_amt {
    //             amount = max_amt;
    //         }
    //     }
    //     if amount < P::MINIMUM_TRADE_AMOUNT_USD {
    //         warn!("Trade amount below platform minimum, skipping trade.");
    //         return Ok(());
    //     }
    //     // Place trade
    //     let trade_id = match direction {
    //         TradeDirection::Buy => self.platform.buy(asset, amount, expiry).await?,
    //         TradeDirection::Sell => self.platform.sell(asset, amount, expiry).await?,
    //     };
    //     info!(
    //         "Placed {:?} trade: asset={}, amount={}, expiry={:?}, trade_id={}",
    //         direction,
    //         asset,
    //         amount,
    //         format_args!("{:?}", expiry),
    //         trade_id
    //     );
    //     self.open_trade = Some(OpenTrade {
    //         asset: asset.clone(),
    //         trade_id,
    //         amount,
    //         direction,
    //         open_time: Utc::now(),
    //     });
    //     self.statistics.update_open_trades(1);
    //     Ok(())
    // }

    // async fn handle_trade_result(&mut self, result: &TradeResult, amount: f64) {
    //     let now = Utc::now();
    //     let profit = match result {
    //         TradeResult::Win(payout) => {
    //             let profit = payout - amount;
    //             self.balance += profit;
    //             profit
    //         }
    //         TradeResult::Lose => {
    //             self.balance -= amount;
    //             -amount
    //         }
    //         TradeResult::Draw => 0.0,
    //     };
    //     info!(
    //         "Trade result: {:?}, profit/loss: {}. New balance: {}",
    //         result, profit, self.balance
    //     );
    //     self.statistics.update_on_trade(result, profit, now);
    //     self.statistics.update_open_trades(0);
    // }

    /// Update statistics after a trade
    async fn update_statistics(&self, trade_id: &P::TradeId, result: &TradeResult) -> TaResult<()> {
        let now = Utc::now();
        let profit = match result {
            TradeResult::Win(payout) => {
                let profit = payout - self.trade_amount;
                // Update balance
                {
                    let mut balance = self
                        .trader
                        .balance
                        .write()
                        .map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))?;
                    *balance += profit;
                }
                profit
            }
            TradeResult::Lose => {
                // Update balance
                {
                    let mut balance = self
                        .trader
                        .balance
                        .write()
                        .map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))?;
                    *balance -= self.trade_amount;
                }
                -self.trade_amount
            }
            TradeResult::Draw => 0.0,
        };

        {
            let mut statistics = self
                .trader
                .statistics
                .write()
                .map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))?;
            statistics.update_on_trade(result, profit, now);
            statistics.remove_opened_trade(trade_id);
        }

        let new_balance = self.current_balance().await?;
        info!(
            "Trade result: {:?}, profit/loss: {}. New balance: {}",
            result, profit, new_balance
        );

        Ok(())
    }
}

/// Builder pattern for creating a Trader
#[derive(Default)]
pub struct TraderBuilder<P: BinaryOptionsPlatform + Default + Serialize + for<'de> Deserialize<'de>>
{
    config: TraderConfig,
    platform: Option<P>,
    trader_mut: Option<TraderMut<P>>,
    trader_mut_builder: Option<TraderMutBuilder<P>>,
}

impl<P: BinaryOptionsPlatform + Default + Serialize + for<'de> Deserialize<'de>> TraderBuilder<P> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_trade_amount(mut self, amount: f64) -> Self {
        self.config.trade_amount = amount;
        self
    }

    pub fn with_mode(mut self, mode: TraderMode) -> Self {
        self.config.mode = mode;
        self
    }

    pub fn with_strategy(mut self, strategy: Strategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    pub fn with_stop_loss(mut self, stop_loss: f64) -> Self {
        self.config.stop_loss = Some(stop_loss);
        self
    }

    pub fn with_take_profit(mut self, take_profit: f64) -> Self {
        self.config.take_profit = Some(take_profit);
        self
    }

    pub fn with_max_trade_amount(mut self, max_amount: f64) -> Self {
        self.config.max_trade_amount = Some(max_amount);
        self
    }

    pub fn with_market_open_only(mut self, market_open_only: bool) -> Self {
        self.config.market_open_only = market_open_only;
        self
    }

    pub fn with_platform(mut self, platform: P) -> Self {
        self.platform = Some(platform);
        self
    }

    pub fn with_trader_mut(mut self, trader_mut: TraderMut<P>) -> Self {
        self.trader_mut = Some(trader_mut);
        self
    }

    /// Set a pre-configured TraderMutBuilder
    pub fn with_trader_mut_builder(mut self, trader_mut_builder: TraderMutBuilder<P>) -> Self {
        self.trader_mut_builder = Some(trader_mut_builder);
        self
    }

    /// Configure TraderMut with a custom modifier
    pub fn with_modifier(mut self, modifier: Modifier) -> Self {
        if let Some(ref mut builder) = self.trader_mut_builder {
            self.trader_mut_builder = Some(std::mem::take(builder).with_modifier(modifier));
        } else {
            self.trader_mut_builder = Some(TraderMutBuilder::new().with_modifier(modifier));
        }
        self
    }

    /// Configure TraderMut with initial balance
    pub fn with_initial_balance(mut self, balance: f64) -> Self {
        if let Some(ref mut builder) = self.trader_mut_builder {
            self.trader_mut_builder = Some(std::mem::take(builder).with_balance(balance));
        } else {
            self.trader_mut_builder = Some(TraderMutBuilder::new().with_balance(balance));
        }
        self
    }

    pub async fn setup_with_credentials(mut self, credentials: P::Creds) -> TaResult<Self> {
        let platform = P::default();
        let initialized_platform = platform.initialize(credentials).await?;
        self.platform = Some(initialized_platform);
        Ok(self)
    }

    pub fn build(self) -> TaResult<Trader<P>> {
        let platform = self.platform.ok_or_else(|| {
            TaError::from(StrategyError::Configuration("Platform not set".to_string()))
        })?;

        let trader_mut = if let Some(trader_mut) = self.trader_mut {
            trader_mut
        } else if let Some(builder) = self.trader_mut_builder {
            builder.build()
        } else {
            TraderMut::default()
        };

        Ok(Trader::new(platform, self.config, trader_mut))
    }
}
