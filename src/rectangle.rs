use glam::Vec3;

use aabb::AABB;
use events::HitEvent;
use geometry::Geometry;
use plane::{Axis, Bounds2D, Plane};
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
        let xy_bounds = Bounds2D::new(p0.x..p1.x, p0.y..p1.y);
        let xz_bounds = Bounds2D::new(p0.x..p1.x, p0.z..p1.z);
        let yz_bounds = Bounds2D::new(p0.y..p1.y, p0.z..p1.z);

        geometry.push(Plane::new(Axis::XY, xy_bounds, p1.z, material_id).into_geometry());
        geometry.push(Plane::new(Axis::XY, xy_bounds, p0.z, material_id).into_reversed());
        geometry.push(Plane::new(Axis::XZ, xz_bounds, p1.y, material_id).into_geometry());
        geometry.push(Plane::new(Axis::XZ, xz_bounds, p0.y, material_id).into_reversed());
        geometry.push(Plane::new(Axis::YZ, yz_bounds, p1.x, material_id).into_geometry());
        geometry.push(Plane::new(Axis::YZ, yz_bounds, p0.x, material_id).into_reversed());

        Rectangle { p0, p1, geometry, material_id }
    }

    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitEvent> {
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
