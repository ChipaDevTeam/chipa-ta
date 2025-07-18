use core::f64;
use std::str::FromStr;

use chipa_ta_macros::{register_trait, AutoImpl};

register_trait! {
    trait Num2String2 {
        fn num2string2(&self) -> String;
        fn string2num2(num: &str) -> Self;

    }
}

impl<T> Num2String2 for T
where
    T: ToString + FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn num2string2(&self) -> String {
        self.to_string()
    }

    fn string2num2(num: &str) -> Self {
        num.parse().unwrap()
    }
}

register_trait! {
    pub trait Num2String {
        #[allow(unused)]
        fn num2string(&self) -> String;
    }
}

#[derive(AutoImpl, Debug)]
#[auto_implement(trait = Num2String)]
#[auto_implement(trait = Num2String2)]
#[auto_implement(method(string2num2 = "str2num"))]
enum Test {
    V1(i32),
    V2(f64),
}

impl Test {
    fn str2num(s: &str) -> Self {
        if let Ok(v) = s.parse::<i32>() {
            Test::V1(v)
        } else if let Ok(v) = s.parse::<f64>() {
            Test::V2(v)
        } else {
            panic!("Failed to parse string to number")
        }
    }
}

impl Num2String for i32 {
    fn num2string(&self) -> String {
        self.to_string()
    }
}

impl Num2String for f64 {
    fn num2string(&self) -> String {
        self.to_string()
    }
}

fn main() {
    let v1 = Test::V1(42);
    let v2 = Test::V2(f64::consts::PI);

    println!("v1 as string: {}", v1.num2string());
    println!("v2 as string: {}", v2.num2string());

    let v1_str = v1.num2string2();
    let v2_str = v2.num2string2();
    println!("v1 as string2: {v1_str}");
    println!("v2 as string2: {v2_str}");
    let v1_num = Test::string2num2(&v1_str);
    let v2_num = Test::string2num2(&v2_str);
    println!("v1 as number: {v1_num:?}");
    println!("v2 as number: {v2_num:?}");
}
