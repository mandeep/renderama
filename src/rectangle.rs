use glam::Vec3A;

use aabb::AABB;
use results::HitResult;
use primitive::Primitive;
use materials::MaterialId;
use plane::{Axis, Bounds2D, Plane};
use ray::Ray;


#[derive(Clone)]
#[allow(dead_code)]
pub struct Rectangle {
    p0: Vec3A,
    p1: Vec3A,
    primitive: Vec<Primitive>,
    material_id: MaterialId,
}

impl Rectangle {
    pub fn new(p0: Vec3A, p1: Vec3A, material_id: MaterialId) -> Rectangle {
        let mut primitive: Vec<Primitive> = Vec::new();
        let xy_bounds = Bounds2D::new(p0.x..p1.x, p0.y..p1.y);
        let xz_bounds = Bounds2D::new(p0.x..p1.x, p0.z..p1.z);
        let yz_bounds = Bounds2D::new(p0.y..p1.y, p0.z..p1.z);

        primitive.push(Plane::new(Axis::XY, xy_bounds, p1.z, material_id).into_primitive());
        primitive.push(Plane::new(Axis::XY, xy_bounds, p0.z, material_id).into_reversed());
        primitive.push(Plane::new(Axis::XZ, xz_bounds, p1.y, material_id).into_primitive());
        primitive.push(Plane::new(Axis::XZ, xz_bounds, p0.y, material_id).into_reversed());
        primitive.push(Plane::new(Axis::YZ, yz_bounds, p1.x, material_id).into_primitive());
        primitive.push(Plane::new(Axis::YZ, yz_bounds, p0.x, material_id).into_reversed());

        Rectangle { p0, p1, primitive, material_id }
    }

    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitResult> {
        self.primitive
        .iter()
        .filter_map(|g| g.hit(ray, position_min, position_max))
        .filter(|hit| hit.parameter.is_finite())
        .min_by(|a, b| a.parameter.partial_cmp(&b.parameter).unwrap())
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        Some(AABB::from(self.p0, self.p1))
    }
}
