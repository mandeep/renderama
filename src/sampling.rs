use rand::rngs::ThreadRng;
use rand::RngExt;
use std::f32::consts::PI;

use glam::Vec3A;

pub fn cosine_sample_hemisphere(rng: &mut ThreadRng) -> Vec3A {
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

pub fn uniform_sample_hemisphere(rng: &mut ThreadRng) -> Vec3A {
    let u = rng.random::<f32>();
    let v = rng.random::<f32>();

    let z = u;
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * PI * v;

    let x = r * phi.cos();
    let y = r * phi.sin();

    Vec3A::new(x, y, z)
}

pub fn uniform_sample_sphere(rng: &mut ThreadRng) -> Vec3A {
    let u = rng.random::<f32>();
    let v = rng.random::<f32>();

    let z = 1.0 - (2.0 * u);
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * PI * v;

    let x = r * phi.cos();
    let y = r * phi.sin();

    Vec3A::new(x, y, z)
}

pub fn uniform_sample_cone(cos_theta_max: f32, rng: &mut ThreadRng) -> Vec3A {
    let r1 = rng.random::<f32>();
    let r2 = rng.random::<f32>();
    let cos_theta = 1.0 + r1 * (cos_theta_max - 1.0); // r1=0 → 1, r1=1 → cos_θ_max
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let phi = 2.0 * PI * r2;

    Vec3A::new(cos_theta, sin_theta, phi)
}