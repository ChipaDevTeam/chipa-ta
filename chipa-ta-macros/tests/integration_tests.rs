use chipa_ta_macros::{register_trait, AutoImpl};

// Simple test trait
register_trait! {
    pub trait TestTrait {
        fn test_method(&self) -> String;
        fn test_method_mut(&mut self) -> i32;
    }
}

// Test structs
#[derive(Clone, Debug, PartialEq)]
struct TestStruct1 {
    value: String,
    counter: i32,
}

impl TestTrait for TestStruct1 {
    fn test_method(&self) -> String {
        format!("TestStruct1: {}", self.value)
    }

    fn test_method_mut(&mut self) -> i32 {
        self.counter += 1;
        self.counter
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TestStruct2 {
    value: f64,
    counter: i32,
}

impl TestTrait for TestStruct2 {
    fn test_method(&self) -> String {
        format!("TestStruct2: {}", self.value)
    }

    fn test_method_mut(&mut self) -> i32 {
        self.counter += 2;
        self.counter
    }
}

// Test enum with AutoImpl
#[derive(AutoImpl, Clone, Debug, PartialEq)]
#[auto_implement(trait = TestTrait, path = "tests/integration_tests.rs")]
enum TestEnum {
    Variant1(TestStruct1),
    Variant2(TestStruct2),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_impl_immutable_method() {
        let enum1 = TestEnum::Variant1(TestStruct1 {
            value: "hello".to_string(),
            counter: 0,
        });

        let enum2 = TestEnum::Variant2(TestStruct2 {
            value: 3.14,
            counter: 0,
        });

        assert_eq!(enum1.test_method(), "TestStruct1: hello");
        assert_eq!(enum2.test_method(), "TestStruct2: 3.14");
    }

    #[test]
    fn test_auto_impl_mutable_method() {
        let mut enum1 = TestEnum::Variant1(TestStruct1 {
            value: "hello".to_string(),
            counter: 0,
        });

        let mut enum2 = TestEnum::Variant2(TestStruct2 {
            value: 3.14,
            counter: 0,
        });

        assert_eq!(enum1.test_method_mut(), 1);
        assert_eq!(enum1.test_method_mut(), 2);

        assert_eq!(enum2.test_method_mut(), 2);
        assert_eq!(enum2.test_method_mut(), 4);
    }
}

// Test for built-in traits (commented out since they depend on chipa-ta)
// You can uncomment these if you add chipa-ta as a dependency
/*
#[cfg(test)]
mod builtin_trait_tests {
    use super::*;

    // Example of how to test built-in traits
    #[test]
    fn test_period_trait() {
        // This would work if chipa-ta was a dependency
        // let test_enum = TestPeriodEnum::Test(TestPeriodStruct { period: 14 });
        // assert_eq!(test_enum.period(), 14);
    }
}
*/
