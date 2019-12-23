use std::time::Duration;

use glam::Vec3;

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
pub fn clamp_rgb(n: f32) -> f32 {
    n.min(255.0).max(0.0)
}

/// Clamp a value between the lower bound and upper bound
pub fn clamp(n: f32, lower_bound: f32, upper_bound: f32) -> f32 {
    let minimum = n.max(lower_bound);
    let maximum = n.min(upper_bound);

    minimum.min(maximum)
}

/// Gamma correct the given luminance
pub fn gamma_correct(luminance: f32, gamma: f32) -> f32 {
    luminance.powf(1.0 / gamma)
}

/// Check if a computed color contains any NaNs
pub fn de_nan(color: &Vec3) -> Vec3 {
    let mut correction = Vec3::new(color.x(), color.y(), color.z());
    if correction.x().is_nan() {
        correction.set_x(0.0);
    }
    if correction.y().is_nan() {
        correction.set_y(0.0);
    }
    if correction.z().is_nan() {
        correction.set_z(0.0);
    }

    correction
}

/// Find the maximum value of a Vec<f32>
pub fn f32_max(vector: &Vec<f32>) -> f32 {
    vector.iter().cloned().fold(0.0 / 0.0, f32::max)
}

/// Find the minimum value of a Vec<f32>
pub fn f32_min(vector: &Vec<f32>) -> f32 {
    vector.iter().cloned().fold(0.0 / 0.0, f32::min)
}
