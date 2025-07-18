#[cfg(feature = "chipa_lang")]
use chipa_lang_utils::{
    errors::{LangError, LangResult},
    Lang, Pair, Rule,
};

use core::fmt;

use chipa_ta_macros::AutoImpl;
use serde::{Deserialize, Serialize};

use crate::indicators::alligator::Alligator;
use crate::indicators::ao::AwesomeOscillator;
use crate::indicators::kc::KeltnerChannel;
use crate::indicators::obv::OnBalanceVolume;
use crate::indicators::sd::StandardDeviation;
use crate::indicators::smma::SmoothedMovingAverage;
use crate::indicators::williams_r::WilliamsR;
use crate::indicators::{BollingerBands, MeanAbsoluteError, StochasticOscillator};
use crate::traits::IndicatorTrait;
use crate::types::OutputShape;
use crate::{
    error::TaResult,
    indicators::{
        AverageTrueRange, ExponentialMovingAverage, MovingAverageConvergenceDivergence,
        RelativeStrengthIndex, SimpleMovingAverage, SuperTrend, TrueRange,
    },
    traits::{Candle, Next, Period, Reset},
    types::OutputType,
};
/// A unified enum for all technical analysis indicators supported by the library.
///
/// The `Indicator` enum provides a type-safe way to work with different technical indicators
/// while maintaining serialization capabilities and consistent interfaces. Each variant wraps
/// a specific indicator implementation with its own parameters and state.
///
/// # Features
/// - **Serializable**: All indicators can be serialized to/from JSON for persistence
/// - **Trait Support**: Implements `Next`, `Reset`, and `Period` traits for consistent usage
/// - **Type Safety**: Each indicator variant enforces its specific parameter constraints
/// - **Dynamic Dispatch**: Allows runtime selection of indicators
///
/// # Usage
///
/// ## Creating Indicators
/// ```rust
/// // Create a Simple Moving Average with period 20
/// let sma = Indicator::sma(20)?;
///
/// // Create an RSI with period 14
/// let rsi = Indicator::rsi(14)?;
///
/// // Create a MACD with custom periods
/// let macd = Indicator::macd(12, 26, 9)?;
/// ```
///
/// ## Processing Data
/// ```rust
/// let mut indicator = Indicator::sma(20)?;
///
/// // Process single values
/// let result = indicator.next(100.0)?;
///
/// // Process candle data
/// let candle = SomeCandle { close: 100.0, /* other fields */ };
/// let result = indicator.next(&candle)?;
/// ```
///
/// ## Serialization
/// ```rust
/// // Serialize to JSON
/// let json = serde_json::to_string(&indicator)?;
///
/// // Deserialize from JSON
/// let restored: Indicator = serde_json::from_str(&json)?;
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AutoImpl)]
#[auto_implement(path = "src/traits.rs")]
#[auto_implement(trait = Period)]
#[auto_implement(trait = Reset)]
#[auto_implement(trait = IndicatorTrait)]
// #[auto_implement(method(from_ct_string = "from_ct_string_custom"))]
#[serde(tag = "type")]
pub enum Indicator {
    /// **None Indicator** - A pass-through indicator that returns input values unchanged.
    ///
    /// Useful as a placeholder or for testing purposes. Always returns the input value
    /// without any processing or state management.
    ///
    /// **Period**: 0 (no historical data required)
    ///
    /// **Output**: Single value (same as input)
    None(NoneIndicator),

    /// **Alligator** - A trend-following indicator with three smoothed moving averages.
    ///
    /// Consists of:
    /// - Jaw: SMMA(13, 8)
    /// - Teeth: SMMA(8, 5)
    /// - Lips: SMMA(5, 3)
    ///
    /// **Use Cases**: Identifying trends, potential reversals
    ///
    /// **Period**: User-defined (commonly 13, 8, 5)
    ///
    /// **Output**: Array [Jaw, Teeth, Lips]
    Alligator(Alligator),

    /// **Awesome Oscillator (AO)** - Momentum indicator based on moving averages.
    ///
    /// Calculates the difference between a short-term and a long-term simple moving average (SMA).
    ///
    /// **Use Cases**: Identifying bullish/bearish momentum, potential trend reversals
    ///
    /// **Period**: User-defined (commonly 5 and 34)
    ///
    /// **Output**: Single value representing the difference between SMAs
    Ao(AwesomeOscillator),

    /// **Average True Range (ATR)** - Measures market volatility.
    ///
    /// ATR calculates the average of true range values over a specified period.
    /// True Range is the maximum of:
    /// - Current High - Current Low
    /// - |Current High - Previous Close|
    /// - |Current Low - Previous Close|
    ///
    /// **Use Cases**: Volatility measurement, position sizing, stop-loss placement
    ///
    /// **Period**: User-defined (commonly 14)
    ///
    /// **Output**: Single value representing average volatility
    ///
    /// **Input Requirements**: Requires OHLC data (High, Low, Close)
    Atr(AverageTrueRange),

