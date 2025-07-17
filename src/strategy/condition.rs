use crate::{
    error::{TaError, TaResult},
    strategy::{wrapper::IndicatorState, MarketData, StrategyError},
    traits::{IndicatorTrait, Period, Reset},
    types::OutputType,
    Indicator,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Operator {
    GreaterThan,
    LessThan,
    Equals,
    GreaterThanOrEqual,
    LessThanOrEqual,
    CrossOver(#[serde(skip)] Option<OutputType>),
    CrossUnder(#[serde(skip)] Option<OutputType>),
}

/// Logical conditions for strategy execution.
/// This enum represents various conditions that can be used to control the flow of a trading strategy.
/// It includes comparisons between indicators and values, logical operations (AND, OR, NOT), and allows for complex condition trees.
/// # Methods
/// 
/// - `validate(&self) -> TaResult<()>`  
///   Validates the condition, ensuring that indicator periods are valid and output shapes are compatible.
/// 
/// - `evaluate(&mut self, data: &MarketData) -> TaResult<bool>`  
///   Evaluates the condition against provided market data, returning whether the condition is met.
/// 
/// - `max_period(&self) -> Option<usize>`  
///   Returns the maximum indicator period contained in the condition, or `None` if there are no indicators.
/// 
/// - `ne(condition: Condition) -> Condition`  
///   Constructs a negated (`Not`) condition.
/// 
/// - `and(conditions: Vec<Condition>) -> Condition`  
///   Constructs a logical `And` condition from a list of conditions.
/// 
/// - `or(conditions: Vec<Condition>) -> Condition`  
///   Constructs a logical `Or` condition from a list of conditions.
/// 
/// - `value(indicator: Indicator, value: OutputType, operator: Operator) -> Condition`  
///   Constructs a value-based condition comparing an indicator to a value using the specified operator.
/// 
/// - `indicator(left: Indicator, right: Indicator, operator: Operator) -> Condition`  
///   Constructs a condition comparing two indicators using the specified operator.
/// 
/// - `greater_than(indicator: Indicator, value: OutputType) -> Condition`  
///   Constructs a condition checking if an indicator is greater than a value.
/// 
/// - `less_than(indicator: Indicator, value: OutputType) -> Condition`  
///   Constructs a condition checking if an indicator is less than a value.
/// 
/// - `equals(indicator: Indicator, value: OutputType) -> Condition`  
///   Constructs a condition checking if an indicator equals a value.
/// 
/// - `greater_than_or_equal(indicator: Indicator, value: OutputType) -> Condition`  
///   Constructs a condition checking if an indicator is greater than or equal to a value.
/// 
/// - `less_than_or_equal(indicator: Indicator, value: OutputType) -> Condition`  
///   Constructs a condition checking if an indicator is less than or equal to a value.
/// 
/// - `cross_over(indicator: Indicator, value: OutputType) -> Condition`  
///   Constructs a condition checking if an indicator crosses over a value.
/// 
/// - `cross_under(indicator: Indicator, value: OutputType) -> Condition`  
///   Constructs a condition checking if an indicator crosses under a value.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Condition {
    /// Compares an indicator to a value using an operator.
    Value {
        indicator: Box<IndicatorState>,
        value: OutputType,
        operator: Operator,
    },
    /// Compares two indicators using an operator.
    Indicator {
        left: Box<IndicatorState>,
        right: Box<IndicatorState>,
        operator: Operator,
    },
    /// Logical AND of multiple conditions.
    And(Vec<Condition>),
    /// Logical OR of multiple conditions.
    Or(Vec<Condition>),
    /// Logical NOT of a condition.
    Not(Box<Condition>),
}

impl Condition {
    /// Validate the condition
    pub fn validate(&self) -> TaResult<()> {
        match self {
            Condition::Value { indicator, value, .. } => {
                if indicator.period() == 0 {
                    return Err(TaError::Strategy(StrategyError::InvalidIndicatorPeriod { period: 0 }));
                }
                if indicator.output_shape() != value.output_shape()? {
                    return Err(TaError::Strategy(StrategyError::IncompatibleShapes {
                        name: "Condition::Value".to_string(),
                        indicator: indicator.output_shape(),
                        value: value.output_shape()?,
                    }));
                }
                Ok(())
            },
            Condition::Indicator { left, right, .. } => {
                if left.period() == 0 || right.period() == 0 {
                    return Err(TaError::Strategy(StrategyError::InvalidIndicatorPeriod { period: 0 }));
                }
                if left.output_shape() != right.output_shape() {
                    return Err(TaError::Strategy(StrategyError::IncompatibleShapes {
                        name: "Condition::Indicator".to_string(),
                        indicator: left.output_shape(),
                        value: right.output_shape(),
                    }));
                }
                Ok(())
            }
            Condition::And(conds) | Condition::Or(conds) => {
                for c in conds {
                    c.validate()?;
                }
                Ok(())
            }
            Condition::Not(c) => c.validate(),
        }
    }

    pub fn update(&mut self, data: &MarketData) -> TaResult<()> {
        match self {
            Condition::Value { indicator, .. } => indicator.update(data),
            Condition::Indicator { left, right, .. } => {
                left.update(data)?;
                right.update(data)?;
                Ok(())
            }
            Condition::And(conds) | Condition::Or(conds) => {
                for c in conds.iter_mut() {
                    c.update(data)?;
                }
                Ok(())
            }
            Condition::Not(c) => c.update(data),
        }
    }

    /// Evaluate the condition against market data.
    pub fn evaluate(&mut self, data: &MarketData) -> TaResult<bool> {
        match self {
            // Condition::GreaterThan { indicator, value } => {
            //     let lhs = indicator.next(data)?;
            //     let rhs = value.resolve(data)?;
            //     Ok(lhs.gt(&rhs))
            // }
            Condition::Value { indicator, value, operator } => {
                let lhs = indicator.prev()?;
                let rhs = value.resolve(data)?;
                operator.evaluate(&lhs, &rhs)
            },
            Condition::Indicator { left, right, operator } => {
                let lhs = left.prev()?;
                let rhs = right.prev()?;
                operator.evaluate(&lhs, &rhs)
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
            Condition::Value { indicator, .. } => Some(indicator.period()),
            Condition::Indicator { left, right, .. } => Some(left.period().max(right.period())),
            Condition::And(conds) | Condition::Or(conds) => {
                conds.iter().filter_map(|c| c.max_period()).max()
            }
            Condition::Not(cond) => cond.max_period(),
        }
    }

    pub fn ne(condition: Condition) -> Condition {
        Condition::Not(Box::new(condition))
    }

    pub fn and(conditions: Vec<Condition>) -> Condition {
        Condition::And(conditions)
    }

    pub fn or(conditions: Vec<Condition>) -> Condition {
        Condition::Or(conditions)
    }

    pub fn value(indicator: Indicator, value: OutputType, operator: Operator) -> Condition {
        Condition::Value { indicator: Box::new(indicator.into()), value, operator }
    }

    pub fn indicator(left: Indicator, right: Indicator, operator: Operator) -> Condition {
        Condition::Indicator { left: Box::new(left.into()), right: Box::new(right.into()), operator }
    }

    pub fn greater_than(indicator: Indicator, value: OutputType) -> Condition {
        Condition::value(indicator, value, Operator::GreaterThan)
    }

    pub fn less_than(indicator: Indicator, value: OutputType) -> Condition {
        Condition::value(indicator, value, Operator::LessThan)
    }

    pub fn equals(indicator: Indicator, value: OutputType) -> Condition {
        Condition::value(indicator, value, Operator::Equals)
    }

    pub fn greater_than_or_equal(indicator: Indicator, value: OutputType) -> Condition {
        Condition::value(indicator, value, Operator::GreaterThanOrEqual)
    }

    pub fn less_than_or_equal(indicator: Indicator, value: OutputType) -> Condition {
        Condition::value(indicator, value, Operator::LessThanOrEqual)
    }

    pub fn cross_over(indicator: Indicator, value: OutputType) -> Condition {
        Condition::value(indicator, value, Operator::CrossOver(None))
    }

    pub fn cross_under(indicator: Indicator, value: OutputType) -> Condition {
        Condition::value(indicator, value, Operator::CrossUnder(None))
    }
}

impl Operator {
    /// Returns the operator as a string.
    pub fn as_str(&self) -> &str {
        match self {
            Operator::GreaterThan => ">",
            Operator::LessThan => "<",
            Operator::Equals => "==",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThanOrEqual => "<=",
            Operator::CrossOver(_) => "crossover",
            Operator::CrossUnder(_) => "crossunder",
        }
    }

    /// Evaluates the result of the operator on two values.
    pub fn evaluate(&mut self, lhs: &OutputType, rhs: &OutputType) -> TaResult<bool> {
        match self {
            Operator::GreaterThan => Ok(lhs.gt(rhs)),
            Operator::LessThan => Ok(lhs.lt(rhs)),
            Operator::Equals => Ok(lhs.eq(rhs)),
            Operator::GreaterThanOrEqual => Ok(lhs.ge(rhs)),
            Operator::LessThanOrEqual => Ok(lhs.le(rhs)),
            Operator::CrossUnder(prev_value) => {
                let result = match prev_value {
                    Some(prev) => (*prev).ge(rhs) && lhs.lt(rhs),
                    None => false, // First evaluation can't detect a crossunder
                };
                *prev_value = Some(lhs.clone());
                Ok(result)
            },
            Operator::CrossOver(prev_value) => {
                let result = match prev_value {
                    Some(prev) => (*prev).le(rhs) && lhs.gt(rhs),
                    None => false, // First evaluation can't detect a crossover
                };
                *prev_value = Some(lhs.clone());
                Ok(result)
            },
        }
    }
}

impl Period for Condition {
    fn period(&self) -> usize {
        self.max_period().unwrap_or(1)
    }
}

impl Reset for Condition {
    fn reset(&mut self) {
        match self {
            Condition::Value { indicator, operator, .. } =>  {
                indicator.reset();
                match operator {
                    Operator::CrossOver(prev_value) | Operator::CrossUnder(prev_value) => {
                        *prev_value = None; // Reset crossover state
                    },
                    _ => {}
                }
            }
            Condition::Indicator { left, right, operator } => {
                left.reset();
                right.reset();
                match operator {
                    Operator::CrossOver(prev_value) | Operator::CrossUnder(prev_value) => {
                        *prev_value = None; // Reset crossover state
                    },
                    _ => {}
                }
            }
            Condition::And(conds) | Condition::Or(conds) => {
                for c in conds {
                    c.reset();
                }
            }
            Condition::Not(c) => c.reset(),
        }
    }
}
