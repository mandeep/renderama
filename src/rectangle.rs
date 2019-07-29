use std::sync::Arc;

use nalgebra::core::Vector3;

use aabb::AABB;
use hitable::{FlipNormals, HitRecord, Hitable};
use materials::Material;
use plane::{Axis, Plane};
use ray::Ray;
use world::World;

pub struct Rectangle {
    p0: Vector3<f32>,
    p1: Vector3<f32>,
    material: Arc<dyn Material>,
    hitables: World,
}

impl Rectangle {
    pub fn new(p0: Vector3<f32>, p1: Vector3<f32>, material: Arc<dyn Material>) -> Rectangle {
        let mut hitables = World::new();

        hitables.add(Plane::from_box(Axis::XY, p0.x, p1.x, p0.y, p1.y, p1.z, material.clone()));

        hitables.add(FlipNormals::of(Plane::from_box(Axis::XY,
                                                     p0.x,
                                                     p1.x,
                                                     p0.y,
                                                     p1.y,
                                                     p0.z,
                                                     material.clone())));

        hitables.add(Plane::from_box(Axis::XZ, p0.x, p1.x, p0.z, p1.z, p1.y, material.clone()));

        hitables.add(FlipNormals::of(Plane::from_box(Axis::XZ,
                                                     p0.x,
                                                     p1.x,
                                                     p0.z,
                                                     p1.z,
                                                     p0.y,
                                                     material.clone())));

        hitables.add(Plane::from_box(Axis::YZ, p0.y, p1.y, p0.z, p1.z, p1.x, material.clone()));

        hitables.add(FlipNormals::of(Plane::from_box(Axis::YZ,
                                                     p0.y,
                                                     p1.y,
                                                     p0.z,
                                                     p1.z,
                                                     p0.x,
                                                     material.clone())));
        Rectangle { p0,
                    p1,
                    material,
                    hitables }
    }
}

impl Hitable for Rectangle {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        self.hitables.hit(&ray, position_min, position_max)
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(AABB::new(self.p0, self.p1))
    }
}