    /// **Bollinger Bands (BB)** - Volatility and mean reversion indicator.
    ///
    /// Consists of three lines:
    /// - Middle Band: Simple Moving Average
    /// - Upper Band: SMA + (Standard Deviation × Multiplier)
    /// - Lower Band: SMA - (Standard Deviation × Multiplier)
    ///
    /// **Use Cases**: Overbought/oversold conditions, volatility analysis, mean reversion trading
    ///
    /// **Period**: User-defined (commonly 20)
    ///
    /// **Parameters**: Period and multiplier (commonly 2.0)
    ///
    /// **Output**: Array [Middle Band, Upper Band, Lower Band]
    Bb(BollingerBands),

    /// **Exponential Moving Average (EMA)** - Trend-following indicator with recent price emphasis.
    ///
    /// Gives more weight to recent prices using an exponential smoothing factor.
    /// More responsive to recent price changes than Simple Moving Average.
    ///
    /// **Formula**: EMA = (Price × Smoothing Factor) + (Previous EMA × (1 - Smoothing Factor))
    ///
    /// **Use Cases**: Trend identification, signal generation, support/resistance levels
    ///
    /// **Period**: User-defined (commonly 12, 26, 50, 200)
    ///
    /// **Output**: Single value representing exponentially weighted average
    Ema(ExponentialMovingAverage),

    /// **Keltner Channel (KC)** - Volatility-based channel indicator.
    ///
    /// Consists of:
    /// - Upper Band: EMA + (ATR × Multiplier)
    /// - Middle Band: EMA
    /// - Lower Band: EMA - (ATR × Multiplier)
    ///
    /// **Use Cases**: Trend following, volatility breakout detection
    ///
    /// **Parameters**: Period (commonly 20), Multiplier (commonly 2.0)
    ///
    /// **Output**: Array [Upper Band, Middle Band, Lower Band]
    Kc(KeltnerChannel),

    /// **Moving Average Convergence Divergence (MACD)** - Momentum and trend-following indicator.
    ///
    /// Consists of:
    /// - MACD Line: Fast EMA - Slow EMA
    /// - Signal Line: EMA of MACD Line
    /// - Histogram: MACD Line - Signal Line
    ///
    /// **Use Cases**: Trend changes, momentum shifts, buy/sell signals
    ///
    /// **Parameters**: Fast period (12), Slow period (26), Signal period (9)
    ///
    /// **Output**: Array [MACD Line, Signal Line, Histogram]
    Macd(MovingAverageConvergenceDivergence),

    /// **Mean Absolute Error (MAE)** - Measures average prediction error magnitude.
    ///
    /// Calculates the average absolute difference between actual and predicted values.
    /// Useful for evaluating the accuracy of other indicators or models.
    ///
    /// **Formula**: MAE = Σ|Actual - Predicted| / n
    ///
    /// **Use Cases**: Model validation, indicator accuracy assessment
    ///
    /// **Period**: User-defined window for error calculation
    ///
    /// **Output**: Single value representing average absolute error
    Mae(MeanAbsoluteError),

    /// **On-Balance Volume (OBV)** - Volume-based trend indicator.
    ///
    /// Measures cumulative buying and selling pressure by adding volume on up days and subtracting volume on down days.
    ///
    /// **Use Cases**: Trend confirmation, volume analysis, divergence detection
    ///
    /// **Output**: Single value representing cumulative volume
    Obv(OnBalanceVolume),

    /// **Relative Strength Index (RSI)** - Momentum oscillator measuring speed and change.
    ///
    /// Oscillates between 0 and 100, indicating overbought (>70) and oversold (<30) conditions.
    /// Compares recent gains to recent losses over a specified period.
    ///
    /// **Formula**: RSI = 100 - (100 / (1 + RS)), where RS = Average Gain / Average Loss
    ///
    /// **Use Cases**: Overbought/oversold identification, divergence analysis, momentum confirmation
    ///
    /// **Period**: User-defined (commonly 14)
    ///
    /// **Output**: Single value between 0 and 100
    Rsi(RelativeStrengthIndex),

    /// **Standard Deviation (SD)** - Measures price volatility and dispersion.
    ///
    /// Calculates the standard deviation of prices over a specified period.
    /// Higher values indicate greater volatility and price dispersion.
    ///
    /// **Use Cases**: Volatility measurement, risk assessment, Bollinger Bands calculation
    ///
    /// **Period**: User-defined (commonly 20)
    ///
    /// **Output**: Single value representing price standard deviation
    Sd(StandardDeviation),

