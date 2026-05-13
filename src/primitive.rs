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
pub enum Primitive {
    Plane(Plane),
    Rectangle(Rectangle),
    Sphere(Sphere),
    Triangle(Triangle),
    TriangleMesh(Box<TriangleMesh>),
    ReverseOrientation(Box<Primitive>),
    TransformedMesh(Box<TransformedMesh>),
    Volume(Box<Volume>)
}

impl Primitive {
    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32) -> Option<HitEvent> {
        match self {
            Primitive::Plane(p) => p.hit(ray, start_distance, end_distance),
            Primitive::Rectangle(r) => r.hit(ray, start_distance, end_distance),
            Primitive::Sphere(s) => s.hit(ray, start_distance, end_distance),
            Primitive::Triangle(t) => t.hit(ray, start_distance, end_distance),
            Primitive::TriangleMesh(m) => m.hit(ray, start_distance, end_distance),
            Primitive::ReverseOrientation(g) => {
                if let Some(mut h) = g.hit(ray, start_distance, end_distance) {
                    h.geometric_normal = -h.geometric_normal;
                    h.shading_normal = -h.shading_normal;
                    Some(h)
                } else {
                    None
                }
            },
            Primitive::TransformedMesh(m) => m.hit(ray, start_distance, end_distance),
            Primitive::Volume(v) => v.hit(ray, start_distance, end_distance)
        }
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        match self {
            Primitive::Plane(p) => p.bounding_box(),
            Primitive::Rectangle(p) => p.bounding_box(),
            Primitive::Sphere(s) => s.bounding_box(),
            Primitive::Triangle(t) => t.bounding_box(),
            Primitive::TriangleMesh(m) => m.bounding_box(),
            Primitive::ReverseOrientation(g) => g.bounding_box(),
            Primitive::TransformedMesh(g) => g.bounding_box(),
            Primitive::Volume(v) => v.bounding_box(),
        }
    }

    #[allow(dead_code)]
    pub fn reversed(self) -> Self {
        Primitive::ReverseOrientation(Box::new(self))
    }
}

macro_rules! impl_from_for_primitive {
    // Direct variants: From<T> wraps as Variant(t)
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl From<$type> for Primitive {
                fn from(value: $type) -> Self {
                    Primitive::$variant(value)
                }
            }
        )*
    };
}

macro_rules! impl_from_boxed_for_primitive {
    // Boxed variants: From<T> wraps as Variant(Box::new(t))
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl From<$type> for Primitive {
                fn from(value: $type) -> Self {
                    Primitive::$variant(Box::new(value))
                }
            }
        )*
    };
}

impl_from_for_primitive! {
    Plane => Plane,
    Rectangle => Rectangle,
    Sphere => Sphere,
    Triangle => Triangle,
}

impl_from_boxed_for_primitive! {
    TriangleMesh => TriangleMesh,
    TransformedMesh => TransformedMesh,
    Volume => Volume,
}