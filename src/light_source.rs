use glam::Vec3;
use rand::rngs::ThreadRng;

use geometry::Geometry;
use plane::Plane;
use sphere::Sphere;

#[derive(Clone)]
pub enum LightSource {
    Plane(Plane),
    Sphere(Sphere),
}

impl LightSource {
    pub fn pdf_value(&self, origin: Vec3, direction: Vec3) -> f32 {
        match self {
            LightSource::Plane(p) => p.pdf_value(origin, direction),
            LightSource::Sphere(s) => s.pdf_value(origin, direction),
        }
    }

    pub fn pdf_from_hit(&self, parameter: f32, direction: Vec3, hit_normal: Vec3) -> f32 {
        match self {
            LightSource::Plane(p) => p.pdf_from_hit(parameter, direction, hit_normal),
            LightSource::Sphere(s) => s.pdf_from_hit(parameter, direction, hit_normal),
        }
    }

    pub fn pdf_random(&self, origin: Vec3, rng: &mut ThreadRng) -> Vec3 {
        match self {
            LightSource::Plane(p) => p.pdf_random(origin, rng),
            LightSource::Sphere(s) => s.pdf_random(origin, rng),
        }
    }

    pub fn to_geometry(&self) -> Geometry {
        match self {
            LightSource::Plane(p) => Geometry::Plane(p.clone()),
            LightSource::Sphere(s) => Geometry::Sphere(s.clone()),
        }
    }
}
