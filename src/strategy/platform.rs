use core::fmt;
use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::TaResult;

/// Represents the result of a binary options trade.
pub enum TradeResult {
    Win(f64),   // payout
    Lose,
    Draw,
}


#[async_trait]
pub trait BinaryOptionsPlatform {
    /// Asset type must implement Display, Clone, Eq, Hash, Send, Sync.
    type Asset: fmt::Display + Clone + PartialEq + Eq + std::hash::Hash + Send + Sync;
    /// Trade ID type.
    type TradeId: fmt::Display + Clone + PartialEq + Eq + std::hash::Hash + Send + Sync;
    /// Candle length type (must be convertible to seconds).
    type CandleLength: Copy + Clone + PartialEq + Eq + std::hash::Hash + Send + Sync;

    /// The minimum trade amount for this platform.
    const MINIMUM_TRADE_AMOUNT: f64;
    /// The maximum number of concurrent candle subscriptions.
    const MAX_CONCURRENT_SUBSCRIPTIONS: usize;

    /// Place a buy order for the given asset and amount.
    async fn buy(&self, asset: &Self::Asset, amount: f64, expiry: Self::CandleLength) -> TaResult<Self::TradeId>;

    /// Place a sell order for the given asset and amount.
    async fn sell(&self, asset: &Self::Asset, amount: f64, expiry: Self::CandleLength) -> TaResult<Self::TradeId>;

    /// Get the payout percentage for the given asset and expiry.
    async fn payout(&self, asset: &Self::Asset, expiry: Self::CandleLength) -> TaResult<f64>;

    /// Get the payout percentages for a list of assets and expiry.
    async fn payouts(&self, assets: &[Self::Asset], expiry: Self::CandleLength) -> TaResult<HashMap<Self::Asset, f64>>;

    /// Get the result of a trade by trade ID.
    async fn result(&self, trade_id: &Self::TradeId) -> TaResult<TradeResult>;

    /// Subscribe to real-time candle data for an asset and candle length.
    async fn subscribe_candles(&self, asset: &Self::Asset, candle_length: Self::CandleLength) -> TaResult<()>;

    /// Unsubscribe from real-time candle data.
    async fn unsubscribe_candles(&self, asset: &Self::Asset, candle_length: Self::CandleLength) -> TaResult<()>;

    /// List all available assets.
    async fn assets(&self) -> TaResult<Vec<Self::Asset>>;

    /// Check if an asset is currently active/tradable.
    async fn is_active(&self, asset: &Self::Asset) -> TaResult<bool>;

    /// Checks if the market is open.
    async fn is_market_open(&self) -> TaResult<bool>;

    /// Get the current account balance.
    async fn balance(&self) -> TaResult<f64>;
}