// Example demonstrating how to use the new TraderMut builder and platform initialization

use chipa_ta::strategy::{
    binary::{
        trader::{Trader, TraderConfig},
        trader_mut::TraderMut,
        modifier::Modifier,
    },
    platform::BinaryOptionsPlatform,
};

// Example usage of the new functionality
async fn example_usage<P>() 
where 
    P: BinaryOptionsPlatform + Default + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    // 1. Create a TraderMut with custom modifier using the builder
    let custom_trader_mut = TraderMut::builder()
        .with_balance(1000.0)
        .with_modifier(Modifier::martingale(2.0, 5, Default::default()).unwrap())
        .build();

    // 2. Use the TraderBuilder with custom TraderMut
    let trader_config = TraderConfig::new()
        .with_trade_amount(10.0)
        .with_stop_loss(500.0)
        .with_take_profit(2000.0);

    // Example credentials (would be actual platform credentials in real use)
    let credentials = P::Creds::default(); // This would be real credentials

    let trader = Trader::builder()
        .with_strategy(Default::default()) // Your actual strategy
        .with_trader_mut(custom_trader_mut)
        .setup_with_credentials(credentials)
        .await
        .unwrap()
        .build()
        .unwrap();

    // 3. Save trader to file
    trader.save("trader_state.json").await.unwrap();

    // 4. Load trader from file and initialize platform
    let loaded_trader = Trader::<P>::load_with_credentials(
        "trader_state.json",
        P::Creds::default(), // Real credentials
    )
    .await
    .unwrap();

    // Alternative: Load and then setup credentials separately
    let mut loaded_trader = Trader::<P>::load("trader_state.json").await.unwrap();
    loaded_trader
        .initialize_platform(P::Creds::default())
        .await
        .unwrap();

    // 5. Use TraderBuilder with modifier configuration
    let trader_with_modifier = Trader::builder()
        .with_trade_amount(25.0)
        .with_modifier(Modifier::martingale(1.5, 3, Default::default()).unwrap())
        .with_initial_balance(500.0)
        .setup_with_credentials(P::Creds::default())
        .await
        .unwrap()
        .build()
        .unwrap();
}

// Usage patterns:

// Pattern 1: Direct TraderMut builder
// let trader_mut = TraderMut::builder()
//     .with_modifier(Modifier::martingale(2.0, 5, reset_condition)?)
//     .with_balance(1000.0)
//     .build();

// Pattern 2: Through TraderBuilder
// let trader = Trader::builder()
//     .with_modifier(modifier)
//     .with_initial_balance(1000.0)
//     .with_trade_amount(50.0)
//     .setup_with_credentials(credentials).await?
//     .build()?;

// Pattern 3: Load and setup platform
// let trader = Trader::load_with_credentials("state.json", credentials).await?;

// Pattern 4: Load and setup separately
// let mut trader = Trader::load("state.json").await?;
// trader.initialize_platform(credentials).await?;
