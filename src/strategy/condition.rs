use crate::{
    error::TaResult,
    strategy::MarketData,
    traits::{Next, Period},
    types::OutputType,
    Indicator,
};
use serde::{Deserialize, Serialize};

/// Logical conditions for strategy execution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Condition {
    /// Checks if an indicator value is greater than a threshold.
    GreaterThan {
        indicator: Indicator,
        value: OutputType,
    },
    /// Checks if an indicator value is less than a threshold.
    LessThan {
        indicator: Indicator,
        value: OutputType,
    },
    /// Checks if an indicator value equals a threshold.
    Equals {
        indicator: Indicator,
        value: OutputType,
    },
    /// Checks if an indicator value is greater than or equal to a threshold.
    GreaterThanOrEqual {
        indicator: Indicator,
        value: OutputType,
    },
    /// Checks if an indicator value is less than or equal to a threshold.
    LessThanOrEqual {
        indicator: Indicator,
        value: OutputType,
    },
    /// Detects when an indicator crosses above a threshold
    /// (previous <= threshold && current > threshold)
    CrossOver {
        indicator: Indicator,
        value: OutputType,
        #[serde(skip)]
        prev_value: Option<OutputType>,
    },
    /// Detects when an indicator crosses below a threshold
    /// (previous >= threshold && current < threshold)
    CrossUnder {
        indicator: Indicator,
        value: OutputType,
        #[serde(skip)]
        prev_value: Option<OutputType>,
    },
    /// Logical AND of multiple conditions.
    And(Vec<Condition>),
    /// Logical OR of multiple conditions.
    Or(Vec<Condition>),
    /// Logical NOT of a condition.
    Not(Box<Condition>),
}

impl Condition {
    /// Evaluate the condition against market data.
    pub fn evaluate(&mut self, data: &MarketData) -> TaResult<bool> {
        match self {
            Condition::GreaterThan { indicator, value } => {
                indicator.next(data)?.cmp_output(value, |x, y| x > y)
            }
            Condition::LessThan { indicator, value } => {
                indicator.next(data)?.cmp_output(value, |x, y| x < y)
            }
            Condition::Equals { indicator, value } => {
                indicator.next(data)?.cmp_output(value, |x, y| x == y)
            }
            Condition::GreaterThanOrEqual { indicator, value } => {
                indicator.next(data)?.cmp_output(value, |x, y| x >= y)
            }
            Condition::LessThanOrEqual { indicator, value } => {
                indicator.next(data)?.cmp_output(value, |x, y| x <= y)
            }
            Condition::CrossOver {
                indicator,
                value,
                prev_value,
            } => {
                let current = indicator.next(data)?;
                let result = match prev_value {
                    Some(prev) => {
                        prev.cmp_output(value, |x, y| x <= y)?
                            && current.cmp_output(value, |x, y| x > y)?
                    }
                    None => false, // First evaluation can't detect a crossover
                };
                *prev_value = Some(current);
                Ok(result)
            }
            Condition::CrossUnder {
                indicator,
                value,
                prev_value,
            } => {
                let current = indicator.next(data)?;
                let result = match prev_value {
                    Some(prev) => {
                        prev.cmp_output(value, |x, y| x >= y)?
                            && current.cmp_output(value, |x, y| x < y)?
                    }
                    None => false, // First evaluation can't detect a crossunder
                };
                *prev_value = Some(current);
                Ok(result)
            }
            Condition::And(conds) => {
                for c in conds.iter_mut() {
                    if !c.evaluate(data)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or(conds) => {
                for c in conds.iter_mut() {
                    if c.evaluate(data)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not(c) => Ok(!c.evaluate(data)?),
        }
    }

    /// Returns the maximum indicator period contained in this condition or `None` if no indicators.
    pub fn max_period(&self) -> Option<usize> {
        match self {
            Condition::GreaterThan { indicator, .. } | Condition::LessThan { indicator, .. } => {
                Some(indicator.period())
            }
            Condition::And(conds) | Condition::Or(conds) => {
                conds.iter().filter_map(|c| c.max_period()).max()
            }
            Condition::Not(cond) => cond.max_period(),
            Condition::Equals { indicator, .. } => Some(indicator.period()),
            Condition::GreaterThanOrEqual { indicator, .. } => Some(indicator.period()),
            Condition::LessThanOrEqual { indicator, .. } => Some(indicator.period()),
            Condition::CrossOver { indicator, .. } => Some(indicator.period()),
            Condition::CrossUnder { indicator, .. } => Some(indicator.period()),
        }
    }
}
