use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use glam::{Vec2, Vec3A};
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
pub fn pick_sphere_point(rng: &mut Pcg64Mcg) -> Vec3A {
    let x: f32 = rng.sample(StandardNormal);
    let y: f32 = rng.sample(StandardNormal);
    let z: f32 = rng.sample(StandardNormal);

    Vec3A::new(x, y, z).normalize()
}

/// Pick a point on a unit disk using concentric mapping.
///
/// References:
/// https://www.pbr-book.org/3ed-2018/Monte_Carlo_Integration/2D_Sampling_with_Multidimensional_Transformations
pub fn pick_disk_point(rng: &mut Pcg64Mcg) -> Vec2 {
    let u: f32 = rng.random_range(-1.0..1.0);
    let v: f32 = rng.random_range(-1.0..1.0);

    if u == 0.0 && v == 0.0 {
        return Vec2::ZERO;
    }

    let (r, theta) = if u.abs() > v.abs() {
        (u, FRAC_PI_4 * (v / u))
    }  else {
        (v, FRAC_PI_2 - FRAC_PI_4 * (u / v))
    };

    let (sin_theta, cos_theta) = theta.sin_cos();

    Vec2::new(r * cos_theta, r * sin_theta)
}

/// Sample a cosine-weighted vector from the hemisphere.
///
/// Matches the cos_theta term in the rendering equation making
/// it necessary for calculating the reflectance of Diffuse materials.
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

/// Sample uniformly on a hemisphere.
///
/// Useful when equal probability across a hemisphere is necessary.
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

/// Sample uniformly over an entire sphere.
///
/// Useful when needing to sample directions for items that
/// scatter/radiate in all directions equally. Volumes, lights, etc.
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

/// Sample uniformly over a cone.
///
/// Needed when sampling items that subtend a cone of directions like
/// the sun or other light sources.
///
/// References: https://pbr-book.org/3ed-2018/Light_Transport_I_Surface_Reflection/Sampling_Light_Sources
pub fn uniform_sample_cone(cos_theta_max: f32, rng: &mut Pcg64Mcg) -> Vec3A {
    let r1 = rng.random::<f32>();
    let r2 = rng.random::<f32>();
    let cos_theta = 1.0 + r1 * (cos_theta_max - 1.0);
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let phi = 2.0 * PI * r2;
    let (sin_phi, cos_phi) = phi.sin_cos();

    Vec3A::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use glam::Vec3A;

    fn get_rng() -> Pcg64Mcg {
        Pcg64Mcg::seed_from_u64(0)
    }

    fn assert_unit_vector(v: Vec3A) {
        let length = v.length();
        assert!((length - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_pick_sphere_point() {
        let mut rng = get_rng();
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;

        for _ in 0..10_000 {
            let v = pick_sphere_point(&mut rng);
            assert_unit_vector(v);
            min_z = min_z.min(v.z);
            max_z = max_z.max(v.z);
        }

        assert!(min_z < -0.9);
        assert!(max_z > 0.9);
    }

    #[test]
    fn test_uniform_sample_sphere() {
        let mut rng = get_rng();
        let mut sum_z = 0.0;

        for _ in 0..10_000 {
            let v = uniform_sample_sphere(&mut rng);
            assert_unit_vector(v);
            sum_z += v.z;
        }

        let avg_z = sum_z / 10_000.0;

        assert!(avg_z.abs() < 0.01);
    }

    #[test]
    fn test_uniform_sample_hemisphere() {
        let mut rng = get_rng();
        let mut sum_z = 0.0;

        for _ in 0..10_000 {
            let v = uniform_sample_hemisphere(&mut rng);
            assert_unit_vector(v);
            assert!(v.z >= 0.0);
            sum_z += v.z;
        }

        let avg_z = sum_z / 10_000.0;
        assert!((avg_z - 0.5).abs() < 0.02);
    }

    #[test]
    fn test_cosine_sample_hemisphere() {
        let mut rng = get_rng();
        let mut sum_z = 0.0;

        for _ in 0..10_000 {
            let v = cosine_sample_hemisphere(&mut rng);
            assert_unit_vector(v);
            assert!(v.z >= 0.0);
            sum_z += v.z;
        }

        let avg_z = sum_z / 10_000.0;
        assert!((avg_z - 0.666).abs() < 0.02);
    }

    #[test]
    fn test_uniform_sample_cone() {
        let mut rng = get_rng();

        // use an angle of 60 degrees
        let cos_theta_max = 0.5;

        for _ in 0..10_000 {
            let v = uniform_sample_cone(cos_theta_max, &mut rng);
            assert_unit_vector(v);

            assert!(v.z >= cos_theta_max - 1e-5);
        }
    }
}