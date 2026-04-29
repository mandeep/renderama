use rand::rngs::ThreadRng;

use glam::Vec3;

use aabb::AABB;
use hitable::HitRecord;
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
    pub fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        match self {
            Geometry::Plane(p) => p.hit(ray, tmin, tmax),
            Geometry::Rectangle(r) => r.hit(ray, tmin, tmax),
            Geometry::Sphere(s) => s.hit(ray, tmin, tmax),
            Geometry::Triangle(t) => t.hit(ray, tmin, tmax),
            Geometry::TriangleMesh(m) => m.hit(ray, tmin, tmax),
            Geometry::ReverseOrientation(g) => {
                if let Some(mut h) = g.hit(ray, tmin, tmax) {
                    h.geometric_normal = -h.geometric_normal;
                    h.shading_normal = -h.shading_normal;
                    Some(h)
                } else {
                    None
                }
            },
            Geometry::TransformedMesh(m) => m.hit(ray, tmin, tmax),
            Geometry::Volume(v) => v.hit(ray, tmin, tmax)
        }
    }

    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        match self {
            Geometry::Plane(p) => p.bounding_box(t0, t1),
            Geometry::Rectangle(p) => p.bounding_box(t0, t1),
            Geometry::Sphere(s) => s.bounding_box(t0, t1),
            Geometry::Triangle(t) => t.bounding_box(t0, t1),
            Geometry::TriangleMesh(m) => m.bounding_box(t0, t1),
            Geometry::ReverseOrientation(g) => g.bounding_box(t0, t1),
            Geometry::TransformedMesh(g) => g.bounding_box(t0, t1),
            Geometry::Volume(v) => v.bounding_box(t0, t1),
        }
    }

    pub fn pdf_value(&self, origin: Vec3, direction: Vec3) -> f32 {
        match self {
            Geometry::Plane(p) => p.pdf_value(origin, direction),
            _ => 0.0
        }
    }

    pub fn pdf_random(&self, origin: Vec3, rng: &mut ThreadRng) -> Vec3 {
        match self {
            Geometry::Plane(p) => p.pdf_random(origin, rng),
            _ => Vec3::new(1.0, 0.0, 0.0)
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