    /// **Simple Moving Average (SMA)** - Basic trend-following indicator.
    ///
    /// Calculates the arithmetic mean of prices over a specified period.
    /// Each price point has equal weight in the calculation.
    ///
    /// **Formula**: SMA = (P1 + P2 + ... + Pn) / n
    ///
    /// **Use Cases**: Trend identification, support/resistance levels, crossover signals
    ///
    /// **Period**: User-defined (commonly 10, 20, 50, 200)
    ///
    /// **Output**: Single value representing arithmetic average
    Sma(SimpleMovingAverage),

    /// **Smoothed Moving Average (SMMA)** - Trend-following indicator.
    ///
    /// Similar to SMA, but gives more weight to recent prices.
    ///
    /// **Formula**: SMMA = (Previous SMMA * (n - 1) + Current Price) / n
    ///
    /// **Use Cases**: Trend identification, smoothing price data
    ///
    /// **Period**: User-defined (commonly 14)
    ///
    /// **Output**: Single value representing smoothed average
    Smma(SmoothedMovingAverage),

    /// **Stochastic Oscillator (STOCH)** - Momentum indicator comparing closing price to price range.
    ///
    /// Consists of two lines:
    /// - %K Line: Raw stochastic value
    /// - %D Line: Moving average of %K (signal line)
    ///
    /// **Formula**: %K = ((Close - Lowest Low) / (Highest High - Lowest Low)) × 100
    ///
    /// **Use Cases**: Overbought/oversold conditions, momentum analysis, divergence identification
    ///
    /// **Parameters**: Period and smoothing period
    ///
    /// **Output**: Array [%K value, %D value]
    ///
    /// **Input Requirements**: Requires OHLC data
    Stoch(StochasticOscillator),

    /// **SuperTrend** - Trend-following overlay indicator.
    ///
    /// Provides dynamic support and resistance levels based on Average True Range.
    /// Shows trend direction and potential reversal points.
    ///
    /// **Components**:
    /// - Trend direction (bullish/bearish)
    /// - Support/resistance level
    ///
    /// **Use Cases**: Trend identification, entry/exit signals, stop-loss placement
    ///
    /// **Parameters**: Multiplier and ATR period
    ///
    /// **Output**: Array [SuperTrend value, Trend direction]
    ///
    /// **Input Requirements**: Requires OHLC data
    SuperTrend(SuperTrend),

    /// **True Range (TR)** - Measures single-period volatility.
    ///
    /// Calculates the true range for each period, which is the maximum of:
    /// - Current High - Current Low
    /// - |Current High - Previous Close|
    /// - |Current Low - Previous Close|
    ///
    /// **Use Cases**: Volatility analysis, ATR calculation, risk management
    ///
    /// **Period**: 1 (single period calculation)
    ///
    /// **Output**: Single value representing current period's true range
    ///
    /// **Input Requirements**: Requires OHLC data
    Tr(TrueRange),

    /// **Williams %R (WILLR)** - Momentum indicator measuring overbought/oversold conditions.
    ///
    /// **Formula**: WILLR = (Highest High - Close) / (Highest High - Lowest Low) * -100
    ///
    /// **Use Cases**: Identifying overbought/oversold levels, potential reversal points
    ///
    /// **Period**: User-defined (commonly 14)
    ///
    /// **Output**: Single value representing Williams %R
    WilliamsR(WilliamsR),
}

/// A placeholder indicator that passes through input values unchanged.
///
/// The `NoneIndicator` serves as a null object pattern implementation,
/// useful for testing, default values, or when no processing is desired.
///
/// # Characteristics
/// - **Stateless**: No internal state or memory
/// - **Zero Period**: Requires no historical data
/// - **Pass-through**: Returns input values without modification
///
/// # Example
/// ```rust
/// let mut none = NoneIndicator;
/// assert_eq!(none.next(42.0)?, 42.0);
/// assert_eq!(none.next(100.0)?, 100.0);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "chipa_lang", derive(Lang))]
#[cfg_attr(feature = "chipa_lang", ct(grammar(None())))]
pub struct NoneIndicator {}

impl NoneIndicator {
    /// Creates a new `NoneIndicator`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Period for NoneIndicator {
    fn period(&self) -> usize {
        0
    }
}

impl IndicatorTrait for NoneIndicator {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(1) // Single value output
    }
}

impl fmt::Display for NoneIndicator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "None")
    }
}

impl fmt::Display for Indicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Indicator: {}",
            match self {
                Self::None(i) => i.name(),
                Self::Alligator(i) => i.name(),
                Self::Ao(i) => i.name(),
                Self::Atr(i) => i.name(),
                Self::Bb(i) => i.name(),
                Self::Ema(i) => i.name(),
                Self::Kc(i) => i.name(),
                Self::Macd(i) => i.name(),
                Self::Mae(i) => i.name(),
                Self::Obv(i) => i.name(),
                Self::Rsi(i) => i.name(),
                Self::Sd(i) => i.name(),
                Self::Sma(i) => i.name(),
                Self::Smma(i) => i.name(),
                Self::Stoch(i) => i.name(),
                Self::SuperTrend(i) => i.name(),
                Self::Tr(i) => i.name(),
                Self::WilliamsR(i) => i.name(),
            }
        )
    }
}

