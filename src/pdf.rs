use std::f32::consts::PI;

use derive_more::IsVariant;
use glam::Vec3A;
use rand::{Rng, RngExt};

use crate::basis::OrthonormalBasis;
use crate::ggx::{ggx_distribution, ggx_g1_masking, ggx_sample_vndf};
use crate::materials::reflect;
use crate::sampling::{cosine_sample_hemisphere, pick_sphere_point};


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
///
/// Deriving the IsVariant trait creates methods for each member of the enum
/// that allows for member checking, e.g. is_delta() is created and can be used
/// to check whether a sampling_strategy is a Delta distribution.
#[derive(IsVariant)]
pub enum PDF {
    Cosine { uvw: OrthonormalBasis },
    Composite {
        uvw: OrthonormalBasis,
        wi: Vec3A,
        normal: Vec3A,
        alpha: f32,
        specular_weight: f32,
        clearcoat_alpha: f32,
        clearcoat_weight: f32,
    },
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
            PDF::Composite { uvw, wi, normal, alpha, specular_weight, clearcoat_alpha, clearcoat_weight } => {
                let diffuse_pdf = PDF::Cosine { uvw: *uvw }.calculate_probability(direction);
                let specular_pdf = PDF::GGX { wi: *wi, normal: *normal, alpha: *alpha }.calculate_probability(direction);
                let clearcoat_pdf = PDF::GGX { wi: *wi, normal: *normal, alpha: *clearcoat_alpha }.calculate_probability(direction);

                let diffuse_weight = (1.0 - *specular_weight - *clearcoat_weight).max(0.0);
                (*clearcoat_weight * clearcoat_pdf) + (*specular_weight * specular_pdf) + (diffuse_weight * diffuse_pdf)
            }
            PDF::Delta => panic!("Delta PDF has no meaningful probability."),
            PDF::GGX { wi, normal, alpha } => {
                let cos_i = normal.dot(*wi);
                if cos_i <= 0.0 {
                    return 0.0;
                }

                let half_vector = wi + direction;
                if half_vector.length_squared() < 1e-14 {
                    return 0.0;
                }

                let half_vector_norm = half_vector.normalize();
                let cos_h = normal.dot(half_vector_norm);
                if cos_h <= 0.0 || direction.dot(half_vector_norm) <= 0.0 {
                    return 0.0;
                }

                ggx_distribution(cos_h, *alpha) * ggx_g1_masking(cos_i, *alpha) / (4.0 * cos_i)
            },
            PDF::Uniform => 1.0 / (4.0 * PI),
        }
    }

    /// Generate a new direction by sampling from the distribution.
    pub fn pick_direction(&self, rng: &mut impl Rng) -> Vec3A {
        match self {
            PDF::Cosine { uvw } => {
                uvw.local(&cosine_sample_hemisphere(rng))
            },
            PDF::Composite { uvw, wi, normal, alpha, specular_weight, clearcoat_alpha, clearcoat_weight } => {
                let u = rng.random::<f32>();
                    if u < *clearcoat_weight {
                        PDF::GGX { wi: *wi, normal: *normal, alpha: *clearcoat_alpha }.pick_direction(rng)
                    } else if u < *clearcoat_weight + *specular_weight {
                        PDF::GGX { wi: *wi, normal: *normal, alpha: *alpha }.pick_direction(rng)
                    } else {
                        PDF::Cosine { uvw: *uvw }.pick_direction(rng)
                    }
            }
            PDF::Delta => panic!("Delta PDF should never be sampled directly."),
            PDF::GGX { wi, normal, alpha } => {
                let h = ggx_sample_vndf(normal, wi, alpha, rng);

                // if the microfacet is hit from behind due to floating point precision
                // errors, then we discard the sampled normal and use the macrosurface normal
                let wi_dot_h = wi.dot(h);
                if wi_dot_h <= 0.0 {
                    return *normal;
                }

                // we reflect wi across h to obtain the outgoing vector wo
                // wi is negative since the reflect function expects the vectors to point
                // towards the surface when we model it here as pointing away from the surface
                // https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf equation (39)
                reflect(-wi, h)
            },
            PDF::Uniform => pick_sphere_point(rng),
        }
    }
}
