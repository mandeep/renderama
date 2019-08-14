use std::time::Duration;

use nalgebra::Vector3;

/// Convert a Duration to a String formatted as HH:MM:SS
pub fn format_time(instant: Duration) -> String {
    let total_seconds = instant.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Clamp a float between 0.0 and 255.0
///
/// This function is used due to an LLVM bug
/// where casting a float to u8 can lead to
/// undefined behavior:
/// https://github.com/rust-lang/rust/issues/10184
pub fn clamp(n: f32) -> f32 {
    n.min(255.0).max(0.0)
}

/// Check if a computed color contains any NaNs
pub fn de_nan(color: &Vector3<f32>) -> Vector3<f32> {
    let mut correction = Vector3::new(color.x, color.y, color.z);
    if correction.x.is_nan() {
        correction.x = 0.0
    };
    if correction.y.is_nan() {
        correction.y = 0.0
    };
    if correction.z.is_nan() {
        correction.z = 0.0
    };
    correction
}