impl Default for Indicator {
    fn default() -> Self {
        Self::None(NoneIndicator {})
    }
}
impl Next<f64> for Indicator {
    type Output = OutputType;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        match self {
            Self::None(indicator) => indicator.next(input).map(OutputType::from),
            Self::Alligator(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(vec![o.0, o.1, o.2])),
            Self::Ema(indicator) => indicator.next(input).map(OutputType::from),
            Self::Sma(indicator) => indicator.next(input).map(OutputType::from),
            Self::Smma(indicator) => indicator.next(input).map(OutputType::from),
            Self::Rsi(indicator) => indicator.next(input).map(OutputType::from),
            Self::Macd(indicator) => indicator
                .next(input)
                .map(|o| o.to_vec())
                .map(OutputType::from),
            Self::Tr(indicator) => indicator.next(input).map(OutputType::from),
            Self::Atr(indicator) => indicator.next(input).map(OutputType::from),
            Self::SuperTrend(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(Vec::from(o))),
            Self::Bb(indicator) => indicator
                .next(input)
                .map(|output| OutputType::Array(vec![output.average, output.upper, output.lower])),
            Self::Stoch(_) => {
                // StochasticOscillator only implements Next<&T>, not Next<f64>
                Err(crate::error::TaError::Unexpected(
                    "StochasticOscillator requires Candle input".to_string(),
                ))
            }
            Self::Sd(indicator) => indicator.next(input).map(OutputType::from),
            Self::Mae(indicator) => indicator.next(input).map(OutputType::from),
            Self::Obv(_) => {
                // OnBalanceVolume only implements Next<&T>, not Next<f64>
                Err(crate::error::TaError::Unexpected(
                    "OnBalanceVolume requires Candle input".to_string(),
                ))
            }
            Self::Ao(indicator) => indicator.next(input).map(OutputType::from),
            Self::Kc(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(Vec::from(o))),
            Self::WilliamsR(indicator) => indicator.next(input).map(OutputType::from),
        }
    }
}

// impl IndicatorTrait for Indicator {
//     fn output_shape(&self) -> OutputShape {
//         match self {
//             Self::None(indicator) => indicator.output_shape(),
//             Self::Alligator(indicator) => indicator.output_shape(),
//             Self::Ao(indicator) => indicator.output_shape(),
//             Self::Ema(indicator) => indicator.output_shape(),
//             Self::Sma(indicator) => indicator.output_shape(),
//             Self::Smma(indicator) => indicator.output_shape(),
//             Self::Rsi(indicator) => indicator.output_shape(),
//             Self::Macd(indicator) => indicator.output_shape(),
//             Self::Tr(indicator) => indicator.output_shape(),
//             Self::Atr(indicator) => indicator.output_shape(),
//             Self::SuperTrend(indicator) => indicator.output_shape(),
//             Self::Bb(indicator) => indicator.output_shape(),
//             Self::Stoch(indicator) => indicator.output_shape(),
//             Self::Sd(indicator) => indicator.output_shape(),
//             Self::Mae(indicator) => indicator.output_shape(),
//             Self::Obv(indicator) => indicator.output_shape(),
//             Self::Kc(indicator) => indicator.output_shape(),
//             Self::WilliamsR(indicator) => indicator.output_shape(),
//         }
//     }
// }

impl<T: Candle> Next<&T> for Indicator {
    type Output = OutputType;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        match self {
            Self::None(indicator) => indicator.next(input).map(OutputType::from),
            Self::Alligator(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(vec![o.0, o.1, o.2])),
            Self::Ao(indicator) => indicator.next(input).map(OutputType::from),
            Self::Ema(indicator) => indicator.next(input).map(OutputType::from),
            Self::Sma(indicator) => indicator.next(input).map(OutputType::from),
            Self::Smma(indicator) => indicator.next(input).map(OutputType::from),
            Self::Rsi(indicator) => indicator.next(input).map(OutputType::from),
            Self::Macd(indicator) => indicator
                .next(input)
                .map(|o| o.to_vec())
                .map(OutputType::from),
            Self::Tr(indicator) => indicator.next(input).map(OutputType::from),
            Self::Atr(indicator) => indicator.next(input).map(OutputType::from),
            Self::SuperTrend(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(Vec::from(o))),
            Self::Bb(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(vec![o.average, o.upper, o.lower])),
            Self::Stoch(indicator) => indicator
                .next(input)
                .map(|(k, d)| OutputType::Array(vec![k, d])),
            Self::Sd(indicator) => indicator.next(input).map(OutputType::from),
            Self::Mae(indicator) => indicator.next(input).map(OutputType::from),
            Self::Obv(indicator) => indicator.next(input).map(OutputType::from),
            Self::Kc(indicator) => indicator
                .next(input)
                .map(|o| OutputType::Array(Vec::from(o))),
            Self::WilliamsR(indicator) => indicator.next(input).map(OutputType::from),
        }
    }
}

