use crate::f32::consts::PI;

use glam::Vec3A;
use rand::{Rng, RngExt};

/// Retrieve the relative luminance of a color in sRGB colorspace.
///
/// Necessary to determine luminance when sampling in the
/// direction of high luminance areas.
///
/// Reference: https://www.w3.org/WAI/GL/wiki/Relative_luminance
fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Given a cdf and random variable u,
/// use binary search to find largest values in the area around u.
fn sample_brightest_pixels(cdf: &[f32], u: f32) -> usize {
    // use rust's built-in binary search algorithm rather than writing
    // our own. find the boundary where probability p is less than u and
    // return the index where this becomes false.
    cdf.partition_point(|&p| p <= u)
        .saturating_sub(1)
        .min(cdf.len().saturating_sub(2))
}

/// EnvironmentMap allows for the use of HDRI image-based lighting.
///
/// We set up conditional distribution functions (CDF) to map pixels to luminance.
/// CDFs lets us compute the probability of a variable taking
/// on a value in a specified range.
///
/// References:
/// https://www.pbr-book.org/3ed-2018/Monte_Carlo_Integration/2D_Sampling_with_Multidimensional_Transformations#eq:2d-discrete-conditional-density
/// https://github.com/mmp/pbrt-v3/blob/master/src/core/sampling.h
/// https://glue.mustafaisik.net/2018/10/image-based-lighting.html
/// https://www.cg.tuwien.ac.at/sites/default/files/course/4411/attachments/06_importance_sampling_0.pdf
#[derive(Clone)]
pub struct EnvironmentMap {
    pixels: Vec<Vec3A>,
    width: usize,
    height: usize,
    marginal_cdf: Vec<f32>,
    conditional_cdf: Vec<f32>,
    total_weight: f32,
    #[allow(unused)]
    max_luminance: f32, // for tone mapping
    intensity: f32,
}

