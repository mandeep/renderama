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

/// Tone map the given luminance globally
pub fn tone_map(luminance: f32) -> f32 {
    luminance / (luminance + 1.0)
}

/// Gamma correct the given luminance
pub fn gamma_correct(luminance: f32, alpha: f32) -> f32 {
    luminance.powf(alpha)
}

/// Check if a computed color contains any NaNs
pub fn de_nan(color: &Vector3<f32>) -> Vector3<f32> {
    let mut correction = Vector3::new(color.x, color.y, color.z);
    (0..3).for_each(|i| {
              if correction[i].is_nan() {
                  correction[i] = 0.0
              }
          });
    correction
}
