use glam::Vec3A;
use rand::rngs::ThreadRng;

use geometry::Geometry;
use plane::Plane;
use sphere::Sphere;

#[derive(Clone)]
pub enum LightGeometry {
    Plane(Plane),
    Sphere(Sphere),
}

#[derive(Clone)]
pub struct Light {
    pub geometry: LightGeometry,
    pub emission: Vec3A,
}

macro_rules! impl_from_geometry {
    ($($t:ty => $v:ident),*) => {
        $(
            impl From<$t> for LightGeometry {
                fn from(m: $t) -> Self {
                    LightGeometry::$v(m)
                }
            }
        )*
    };
}

impl_from_geometry!(
    Plane => Plane,
    Sphere => Sphere
);

impl From<Geometry> for LightGeometry {
    fn from(geom: Geometry) -> Self {
        match geom {
            Geometry::Plane(p) => LightGeometry::Plane(p),
            Geometry::Sphere(s) => LightGeometry::Sphere(s),
            Geometry::ReverseOrientation(inner) => {
                // convert a ReverseOrientation Geometry type back into a Plane
                Self::from(*inner)
            },
            _ => panic!("This geometry type cannot be used as a light source!"),
        }
    }
}

impl Light {
    pub fn new(geometry: LightGeometry, emission: Vec3A) -> Light {
        Light { geometry, emission }
    }
    pub fn evaluate_sampling_weight(&self, origin: Vec3A, direction: Vec3A) -> f32 {
        match &self.geometry {
            LightGeometry::Plane(p) => p.evaluate_sampling_weight(origin, direction),
            LightGeometry::Sphere(s) => s.evaluate_sampling_weight(origin, direction),
        }
    }

    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut ThreadRng) -> Vec3A {
        match &self.geometry {
            LightGeometry::Plane(p) => p.sample_direction_to_light(origin, rng),
            LightGeometry::Sphere(s) => s.sample_direction_to_light(origin, rng),
        }
    }

    /// Upper bound on `t` for a shadow ray occlusion test that excludes the light surface itself.
    /// `light_distance` must be measured from the same origin as the shadow ray.
    pub fn calculate_distance_from(&self, light_distance: f32) -> f32 {
        match &self.geometry {
            LightGeometry::Plane(_) => light_distance - 1e-3,
            LightGeometry::Sphere(s) => light_distance - s.radius.abs() - 1e-3,
        }
    }

    pub fn to_geometry(&self) -> Geometry {
        match &self.geometry {
            LightGeometry::Plane(p) => Geometry::Plane(p.clone()),
            LightGeometry::Sphere(s) => Geometry::Sphere(s.clone()),
        }
    }
}