impl EnvironmentMap {
    /// Create a new EnvironmentMap from the image at the given path.
    pub fn new(filename: &str, intensity: f32) -> EnvironmentMap {
        let img = image::open(filename)
            .expect(&format!("Failed to open environment map '{}'", filename))
            .into_rgb32f();

        let width = img.width() as usize;
        let height = img.height() as usize;

        let pixels = img.pixels()
            .map(|pixel| Vec3A::new(pixel[0], pixel[1], pixel[2]))
            .collect();

        // conditional_pdf stores the probabilities for picking a
        // specific pixel within a single row
        let mut conditional_cdf = vec![0.0f32; height * (width + 1)];

        // stores the total brightness of each row
        let mut marginal_weights = vec![0.0f32; height];

        let mut max_luminance = 0.0f32;

        // compute the horizontal probabilities
        for j in 0..height {
            // since HDRI images are equirectangular projections we
            // need to account for the stretching at the poles
            let sin_theta = (PI * (j as f32 + 0.5) / height as f32).sin();

            let row = j * (width + 1);
            conditional_cdf[row] = 0.0;

            // calculate the luminance for every pixel in this row and weigh by sin_theta.
            for i in 0..width {
                let pixel = img.get_pixel(i as u32, j as u32);
                let luminance = luminance(pixel[0], pixel[1], pixel[2]) * sin_theta;
                max_luminance = max_luminance.max(luminance);
                conditional_cdf[row + i + 1] = conditional_cdf[row + i] + luminance;
            }

            // the row's total sum of brightness at conditional_cdf[row + width]
            // is saved to marginal_weights. every row is then divided by the
            // row_sum to normalize the row and create a valid probability distribution
            marginal_weights[j] = conditional_cdf[row + width];
            let row_sum = marginal_weights[j];
            if row_sum > 0.0 {
                for i in 1..=width {
                    conditional_cdf[row + i] /= row_sum;
                }
            }
        }

        // build a running total of the brightness of each row
        let mut marginal_cdf = vec![0.0f32; height + 1];
        for j in 0..height {
            marginal_cdf[j + 1] = marginal_cdf[j] + marginal_weights[j];
        }

        // normalize the marginal_cdf totals
        let total_weight = marginal_cdf[height];
        if total_weight > 0.0 {
            for j in 1..=height {
                marginal_cdf[j] /= total_weight;
            }
        }

        EnvironmentMap { pixels, width, height, marginal_cdf, conditional_cdf, total_weight, max_luminance, intensity }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates.
    ///
    /// Converts the direction into spherical UV coordinates via atan2 and asin
    /// and performs a nearest neighbor pixel fetch.
    pub fn sample_map(&self, direction: &Vec3A) -> Vec3A {
        let u = 0.5 + direction.z.atan2(direction.x) / (2.0 * PI);
        let v = 0.5 - direction.y.asin() / PI;

        let i = ((u * self.width as f32) as usize).min(self.width - 1);
        let j = ((v * self.height as f32) as usize).min(self.height - 1);

        let pixel = self.pixels[j * self.width + i];

        pixel * self.intensity
    }

    /// Compute the weight on how likely we will sample in the given direction.
    ///
    /// After converting the direction into spherical UV coordinates,
    /// we retrieve the luminance and compute the weight of that luminance.
    pub fn evaluate_sampling_weight(&self, direction: &Vec3A) -> (Vec3A, f32) {
        if self.total_weight <= 0.0 {
            return (Vec3A::ZERO, 0.0);
        }

        let u = 0.5 + direction.z.atan2(direction.x) / (2.0 * PI);
        let v = 0.5 - direction.y.asin() / PI;

        let i = ((u * self.width as f32) as usize).min(self.width - 1);
        let j = ((v * self.height as f32) as usize).min(self.height - 1);

        let pixel = self.pixels[j * self.width + i];

        // since we're working in the solid angle domain we need to transform
        // p(i, j) to p(w).
        // https://glue.mustafaisik.net/2018/10/image-based-lighting.html
        let luminance = luminance(pixel.x, pixel.y, pixel.z);

        let weight = luminance * self.width as f32 * self.height as f32 / (self.total_weight * 2.0 * PI * PI);

        (pixel * self.intensity, weight)
    }

    /// Find areas of high luminance in the conditional_cdf and
    /// return the direction to that area.
    pub fn sample_direction_to_light(&self, rng: &mut impl Rng) -> (Vec3A, Vec3A, f32) {
        let u1 = rng.random::<f32>();
        let u2 = rng.random::<f32>();

        // sample the brightest rows using random variable u1
        let j = sample_brightest_pixels(&self.marginal_cdf, u1);

        // now that we've found a row of high density, we sample
        // the column with random variable u2
        let row = j * (self.width + 1);
        let i = sample_brightest_pixels(&self.conditional_cdf[row..row + self.width + 1], u2);

        // look into continuous inverse sampling transform instead of
        // using a jittered sampling appraoch.
        let u = (i as f32 + 0.5) / self.width as f32;
        let v = (j as f32 + 0.5) / self.height as f32;

        // convert cartesian coordinates to spherical coordinates
        let phi = (u - 0.5) * 2.0 * PI;
        let elevation = (0.5 - v) * PI;
        let (sin_el, cos_el) = elevation.sin_cos();
        let (sin_phi, cos_phi) = phi.sin_cos();

        // map spherical angles to 3D direction vector in direction of light source
        let direction = Vec3A::new(cos_el * cos_phi, sin_el, cos_el * sin_phi);

        let pixel = self.pixels[j * self.width + i];
        let color = pixel * self.intensity;

        // since we're working in the solid angle domain we need to transform
        // p(i, j) to p(w).
        // https://glue.mustafaisik.net/2018/10/image-based-lighting.html
        let luminance = luminance(pixel.x, pixel.y, pixel.z);
        let weight = luminance * self.width as f32 * self.height as f32 / (self.total_weight * 2.0 * PI * PI);

        (direction, color, weight)
    }
}
