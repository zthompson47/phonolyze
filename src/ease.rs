#![warn(missing_docs)]
//! Easing functions

/// Fifth order curve ease in
pub fn quint_ease_in(x: f32, min: f32, max: f32) -> f32 {
    assert!(x >= 0.);
    assert!(x <= 1.0);
    (max - min) * x.powi(5) + min
}

/// Inverse fifth order curve ease in
pub fn inv_quint_ease_in(x: f32, min: f32, max: f32) -> f32 {
    assert!(x >= min);
    assert!(x <= max);
    ((x - min) / (max - min)).powf(1. / 5.)
}

// Map t which is in range [a, b] to range [c, d]
fn _remap(t: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    (t - a) * ((d - c) / (b - a)) + c
}
