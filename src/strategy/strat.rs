use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{error::TaResult, strategy::{Action, MarketData, StrategyNode}, traits::{Period, Reset}};

#[derive(Clone)]
pub enum State {
    Progress(usize),
    Ready
}

#[derive(Clone)]
pub struct Strategy {
    pub nodes: StrategyNode,
    pub state: State
}

impl Deref for Strategy {
    type Target = StrategyNode;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            nodes: StrategyNode::default(),
            state: State::Progress(0),
        }
    }
}

impl Strategy {
    pub fn new(nodes: StrategyNode) -> Self {
        Self { nodes, state: State::Progress(0) }
    }

    pub fn evaluate(&mut self, data: &MarketData) -> TaResult<Option<Action>> {
        self.next();
        match self.state {
            State::Progress(_) => {
                self.nodes.update(data)?;
                Ok(None)
            },
            State::Ready => self.nodes.evaluate(data).map(Some),
        }
    }

    fn next(&mut self) {
        if let State::Progress(index) = self.state {
            if index < self.nodes.period() {
                self.state = State::Progress(index + 1);
            } else {
                self.state = State::Ready;
            }
        }
    }
}

impl Reset for Strategy {
    fn reset(&mut self) {
        self.state = State::Progress(0);
        self.nodes.reset();
    }
}

impl Period for Strategy {
    fn period(&self) -> usize {
        self.nodes.period()
    }
}

/// Custom implementation of the Serialize and Deserialize traits for Strategy
impl Serialize for Strategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.nodes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Strategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nodes = StrategyNode::deserialize(deserializer)?;
        Ok(Self::new(nodes))
    }
}