use std::sync::Arc;

use rand::Rng;

use crate::aabb::AABB;
use crate::plane::Plane;
use crate::ray::Ray;
use crate::rectangle::Rectangle;
use crate::results::HitResult;
use crate::sphere::Sphere;
use crate::triangle::{Triangle, TriangleMesh};
use crate::transformations::{MotionMesh, TransformedMesh};
use crate::volume::Volume;


#[derive(Clone)]
/// The Primitive enum allows us to statically dispatch all shapes currently
/// accepted by the integrators.
pub enum Primitive {
    Plane(Plane),
    Rectangle(Rectangle),
    Sphere(Sphere),
    Triangle(Triangle),
    TriangleMesh(Arc<TriangleMesh>),
    TransformedMesh(Arc<TransformedMesh>),
    MotionMesh(Arc<MotionMesh>),
    Volume(Arc<Volume>)
}

impl Primitive {
    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut impl Rng) -> Option<HitResult> {
        match self {
            Primitive::Plane(plane) => plane.hit(ray, start_distance, end_distance),
            Primitive::Rectangle(rectangle) => rectangle.hit(ray, start_distance, end_distance),
            Primitive::Sphere(sphere) => sphere.hit(ray, start_distance, end_distance),
            Primitive::Triangle(triangle) => triangle.hit(ray, start_distance, end_distance),
            Primitive::TriangleMesh(mesh) => mesh.hit(ray, start_distance, end_distance, rng),
            Primitive::TransformedMesh(mesh) => mesh.hit(ray, start_distance, end_distance, rng),
            Primitive::MotionMesh(mesh) => mesh.hit(ray, start_distance, end_distance, rng),
            Primitive::Volume(volume) => volume.hit(ray, start_distance, end_distance, rng)
        }
    }

    pub fn hits_anything(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut impl Rng) -> bool {
        match self {
            Primitive::Plane(plane) => plane.hit(ray, start_distance, end_distance).is_some(),
            Primitive::Rectangle(rectangle) => rectangle.hit(ray, start_distance, end_distance).is_some(),
            Primitive::Sphere(sphere) => sphere.hit(ray, start_distance, end_distance).is_some(),
            Primitive::Triangle(triangle) => triangle.hits_anything(ray, start_distance, end_distance),
            Primitive::TriangleMesh(mesh) => mesh.hits_anything(ray, start_distance, end_distance, rng),
            Primitive::TransformedMesh(mesh) => mesh.hits_anything(ray, start_distance, end_distance, rng),
            Primitive::MotionMesh(mesh) => mesh.hits_anything(ray, start_distance, end_distance, rng),
            Primitive::Volume(volume) => volume.hit(ray, start_distance, end_distance, rng).is_some()
        }
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        match self {
            Primitive::Plane(plane) => plane.bounding_box(),
            Primitive::Rectangle(rectangle) => rectangle.bounding_box(),
            Primitive::Sphere(sphere) => sphere.bounding_box(),
            Primitive::Triangle(triangle) => triangle.bounding_box(),
            Primitive::TriangleMesh(mesh) => mesh.bounding_box(),
            Primitive::TransformedMesh(mesh) => mesh.bounding_box(),
            Primitive::MotionMesh(mesh) => mesh.bounding_box(),
            Primitive::Volume(volume) => volume.bounding_box(),
        }
    }
}

macro_rules! impl_from_for_primitive {
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
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl From<$type> for Primitive {
                fn from(value: $type) -> Self {
                    Primitive::$variant(Arc::new(value))
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
    MotionMesh => MotionMesh,
    TriangleMesh => TriangleMesh,
    TransformedMesh => TransformedMesh,
    Volume => Volume,
}


#[cfg(test)]
mod tests {
    use crate::materials::MaterialId;
    use super::*;
    use glam::Vec3A;
    use rand::{rng, SeedableRng};
    use rand_pcg::Pcg64Mcg;

    #[test]
    fn test_primitive_from_impl() {
        let mut rng = Pcg64Mcg::from_rng(&mut rng());
        let material = MaterialId::new(0);
        let sphere: Sphere = Sphere::new(Vec3A::ZERO, 1.0, material);
        let primitive: Primitive = sphere.into();
        let ray = Ray::new(Vec3A::new(0.0, 0.0, -1.0), Vec3A::new(0.0, 0.0, 1.0), 0.0);
        let hit_result = primitive.hit(&ray, 0.0, f32::INFINITY, &mut rng);
        assert!(hit_result.is_some());

        let aabb = primitive.bounding_box();
        assert!(aabb.is_some());
        assert_eq!(aabb.unwrap().minimum, Vec3A::new(-1.0, -1.0, -1.0));
        assert_eq!(aabb.unwrap().maximum, Vec3A::ONE);
    }
}