// impl Reset for Indicator {
//     fn reset(&mut self) {
//         match self {
//             Self::None(indicator) => indicator.reset(),
//             Self::Alligator(indicator) => indicator.reset(),
//             Self::Ao(indicator) => indicator.reset(),
//             Self::Ema(indicator) => indicator.reset(),
//             Self::Sma(indicator) => indicator.reset(),
//             Self::Smma(indicator) => indicator.reset(),
//             Self::Rsi(indicator) => indicator.reset(),
//             Self::Macd(indicator) => indicator.reset(),
//             Self::Tr(indicator) => indicator.reset(),
//             Self::Atr(indicator) => indicator.reset(),
//             Self::SuperTrend(indicator) => indicator.reset(),
//             Self::Bb(indicator) => indicator.reset(),
//             Self::Stoch(indicator) => indicator.reset(),
//             Self::Sd(indicator) => indicator.reset(),
//             Self::Mae(indicator) => indicator.reset(),
//             Self::Obv(indicator) => indicator.reset(),
//             Self::Kc(indicator) => indicator.reset(),
//             Self::WilliamsR(indicator) => indicator.reset(),
//         }
//     }
// }

// impl Period for Indicator {
//     fn period(&self) -> usize {
//         match self {
//             Self::None(indicator) => indicator.period(),
//             Self::Alligator(indicator) => indicator.period(),
//             Self::Ao(indicator) => indicator.period(),
//             Self::Ema(indicator) => indicator.period(),
//             Self::Sma(indicator) => indicator.period(),
//             Self::Smma(indicator) => indicator.period(),
//             Self::Rsi(indicator) => indicator.period(),
//             Self::Macd(indicator) => indicator.period(),
//             Self::Tr(indicator) => indicator.period(),
//             Self::Atr(indicator) => indicator.period(),
//             Self::SuperTrend(indicator) => indicator.period(),
//             Self::Bb(indicator) => indicator.period(),
//             Self::Stoch(indicator) => indicator.period(),
//             Self::Sd(indicator) => indicator.period(),
//             Self::Mae(indicator) => indicator.period(),
//             Self::Obv(indicator) => indicator.period(),
//             Self::Kc(indicator) => indicator.period(),
//             Self::WilliamsR(indicator) => indicator.period(),
//         }
//     }
// }

#[cfg(feature = "chipa_lang")]
impl Lang for Indicator {
    fn from_ct(input: &str) -> LangResult<Self> {
        match input {
            _ if input.starts_with("None") => Ok(Indicator::none()),
            _ if input.starts_with("Alligator") => {
                Alligator::from_ct(input).map(Indicator::Alligator)
            }
            _ if input.starts_with("Ao") => AwesomeOscillator::from_ct(input).map(Indicator::Ao),
            _ if input.starts_with("Atr") => AverageTrueRange::from_ct(input).map(Indicator::Atr),
            _ if input.starts_with("Bb") => BollingerBands::from_ct(input).map(Indicator::Bb),
            _ if input.starts_with("Ema") => {
                ExponentialMovingAverage::from_ct(input).map(Indicator::Ema)
            }
            _ if input.starts_with("Kc") => KeltnerChannel::from_ct(input).map(Indicator::Kc),
            _ if input.starts_with("Macd") => {
                MovingAverageConvergenceDivergence::from_ct(input).map(Indicator::Macd)
            }
            _ if input.starts_with("Mae") => MeanAbsoluteError::from_ct(input).map(Indicator::Mae),
            _ if input.starts_with("Obv") => OnBalanceVolume::from_ct(input).map(Indicator::Obv),
            _ if input.starts_with("Rsi") => {
                RelativeStrengthIndex::from_ct(input).map(Indicator::Rsi)
            }
            _ if input.starts_with("Sd") => StandardDeviation::from_ct(input).map(Indicator::Sd),
            _ if input.starts_with("Sma") => {
                SimpleMovingAverage::from_ct(input).map(Indicator::Sma)
            }
            _ if input.starts_with("Smma") => {
                SmoothedMovingAverage::from_ct(input).map(Indicator::Smma)
            }
            _ if input.starts_with("Stoch") => {
                StochasticOscillator::from_ct(input).map(Indicator::Stoch)
            }
            _ if input.starts_with("SuperTrend") => {
                SuperTrend::from_ct(input).map(Indicator::SuperTrend)
            }
            _ if input.starts_with("Tr") => TrueRange::from_ct(input).map(Indicator::Tr),
            _ if input.starts_with("WilliamsR") => {
                WilliamsR::from_ct(input).map(Indicator::WilliamsR)
            }
            _ => Err(LangError::ParseError(format!(
                "Unknown indicator type: {input}"
            ))),
        }
    }

