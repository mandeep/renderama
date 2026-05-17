use std::f32::consts::PI;

use glam::Vec3A;
use rand_pcg::Pcg64Mcg;

use basis::OrthonormalBasis;
use ggx::{ggx_distribution, ggx_g1_masking, ggx_sample_vndf};
use sampling::{cosine_sample_hemisphere, pick_sphere_point};


#[allow(unused)]
/// The balance heuristic weighs samples by their relative PDF contribution.
///
/// Reference: https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/Importance_Sampling
pub fn balance_heuristic(f_pdf: f32, g_pdf: f32) -> f32 {
    let sum = f_pdf + g_pdf;
    if sum > 0.0 { f_pdf / sum } else { 0.0 }
}

/// The power heuristic weighs samples to reduce variance
///
/// See https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/Importance_Sampling
/// for more information
pub fn power_heuristic(f_pdf: f32, g_pdf: f32) -> f32 {
    let f2 = f_pdf * f_pdf;
    let g2 = g_pdf * g_pdf;
    f2 / (f2 + g2)
}

/// PDF enum houses the different ways we can sample directions
/// to determine how likely a ray is to be sampled in that direction.
pub enum PDF {
    Cosine { uvw: OrthonormalBasis },
    Delta,
    GGX { wi: Vec3A, normal: Vec3A, alpha: f32 },
    Uniform,
}

impl PDF {
    /// Calculate the PDF value for the given direction.
    pub fn calculate_probability(&self, direction: Vec3A) -> f32 {
        match self {
            PDF::Cosine { uvw } => {
                let cosine = direction.dot(uvw.w());
                if cosine > 0.0 { cosine / PI } else { 0.0 }
            },
            PDF::Delta => panic!("Delta PDF has no meaningful probability."),
            PDF::GGX { wi, normal, alpha } => {
                let cos_i = normal.dot(*wi);
                if cos_i <= 0.0 { return 0.0; }
                let h_unnorm = *wi + direction;
                if h_unnorm.length_squared() < 1e-14 { return 0.0; }
                let h = h_unnorm.normalize();
                let cos_h = normal.dot(h);
                if cos_h <= 0.0 || direction.dot(h) <= 0.0 { return 0.0; }

                ggx_distribution(cos_h, *alpha) * ggx_g1_masking(cos_i, *alpha) / (4.0 * cos_i)
            },
            PDF::Uniform => 1.0 / (4.0 * PI),
        }
    }

    /// Generate a new direction by sampling from the distribution.
    pub fn pick_direction(&self, rng: &mut Pcg64Mcg) -> Vec3A {
        match self {
            PDF::Cosine { uvw } => {
                uvw.local(&cosine_sample_hemisphere(rng))
            },
            PDF::Delta => panic!("Delta PDF should never be sampled directly."),
            PDF::GGX { wi, normal, alpha } => {
                let h = ggx_sample_vndf(*normal, *wi, *alpha, rng);
                let wi_dot_h = wi.dot(h);
                if wi_dot_h <= 0.0 { return *normal; }
                2.0 * wi_dot_h * h - *wi
            },
            PDF::Uniform => pick_sphere_point(rng),
        }
    }
}
