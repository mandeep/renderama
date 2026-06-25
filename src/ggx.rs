//! GGX functions to be used in accordance with the
//! Cook-Torrance BRDF model.
//!
//! A Reflectance Model for Computer Graphics
//! Robert L. Cook, Kenneth E. Torrance
//! https://dl.acm.org/doi/pdf/10.1145/357290.357293
//!
//! Microfacet Models for Refraction through Rough Surfaces
//! Bruce Walter, Steve Marschner, Hongsong Li, Kenneth E. Torrance
//! https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf
//!
//! Understanding the Masking-Shadowing Function in Microfacet-based BRDFs
//! Eric Heitz
//! https://inria.hal.science/hal-01024289v1/document
//!
//! Sampling the GGX Distribution of Visible Normals
//! Eric Heitz
//! https://pdfs.semanticscholar.org/63bc/928467d760605cdbf77a25bb7c3ad957e40e.pdf 
//!
//! Roughness Using Microfacet Theory
//! Pharr et al.
//! https://pbr-book.org/4ed/Reflection_Models/Roughness_Using_Microfacet_Theory#x5-TheHalf-DirectionTransform

use std::f32::consts::PI;

use glam::Vec3A;
use rand::RngExt;
use rand_pcg::Pcg64Mcg;

use crate::basis::OrthonormalBasis;


/// G1 term of the Smith masking function used in GGX.
///
/// Calculates how much a surface's microfacets are masked/shadowed
/// from the view direction.
///
/// References:
/// Understanding the Masking-Shadowing Function in Microfacet-based BRDFs
/// https://inria.hal.science/hal-01024289/
///
/// Physically Based Rendering in Filament
/// https://google.github.io/filament/Filament.md.html#materialsystem/specularbrdf/geometricshadowingspecularg
pub fn ggx_g1_masking(cosine_view: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;

    // the tangent form of the G1 function has been algebraically formed into a cosine
    // oriented form as seen in the references
    let numerator = 2.0 * cosine_view;
    let denominator = cosine_view + (a2 + (1.0 - a2) * cosine_view * cosine_view).sqrt();
    numerator / denominator
}

/// ggx_distribution represents the facet slope distribution function D
/// in the microfacet BRDF model.
///
/// It represents the fraction of facets that are oriented in the
/// direction of the half vector. This function determines the
/// size and shape of specular highlights.
///
/// References:
/// https://pharr.org/matt/blog/images/average-irregularity-representation-of-a-rough-surface-for-ray-reflection.pdf
pub fn ggx_distribution(half_vector: f32, alpha: f32) -> f32 {
    let alpha = alpha.max(1e-3);
    let a2 = alpha * alpha;
    let denominator = half_vector * half_vector * (a2 - 1.0) + 1.0;
    a2 / (PI * denominator * denominator)
}

/// ggx_geometry represents the geometrical attenuation factor G
/// in the microfacet BRDF model. It accounts for the shadowing
/// and masking of one facet by another.
///
/// References:
/// https://dl.acm.org/doi/pdf/10.1145/357290.357293
pub fn ggx_geometry(cos_i: f32, cos_o: f32, alpha: f32) -> f32 {
    ggx_g1_masking(cos_i, alpha) * ggx_g1_masking(cos_o, alpha)
}

/// The height correlated function correctly accounts for glossy
/// highlights at grazing angles.
///
/// References:
/// https://inria.hal.science/hal-01024289v1/document
/// https://media.gdcvault.com/gdc2017/Presentations/Hammon_Earl_PBR_Diffuse_Lighting.pdf
/// https://pbr-book.org/4ed/Reflection_Models/Roughness_Using_Microfacet_Theory#sec:torrance-sparrow-shadowing-masking
pub fn ggx_height_correlated_geometry(cos_i: f32, cos_o: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;

    let denominator_i = cos_o * (a2 + (1.0 - a2) * cos_i * cos_i).sqrt();
    let denominator_o = cos_i * (a2 + (1.0 - a2) * cos_o * cos_o).sqrt();

    (2.0 * cos_i * cos_o) / (denominator_i + denominator_o)
}

