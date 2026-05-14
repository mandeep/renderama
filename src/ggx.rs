use std::f32::consts::PI;

use glam::Vec3A;
use rand::RngExt;
use rand_pcg::Pcg64Mcg;

use basis::OrthonormalBasis;


/// G1 term of the Smith masking function used in GGX
/// Calculate's how much a surface's microfacets are masked/shadowed
/// from the view direction
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


pub fn ggx_distribution(cosine_half_vector: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let denominator = cosine_half_vector * cosine_half_vector * (a2 - 1.0) + 1.0;
    a2 / (PI * denominator * denominator)
}

pub fn ggx_geometry(cos_i: f32, cos_o: f32, alpha: f32) -> f32 {
    ggx_g1_masking(cos_i, alpha) * ggx_g1_masking(cos_o, alpha)
}

/// Sample a visible GGX half-vector (Heitz 2018).
/// Guarantees wo = reflect(wi, h) is above the surface, eliminating the
/// unbounded weight that causes fireflies with plain NDF sampling.
/// Reference: https://www.jcgt.org/published/0007/04/01/paper.pdf
/// Full code implementation on page 10
pub fn ggx_sample_vndf(normal: Vec3A, wi: Vec3A, alpha: f32, rng: &mut Pcg64Mcg) -> Vec3A {
    let uvw = OrthonormalBasis::new(&normal);
    let vh = Vec3A::new(wi.dot(uvw.u()), wi.dot(uvw.v()), wi.dot(uvw.w()));
    if vh.z <= 0.0 { return uvw.w(); }

    // Transform the view direction into the hemisphere configuration
    let wi_s = Vec3A::new(alpha * vh.x, alpha * vh.y, vh.z).normalize();

    // ONB around wi_s
    let lensq = wi_s.x * wi_s.x + wi_s.y * wi_s.y;
    let t1 = if lensq > 1e-10 {
        Vec3A::new(-wi_s.y, wi_s.x, 0.0) / lensq.sqrt()
    } else {
        Vec3A::new(1.0, 0.0, 0.0)
    };
    let t2 = wi_s.cross(t1);

    // Sample projected area
    let u1 = rng.random::<f32>();
    let u2 = rng.random::<f32>();
    let r = u1.sqrt();
    let phi = 2.0 * PI * u2;
    let p1 = r * phi.cos();
    let p2_raw = r * phi.sin();
    let s = 0.5 * (1.0 + wi_s.z);
    let p2 = (1.0 - s) * (1.0 - p1 * p1).max(0.0).sqrt() + s * p2_raw;

    // Reproject onto unit hemisphere
    let nh = p1 * t1 + p2 * t2 + (1.0 - p1 * p1 - p2 * p2).max(0.0).sqrt() * wi_s;

    // Unstretch back to GGX normal
    let nh_local = Vec3A::new(alpha * nh.x, alpha * nh.y, nh.z.max(0.0)).normalize();
    uvw.local(&nh_local)
}