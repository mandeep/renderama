use std::f32::consts::PI;

use glam::Vec3A;
use rand::RngExt;
use rand_pcg::Pcg64Mcg;
use rand_distr::StandardNormal;

/// Pick a random point on the unit sphere
///
/// We can use a Gaussian distribution to uniformly generate points
/// on the unit sphere. If a uniform distribution were used instead,
/// the points would tend to aggregate to the poles of the sphere.
/// A vector is created from the sample points taken for each coordinate
/// axis and the unit vector of this newly created vector is returned.
///
/// Reference: http://mathworld.wolfram.com/SpherePointPicking.html
///
pub fn pick_sphere_point(rng: &mut Pcg64Mcg) -> Vec3A {
    let x: f32 = rng.sample(StandardNormal);
    let y: f32 = rng.sample(StandardNormal);
    let z: f32 = rng.sample(StandardNormal);

    Vec3A::new(x, y, z).normalize()
}

pub fn cosine_sample_hemisphere(rng: &mut Pcg64Mcg) -> Vec3A {
    let r1 = rng.random::<f32>();
    let r2 = rng.random::<f32>();

    let phi = 2.0 * PI * r1;

    let r = r2.sqrt();

    // originally x and y were both multiplied by 2.0
    // I'm not sure why this was the case since it's incorrect
    // but I do recall there was a reason for it. If I ever figure
    // it out again just know this is where it happened
    let (sin_phi, cos_phi) = phi.sin_cos();
    let x = cos_phi * r;
    let y = sin_phi * r;
    let z = (1.0 - r2).sqrt();

    Vec3A::new(x, y, z)
}

#[allow(dead_code)]
pub fn uniform_sample_hemisphere(rng: &mut Pcg64Mcg) -> Vec3A {
    let u = rng.random::<f32>();
    let v = rng.random::<f32>();

    let z = u;
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * PI * v;

    let x = r * phi.cos();
    let y = r * phi.sin();

    Vec3A::new(x, y, z)
}

pub fn uniform_sample_sphere(rng: &mut Pcg64Mcg) -> Vec3A {
    let u = rng.random::<f32>();
    let v = rng.random::<f32>();

    let z = 1.0 - (2.0 * u);
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * PI * v;

    let x = r * phi.cos();
    let y = r * phi.sin();

    Vec3A::new(x, y, z)
}

pub fn uniform_sample_cone(cos_theta_max: f32, rng: &mut Pcg64Mcg) -> Vec3A {
    let r1 = rng.random::<f32>();
    let r2 = rng.random::<f32>();
    let cos_theta = 1.0 + r1 * (cos_theta_max - 1.0); // r1=0 → 1, r1=1 → cos_θ_max
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let phi = 2.0 * PI * r2;

    Vec3A::new(cos_theta, sin_theta, phi)
}