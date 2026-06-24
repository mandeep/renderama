use std::sync::Arc;

use glam::Vec3A;
use rand_pcg::Pcg64Mcg;
use rand::RngExt;

use crate::bvh::BVH;
use crate::primitive::Primitive;
use crate::plane::Plane;
use crate::ray::Ray;
use crate::sampling::uniform_sample_triangle;
use crate::sphere::Sphere;
use crate::triangle::Triangle;


#[derive(Clone)]
/// Enum that holds all primitives that can be used as a Light
pub enum LightPrimitive {
    MeshLight(MeshLight),
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
    MeshLight => MeshLight,
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
    pub fn new(primitive: impl Into<LightPrimitive>, intensity: Vec3A) -> Light {
        Light { primitive: primitive.into(), intensity }
    }

    /// Dispatch the weight evaluation to the primitive.
    pub fn evaluate_sampling_weight(&self, ray: &Ray, rng: &mut Pcg64Mcg) -> f32 {
        match &self.primitive {
            LightPrimitive::MeshLight(mesh_light) => mesh_light.evaluate_sampling_weight(ray, rng),
            LightPrimitive::Plane(plane) => plane.evaluate_sampling_weight(ray),
            LightPrimitive::Sphere(sphere) => sphere.evaluate_sampling_weight(ray),
        }
    }

    /// Dispatch the importance sampling to the primitive.
    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut Pcg64Mcg) -> Vec3A {
        match &self.primitive {
            LightPrimitive::MeshLight(mesh_light) => mesh_light.sample_direction_to_light(origin, rng),
            LightPrimitive::Plane(plane) => plane.sample_direction_to_light(origin, rng),
            LightPrimitive::Sphere(sphere) => sphere.sample_direction_to_light(origin, rng),
        }
    }

    /// Calculate the exact distance from the light source for the use with shadpw rays.
    pub fn calculate_distance_from(&self, light_distance: f32) -> f32 {
        match &self.primitive {
            LightPrimitive::MeshLight(_)
            | LightPrimitive::Plane(_) => light_distance - 1e-3,
            LightPrimitive::Sphere(sphere) => light_distance - sphere.radius.abs() - 1e-3,
        }
    }
}

/// MeshLight is a light primitive used when loading emissives
/// from obj files.
///
/// When loading objs, any mesh that contains an emissive material
/// will have a MeshLight created for it. This way both bsdf sampling
/// and light sampling still occur for loaded meshes.
#[derive(Clone)]
pub struct MeshLight {
    triangles: Vec<Triangle>,
    cdf: Vec<f32>,
    total_area: f32,
    accelerator: BVH,
}

impl MeshLight {
    /// Create a new MeshLight from the given triangles
    pub fn new(triangles: Vec<Triangle>) -> Self {
        let mut light_triangles = Vec::new();
        let mut cdf = Vec::new();
        let mut total_area = 0.0;

        for triangle in triangles {
            let area = triangle.area();

            if area > f32::EPSILON {
                total_area += area;
                cdf.push(total_area);
                light_triangles.push(triangle);
            }
        }

        // TODO: architect a way that we don't need to clone the triangles
        let mut geometries: Vec<Primitive> = light_triangles
            .iter()
            .cloned()
            .map(Primitive::Triangle)
            .collect();

        let accelerator = BVH::new(&mut geometries);

        MeshLight { triangles: light_triangles, cdf, total_area, accelerator }
    }

    /// Sample a random triangle from the triangles vector
    fn sample_triangle(&self, rng: &mut Pcg64Mcg) -> &Triangle {
        let target = rng.random::<f32>() * self.total_area;

        let index = self
            .cdf
            .partition_point(|&x| x < target)
            .min(self.triangles.len() - 1);

        &self.triangles[index]
    }

    /// Sample the direction to this light source from the given origin
    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut Pcg64Mcg) -> Vec3A {
        let triangle = self.sample_triangle(rng);
        let barycentric = uniform_sample_triangle(rng);

        let point_on_light = triangle.interpolate_position(barycentric);

        point_on_light - origin
    }

    /// Evaluate the sampling weight of this MeshLight
    pub fn evaluate_sampling_weight(&self, ray: &Ray, rng: &mut Pcg64Mcg) -> f32 {
        // TODO: find a way to remove this accelerator call
        let Some(hit) = self.accelerator.hit(ray, 1e-4, f32::INFINITY, rng) else {
            return 0.0;
        };

        let cos_light = hit.geometric_normal.dot(-ray.direction);

        if cos_light <= 0.0 {
            return 0.0;
        }

        let distance_squared = hit.parameter * hit.parameter;

        distance_squared / (cos_light * self.total_area)
    }
}
