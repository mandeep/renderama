use glam::Vec3A;
use rand::rngs::ThreadRng;

use primitive::Primitive;
use plane::Plane;
use sphere::Sphere;

#[derive(Clone)]
pub enum LightPrimitive {
    Plane(Plane),
    Sphere(Sphere),
}

#[derive(Clone)]
pub struct Light {
    pub primitive: LightPrimitive,
    pub emission: Vec3A,
}

macro_rules! impl_from_primitive {
    ($($t:ty => $v:ident),*) => {
        $(
            impl From<$t> for LightPrimitive {
                fn from(m: $t) -> Self {
                    LightPrimitive::$v(m)
                }
            }
        )*
    };
}

impl_from_primitive!(
    Plane => Plane,
    Sphere => Sphere
);

impl From<Primitive> for LightPrimitive {
    fn from(geom: Primitive) -> Self {
        match geom {
            Primitive::Plane(p) => LightPrimitive::Plane(p),
            Primitive::Sphere(s) => LightPrimitive::Sphere(s),
            Primitive::ReverseOrientation(inner) => {
                // convert a ReverseOrientation Primitive type back into a Plane
                Self::from(*inner)
            },
            _ => panic!("This primitive type cannot be used as a light source!"),
        }
    }
}

impl Light {
    pub fn new(primitive: LightPrimitive, emission: Vec3A) -> Light {
        Light { primitive, emission }
    }
    pub fn evaluate_sampling_weight(&self, origin: Vec3A, direction: Vec3A) -> f32 {
        match &self.primitive {
            LightPrimitive::Plane(p) => p.evaluate_sampling_weight(origin, direction),
            LightPrimitive::Sphere(s) => s.evaluate_sampling_weight(origin, direction),
        }
    }

    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut ThreadRng) -> Vec3A {
        match &self.primitive {
            LightPrimitive::Plane(p) => p.sample_direction_to_light(origin, rng),
            LightPrimitive::Sphere(s) => s.sample_direction_to_light(origin, rng),
        }
    }

    /// Upper bound on `t` for a shadow ray occlusion test that excludes the light surface itself.
    /// `light_distance` must be measured from the same origin as the shadow ray.
    pub fn calculate_distance_from(&self, light_distance: f32) -> f32 {
        match &self.primitive {
            LightPrimitive::Plane(_) => light_distance - 1e-3,
            LightPrimitive::Sphere(s) => light_distance - s.radius.abs() - 1e-3,
        }
    }

    #[allow(dead_code)]
    pub fn to_primitive(&self) -> Primitive {
        match &self.primitive {
            LightPrimitive::Plane(p) => Primitive::Plane(p.clone()),
            LightPrimitive::Sphere(s) => Primitive::Sphere(s.clone()),
        }
    }
}
