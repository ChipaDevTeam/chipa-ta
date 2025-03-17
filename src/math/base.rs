use crate::error::{TaError, TaResult};

pub fn mean(prices: &[f64]) -> TaResult<f64> {
    if prices.is_empty() {
        return Err(TaError::EmptyIterator("mean".to_string()));
    };
    let sum: f64 = prices.iter().sum();
    Ok(sum / prices.len() as f64)
}

pub fn median(prices: &[f64]) -> TaResult<f64> {
    if prices.is_empty() {
        return Err(TaError::EmptyIterator("median".to_string()));
    };

    let mut ordered_prices = prices
        .iter()
        .filter_map(|f| if f.is_nan() { None } else { Some(*f) })
        .collect::<Vec<f64>>();
    ordered_prices.sort_by(cmp_f64);
    let middle: usize = prices.len() / 2;
    if prices.len() % 2 == 0 {
        return mean(&ordered_prices[middle - 1..middle + 1]);
    };

    Ok(ordered_prices[middle])
}

use std::{cmp::Ordering, collections::HashMap};

pub fn cmp_f64(a: &f64, b: &f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}

// use itertools::Itertools;

pub fn most_frequent(vector: Vec<i64>) -> f64 {
    let counts = vector.into_iter().fold(HashMap::new(), |mut acc, x| {
        *acc.entry(x).or_insert(0) += 1;
        acc
    });

    let max_count = *counts.values().max().unwrap_or(&0);
    let most_frequent = counts
        .into_iter()
        .filter(|(_, count)| count == &max_count)
        .collect::<Vec<_>>();

    if most_frequent.is_empty() {
        0.0
    } else {
        most_frequent
            .iter()
            .map(|&(count, _)| count as f64)
            .sum::<f64>()
            / most_frequent.len() as f64
    }
}

pub fn mode(prices: &[f64]) -> TaResult<f64> {
    if prices.is_empty() {
        return Err(TaError::EmptyIterator("median".to_string()));
    };

    let rounded_prices = prices.iter().map(|x| x.round() as i64).collect();
    Ok(most_frequent(rounded_prices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_mode_round_up() {
        let prices = vec![100.2, 100.46, 100.53, 101.08, 101.19];
        assert_eq!(Ok(101.0), mode(&prices));
    }

    #[test]
    fn single_mode_round_down() {
        let prices = vec![100.2, 100.46, 100.35, 101.08, 101.19];
        assert_eq!(Ok(100.0), mode(&prices));
    }
}
