use std::sync::Arc;

use glam::Vec3A;
use rand_pcg::Pcg64Mcg;

use primitive::Primitive;
use plane::Plane;
use ray::Ray;
use sphere::Sphere;

#[derive(Clone)]
/// Enum that holds all primitives that can be used as a Light
pub enum LightPrimitive {
    Plane(Plane),
    Sphere(Sphere),
}

#[derive(Clone)]
/// A light source used to add light emission in a scene
pub struct Light {
    pub primitive: LightPrimitive,
    pub intensity: Vec3A,
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

/// Provide easy conversion from Primitive to LightPrimitive.
///
/// This trait allows the caller to convert to LightPrimitive using
/// the .into() method.
impl From<Primitive> for LightPrimitive {
    fn from(primitive: Primitive) -> Self {
        match primitive {
            Primitive::Plane(plane) => LightPrimitive::Plane(plane),
            Primitive::Sphere(sphere) => LightPrimitive::Sphere(sphere),
            Primitive::ReverseOrientation(primitive) => {
                let inner_primitive = Arc::unwrap_or_clone(primitive);
                LightPrimitive::from(inner_primitive)
            },
            _ => panic!("This primitive type cannot be used as a light source!"),
        }
    }
}

impl Light {
    /// Create a new Light from the given primtive and intensity.
    pub fn new(primitive: LightPrimitive, intensity: Vec3A) -> Light {
        Light { primitive, intensity }
    }

    /// Dispatch the weight evaluation to the primitive.
    pub fn evaluate_sampling_weight(&self, ray: &Ray) -> f32 {
        match &self.primitive {
            LightPrimitive::Plane(plane) => plane.evaluate_sampling_weight(ray),
            LightPrimitive::Sphere(sphere) => sphere.evaluate_sampling_weight(ray),
        }
    }

    /// Dispatch the importance sampling to the primitive.
    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut Pcg64Mcg) -> Vec3A {
        match &self.primitive {
            LightPrimitive::Plane(plane) => plane.sample_direction_to_light(origin, rng),
            LightPrimitive::Sphere(sphere) => sphere.sample_direction_to_light(origin, rng),
        }
    }

    /// Calculate the exact distance from the light source for the use with shadpw rays.
    pub fn calculate_distance_from(&self, light_distance: f32) -> f32 {
        match &self.primitive {
            LightPrimitive::Plane(_) => light_distance - 1e-3,
            LightPrimitive::Sphere(sphere) => light_distance - sphere.radius.abs() - 1e-3,
        }
    }
}
