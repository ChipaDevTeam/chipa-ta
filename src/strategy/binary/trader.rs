use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::{TaError, TaResult}, strategy::{
    binary::{
        info::BinaryInfo,
        trader_mut::TraderMut,
    }, platform::{BinaryOptionsPlatform, TradeResult}, strat::Strategy, Action, MarketData, StrategyError
}};

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



#[derive(Default)]
pub struct TraderConfig {
    pub trade_amount: f64,
    pub mode: TraderMode,
    pub strategy: Strategy,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub max_trade_amount: Option<f64>,
}

impl<P: BinaryOptionsPlatform + Default> Trader<P> {
    pub fn new(
        platform: P,
        config: TraderConfig,
        trader: TraderMut<P>,
    ) -> Self {
        Self {
            platform,
            info: BinaryInfo::default(),
            trade_amount: config.trade_amount,
            strategy: config.strategy,
            max_trade_amount: config.max_trade_amount,
            stop_loss: config.stop_loss,
            take_profit: config.take_profit,
            market_open_only: true,
            mode: config.mode,
            trader,
        }
    }


    pub async fn run(&mut self, assets: impl IntoIterator<Item = impl Into<P::Asset>>, candle_length: P::CandleLength) -> crate::error::TaResult<()> {
        // Initialize the trader
        let assets: Vec<_> = assets.into_iter().map(|a| a.into()).collect();
        if assets.is_empty() {
            return Err(TaError::from(StrategyError::EmptyIterator("No assets provided".to_string())));
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
                self.start_single_threaded_loop(assets.first().cloned().ok_or(TaError::from(StrategyError::EmptyIterator("Expected at least 1 asset".to_string())))?, candle_length).await?;
            }
        }
        Ok(())
    }

    async fn init(&mut self, assets: &[P::Asset]) -> crate::error::TaResult<()> {
        // Initialize the trader
        info!("Initializing trader with assets: {:?}", assets.iter().map(|a| a.to_string()).collect::<Vec<_>>());
        self.trader.init(&self.platform).await?;
        Ok(())
    }

    async fn start_single_threaded_loop(&mut self, asset: P::Asset, candle_length: P::CandleLength) -> TaResult<()> {
        // Start the single-threaded trading loop
        info!("Starting single-threaded trading loop");
        // Here you would implement the logic for the single-threaded loop
        let mut strategy = self.strategy.clone();
        let mut last_result: Option<TradeResult> = None;

        loop {
            // Fetch market data for the asset
            info!("Created realtime subscription for asset: {:?}", asset.to_string());
            let data = self.platform.subscribe_candles(&asset, candle_length).await?;
            ::futures_util::pin_mut!(data);
            while let Some(candle) = data.next().await {
                // Step through the strategy
                info!("Received new candle data: {:?}", candle);
                if let Some((trade_id, result)) = self.step(&asset, candle_length, candle, &mut strategy, &last_result).await? {
                    info!("Trade executed with ID: {}, Result: {:?}", trade_id, result);
                    last_result = Some(result);
                } else {
                    info!("No trade executed this step.");
                }
            }

        }   

    }

    async fn step(&self, asset: &P::Asset, expiry: P::CandleLength, data: MarketData, strategy: &mut Strategy, last_result: &Option<TradeResult>) -> crate::error::TaResult<Option<(P::TradeId, TradeResult)>> {
        if let Some(action) = strategy.evaluate(&data)? {
            info!("Evaluating action: {:?}", action);
            match action {
                Action::Buy | Action::StrongBuy => {
                    // Place a buy trade
                    let id = self.platform.buy(asset, self.trader.modify(self.trade_amount, last_result)?, expiry).await?;
                    info!("Placed buy trade for asset: {}, amount: {}, expiry: {:?}", asset, self.trade_amount, expiry);

                    let result = self.platform.result(&id).await?;
                    info!("Trade result for buy action for asset '{}' of id '{}': {:?}", asset, id, result);
                    return Ok(Some((id, result)));
                }
                Action::Sell | Action::StrongSell => {
                    // Place a sell trade
                    let id = self.platform.sell(asset, self.trader.modify(self.trade_amount, last_result)?, expiry).await?;
                    info!("Placed buy trade for asset: {}, amount: {}, expiry: {:?}", asset, self.trade_amount, expiry);

                    let result = self.platform.result(&id).await?;
                    info!("Trade result for buy action for asset '{}' of id '{}': {:?}", asset, id, result);
                    return Ok(Some((id, result)));
                }
                Action::Hold => {
                    info!("Holding position, no trade action taken.");
                }
            }
        } else {
            info!("No action determined by strategy, skipping trade.");
        }
        // This is where you would implement the logic for a single step in the trading loop
        // For example, you could call self.trader.on_new_candle(data, asset, expiry).await?;
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
}
