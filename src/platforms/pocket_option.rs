use std::{collections::HashMap, sync::Arc, time::Duration};

use binary_options_tools::pocketoption::{pocket_client::PocketOption, types::update::DataCandle, ws::stream::StreamAsset};
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::{TaError, TaResult}, helper_types::Bar, platforms::PlatformError, strategy::{platform::{BinaryOptionsPlatform, TradeResult}, MarketData}};

#[derive(Default)]
pub struct PocketOptionPlatform {
    inner: Option<PocketOption>,
}

impl Serialize for PocketOptionPlatform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("PocketOption")
    }
}

impl<'de> Deserialize<'de> for PocketOptionPlatform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "PocketOption" {
            Ok(PocketOptionPlatform { inner: None })
        } else {
            Err(serde::de::Error::custom("Expected 'PocketOption'"))
        }
    }
}


impl PocketOptionPlatform {
    pub fn set_inner(&mut self, client: PocketOption) {
        self.inner = Some(client);
    }

    pub fn inner(&self) -> Option<&PocketOption> {
        self.inner.as_ref()
    }

    fn take(&self) -> Result<&PocketOption, PlatformError> {
        self.inner.as_ref().ok_or_else(|| PlatformError::NotInitialized)
    }
}

fn to_market(data: DataCandle) -> MarketData {
    let bar = Bar::new()
        .set_open(data.open)
        .set_high(data.high)
        .set_low(data.low)
        .set_close(data.close)
        .set_price((data.high + data.low + data.close) / 3.0) // Typical price
        .set_volume(0.0);
    MarketData::Bar(bar)
}


#[async_trait::async_trait]
impl BinaryOptionsPlatform for PocketOptionPlatform {
    type Asset = String; // Assuming asset is represented as a String
    type TradeId = Uuid; // Assuming trade ID is represented as a String
    type CandleLength = u32; // Assuming candle length is represented in seconds
    type Creds = String;

    const MINIMUM_TRADE_AMOUNT_USD: f64 = 1.0;
    const MAXIMUM_TRADE_AMOUNT_USD: f64 = 20_000.0;
    const MAX_CONCURRENT_SUBSCRIPTIONS: usize = 4;

    async fn initialize(&self, credentials: Self::Creds) -> TaResult<Self> {
        let client = PocketOption::new(credentials).await.map_err(PlatformError::from)?;
        let mut platform = PocketOptionPlatform::default();
        platform.set_inner(client);
        Ok(platform)
    }

    async fn buy(
        &self,
        asset: &Self::Asset,
        amount: f64,
        expiry: Self::CandleLength,
    ) -> TaResult<Self::TradeId> {
        let (trade_id, _deal) = self
            .take()?
            .buy(asset, amount, expiry)
            .await
            .map_err(PlatformError::from)?;
        Ok(trade_id)
    }

    async fn sell(
        &self,
        asset: &Self::Asset,
        amount: f64,
        expiry: Self::CandleLength,
    ) -> TaResult<Self::TradeId> {
        let (trade_id, _deal) = self
            .take()?
            .sell(asset, amount, expiry)
            .await
            .map_err(PlatformError::from)?;
        Ok(trade_id)
    }

    async fn payout(&self, asset: &Self::Asset, _expiry: Self::CandleLength) -> TaResult<f64> {
        let payout_data = self.take()?.get_payout().await;
        let payout_percentage = payout_data.get(asset).copied().unwrap_or(0) as f64; // Default to 0% if not found
        Ok(payout_percentage / 100.0) // Convert percentage to decimal
    }

    async fn payouts(
        &self,
        assets: &[Self::Asset],
        _expiry: Self::CandleLength,
    ) -> TaResult<HashMap<Self::Asset, f64>> {
        let payout_data = self.take()?.get_payout().await;
        let mut result = HashMap::new();

        for asset in assets {
            let payout_percentage = payout_data.get(asset).copied().unwrap_or(0) as f64; // Default to 0% if not found
            result.insert(asset.clone(), payout_percentage / 100.0);
        }

        Ok(result)
    }

    async fn result(&self, trade_id: &Self::TradeId) -> TaResult<TradeResult> {
        // Check if the trade is in closed deals
        let deal = self
            .take()?
            .check_results(*trade_id)
            .await
            .map_err(|e| TaError::Unexpected(e.to_string()))?;

        match deal.profit {
            profit if profit > 0.0 => Ok(TradeResult::Win(profit)),
            profit if profit < 0.0 => Ok(TradeResult::Lose),
            _ => Ok(TradeResult::Draw),
        }
    }


    async fn subscribe_candles(
        &self,
        asset: &Self::Asset,
        candle_length: Self::CandleLength,
    ) -> TaResult<impl Stream<Item = MarketData>> {
        // Implement the logic for subscribing to candles
        let stream = self.take()?.subscribe_symbol_timed(asset, Duration::from_secs(candle_length as u64)).await.map_err(PlatformError::from)?;
                // Convert the stream of DataCandle to MarketData
        let market_data_stream = StreamAsset::to_stream_static(Arc::new(stream)).map(|result| {
            result
                .map(to_market)
                .unwrap_or_else(|_| MarketData::Float(0.0)) // Fallback value
        });

        Ok(market_data_stream)

    }

    async fn unsubscribe_candles(
        &self,
        _asset: &Self::Asset,
        _candle_length: Self::CandleLength,
    ) -> TaResult<()> {
        // PocketOption doesn't have explicit unsubscribe, streams are dropped automatically
        Ok(())
    }

    async fn assets(&self) -> TaResult<Vec<Self::Asset>> {
        // For now, return a default list of common assets
        // In a real implementation, you'd fetch this from the platform
        let payout = self.take()?.get_payout().await;
        let assets: Vec<String> = payout.keys().cloned().collect();
        Ok(assets)
    }

    async fn is_active(&self, _asset: &Self::Asset) -> TaResult<bool> {
        // For testing purposes, assume all assets are active
        // In a real implementation, you might check the asset's status on the platform
        Ok(true)
    }

    async fn is_market_open(&self) -> TaResult<bool> {
        // PocketOption is typically open 24/7 for most assets
        // In a real implementation, you might check specific market hours
        Ok(true)
    }

    async fn balance(&self) -> TaResult<f64> {
        let balance_data = self.take()?.get_balance().await;
        Ok(balance_data.balance)
    }

}