    fn from_pair(pair: Pair<Rule>) -> LangResult<Self> {
        match pair.as_rule() {
            Rule::None => Ok(Indicator::none()),
            Rule::Alligator => Alligator::from_pair(pair).map(Indicator::Alligator),
            Rule::Ao => AwesomeOscillator::from_pair(pair).map(Indicator::Ao),
            Rule::Atr => AverageTrueRange::from_pair(pair).map(Indicator::Atr),
            Rule::Bb => BollingerBands::from_pair(pair).map(Indicator::Bb),
            Rule::Ema => ExponentialMovingAverage::from_pair(pair).map(Indicator::Ema),
            Rule::Kc => KeltnerChannel::from_pair(pair).map(Indicator::Kc),
            Rule::Macd => MovingAverageConvergenceDivergence::from_pair(pair).map(Indicator::Macd),
            Rule::Mae => MeanAbsoluteError::from_pair(pair).map(Indicator::Mae),
            Rule::Obv => OnBalanceVolume::from_pair(pair).map(Indicator::Obv),
            Rule::Rsi => RelativeStrengthIndex::from_pair(pair).map(Indicator::Rsi),
            Rule::Sd => StandardDeviation::from_pair(pair).map(Indicator::Sd),
            Rule::Sma => SimpleMovingAverage::from_pair(pair).map(Indicator::Sma),
            Rule::Smma => SmoothedMovingAverage::from_pair(pair).map(Indicator::Smma),
            Rule::Stoch => StochasticOscillator::from_pair(pair).map(Indicator::Stoch),
            Rule::SuperTrend => SuperTrend::from_pair(pair).map(Indicator::SuperTrend),
            Rule::Tr => TrueRange::from_pair(pair).map(Indicator::Tr),
            Rule::WilliamsR => WilliamsR::from_pair(pair).map(Indicator::WilliamsR),
            _ => Err(LangError::ParseError(format!(
                "Unexpected rule for Indicator: {:?}",
                pair.as_rule()
            ))),
        }
    }

    fn to_ct(&self) -> String {
        match self {
            Self::None(_) => "None()".to_string(),
            Self::Alligator(indicator) => indicator.to_ct(),
            Self::Ao(indicator) => indicator.to_ct(),
            Self::Atr(indicator) => indicator.to_ct(),
            Self::Bb(indicator) => indicator.to_ct(),
            Self::Ema(indicator) => indicator.to_ct(),
            Self::Kc(indicator) => indicator.to_ct(),
            Self::Macd(indicator) => indicator.to_ct(),
            Self::Mae(indicator) => indicator.to_ct(),
            Self::Obv(indicator) => indicator.to_ct(),
            Self::Rsi(indicator) => indicator.to_ct(),
            Self::Sd(indicator) => indicator.to_ct(),
            Self::Sma(indicator) => indicator.to_ct(),
            Self::Smma(indicator) => indicator.to_ct(),
            Self::Stoch(indicator) => indicator.to_ct(),
            Self::SuperTrend(indicator) => indicator.to_ct(),
            Self::Tr(indicator) => indicator.to_ct(),
            Self::WilliamsR(indicator) => indicator.to_ct(),
        }
    }
}

impl Reset for NoneIndicator {
    fn reset(&mut self) {}
}

impl Next<f64> for NoneIndicator {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        Ok(input)
    }
}

impl<T: Candle> Next<&T> for NoneIndicator {
    type Output = f64;

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        self.next(input.close())
    }
}

impl Indicator {
    // fn from_ct_string_custom(s: &str) -> TaResult<Self> {
    //     // Custom parsing logic for Chipa Trade language
    //     // This is a placeholder; actual implementation will depend on the language syntax
    //     Err(crate::error::TaError::Unexpected(
    //         "Custom from_ct_string not implemented".to_string(),
    //     ))
    // }

    /// Creates a new None indicator (pass-through).
    ///
    /// Returns an indicator that simply passes input values through unchanged.
    /// Useful as a placeholder or for testing purposes.
    ///
    /// # Example
    /// ```rust
    /// let indicator = Indicator::none();
    /// ```
    pub fn none() -> Self {
        Self::None(NoneIndicator {})
    }

    /// Creates a new Exponential Moving Average indicator.
    ///
    /// EMA gives more weight to recent prices, making it more responsive
    /// to price changes than a Simple Moving Average.
    ///
    /// # Arguments
    /// * `period` - Number of periods for the EMA calculation (must be > 0)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created EMA indicator
    /// * `Err(TaError)` - If period is 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let ema = Indicator::ema(14)?; // 14-period EMA
    /// ```
    pub fn ema(period: usize) -> TaResult<Self> {
        Ok(Self::Ema(ExponentialMovingAverage::new(period)?))
    }