/// Generates a microfacet normal (half vector) based only on the
/// microfacets that are visible in the camera's viewing angle.
///
/// Reference: https://www.jcgt.org/published/0007/04/01/paper.pdf
/// Full code implementation on page 10
pub fn ggx_sample_vndf(normal: &Vec3A, wi: &Vec3A, alpha: &f32, rng: &mut Pcg64Mcg) -> Vec3A {
    let uvw = OrthonormalBasis::new(&normal);
    // convert world to local
    let vh = Vec3A::new(wi.dot(uvw.u()), wi.dot(uvw.v()), wi.dot(uvw.w()));
    if vh.z <= 0.0 {
        return uvw.w();
    }

    let wi_s = Vec3A::new(alpha * vh.x, alpha * vh.y, vh.z).normalize();

    let lensq = wi_s.x * wi_s.x + wi_s.y * wi_s.y;

    let t1 = if lensq > 1e-10 {
        Vec3A::new(-wi_s.y, wi_s.x, 0.0) / lensq.sqrt()
    } else {
        Vec3A::new(1.0, 0.0, 0.0)
    };

    let t2 = wi_s.cross(t1);

    // cosine sample hemisphere but not using function from sampling module
    // as it would break tests. fix this when we refactor the function
    let u1 = rng.random::<f32>();
    let u2 = rng.random::<f32>();
    let r = u1.sqrt();
    let phi = 2.0 * PI * u2;
    let p1 = r * phi.cos();
    let p2_raw = r * phi.sin();
    let s = 0.5 * (1.0 + wi_s.z);
    let p2 = (1.0 - s) * (1.0 - p1 * p1).max(0.0).sqrt() + s * p2_raw;

    let nh = p1 * t1 + p2 * t2 + (1.0 - p1 * p1 - p2 * p2).max(0.0).sqrt() * wi_s;

    let nh_local = Vec3A::new(alpha * nh.x, alpha * nh.y, nh.z.max(0.0)).normalize();

    uvw.local(&nh_local)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;
    use rand_pcg::Pcg64Mcg;
    use rand::SeedableRng;

    #[test]
    fn test_ggx_g1_masking() {
        // ggx_g1_masking returns 1.0 when the surface is 100% visible to the camera
        let alpha = 1.0;
        let cosine = 1.0;
        let result = ggx_g1_masking(cosine, alpha);
        assert!((result - 1.0).abs() < f32::EPSILON);

        let alpha = 0.0;
        let result_smooth = ggx_g1_masking(cosine, alpha);
        assert!((result_smooth - 1.0).abs() < f32::EPSILON);

        // at a grazing angle, the microfacets shadow each other and 0.0 should be returned
        let cosine = 0.0;
        let alpha = 0.5;
        let result = ggx_g1_masking(cosine, alpha);
        assert!((result - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ggx_distribution() {
        let alpha = 1.0;
        let half_vector = 1.0;
        let expected = 1.0 / PI;
        let result = ggx_distribution(half_vector, alpha);
        // perfect alignment and rough surface means function should return alpha / PI
        assert!((result - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ggx_geometry() {
        let cos_i = 1.0;
        let cos_o = 1.0;
        let alpha = 1.0;
        let result = ggx_geometry(cos_i, cos_o, alpha);
        // 1.0 * 1.0 should be 1.0
        assert!((result - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ggx_height_correlated_geometry() {
        let cos_i = 1.0;
        let cos_o = 1.0;
        let alpha = 1.0;
        let result = ggx_height_correlated_geometry(cos_i, cos_o, alpha);
        // numerator should equal denominator
        assert!((result - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ggx_sample_vndf() {
        let mut rng = Pcg64Mcg::seed_from_u64(0);

        let normal = Vec3A::Z;
        let wi = Vec3A::new(1.0, 0.0, 1.0).normalize(); // 45 degree angle of incident
        let alpha = 0.5;

        let sampled_half_vector = ggx_sample_vndf(&normal, &wi, &alpha, &mut rng);

        assert!((sampled_half_vector.length_squared() - 1.0).abs() < f32::EPSILON);
        assert!(sampled_half_vector.z >= 0.0);
    }

    #[test]
    fn test_ggx_sample_vndf_backface() {
        let mut rng = Pcg64Mcg::seed_from_u64(12345);
        let normal = Vec3A::Z;

        let wi = Vec3A::new(0.0, 0.0, -1.0); 
        let alpha = 0.5;

        let sampled_half_vector = ggx_sample_vndf(&normal, &wi, &alpha, &mut rng);

        // test that uvw.z() is returned in degenerate cases
        assert!((sampled_half_vector.x - normal.x).abs() < f32::EPSILON);
        assert!((sampled_half_vector.y - normal.y).abs() < f32::EPSILON);
        assert!((sampled_half_vector.z - normal.z).abs() < f32::EPSILON);
    }
}