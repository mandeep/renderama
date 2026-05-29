use std::time::Duration;

use glam::{BVec3A, Vec3A};
use image::RgbImage;
use rayon::prelude::*;

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

/// Convert from linear f32 to srgb u8
pub fn linear_to_srgb(linear_f32: f32) -> u8 {
    let clamped = clamp(linear_f32, 0.0, 1.0);
    let gamma_corrected = gamma_correct(clamped, 2.2);

    clamp_rgb(gamma_corrected * 255.0) as u8
}


/// Check if a computed color contains any NaNs or infinites
pub fn de_nan(color: &Vec3A) -> Vec3A {
    let mask = BVec3A::new(color.x.is_finite(), color.y.is_finite(), color.z.is_finite());

    Vec3A::select(mask, *color, Vec3A::ZERO)
}

/// Apply gamma correction to an Rgb8 image buffer in-place.
pub fn gamma_correct_buffer(image: &mut RgbImage, gamma: f32) {
    let mut lut = [0u8; 256];

    for (i, value) in lut.iter_mut().enumerate() {
        let luminance = i as f32 / 255.0;
        let corrected_value = gamma_correct(luminance, gamma);
        *value = clamp_rgb(corrected_value * 255.0) as u8;
    }

    image.as_mut().par_iter_mut().for_each(|pixel| {
        *pixel = lut[*pixel as usize];
    });
}


#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;
    use image::Rgb;

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

    #[test]
    fn test_gamma_correct_buffer() {
        let mut img = RgbImage::new(2, 2);

        img.put_pixel(0, 0, Rgb([0, 0, 0]));
        img.put_pixel(1, 0, Rgb([255, 255, 255]));
        img.put_pixel(0, 1, Rgb([128, 128, 128]));
        img.put_pixel(1, 1, Rgb([64, 64, 64]));

        gamma_correct_buffer(&mut img, 2.2);

        // ((pixel / 255.0) ^ (1.0 / 2.2)) * 255.0
        assert_eq!(img.get_pixel(0, 0).0, [0, 0, 0]);
        assert_eq!(img.get_pixel(1, 0).0, [255, 255, 255]);
        assert_eq!(img.get_pixel(0, 1).0, [186, 186, 186]);
        assert_eq!(img.get_pixel(1, 1).0, [136, 136, 136]);
    }
}