    /// Creates a new Simple Moving Average indicator.
    ///
    /// SMA calculates the arithmetic mean of prices over the specified period.
    /// All price points have equal weight in the calculation.
    ///
    /// # Arguments
    /// * `period` - Number of periods for the SMA calculation (must be > 0)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created SMA indicator
    /// * `Err(TaError)` - If period is 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let sma = Indicator::sma(20)?; // 20-period SMA
    /// ```
    pub fn sma(period: usize) -> TaResult<Self> {
        Ok(Self::Sma(SimpleMovingAverage::new(period)?))
    }

    /// Creates a new Relative Strength Index indicator.
    ///
    /// RSI is a momentum oscillator that measures the speed and change
    /// of price movements, oscillating between 0 and 100.
    ///
    /// # Arguments
    /// * `period` - Number of periods for RSI calculation (must be > 0, commonly 14)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created RSI indicator
    /// * `Err(TaError)` - If period is 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let rsi = Indicator::rsi(14)?; // 14-period RSI
    /// ```
    pub fn rsi(period: usize) -> TaResult<Self> {
        Ok(Self::Rsi(RelativeStrengthIndex::new(period)?))
    }

    /// Creates a new MACD (Moving Average Convergence Divergence) indicator.
    ///
    /// MACD consists of the MACD line (fast EMA - slow EMA), signal line
    /// (EMA of MACD line), and histogram (MACD line - signal line).
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast EMA (commonly 12)
    /// * `slow_period` - Period for slow EMA (commonly 26, must be > fast_period)
    /// * `signal_period` - Period for signal line EMA (commonly 9)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created MACD indicator
    /// * `Err(TaError)` - If periods are invalid or slow_period <= fast_period
    ///
    /// # Example
    /// ```rust
    /// let macd = Indicator::macd(12, 26, 9)?; // Standard MACD(12,26,9)
    /// ```
    pub fn macd(fast_period: usize, slow_period: usize, signal_period: usize) -> TaResult<Self> {
        Ok(Self::Macd(MovingAverageConvergenceDivergence::new(
            fast_period,
            slow_period,
            signal_period,
        )?))
    }

    /// Creates a new True Range indicator.
    ///
    /// True Range measures the volatility of a single period by calculating
    /// the maximum of three price differences.
    ///
    /// # Returns
    /// A True Range indicator (never fails)
    ///
    /// # Example
    /// ```rust
    /// let tr = Indicator::tr(); // True Range indicator
    /// ```
    pub fn tr() -> Self {
        Self::Tr(TrueRange::new())
    }

    /// Creates a new Average True Range indicator.
    ///
    /// ATR smooths the True Range values over a specified period,
    /// providing a measure of average volatility.
    ///
    /// # Arguments
    /// * `period` - Number of periods for ATR calculation (must be > 0, commonly 14)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created ATR indicator
    /// * `Err(TaError)` - If period is 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let atr = Indicator::atr(14)?; // 14-period ATR
    /// ```
    pub fn atr(period: usize) -> TaResult<Self> {
        Ok(Self::Atr(AverageTrueRange::new(period)?))
    }

    /// Creates a new SuperTrend indicator.
    ///
    /// SuperTrend is a trend-following indicator that provides dynamic
    /// support and resistance levels based on Average True Range.
    ///
    /// # Arguments
    /// * `multiplier` - Multiplier for ATR calculation (commonly 2.0-3.0)
    /// * `period` - Period for ATR calculation (commonly 10-14)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created SuperTrend indicator
    /// * `Err(TaError)` - If period is 0 or parameters are invalid
    ///
    /// # Example
    /// ```rust
    /// let supertrend = Indicator::super_trend(3.0, 10)?; // SuperTrend(3.0, 10)
    /// ```
    pub fn super_trend(multiplier: f64, period: usize) -> TaResult<Self> {
        Ok(Self::SuperTrend(SuperTrend::new(multiplier, period)?))
    }

    /// Creates a new Bollinger Bands indicator.
    ///
    /// Bollinger Bands consist of a middle band (SMA) and upper/lower bands
    /// that are standard deviations away from the middle band.
    ///
    /// # Arguments
    /// * `period` - Period for SMA and standard deviation calculation (commonly 20)
    /// * `k` - Number of standard deviations for bands (commonly 2.0)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created Bollinger Bands indicator
    /// * `Err(TaError)` - If period is 0 or parameters are invalid
    ///
    /// # Example
    /// ```rust
    /// let bb = Indicator::bb(20, 2.0)?; // Bollinger Bands(20, 2.0)
    /// ```
    pub fn bb(period: usize, k: f64) -> TaResult<Self> {
        Ok(Self::Bb(BollingerBands::new(period, k)?))
    }

