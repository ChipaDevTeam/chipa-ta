# TraderMut Builder and Platform Initialization Guide

This guide explains the new features added to the trader system for better configuration flexibility and platform initialization.

## Features Added

### 1. TraderMut Builder Pattern

The `TraderMutBuilder` allows you to configure TraderMut instances with custom components:

```rust
use chipa_ta::strategy::binary::{
    trader_mut::TraderMut,
    modifier::Modifier,
};

// Create a TraderMut with custom configuration
let trader_mut = TraderMut::builder()
    .with_balance(1000.0)
    .with_modifier(Modifier::martingale(2.0, 5, reset_condition)?)
    .with_statistics(custom_statistics)
    .build();
```

### 2. Enhanced TraderBuilder

The `TraderBuilder` now supports:

- Direct modifier configuration
- Initial balance setting
- TraderMutBuilder integration

```rust
// Configure modifier directly through TraderBuilder
let trader = Trader::builder()
    .with_trade_amount(50.0)
    .with_modifier(Modifier::martingale(2.0, 5, reset_condition)?)
    .with_initial_balance(1000.0)
    .setup_with_credentials(credentials).await?
    .build()?;
```

### 3. Platform Initialization for Loaded Traders

When loading a trader from a file, you can now initialize the platform:

```rust
// Method 1: Load with credentials in one step
let trader = Trader::load_with_credentials(
    "trader_state.json",
    credentials
).await?;

// Method 2: Load and initialize separately
let mut trader = Trader::load("trader_state.json").await?;
trader.initialize_platform(credentials).await?;

// Method 3: Setup platform credentials later
trader.setup_platform_credentials(new_credentials).await?;
```

## Usage Patterns

### Pattern 1: Custom Modifier Selection

```rust
// Create different modifiers
let martingale = Modifier::martingale(2.0, 5, MartingaleResetCondition::OnWin)?;
let no_modifier = Modifier::none();

// Use with TraderMut builder
let trader_mut = TraderMut::builder()
    .with_modifier(martingale)
    .with_balance(500.0)
    .build();

// Use with Trader builder
let trader = Trader::builder()
    .with_modifier(no_modifier)
    .with_trade_amount(25.0)
    .setup_with_credentials(credentials).await?
    .build()?;
```

### Pattern 2: Loading and Initializing Traders

```rust
// Save trader state
trader.save("my_trader.json").await?;

// Load with immediate platform setup
let reloaded_trader = Trader::load_with_credentials(
    "my_trader.json",
    platform_credentials
).await?;

// Or load and setup separately for more control
let mut trader = Trader::load("my_trader.json").await?;
trader.initialize_platform(platform_credentials).await?;
```

### Pattern 3: Complete Configuration Example

```rust
use chipa_ta::strategy::binary::{
    trader::{Trader, TraderConfig, TraderMode},
    trader_mut::TraderMut,
    modifier::Modifier,
    martingale::{Martingale, MartingaleResetCondition},
};

// Create a fully configured trader
let trader = Trader::builder()
    .with_trade_amount(50.0)
    .with_mode(TraderMode::Singlethreaded)
    .with_strategy(my_strategy)
    .with_stop_loss(200.0)
    .with_take_profit(1500.0)
    .with_modifier(Modifier::martingale(
        1.8,  // multiplier
        4,    // max doublings
        MartingaleResetCondition::OnWin
    )?)
    .with_initial_balance(800.0)
    .setup_with_credentials(credentials).await?
    .build()?;

// Save and reload
trader.save("full_config_trader.json").await?;
let reloaded = Trader::load_with_credentials(
    "full_config_trader.json",
    fresh_credentials
).await?;
```

## Benefits

1. **Manual Modifier Selection**: You can now choose exactly which modifier to use
2. **Flexible Configuration**: Builder pattern allows step-by-step configuration
3. **Platform Persistence**: Loading traders from files now properly handles platform initialization
4. **Credential Management**: Separate credential handling from trader state
5. **Better Error Handling**: Clear separation of concerns between trader state and platform connection

## Migration Guide

### Old Way:

```rust
// Limited configuration options
let trader_mut = TraderMut::default();
let trader = Trader::new(platform, config, trader_mut);

// Loading didn't handle platform initialization
let loaded_trader = Trader::load("state.json").await?; // Platform not initialized!
```

### New Way:

```rust
// Flexible configuration
let trader_mut = TraderMut::builder()
    .with_modifier(your_chosen_modifier)
    .build();

let trader = Trader::builder()
    .with_trader_mut(trader_mut)
    .setup_with_credentials(credentials).await?
    .build()?;

// Loading with proper platform initialization
let loaded_trader = Trader::load_with_credentials("state.json", credentials).await?;
```

## Error Handling

The new methods properly handle errors related to:

- Platform initialization failures
- Invalid credentials
- File I/O errors during save/load
- Lock poisoning in concurrent scenarios

All methods return `TaResult<T>` for consistent error handling across the API.
