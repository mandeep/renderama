use rand::rngs::ThreadRng;
use rand::Rng;
use std::f32::consts::PI;

use glam::Vec3;

pub fn cosine_sample_hemisphere(rng: &mut ThreadRng) -> Vec3 {
    let r1 = rng.gen::<f32>();
    let r2 = rng.gen::<f32>();

    let phi = 2.0 * PI * r1;

    let r = r2.sqrt();

    // originally x and y were both multiplied by 2.0
    // I'm not sure why this was the case since it's incorrect
    // but I do recall there was a reason for it. If I ever figure
    // it out again just know this is where it happened
    let x = phi.cos() * r;
    let y = phi.sin() * r;
    let z = (1.0 - r2).sqrt();

    Vec3::new(x, y, z)
}

pub fn uniform_sample_hemisphere(rng: &mut ThreadRng) -> Vec3 {
    let u = rng.gen::<f32>();
    let v = rng.gen::<f32>();

    let z = u;
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * PI * v;

    let x = r * phi.cos();
    let y = r * phi.sin();

    Vec3::new(x, y, z)
}

pub fn uniform_sample_sphere(rng: &mut ThreadRng) -> Vec3 {
    let u = rng.gen::<f32>();
    let v = rng.gen::<f32>();

    let z = 1.0 - (2.0 * u);
    let r = (1.0 - z * z).sqrt();
    let phi = 2.0 * PI * v;

    let x = r * phi.cos();
    let y = r * phi.sin();

    Vec3::new(x, y, z)
}
