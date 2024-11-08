pub fn ieee_754_total_ordering_value(value: f64) -> i64 {
    // Use IEEE 754 Total Ordering
    // https://github.com/rust-lang/rust/blob/ecd3dbab4ed82abfa05e22069261e565239449cf/library/core/src/num/f64.rs#LL1336C14-L1336C14
    let mut series_value = value.to_bits() as i64;
    series_value ^= (((series_value >> 63) as u64) >> 1) as i64;
    series_value
}

pub fn reverse_ieee_754_total_ordering_value(value: i64) -> f64 {
    let series_value = value ^ ((((value >> 63) as u64) >> 1) as i64);
    f64::from_bits(series_value as u64)
}
