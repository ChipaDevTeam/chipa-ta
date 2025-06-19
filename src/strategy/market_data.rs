use crate::{helper_types::Bar, traits::Candle};

/// Market data passed to strategies and indicators.
/// Contains OHLCV values.
#[derive(Debug, Clone)]
pub enum MarketData {
    Bar(Bar), // Boxed trait object for dynamic dispatch
    // Add more variants as needed for other Candle implementors
    Float(f64),
}

impl MarketData {
    /// Returns the typical price ((high + low + close) / 3).
    pub fn typical_price(&self) -> f64 {
        match self {
            MarketData::Bar(bar) => bar.typical_price(),
            MarketData::Float(value) => *value,
        }
    }
}

impl Candle for MarketData {
    fn open(&self) -> f64 {
        match self {
            MarketData::Bar(bar) => bar.open(),
            MarketData::Float(value) => *value,
        }
    }

    fn close(&self) -> f64 {
        match self {
            MarketData::Bar(bar) => bar.close(),
            MarketData::Float(value) => *value,
        }
    }

    fn high(&self) -> f64 {
        match self {
            MarketData::Bar(bar) => bar.high(),
            MarketData::Float(value) => *value,
        }
    }

    fn low(&self) -> f64 {
        match self {
            MarketData::Bar(bar) => bar.low(),
            MarketData::Float(value) => *value,
        }
    }

    fn price(&self) -> f64 {
        match self {
            MarketData::Bar(bar) => bar.price(),
            MarketData::Float(value) => *value,
        }
    }

    fn volume(&self) -> f64 {
        match self {
            MarketData::Bar(bar) => bar.volume(),
            MarketData::Float(_) => f64::NAN, // Volume not applicable for Float variant
        }
    }
}
