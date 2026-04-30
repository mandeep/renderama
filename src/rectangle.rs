use glam::Vec3;

use aabb::AABB;
use geometry::Geometry;
use hitable::HitRecord;
use plane::{Axis, Plane};
use ray::Ray;

#[derive(Clone)]
pub struct Rectangle {
    p0: Vec3,
    p1: Vec3,
    geometry: Vec<Geometry>,
    material_id: u32,
}

impl Rectangle {
    pub fn new(p0: Vec3, p1: Vec3, material_id: u32) -> Rectangle {
        let mut geometry: Vec<Geometry> = Vec::new();

        geometry.push(
            Geometry::Plane(
                Plane::from_box(Axis::XY,
                                     p0.x,
                                     p1.x,
                                     p0.y,
                                     p1.y,
                                     p1.z,
                                     material_id)));

        geometry.push(
            Geometry::ReverseOrientation(Box::new(
                Geometry::Plane(Plane::from_box(Axis::XY,
                                                     p0.x,
                                                     p1.x,
                                                     p0.y,
                                                     p1.y,
                                                     p0.z,
                                                     material_id)))));

        geometry.push(Geometry::Plane(
            Plane::from_box(Axis::XZ,
                                     p0.x,
                                     p1.x,
                                     p0.z,
                                     p1.z,
                                     p1.y,
                                     material_id)));

        geometry.push(Geometry::ReverseOrientation(Box::new(
                Geometry::Plane(Plane::from_box(Axis::XZ,
                                                     p0.x,
                                                     p1.x,
                                                     p0.z,
                                                     p1.z,
                                                     p0.y,
                                                     material_id)))));

        geometry.push(Geometry::Plane(
            Plane::from_box(Axis::YZ,
                                     p0.y,
                                     p1.y,
                                     p0.z,
                                     p1.z,
                                     p1.x,
                                     material_id)));

        geometry.push(Geometry::ReverseOrientation(Box::new(
            Geometry::Plane(Plane::from_box(Axis::YZ,
                                                     p0.y,
                                                     p1.y,
                                                     p0.z,
                                                     p1.z,
                                                     p0.x,
                                                     material_id)))));
        Rectangle { p0,
                    p1,
                    geometry,
                    material_id
                  }
    }

    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        self.geometry
        .iter()
        .filter_map(|g| g.hit(ray, position_min, position_max))
        .filter(|hit| hit.parameter.is_finite())
        .min_by(|a, b| a.parameter.partial_cmp(&b.parameter).unwrap())
    }

    pub fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(AABB::from(self.p0, self.p1))
    }
}
