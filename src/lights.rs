use glam::Vec3A;
use rand::{Rng, RngExt};

use crate::bvh::BVH;
use crate::extensions::DummyRng;
use crate::materials::{Material, MaterialId};
use crate::primitive::Primitive;
use crate::plane::{Axis, Bounds2D, Orientation, Plane};
use crate::ray::Ray;
use crate::sampling::uniform_sample_triangle;
use crate::sphere::Sphere;
use crate::texture::{Texture, TextureId};
use crate::triangle::Triangle;

#[derive(Clone)]
/// A light source used to add light emission in a scene
pub enum Light {
    Point(PointLight),
    Area(AreaLight),
    Mesh(MeshLight),
}

#[derive(Clone, Copy)]
pub struct LightSample {
    pub direction: Vec3A,
    pub radiance: Vec3A,
}

impl Light {
    pub fn evaluate_sampling_weight(&self, ray: &Ray) -> f32 {
        match self {
            Light::Point(light) => light.evaluate_sampling_weight(ray),
            Light::Area(light) => light.evaluate_sampling_weight(ray),
            Light::Mesh(light) => light.evaluate_sampling_weight(ray),
        }
    }

    pub fn sample_direction_and_radiance(&self, origin: Vec3A, materials: &[Material], textures: &[Texture], rng: &mut impl Rng) -> LightSample {
        match self {
            Light::Point(light) => LightSample {
                direction: light.sample_direction_to_light(origin, rng),
                radiance: light.intensity(textures),
            },
            Light::Area(light) => LightSample {
                direction: light.sample_direction_to_light(origin, rng),
                radiance: light.intensity(textures),
            },
            Light::Mesh(light) => light.sample_direction_and_radiance(origin, materials, textures, rng),
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
    intensity: TextureId,
}

impl PointLight {
    pub fn new(center: Vec3A, radius: f32, material_id: MaterialId, intensity: TextureId) -> PointLight {
        let sphere = Sphere::new(center, radius, material_id);
        PointLight { sphere, intensity }
    }

    pub fn from(sphere: Sphere, intensity: TextureId) -> PointLight {
        PointLight { sphere, intensity }
    }

    pub fn intensity(&self, textures: &[Texture]) -> Vec3A {
        let intensity = textures[self.intensity.index()].sample_texture(0.5, 0.5);
        intensity
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
    intensity: TextureId,
}

impl AreaLight {
    pub fn new(axis: Axis, bounds: Bounds2D, offset: f32, orientation: Orientation, material_id: MaterialId, intensity: TextureId) -> AreaLight {
        let plane = Plane::new(axis, bounds, offset, orientation, material_id);
        AreaLight { plane, intensity }
    }

    pub fn from(plane: Plane, intensity: TextureId) -> AreaLight {
        AreaLight { plane, intensity }
    }

    pub fn intensity(&self, textures: &[Texture]) -> Vec3A {
        let intensity = textures[self.intensity.index()].sample_texture(0.5, 0.5);
        intensity
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
    cdf: Vec<f32>,
    total_area: f32,
    material_id: MaterialId,
    accelerator: BVH,
}

impl MeshLight {
    /// Create a new MeshLight from the given triangles
    pub fn new(triangles: Vec<Triangle>, material_id: MaterialId) -> MeshLight {
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

        let geometries: Vec<Primitive> = light_triangles
            .into_iter()
            .map(Primitive::Triangle)
            .collect();

        let accelerator = BVH::new(geometries);

        MeshLight { cdf, total_area, material_id, accelerator }
    }

    /// Sample a random triangle from the accelerator
    fn sample_triangle(&self, rng: &mut impl Rng) -> &Triangle {
        let target = rng.random::<f32>() * self.total_area;

        let index = self
            .cdf
            .partition_point(|&x| x < target)
            .min(self.cdf.len() - 1);

        match self.accelerator.primitive(index) {
            Primitive::Triangle(triangle) => triangle,
            _ => panic!("Found a Primitive other than Triangle inside the accelerator."),
        }
    }

    /// Sample a point on the light and evaluate its material at that point's UV.
    pub fn sample_direction_and_radiance(&self, origin: Vec3A, materials: &[Material], textures: &[Texture], rng: &mut impl Rng) -> LightSample {
        let triangle = self.sample_triangle(rng);
        let barycentric = uniform_sample_triangle(rng);

        let point_on_light = triangle.interpolate_position(barycentric);
        let uv = triangle.interpolate_uv(barycentric);
        let radiance = materials[self.material_id.index()].evaluate_emission_at_uv(uv.x, uv.y, textures);

        LightSample { direction: point_on_light - origin, radiance}
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