    /// Creates a new Stochastic Oscillator indicator.
    ///
    /// Stochastic Oscillator compares the closing price to the price range
    /// over a specified period, generating %K and %D values.
    ///
    /// # Arguments
    /// * `period` - Period for %K calculation (commonly 14)
    /// * `smoothing_period` - Period for %D smoothing (commonly 3)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created Stochastic Oscillator
    /// * `Err(TaError)` - If periods are 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let stoch = Indicator::stoch(14, 3)?; // Stochastic(14, 3)
    /// ```
    pub fn stoch(period: usize, smoothing_period: usize) -> TaResult<Self> {
        Ok(Self::Stoch(StochasticOscillator::new(
            period,
            smoothing_period,
        )?))
    }

    /// Creates a new Mean Absolute Error indicator.
    ///
    /// MAE measures the average magnitude of errors between actual
    /// and predicted values over a specified period.
    ///
    /// # Arguments
    /// * `period` - Period for error calculation (must be > 0)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created MAE indicator
    /// * `Err(TaError)` - If period is 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let mae = Indicator::mae(10)?; // 10-period MAE
    /// ```
    pub fn mae(period: usize) -> TaResult<Self> {
        Ok(Self::Mae(MeanAbsoluteError::new(period)?))
    }

    /// Creates a new Standard Deviation indicator.
    ///
    /// Standard Deviation measures the amount of variation or dispersion
    /// of prices from their average value over a specified period.
    ///
    /// # Arguments
    /// * `period` - Period for standard deviation calculation (must be > 0)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created Standard Deviation indicator
    /// * `Err(TaError)` - If period is 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let sd = Indicator::sd(20)?; // 20-period Standard Deviation
    /// ```
    pub fn sd(period: usize) -> TaResult<Self> {
        Ok(Self::Sd(StandardDeviation::new(period)?))
    }

    /// Creates a new Keltner Channel indicator.
    ///
    /// Keltner Channel is a volatility-based envelope set above and below an
    /// exponential moving average (EMA). The distance from the EMA is based on
    /// the Average True Range (ATR).
    ///
    /// # Arguments
    /// * `period` - Period for EMA and ATR calculation (commonly 20)
    /// * `multiplier` - Multiplier for ATR (commonly 2.0)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created Keltner Channel indicator
    /// * `Err(TaError)` - If period is 0 or parameters are invalid
    ///
    /// # Example
    /// ```rust
    /// let kc = Indicator::kc(20, 2.0)?; // Keltner Channel(20, 2.0)
    /// ```
    pub fn kc(period: usize, multiplier: f64) -> TaResult<Self> {
        Ok(Self::Kc(KeltnerChannel::new(period, multiplier)?))
    }

    /// Creates a new On-Balance Volume indicator.
    /// On-Balance Volume (OBV) is a volume-based indicator that uses volume flow
    /// to predict price movements. It adds volume on up days and subtracts volume
    /// on down days, creating a cumulative volume line.
    ///
    /// # Arguments
    ///
    /// # Returns
    /// * `Indicator` - Successfully created OBV indicator
    ///
    /// # Example
    /// ```rust
    /// let obv = Indicator::obv();
    /// ```
    pub fn obv() -> Self {
        Self::Obv(OnBalanceVolume::new())
    }

    /// Creates a new Awesome Oscillator indicator.
    /// Awesome Oscillator (AO) is a momentum indicator that calculates the difference
    /// between a short-term and a long-term simple moving average (SMA).
    ///
    /// # Arguments
    /// * `short_period` - Period for the short-term SMA
    /// * `long_period` - Period for the long-term SMA
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created Awesome Oscillator
    /// * `Err(TaError)` - If periods are 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let ao = Indicator::ao(5, 34)?;
    /// ```
    pub fn ao(short_period: usize, long_period: usize) -> TaResult<Self> {
        Ok(Self::Ao(AwesomeOscillator::new(short_period, long_period)?))
    }

    /// Creates a new Williams %R indicator.
    /// Williams %R is a momentum indicator that measures overbought and oversold levels.
    ///
    /// # Arguments
    /// * `period` - Period for calculation (must be > 0)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created Williams %R indicator
    /// * `Err(TaError)` - If period is 0 or invalid
    ///
    /// # Example
    /// ```rust
    /// let williams_r = Indicator::williams_r(14)?;
    /// ```
    pub fn williams_r(period: usize) -> TaResult<Self> {
        Ok(Self::WilliamsR(WilliamsR::new(period)?))
    }

    /// Creates a new Smoothed Moving Average indicator.
    /// Smoothed Moving Average (SMMA) is similar to SMA but gives more weight to recent prices.
    ///
    /// # Arguments
    /// * `period` - Period for calculation (must be > 1)
    ///
    /// # Returns
    /// * `Ok(Indicator)` - Successfully created SMMA indicator
    /// * `Err(TaError)` - If period is 0, 1 or invalid
    ///
    /// # Example
    /// ```rust
    /// let smma = Indicator::smma(14)?;
    /// ```
    pub fn smma(period: usize) -> TaResult<Self> {
        Ok(Self::Smma(SmoothedMovingAverage::new(period)?))
    }
}

#[cfg(test)]
mod tests {}
