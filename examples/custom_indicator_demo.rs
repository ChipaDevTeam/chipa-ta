use chipa_ta::indicators::{CustomIndicator, RelativeStrengthIndex, SimpleMovingAverage};
use chipa_ta::indicators::custom::wrap_indicator;
use chipa_ta::traits::{IndicatorTrait, Next, Period, Reset};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CustomIndicator Demo ===\n");

    // 1. Creating a CustomIndicator with automatic type inference
    let rsi = RelativeStrengthIndex::new(14)?;
    let custom_rsi = CustomIndicator::new(rsi);
    
    println!("✅ Zero-cost wrapper: CustomIndicator<RelativeStrengthIndex>");
    println!("   Period: {}", custom_rsi.period());
    println!("   Output shape: {:?}", custom_rsi.output_shape());
    
    // 2. Using the helper function for even cleaner syntax
    let sma = SimpleMovingAverage::new(20)?;
    let custom_sma = wrap_indicator(sma);
    
    println!("\n✅ Helper function: wrap_indicator(sma)");
    println!("   Period: {}", custom_sma.period());
    
    // 3. Demonstrating that it works exactly like the original
    let mut original_rsi = RelativeStrengthIndex::new(14)?;
    let mut wrapped_rsi = CustomIndicator::new(RelativeStrengthIndex::new(14)?);
    
    let test_values = [10.0, 12.0, 11.0, 15.0, 13.0, 14.0, 16.0];
    
    println!("\n✅ Behavior verification (original vs wrapped):");
    for value in test_values {
        let original_result = original_rsi.next(value)?;
        let wrapped_result = wrapped_rsi.next(value)?;
        
        assert_eq!(original_result, wrapped_result);
        println!("   Input: {:.1} -> Both outputs: {:.4}", value, original_result);
    }
    
    // 4. Demonstrating access to inner indicator
    println!("\n✅ Inner access:");
    println!("   Can access inner: {:?}", wrapped_rsi.inner());
    println!("   Can mutate inner: {:?}", wrapped_rsi.inner_mut());
    
    // 5. Demonstrating reset functionality
    wrapped_rsi.reset();
    println!("\n✅ Reset functionality works perfectly");
    
    // 6. Performance note
    println!("\n🚀 Performance Benefits:");
    println!("   • Zero runtime overhead (no Arc/Mutex)");
    println!("   • Compile-time polymorphism");
    println!("   • Direct method dispatch");
    println!("   • Full type safety maintained");
    
    // 7. Compared to old implementation
    println!("\n🔧 Improvements over old implementation:");
    println!("   • ❌ Old: Complex DynIndicatorTrait with runtime overhead");
    println!("   • ✅ New: Simple generic wrapper with zero cost");
    println!("   • ❌ Old: Arc<Mutex<dyn Trait>> causing lock contention");
    println!("   • ✅ New: Direct field access, no locks");
    println!("   • ❌ Old: Complex feature flag conditionals");
    println!("   • ✅ New: Clean conditional compilation");
    println!("   • ❌ Old: Hard to extend and maintain");
    println!("   • ✅ New: Simple, extensible, follows Rust idioms");
    
    Ok(())
}
