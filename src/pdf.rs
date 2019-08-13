use std::f32::consts::PI;

use nalgebra::Vector3;
use rand::Rng;

pub fn random_cosine_direction(rng: &mut rand::rngs::ThreadRng) -> Vector3<f32> {
    let r1 = rng.gen::<f32>();
    let r2 = rng.gen::<f32>();
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * PI * r1;
    let x = phi.cos() * 2.0 * r2.sqrt();
    let y = phi.sin() * 2.0 * r2.sqrt();
    Vector3::new(x, y, z)
}
