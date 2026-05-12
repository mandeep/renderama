use rand::rngs::ThreadRng;

use glam::Vec3A;

use aabb::AABB;
use events::HitEvent;
use plane::Plane;
use ray::Ray;
use rectangle::Rectangle;
use sphere::Sphere;
use triangle::{Triangle, TriangleMesh};
use transformations::TransformedMesh;
use volume::Volume;


#[derive(Clone)]
pub enum Geometry {
    Plane(Plane),
    Rectangle(Rectangle),
    Sphere(Sphere),
    Triangle(Triangle),
    TriangleMesh(Box<TriangleMesh>),
    ReverseOrientation(Box<Geometry>),
    TransformedMesh(Box<TransformedMesh>),
    Volume(Box<Volume>)
}

impl Geometry {
    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32) -> Option<HitEvent> {
        match self {
            Geometry::Plane(p) => p.hit(ray, start_distance, end_distance),
            Geometry::Rectangle(r) => r.hit(ray, start_distance, end_distance),
            Geometry::Sphere(s) => s.hit(ray, start_distance, end_distance),
            Geometry::Triangle(t) => t.hit(ray, start_distance, end_distance),
            Geometry::TriangleMesh(m) => m.hit(ray, start_distance, end_distance),
            Geometry::ReverseOrientation(g) => {
                if let Some(mut h) = g.hit(ray, start_distance, end_distance) {
                    h.geometric_normal = -h.geometric_normal;
                    h.shading_normal = -h.shading_normal;
                    Some(h)
                } else {
                    None
                }
            },
            Geometry::TransformedMesh(m) => m.hit(ray, start_distance, end_distance),
            Geometry::Volume(v) => v.hit(ray, start_distance, end_distance)
        }
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        match self {
            Geometry::Plane(p) => p.bounding_box(),
            Geometry::Rectangle(p) => p.bounding_box(),
            Geometry::Sphere(s) => s.bounding_box(),
            Geometry::Triangle(t) => t.bounding_box(),
            Geometry::TriangleMesh(m) => m.bounding_box(),
            Geometry::ReverseOrientation(g) => g.bounding_box(),
            Geometry::TransformedMesh(g) => g.bounding_box(),
            Geometry::Volume(v) => v.bounding_box(),
        }
    }

    pub fn evaluate_sampling_weight(&self, origin: Vec3A, direction: Vec3A) -> f32 {
        match self {
            Geometry::Plane(p) => p.evaluate_sampling_weight(origin, direction),
            Geometry::Sphere(s) => s.evaluate_sampling_weight(origin, direction),
            _ => 0.0
        }
    }

    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut ThreadRng) -> Vec3A {
        match self {
            Geometry::Plane(p) => p.sample_direction_to_light(origin, rng),
            Geometry::Sphere(s) => s.sample_direction_to_light(origin, rng),
            _ => Vec3A::new(1.0, 0.0, 0.0)
        }
    }

    pub fn reversed(self) -> Self {
        Geometry::ReverseOrientation(Box::new(self))
    }
}

macro_rules! impl_from_for_geometry {
    // Direct variants: From<T> wraps as Variant(t)
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl From<$type> for Geometry {
                fn from(value: $type) -> Self {
                    Geometry::$variant(value)
                }
            }
        )*
    };
}

macro_rules! impl_from_boxed_for_geometry {
    // Boxed variants: From<T> wraps as Variant(Box::new(t))
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl From<$type> for Geometry {
                fn from(value: $type) -> Self {
                    Geometry::$variant(Box::new(value))
                }
            }
        )*
    };
}

impl_from_for_geometry! {
    Plane => Plane,
    Rectangle => Rectangle,
    Sphere => Sphere,
    Triangle => Triangle,
}

impl_from_boxed_for_geometry! {
    TriangleMesh => TriangleMesh,
    TransformedMesh => TransformedMesh,
    Volume => Volume,
}