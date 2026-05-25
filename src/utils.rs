use std::time::Duration;

use glam::{BVec3A, Vec3A};

/// Convert a Duration to a String formatted as HH:MM:SS
pub fn format_time(instant: Duration) -> String {
    let total_seconds = instant.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Check if a computed color contains any NaNs or infinites
pub fn de_nan(color: &Vec3A) -> Vec3A {
    let mask = BVec3A::new(color.x.is_finite(), color.y.is_finite(), color.z.is_finite());

    Vec3A::select(mask, *color, Vec3A::ZERO)
}


#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;

    #[test]
    fn test_de_nan() {
        let color = Vec3A::new(1.0 / 0.0, 2.0 / 0.0, 3.0 / 0.0);
        let corrected = de_nan(&color);
        assert_eq!(corrected, Vec3A::ZERO);

        let color = Vec3A::new(f32::NAN, 0.0, 0.0);
        let corrected = de_nan(&color);
        assert_eq!(corrected, Vec3A::ZERO);

        let color = Vec3A::new(f32::NAN, 10.0 / 0.0, 1.0);
        let corrected = de_nan(&color);
        assert_eq!(corrected, Vec3A::new(0.0, 0.0, 1.0));
    }
}