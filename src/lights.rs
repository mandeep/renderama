use glam::Vec3A;
use rand::{Rng, RngExt};

use crate::bvh::BVH;
use crate::extensions::DummyRng;
use crate::materials::MaterialId;
use crate::primitive::Primitive;
use crate::plane::{Axis, Bounds2D, Orientation, Plane};
use crate::ray::Ray;
use crate::sampling::uniform_sample_triangle;
use crate::sphere::Sphere;
use crate::triangle::Triangle;

#[derive(Clone)]
/// A light source used to add light emission in a scene
pub enum Light {
    Point(PointLight),
    Area(AreaLight),
    Mesh(MeshLight),
}

impl Light {
    pub fn intensity(&self) -> Vec3A {
        match self {
            Light::Point(light) => light.intensity,
            Light::Area(light) => light.intensity,
            Light::Mesh(light) => light.intensity,
        }
    }

    pub fn evaluate_sampling_weight(&self, ray: &Ray) -> f32 {
        match self {
            Light::Point(light) => light.evaluate_sampling_weight(ray),
            Light::Area(light) => light.evaluate_sampling_weight(ray),
            Light::Mesh(light) => light.evaluate_sampling_weight(ray),
        }
    }

    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut impl Rng) -> Vec3A {
        match self {
            Light::Point(light) => light.sample_direction_to_light(origin, rng),
            Light::Area(light) => light.sample_direction_to_light(origin, rng),
            Light::Mesh(light) => light.sample_direction_to_light(origin, rng),
        }
    }

    pub fn calculate_distance_from(&self, light_distance: f32) -> f32 {
        match &self {
            Light::Point(light) => light_distance - light.sphere.radius.abs() - 1e-3,
            Light::Area(_) | Light::Mesh(_) => light_distance - 1e-3,
        }
    }
}

macro_rules! impl_from_light {
    ($light:ty => $variant:ident) => {
        impl From<$light> for Light {
            fn from(light: $light) -> Self {
                Light::$variant(light)
            }
        }
    };
}

impl_from_light!(PointLight => Point);
impl_from_light!(AreaLight => Area);
impl_from_light!(MeshLight => Mesh);

/// PointLight is a spherical light with light emitting in all directions
///
/// Typically, a PointLight is a single point modeled as a light source with
/// a Delta distribution, however this implementation of a point light uses
/// an underlying sphere with radius to better simulate realistic light sources.
#[derive(Clone)]
pub struct PointLight {
    sphere: Sphere,
    intensity: Vec3A,
}

impl PointLight {
    pub fn new(center: Vec3A, radius: f32, material_id: MaterialId, intensity: Vec3A) -> PointLight {
        let sphere = Sphere::new(center, radius, material_id);
        PointLight { sphere, intensity }
    }

    pub fn from(sphere: Sphere, intensity: Vec3A) -> PointLight {
        PointLight { sphere, intensity }
    }

    pub fn evaluate_sampling_weight(&self, ray: &Ray) -> f32 {
        self.sphere.evaluate_sampling_weight(ray)
    }

    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut impl Rng) -> Vec3A {
        self.sphere.sample_direction_to_light(origin, rng)
    }
}

/// AreaLight is a plane emitting light from a single side of its surface
///
/// Some DCCs allow area lights to emit light from both surfaces, however here
/// Orientation is used so that only a single side emits light.
#[derive(Clone)]
pub struct AreaLight {
    plane: Plane,
    intensity: Vec3A,
}

impl AreaLight {
    pub fn new(axis: Axis, bounds: Bounds2D, offset: f32, orientation: Orientation, material_id: MaterialId, intensity: Vec3A) -> AreaLight {
        let plane = Plane::new(axis, bounds, offset, orientation, material_id);
        AreaLight { plane, intensity }
    }

    pub fn from(plane: Plane, intensity: Vec3A) -> AreaLight {
        AreaLight { plane, intensity }
    }

    pub fn evaluate_sampling_weight(&self, ray: &Ray) -> f32 {
        self.plane.evaluate_sampling_weight(ray)
    }

    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut impl Rng) -> Vec3A {
        self.plane.sample_direction_to_light(origin, rng)

    }
}

/// MeshLight is a light used when loading emissives
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
    intensity: Vec3A,
    accelerator: BVH,
}

impl MeshLight {
    /// Create a new MeshLight from the given triangles
    pub fn new(triangles: Vec<Triangle>, intensity: Vec3A) -> MeshLight {
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

        MeshLight { triangles: light_triangles, cdf, total_area, intensity, accelerator }
    }

    /// Sample a random triangle from the triangles vector
    fn sample_triangle(&self, rng: &mut impl Rng) -> &Triangle {
        let target = rng.random::<f32>() * self.total_area;

        let index = self
            .cdf
            .partition_point(|&x| x < target)
            .min(self.triangles.len() - 1);

        &self.triangles[index]
    }

    /// Sample the direction to this light source from the given origin
    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut impl Rng) -> Vec3A {
        let triangle = self.sample_triangle(rng);
        let barycentric = uniform_sample_triangle(rng);

        let point_on_light = triangle.interpolate_position(barycentric);

        point_on_light - origin
    }

    /// Evaluate the sampling weight of this MeshLight
    pub fn evaluate_sampling_weight(&self, ray: &Ray) -> f32 {
        // because MeshLight only traverses Triangles we don't need the rng that is used
        // for the hit method of Volume types
        let mut dummy_rng = DummyRng;
        // TODO: find a way to remove this accelerator call
        let Some(hit) = self.accelerator.hit(ray, 1e-4, f32::INFINITY, &mut dummy_rng) else {
            return 0.0;
        };

        let cos_light = -ray.direction.dot(hit.geometric_normal);

        if cos_light <= 0.0 {
            return 0.0;
        }

        let distance_squared = hit.parameter * hit.parameter;

        distance_squared / (cos_light * self.total_area)
    }
}
