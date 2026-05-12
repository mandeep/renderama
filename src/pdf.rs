use std::f32::consts::PI;

use glam::Vec3A;
use rand::rngs::ThreadRng;
use rand::RngExt;

use basis::OrthonormalBasis;
use geometry::Geometry;
use ggx::{ggx_distribution, ggx_g1_masking, ggx_sample_vndf};
use integrator::pick_sphere_point;
use sampling::cosine_sample_hemisphere;


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

pub enum MaterialPDF {
    Cosine { uvw: OrthonormalBasis },
    GGX { wi: Vec3A, normal: Vec3A, alpha: f32 },
    Importance { origin: Vec3A, geometry: Geometry },
    Uniform,
}

impl MaterialPDF {
    pub fn calculate_probability(&self, direction: Vec3A) -> f32 {
        match self {
            MaterialPDF::Cosine { uvw } => {
                let cosine = direction.dot(uvw.w());
                if cosine > 0.0 { cosine / PI } else { 0.0 }
            }
            MaterialPDF::Importance { origin, geometry } => {
                geometry.evaluate_sampling_weight(*origin, direction)
            }
            MaterialPDF::GGX { wi, normal, alpha } => {
                let cos_i = normal.dot(*wi);
                if cos_i <= 0.0 { return 0.0; }
                let h_unnorm = *wi + direction;
                if h_unnorm.length_squared() < 1e-14 { return 0.0; }
                let h = h_unnorm.normalize();
                let cos_h = normal.dot(h);
                if cos_h <= 0.0 || direction.dot(h) <= 0.0 { return 0.0; }
                // VNDF PDF: D * G1(wi) / (4 * cos_i)  [wo·h = wi·h cancels]
                ggx_distribution(cos_h, *alpha) * ggx_g1_masking(cos_i, *alpha) / (4.0 * cos_i)
            }
            MaterialPDF::Uniform => 1.0 / (4.0 * PI),
        }
    }

    pub fn pick_direction(&self, rng: &mut ThreadRng) -> Vec3A {
        match self {
            MaterialPDF::Cosine { uvw } => {
                uvw.local(&cosine_sample_hemisphere(rng))
            }
            MaterialPDF::Importance { origin, geometry } => {
                geometry.sample_direction_to_light(*origin, rng)
            },
            MaterialPDF::GGX { wi, normal, alpha } => {
                let h = ggx_sample_vndf(*normal, *wi, *alpha, rng);
                let wi_dot_h = wi.dot(h);
                if wi_dot_h <= 0.0 { return *normal; }
                2.0 * wi_dot_h * h - *wi
            }
            MaterialPDF::Uniform => pick_sphere_point(rng),
        }
    }
}

pub struct HybridPDF<'a> {
        material_pdf: &'a MaterialPDF,
        importance_pdf: &'a MaterialPDF,
}

impl<'a> HybridPDF<'a> {
    pub fn new(material_pdf: &'a MaterialPDF, importance_pdf: &'a MaterialPDF) -> HybridPDF<'a> {
        HybridPDF { material_pdf, importance_pdf }
    }

    pub fn value(&self, direction: Vec3A) -> f32 {
        0.5 * self.material_pdf.calculate_probability(direction) + 0.5 * self.importance_pdf.calculate_probability(direction)
    }

    pub fn generate(&self, rng: &mut ThreadRng) -> Vec3A {
        if rng.random::<f32>() < 0.5 {
            self.material_pdf.pick_direction(rng)
        } else {
            self.importance_pdf.pick_direction(rng)
        }
    }
}