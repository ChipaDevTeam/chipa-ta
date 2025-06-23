use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::{error::{TaError, TaResult}, strategy::{binary::{modifier::{AmountModifier, Modifier}, statistics::Statistics, trader::OpenTrade}, platform::{BinaryOptionsPlatform, TradeResult}, StrategyError}};

#[derive(Serialize, Deserialize)]
pub struct TraderMut<P: BinaryOptionsPlatform + Default> {
    #[serde(with= "arc_rwlock_serde")]
    pub statistics: Arc<RwLock<Statistics<P>>>,
    #[serde(with= "arc_rwlock_serde")]
    pub balance: Arc<RwLock<f64>>,
    #[serde(with= "arc_rwlock_serde")]
    pub modifier: Arc<RwLock<Modifier>>,
    #[serde(skip)]
    pub open_trade: Arc<RwLock<Vec<OpenTrade<P>>>>,
}

impl<P: BinaryOptionsPlatform + Default> Default for TraderMut<P> {
    fn default() -> Self {
        Self {
            statistics: Arc::new(RwLock::new(Statistics::default())),
            balance: Arc::new(RwLock::new(0.0)),
            modifier: Arc::new(RwLock::new(Modifier::default())),
            open_trade: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl<P: BinaryOptionsPlatform + Default> TraderMut<P> {
    pub fn new(
        statistics: Statistics<P>,
        balance: f64,
        modifier: Modifier,
    ) -> Self {
        Self {
            statistics: Arc::new(RwLock::new(statistics)),
            balance: Arc::new(RwLock::new(balance)),
            modifier: Arc::new(RwLock::new(modifier)),
            open_trade: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn init(&self, platform: &P) -> TaResult<()> {
        let balance = platform.balance().await?;

        *self.balance.write().map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))? = balance;
        Ok(())
    }

    pub fn modify(&self, amount: f64, last_result: &Option<TradeResult>) -> TaResult<f64> {
        let mut modifier = self.modifier.write().map_err(|e| TaError::from(StrategyError::Poison(e.to_string())))?;
        Ok(modifier.modify(amount, last_result))
    }
}


mod arc_rwlock_serde {
    use serde::{Deserialize, Serialize};
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use std::sync::{Arc, RwLock};

    use crate::error::TaError;
    use crate::strategy::StrategyError;

    pub fn serialize<S, T>(val: &Arc<RwLock<T>>, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
              T: Serialize,
    {
        let guard = val.read().map_err(|e| TaError::from(StrategyError::Poison(e.to_string()))).map_err(serde::ser::Error::custom)?;
        T::serialize(&*guard, s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Arc<RwLock<T>>, D::Error>
        where D: Deserializer<'de>,
              T: Deserialize<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(d)?)))
    }
}
