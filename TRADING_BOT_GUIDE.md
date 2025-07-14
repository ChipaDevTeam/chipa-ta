# Trading Bot Implementation Guide with PocketOption

This comprehensive guide demonstrates how to implement a trading bot using the `Trader` struct from the chipa-ta library with the real PocketOption platform. We'll walk through creating a complete binary options trading bot that uses technical analysis strategies.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Basic Setup](#basic-setup)
3. [Creating a Simple Trading Bot](#creating-a-simple-trading-bot)
4. [Advanced Bot with Custom Strategy](#advanced-bot-with-custom-strategy)
5. [Error Handling and Logging](#error-handling-and-logging)
6. [Configuration and Persistence](#configuration-and-persistence)
7. [Complete Production Example](#complete-production-example)
8. [Running the Bot](#running-the-bot)

## Prerequisites

Before starting, you'll need:

1. **PocketOption Account**: A valid PocketOption account with trading access
2. **Session ID (SSID)**: Your authentication session ID from PocketOption
3. **Rust Environment**: Rust 1.70+ with Cargo
4. **Dependencies**: The required crates listed below

## Basic Setup

First, add the necessary dependencies to your `Cargo.toml`:

```toml
[dependencies]
chipa-ta = { path = ".", features = ["pocket_option"] }
binary-options-tools = "0.1"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
clap = { version = "4.0", features = ["derive"] }
```

## Creating a Simple Trading Bot

Here's a basic example of how to create and run a trading bot with PocketOption:

```rust
use chipa_ta::{
    platforms::PocketOptionPlatform,
    strategy::{
        binary::{
            trader::{Trader, TraderConfig, TraderMode},
            trader_mut::TraderMut,
        },
        platform::BinaryOptionsPlatform,
        strat::Strategy,
        Action, MarketData,
    },
    error::TaResult,
};
use std::time::Duration;
use tracing::{info, warn, error};
use clap::Parser;

#[derive(Parser)]
#[command(name = "trading-bot")]
#[command(about = "A binary options trading bot for PocketOption")]
struct Args {
    /// PocketOption session ID (SSID)
    #[arg(short, long)]
    ssid: String,

    /// Trade amount in USD
    #[arg(short, long, default_value = "10.0")]
    amount: f64,

    /// Asset to trade (e.g., EURUSD)
    #[arg(short = 'A', long, default_value = "EURUSD")]
    asset: String,

    /// Candle length in seconds
    #[arg(short, long, default_value = "60")]
    candle_length: u32,

    /// Stop loss amount
    #[arg(long)]
    stop_loss: Option<f64>,

    /// Take profit amount
    #[arg(long)]
    take_profit: Option<f64>,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    // Create and run the bot
    let mut bot = create_simple_bot(args).await?;

    // Run the bot
    info!("Starting PocketOption trading bot...");
    bot.run(vec![bot.config.asset.clone()], bot.config.candle_length).await?;

    Ok(())
}

struct BotConfig {
    ssid: String,
    amount: f64,
    asset: String,
    candle_length: u32,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

async fn create_simple_bot(args: Args) -> TaResult<SimpleBot> {
    let config = BotConfig {
        ssid: args.ssid,
        amount: args.amount,
        asset: args.asset,
        candle_length: args.candle_length,
        stop_loss: args.stop_loss,
        take_profit: args.take_profit,
    };

    // Create a basic trading configuration
    let mut trader_config = TraderConfig::new()
        .with_trade_amount(config.amount)
        .with_mode(TraderMode::Singlethreaded)
        .with_market_open_only(true);

    if let Some(stop_loss) = config.stop_loss {
        trader_config = trader_config.with_stop_loss(stop_loss);
    }

    if let Some(take_profit) = config.take_profit {
        trader_config = trader_config.with_take_profit(take_profit);
    }

    // Setup the trader with PocketOption credentials
    let trader = Trader::<PocketOptionPlatform>::setup(trader_config, config.ssid.clone()).await?;

    Ok(SimpleBot { trader, config })
}

struct SimpleBot {
    trader: Trader<PocketOptionPlatform>,
    config: BotConfig,
}

impl SimpleBot {
    async fn run(&mut self, assets: Vec<String>, candle_length: u32) -> TaResult<()> {
        self.trader.run(assets, candle_length).await
    }
}
```

## Advanced Bot with Custom Strategy

Here's a more sophisticated example using technical indicators and a complex strategy:

```rust
use chipa_ta::{
    indicators::{SimpleMovingAverage, RelativeStrengthIndex, Indicator},
    strategy::{
        node::StrategyNode,
        condition::{Condition, Operator},
        strat::Strategy,
        Action,
    },
    types::OutputType,
    traits::Period,
};

async fn create_advanced_bot(args: Args) -> TaResult<AdvancedBot> {
    let config = load_or_create_config(args).await?;

    // Create technical indicators for the strategy
    let sma_fast = Indicator::SimpleMovingAverage(SimpleMovingAverage::new(10)?);
    let sma_slow = Indicator::SimpleMovingAverage(SimpleMovingAverage::new(20)?);
    let rsi = Indicator::RelativeStrengthIndex(RelativeStrengthIndex::new(14)?);

    // Create trading conditions
    let bullish_crossover = Condition::and(vec![
        // Fast SMA crosses above Slow SMA (golden cross)
        Condition::Indicator {
            left: sma_fast.clone(),
            right: sma_slow.clone(),
            operator: Operator::GreaterThan,
        },
        // RSI is not overbought (< 70)
        Condition::Value {
            indicator: rsi.clone(),
            value: OutputType::Single(70.0),
            operator: Operator::LessThan,
        },
        // RSI is above 50 (bullish momentum)
        Condition::Value {
            indicator: rsi.clone(),
            value: OutputType::Single(50.0),
            operator: Operator::GreaterThan,
        },
    ]);

    let bearish_crossover = Condition::and(vec![
        // Fast SMA crosses below Slow SMA (death cross)
        Condition::Indicator {
            left: sma_fast.clone(),
            right: sma_slow.clone(),
            operator: Operator::LessThan,
        },
        // RSI is not oversold (> 30)
        Condition::Value {
            indicator: rsi.clone(),
            value: OutputType::Single(30.0),
            operator: Operator::GreaterThan,
        },
        // RSI is below 50 (bearish momentum)
        Condition::Value {
            indicator: rsi.clone(),
            value: OutputType::Single(50.0),
            operator: Operator::LessThan,
        },
    ]);

    // Create strategy decision tree
    let strategy = StrategyNode::If {
        condition: bullish_crossover,
        then_branch: Box::new(StrategyNode::Action(Action::Buy)),
        else_branch: Some(Box::new(StrategyNode::If {
            condition: bearish_crossover,
            then_branch: Box::new(StrategyNode::Action(Action::Sell)),
            else_branch: Some(Box::new(StrategyNode::Action(Action::Hold))),
        })),
    };

    // Create configuration with the advanced strategy
    let trader_config = TraderConfig::new()
        .with_trade_amount(config.trade_amount)
        .with_strategy(Strategy::Node(strategy))
        .with_stop_loss(config.stop_loss.unwrap_or(500.0))
        .with_take_profit(config.take_profit.unwrap_or(2000.0))
        .with_max_trade_amount(config.max_trade_amount.unwrap_or(100.0))
        .with_market_open_only(true);

    // Setup trader with PocketOption
    let trader = Trader::<PocketOptionPlatform>::setup(trader_config, config.ssid.clone()).await?;

    Ok(AdvancedBot { trader, config })
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct AdvancedBotConfig {
    pub ssid: String,
    pub trade_amount: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub max_trade_amount: Option<f64>,
    pub assets: Vec<String>,
    pub candle_length: u32,
    pub strategy_type: String,
    pub risk_management: RiskManagement,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct RiskManagement {
    pub max_daily_loss: f64,
    pub max_concurrent_trades: usize,
    pub martingale_enabled: bool,
    pub martingale_multiplier: f64,
    pub martingale_max_level: usize,
}

impl Default for AdvancedBotConfig {
    fn default() -> Self {
        Self {
            ssid: String::new(),
            trade_amount: 10.0,
            stop_loss: Some(500.0),
            take_profit: Some(2000.0),
            max_trade_amount: Some(100.0),
            assets: vec!["EURUSD".to_string(), "GBPUSD".to_string()],
            candle_length: 60,
            strategy_type: "sma_rsi_crossover".to_string(),
            risk_management: RiskManagement {
                max_daily_loss: 200.0,
                max_concurrent_trades: 3,
                martingale_enabled: false,
                martingale_multiplier: 2.0,
                martingale_max_level: 3,
            },
        }
    }
}

struct AdvancedBot {
    trader: Trader<PocketOptionPlatform>,
    config: AdvancedBotConfig,
}
```

## Error Handling and Logging

Implement robust error handling and recovery mechanisms:

```rust
use tracing::{error, warn, info, debug};
use std::time::Duration;

impl AdvancedBot {
    async fn run_with_recovery(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut attempts = 0;
        let max_attempts = 5;
        let mut backoff = Duration::from_secs(30);

        while attempts < max_attempts {
            match self.run_trading_session().await {
                Ok(_) => {
                    info!("Trading session completed successfully");
                    break;
                }
                Err(e) => {
                    attempts += 1;
                    error!("Trading session failed (attempt {}/{}): {}", attempts, max_attempts, e);

                    // Check if it's a recoverable error
                    if self.is_recoverable_error(&e) && attempts < max_attempts {
                        warn!("Attempting recovery in {:?}...", backoff);
                        tokio::time::sleep(backoff).await;

                        // Exponential backoff
                        backoff = std::cmp::min(backoff * 2, Duration::from_secs(300));

                        // Try to reinitialize the platform
                        if let Err(init_error) = self.reinitialize_platform().await {
                            error!("Failed to reinitialize platform: {}", init_error);
                            continue;
                        }

                        info!("Platform reinitialized successfully");
                    } else {
                        error!("Non-recoverable error or max attempts reached. Stopping bot.");
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn run_trading_session(&mut self) -> TaResult<()> {
        info!("Starting trading session with config: {:?}", self.config.strategy_type);

        // Save initial state
        self.save_state().await?;

        // Pre-flight checks
        self.validate_trading_conditions().await?;

        // Run the main trading loop
        for asset in &self.config.assets.clone() {
            info!("Starting trading on asset: {}", asset);

            // Check daily loss limits before starting
            if self.check_daily_loss_limit().await? {
                warn!("Daily loss limit reached. Skipping remaining assets.");
                break;
            }

            // Run trading for this asset
            if let Err(e) = self.trader.run(vec![asset.clone()], self.config.candle_length).await {
                error!("Error trading {}: {}", asset, e);

                // Don't stop for individual asset errors
                continue;
            }
        }

        // Save final state
        self.save_state().await?;

        Ok(())
    }

    async fn validate_trading_conditions(&self) -> TaResult<()> {
        // Check if market is open
        if !self.trader.platform.is_market_open().await? {
            return Err(TaError::Unexpected("Market is closed".to_string()));
        }

        // Check account balance
        let balance = self.trader.platform.balance().await?;
        if balance < self.config.trade_amount {
            return Err(TaError::Unexpected(format!(
                "Insufficient balance: {} < {}",
                balance,
                self.config.trade_amount
            )));
        }

        // Check if assets are active
        for asset in &self.config.assets {
            if !self.trader.platform.is_active(asset).await? {
                warn!("Asset {} is not active", asset);
            }
        }

        info!("All trading conditions validated successfully");
        Ok(())
    }

    async fn check_daily_loss_limit(&self) -> TaResult<bool> {
        let current_balance = self.trader.current_balance().await?;
        let starting_balance = self.get_starting_daily_balance().await?;
        let daily_loss = starting_balance - current_balance;

        if daily_loss >= self.config.risk_management.max_daily_loss {
            warn!(
                "Daily loss limit reached: ${:.2} >= ${:.2}",
                daily_loss,
                self.config.risk_management.max_daily_loss
            );
            return Ok(true);
        }

        Ok(false)
    }

    fn is_recoverable_error(&self, error: &Box<dyn std::error::Error>) -> bool {
        let error_str = error.to_string().to_lowercase();

        // Define recoverable error patterns
        let recoverable_patterns = [
            "connection",
            "timeout",
            "network",
            "websocket",
            "temporary",
            "rate limit",
        ];

        recoverable_patterns.iter().any(|pattern| error_str.contains(pattern))
    }

    async fn reinitialize_platform(&mut self) -> TaResult<()> {
        info!("Reinitializing PocketOption platform...");

        // Create new trader with same config
        let trader_config = TraderConfig::new()
            .with_trade_amount(self.config.trade_amount)
            .with_stop_loss(self.config.stop_loss.unwrap_or(500.0))
            .with_take_profit(self.config.take_profit.unwrap_or(2000.0))
            .with_max_trade_amount(self.config.max_trade_amount.unwrap_or(100.0));

        self.trader = Trader::<PocketOptionPlatform>::setup(
            trader_config,
            self.config.ssid.clone()
        ).await?;

        Ok(())
    }
}
```

## Configuration and Persistence

Create a robust configuration system with state persistence:

```rust
use std::path::Path;
use serde_json;

impl AdvancedBotConfig {
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ssid.is_empty() {
            return Err("SSID cannot be empty".to_string());
        }

        if self.trade_amount <= 0.0 {
            return Err("Trade amount must be positive".to_string());
        }

        if self.assets.is_empty() {
            return Err("At least one asset must be specified".to_string());
        }

        if self.candle_length < 30 {
            return Err("Candle length must be at least 30 seconds".to_string());
        }

        Ok(())
    }
}

impl AdvancedBot {
    async fn save_state(&self) -> TaResult<()> {
        // Save trader state
        self.trader.save("bot_state.json").await?;

        // Save configuration
        self.config.save_to_file("bot_config.json").await
            .map_err(|e| TaError::Unexpected(e.to_string()))?;

        info!("Bot state and configuration saved");
        Ok(())
    }

    async fn load_state(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration
        let config = if Path::new(config_path).exists() {
            AdvancedBotConfig::load_from_file(config_path).await?
        } else {
            let default_config = AdvancedBotConfig::default();
            default_config.save_to_file(config_path).await?;
            info!("Created default configuration file at {}", config_path);
            default_config
        };

        // Validate configuration
        config.validate()?;

        // Try to load existing trader state
        let trader = if Path::new("bot_state.json").exists() {
            match Trader::<PocketOptionPlatform>::load("bot_state.json").await {
                Ok(trader) => {
                    info!("Loaded existing bot state");
                    trader
                }
                Err(e) => {
                    warn!("Failed to load bot state, creating new: {}", e);
                    create_new_trader(&config).await?
                }
            }
        } else {
            create_new_trader(&config).await?
        };

        Ok(Self { trader, config })
    }

    async fn get_starting_daily_balance(&self) -> TaResult<f64> {
        // In a real implementation, you'd load this from a daily state file
        // For now, return current balance as starting balance
        self.trader.current_balance().await
    }
}

async fn create_new_trader(config: &AdvancedBotConfig) -> TaResult<Trader<PocketOptionPlatform>> {
    let trader_config = TraderConfig::new()
        .with_trade_amount(config.trade_amount)
        .with_stop_loss(config.stop_loss.unwrap_or(500.0))
        .with_take_profit(config.take_profit.unwrap_or(2000.0))
        .with_max_trade_amount(config.max_trade_amount.unwrap_or(100.0));

    Trader::<PocketOptionPlatform>::setup(trader_config, config.ssid.clone()).await
}
```

## Complete Production Example

Here's a complete, production-ready example with all features:

```rust
use chipa_ta::{
    platforms::PocketOptionPlatform,
    strategy::{
        binary::{
            trader::{Trader, TraderConfig, TraderMode},
            trader_mut::TraderMut,
        },
        platform::BinaryOptionsPlatform,
        strat::Strategy,
        Action, MarketData,
    },
    error::TaResult,
    indicators::{SimpleMovingAverage, RelativeStrengthIndex, Indicator},
};
use clap::Parser;
use std::{path::Path, time::Duration};
use tracing::{info, warn, error};

#[derive(Parser)]
#[command(name = "pocket-trading-bot")]
#[command(version = "1.0")]
#[command(about = "Professional binary options trading bot for PocketOption")]
struct CliArgs {
    /// Configuration file path
    #[arg(short, long, default_value = "bot_config.json")]
    config: String,

    /// PocketOption session ID (overrides config file)
    #[arg(long)]
    ssid: Option<String>,

    /// Run in demo mode (paper trading)
    #[arg(long)]
    demo: bool,

    /// Dry run - analyze only, don't place trades
    #[arg(long)]
    dry_run: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    // Initialize logging
    let level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting PocketOption Trading Bot v1.0");

    // Load configuration and create bot
    let mut bot = PocketTradingBot::initialize(args).await?;

    // Run the bot with recovery mechanisms
    match bot.run_with_recovery().await {
        Ok(_) => {
            info!("✅ Trading bot completed successfully");
        }
        Err(e) => {
            error!("❌ Trading bot failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

struct PocketTradingBot {
    trader: Trader<PocketOptionPlatform>,
    config: AdvancedBotConfig,
    cli_args: CliArgs,
    session_stats: SessionStats,
}

#[derive(Default)]
struct SessionStats {
    trades_placed: u32,
    wins: u32,
    losses: u32,
    draws: u32,
    total_profit: f64,
    session_start: chrono::DateTime<chrono::Utc>,
    last_trade_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl PocketTradingBot {
    async fn initialize(args: CliArgs) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔧 Initializing trading bot...");

        // Load configuration
        let mut config = AdvancedBotConfig::load_from_file(&args.config).await
            .unwrap_or_else(|_| {
                info!("📝 Creating default configuration");
                AdvancedBotConfig::default()
            });

        // Override SSID from command line if provided
        if let Some(ssid) = &args.ssid {
            config.ssid = ssid.clone();
        }

        // Validate configuration
        config.validate()?;

        // Save updated configuration
        config.save_to_file(&args.config).await?;
        info!("💾 Configuration saved to {}", args.config);

        // Create trader
        let trader = create_advanced_trader(&config).await?;

        let session_stats = SessionStats {
            session_start: chrono::Utc::now(),
            ..Default::default()
        };

        info!("✅ Bot initialized successfully");
        info!("📊 Trading {} assets with ${} per trade",
              config.assets.len(),
              config.trade_amount);

        Ok(Self {
            trader,
            config,
            cli_args: args,
            session_stats,
        })
    }

    async fn run_with_recovery(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 5;

        loop {
            match self.run_trading_session().await {
                Ok(_) => {
                    info!("✅ Trading session completed successfully");
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    error!("❌ Trading session failed (attempt {}/{}): {}",
                           retry_count, MAX_RETRIES, e);

                    if retry_count >= MAX_RETRIES {
                        error!("🛑 Maximum retry attempts reached. Stopping bot.");
                        return Err(e);
                    }

                    let delay = Duration::from_secs(30 * retry_count as u64);
                    warn!("⏳ Retrying in {:?}...", delay);
                    tokio::time::sleep(delay).await;

                    // Try to reinitialize
                    if let Err(init_err) = self.reinitialize().await {
                        error!("Failed to reinitialize: {}", init_err);
                        continue;
                    }
                }
            }
        }

        self.print_session_summary();
        Ok(())
    }

    async fn run_trading_session(&mut self) -> TaResult<()> {
        info!("🎯 Starting trading session");

        // Pre-flight checks
        self.validate_conditions().await?;

        // Save initial state
        self.save_state().await?;

        for asset in &self.config.assets.clone() {
            info!("📈 Trading asset: {}", asset);

            if self.check_stop_conditions().await? {
                warn!("🛑 Stop conditions met. Ending session.");
                break;
            }

            // Run trading for this asset
            if let Err(e) = self.trader.run(vec![asset.clone()], self.config.candle_length).await {
                error!("Error trading {}: {}", asset, e);
                continue;
            }
        }

        Ok(())
    }

    async fn validate_conditions(&self) -> TaResult<()> {
        // Check market status
        if !self.trader.platform.is_market_open().await? {
            return Err(TaError::Unexpected("Market is closed".to_string()));
        }

        // Check balance
        let balance = self.trader.platform.balance().await?;
        info!("💰 Current balance: ${:.2}", balance);

        if balance < self.config.trade_amount {
            return Err(TaError::Unexpected(format!(
                "Insufficient balance: ${:.2} < ${:.2}",
                balance, self.config.trade_amount
            )));
        }

        // Validate assets
        let available_assets = self.trader.platform.assets().await?;
        for asset in &self.config.assets {
            if !available_assets.contains(asset) {
                warn!("⚠️  Asset {} not available", asset);
            }
        }

        info!("✅ All conditions validated");
        Ok(())
    }

    async fn check_stop_conditions(&self) -> TaResult<bool> {
        let current_balance = self.trader.current_balance().await?;

        // Check stop loss
        if let Some(stop_loss) = self.config.stop_loss {
            if current_balance <= stop_loss {
                warn!("🔴 Stop loss triggered: ${:.2} <= ${:.2}", current_balance, stop_loss);
                return Ok(true);
            }
        }

        // Check take profit
        if let Some(take_profit) = self.config.take_profit {
            if current_balance >= take_profit {
                info!("🟢 Take profit reached: ${:.2} >= ${:.2}", current_balance, take_profit);
                return Ok(true);
            }
        }

        // Check daily loss limit
        let daily_loss = self.calculate_daily_loss().await?;
        if daily_loss >= self.config.risk_management.max_daily_loss {
            warn!("🔴 Daily loss limit reached: ${:.2}", daily_loss);
            return Ok(true);
        }

        Ok(false)
    }

    async fn calculate_daily_loss(&self) -> TaResult<f64> {
        // In production, you'd track the starting balance of the day
        // For now, we'll use session profit/loss
        Ok(-self.session_stats.total_profit.min(0.0))
    }

    fn print_session_summary(&self) {
        let duration = chrono::Utc::now() - self.session_stats.session_start;
        let win_rate = if self.session_stats.trades_placed > 0 {
            (self.session_stats.wins as f64 / self.session_stats.trades_placed as f64) * 100.0
        } else {
            0.0
        };

        info!("📊 === SESSION SUMMARY ===");
        info!("⏱️  Duration: {:?}", duration);
        info!("🎯 Trades placed: {}", self.session_stats.trades_placed);
        info!("🟢 Wins: {}", self.session_stats.wins);
        info!("🔴 Losses: {}", self.session_stats.losses);
        info!("⚪ Draws: {}", self.session_stats.draws);
        info!("📈 Win rate: {:.1}%", win_rate);
        info!("💰 Total profit: ${:.2}", self.session_stats.total_profit);
        info!("========================");
    }

    async fn save_state(&self) -> TaResult<()> {
        self.trader.save("bot_state.json").await?;
        self.config.save_to_file(&self.cli_args.config).await
            .map_err(|e| TaError::Unexpected(e.to_string()))?;
        Ok(())
    }

    async fn reinitialize(&mut self) -> TaResult<()> {
        info!("🔄 Reinitializing trader...");
        self.trader = create_advanced_trader(&self.config).await?;
        info!("✅ Trader reinitialized");
        Ok(())
    }
}

async fn create_advanced_trader(config: &AdvancedBotConfig) -> TaResult<Trader<PocketOptionPlatform>> {
    // Create technical indicators
    let sma_fast = Indicator::SimpleMovingAverage(SimpleMovingAverage::new(10)?);
    let sma_slow = Indicator::SimpleMovingAverage(SimpleMovingAverage::new(20)?);
    let rsi = Indicator::RelativeStrengthIndex(RelativeStrengthIndex::new(14)?);

    // Create strategy (simplified for example)
    let strategy = Strategy::default(); // You would implement your actual strategy here

    let trader_config = TraderConfig::new()
        .with_trade_amount(config.trade_amount)
        .with_strategy(strategy)
        .with_stop_loss(config.stop_loss.unwrap_or(500.0))
        .with_take_profit(config.take_profit.unwrap_or(2000.0))
        .with_max_trade_amount(config.max_trade_amount.unwrap_or(100.0))
        .with_market_open_only(true);

    Trader::<PocketOptionPlatform>::setup(trader_config, config.ssid.clone()).await
}
```

## Running the Bot

### Command Line Usage

```bash
# Basic usage with SSID
cargo run --release -- --ssid "your-pocket-option-ssid-here"

# Using a custom configuration file
cargo run --release -- --config my_config.json --ssid "your-ssid"

# Demo mode (paper trading)
cargo run --release -- --demo --ssid "demo-ssid"

# Dry run (analysis only)
cargo run --release -- --dry-run --config my_config.json

# Verbose logging
cargo run --release -- --verbose --ssid "your-ssid"
```

### Configuration File Example

Create a `bot_config.json` file:

```json
{
  "ssid": "your-pocket-option-session-id",
  "trade_amount": 25.0,
  "stop_loss": 800.0,
  "take_profit": 1500.0,
  "max_trade_amount": 100.0,
  "assets": ["EURUSD", "GBPUSD", "USDJPY", "AUDUSD"],
  "candle_length": 60,
  "strategy_type": "sma_rsi_crossover",
  "risk_management": {
    "max_daily_loss": 200.0,
    "max_concurrent_trades": 3,
    "martingale_enabled": false,
    "martingale_multiplier": 2.0,
    "martingale_max_level": 3
  }
}
```

### Important Notes

1. **Session ID (SSID)**: You must obtain this from PocketOption after logging in
2. **Risk Management**: Always set appropriate stop loss and take profit levels
3. **Testing**: Start with small amounts and test thoroughly
4. **Monitoring**: Monitor the bot's performance and logs regularly
5. **Compliance**: Ensure you comply with PocketOption's terms of service
6. **Backup**: Regularly backup your configuration and state files

### Safety Recommendations

- Start with demo trading to test your strategies
- Use small trade amounts initially
- Set strict stop loss limits
- Monitor the bot regularly
- Keep your SSID secure and never share it
- Regularly update the bot and dependencies
- Have a manual override mechanism

This guide provides a solid foundation for building a professional trading bot with the chipa-ta library and PocketOption platform. Remember to always trade responsibly and within your risk tolerance.
