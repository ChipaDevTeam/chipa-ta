# Chipa-TA Macros

This crate provides procedural macros for automatically implementing traits on enums. This is particularly useful for creating polymorphic enums where all variants implement the same trait.

## Features

- **`AutoImpl` derive macro**: Automatically implement traits on enums by delegating to inner types
- **`auto_method` attribute**: Mark trait methods for auto-implementation
- **`auto_trait` attribute**: Mark traits as auto-implementable
- **`impl_custom_trait` macro**: Generate custom trait implementations
- Support for generic traits and associated types

## Usage

### Basic Usage

```rust
use chipa_ta_macros::AutoImpl;

// Define your structs that implement a trait
#[derive(Clone, Debug)]
struct TypeA {
    value: i32,
}

#[derive(Clone, Debug)]
struct TypeB {
    value: f64,
}

// Both implement the same trait
trait MyTrait {
    fn process(&self) -> String;
}

impl MyTrait for TypeA {
    fn process(&self) -> String {
        format!("TypeA: {}", self.value)
    }
}

impl MyTrait for TypeB {
    fn process(&self) -> String {
        format!("TypeB: {}", self.value)
    }
}

// Use AutoImpl to automatically implement the trait for the enum
#[derive(AutoImpl, Clone, Debug)]
#[auto_implement(trait_name = "MyTrait")]
enum MyEnum {
    A(TypeA),
    B(TypeB),
}

// Now you can use the enum polymorphically
fn main() {
    let items = vec![
        MyEnum::A(TypeA { value: 42 }),
        MyEnum::B(TypeB { value: 3.14 }),
    ];
    
    for item in items {
        println!("{}", item.process());
    }
}
```

### Generic Traits

For traits with generic parameters and associated types:

```rust
use chipa_ta_macros::AutoImpl;

// Generic trait with associated type
trait Process<T> {
    type Output;
    fn process(&mut self, input: T) -> Result<Self::Output, String>;
}

#[derive(AutoImpl, Clone, Debug)]
#[auto_implement(trait_name = "Process", generics = "T", associated_types = "Output")]
enum Processor<T> {
    ProcessorA(ProcessorAImpl),
    ProcessorB(ProcessorBImpl),
}
```

### Real-World Example: Technical Indicators

```rust
use chipa_ta_macros::AutoImpl;
use chipa_ta::traits::{Indicator, Reset, Period};

#[derive(AutoImpl, Clone, Debug, PartialEq)]
#[auto_implement(trait_name = "Indicator")]
enum TechnicalIndicator {
    Sma(SimpleMovingAverage),
    Ema(ExponentialMovingAverage),
    Rsi(RelativeStrengthIndex),
    Macd(MacdIndicator),
}

// The enum now implements Indicator automatically!
fn analyze_indicators(indicators: Vec<TechnicalIndicator>) {
    for indicator in indicators {
        println!("Indicator: {}", indicator.name());
        println!("Period: {}", indicator.period());
        println!("Output Shape: {:?}", indicator.output_shape());
    }
}
```

## Supported Traits

The macro currently has built-in support for these traits from the chipa-ta crate:

### `Indicator`
- `output_shape(&self) -> OutputShape`
- `name(&self) -> String`
- `to_ct_string(&self) -> String`
- `from_ct_string(s: &str) -> TaResult<Self>`

### `Reset`
- `reset(&mut self)`

### `Period`
- `period(&self) -> usize`

### `Candle`
- `open(&self) -> f64`
- `close(&self) -> f64`
- `high(&self) -> f64`
- `low(&self) -> f64`
- `price(&self) -> f64`
- `volume(&self) -> f64`
- `to_bar(&self) -> Bar`

### `Next<T>` (Generic trait)
- `next(&mut self, input: T) -> TaResult<Self::Output>`
- `next_batched<A>(&mut self, input: A) -> TaResult<Vec<Self::Output>>`

## Advanced Usage

### Multiple Trait Implementations

You can implement multiple traits on the same enum by using the macro multiple times:

```rust
// First, create separate enums for each trait
#[derive(AutoImpl, Clone, Debug, PartialEq)]
#[auto_implement(trait_name = "Indicator")]
enum IndicatorEnum {
    Sma(SimpleMovingAverage),
    Ema(ExponentialMovingAverage),
}

#[derive(AutoImpl, Clone, Debug, PartialEq)]
#[auto_implement(trait_name = "Reset")]
enum ResetEnum {
    Sma(SimpleMovingAverage),
    Ema(ExponentialMovingAverage),
}

// Or implement them manually if you need both on the same enum
#[derive(Clone, Debug, PartialEq)]
enum TechnicalIndicator {
    Sma(SimpleMovingAverage),
    Ema(ExponentialMovingAverage),
}

#[derive(AutoImpl)]
#[auto_implement(trait_name = "Indicator")]
impl Indicator for TechnicalIndicator {}

#[derive(AutoImpl)]
#[auto_implement(trait_name = "Reset")]
impl Reset for TechnicalIndicator {}
```

### Custom Trait Implementation

For traits not built into the macro, you can use the `impl_custom_trait` macro:

```rust
use chipa_ta_macros::impl_custom_trait;

impl_custom_trait! {
    trait CustomTrait for MyEnum {
        fn custom_method(&self, param: i32) -> String;
        fn another_method(&mut self) -> Result<(), Error>;
    }
}
```

## Requirements

- All enum variants must be tuple structs with exactly one field
- All variant types must implement the trait you're auto-implementing
- The trait must be in scope when using the macro

## Error Handling

The macro will produce compile-time errors if:
- The enum has variants that aren't tuple structs with one field
- The specified trait doesn't exist or isn't in scope
- The trait name is malformed

## Limitations

- Currently only supports traits with specific method signatures
- Associated types are handled in a simplified way (using the first variant's type)
- Generic constraints are not automatically inferred
- Default trait implementations are not preserved

## Contributing

To add support for a new trait, update the `get_trait_methods` function in `src/lib.rs` with the trait's method signatures.

## License

This crate follows the same license as the parent chipa-ta project.
