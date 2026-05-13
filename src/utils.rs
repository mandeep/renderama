use std::time::Duration;

use glam::Vec3A;

/// Convert a Duration to a String formatted as HH:MM:SS
pub fn format_time(instant: Duration) -> String {
    let total_seconds = instant.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Check if a computed color contains any NaNs
pub fn de_nan(color: &Vec3A) -> Vec3A {
    let mut correction = Vec3A::new(color.x, color.y, color.z);
    if correction.x.is_nan() {
        correction.x= 0.0;
    }
    if correction.y.is_nan() {
        correction.y= 0.0;
    }
    if correction.z.is_nan() {
        correction.z= 0.0;
    }

    correction
}
