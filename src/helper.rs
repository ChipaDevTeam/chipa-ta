pub fn max3(a: f64, b: f64, c: f64) -> f64 {
    a.max(b).max(c)
}

pub fn round(num: f64) -> f64 {
    (num * 1000.0).round() / 1000.00
}
