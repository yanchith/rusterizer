pub(crate) fn clamp(val: f64, min: f64, max: f64) -> f64 {
    f64::min(max, f64::max(min, val))
}
