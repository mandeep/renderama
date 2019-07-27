use std::sync::Arc;

use nalgebra::core::Vector3;

use aabb::AABB;
use hitable::{FlipNormals, HitRecord, Hitable};
use materials::Material;
use ray::Ray;
use rectangle::{Plane, Rectangle};
use world::World;

pub struct Prism {
    p0: Vector3<f32>,
    p1: Vector3<f32>,
    material: Arc<dyn Material>,
    hitables: World,
}

impl Prism {
    pub fn new(p0: Vector3<f32>, p1: Vector3<f32>, material: Arc<dyn Material>) -> Prism {
        let mut hitables = World::new();

        hitables.add(Rectangle::from_box(Plane::XY,
                                         p0.x,
                                         p1.x,
                                         p0.y,
                                         p1.y,
                                         p1.z,
                                         material.clone()));

        hitables.add(FlipNormals::of(Rectangle::from_box(Plane::XY,
                                                         p0.x,
                                                         p1.x,
                                                         p0.y,
                                                         p1.y,
                                                         p0.z,
                                                         material.clone())));

        hitables.add(Rectangle::from_box(Plane::XZ,
                                         p0.x,
                                         p1.x,
                                         p0.z,
                                         p1.z,
                                         p1.y,
                                         material.clone()));

        hitables.add(FlipNormals::of(Rectangle::from_box(Plane::XZ,
                                                         p0.x,
                                                         p1.x,
                                                         p0.z,
                                                         p1.z,
                                                         p0.y,
                                                         material.clone())));

        hitables.add(Rectangle::from_box(Plane::YZ,
                                         p0.y,
                                         p1.y,
                                         p0.z,
                                         p1.z,
                                         p1.x,
                                         material.clone()));

        hitables.add(FlipNormals::of(Rectangle::from_box(Plane::YZ,
                                                         p0.y,
                                                         p1.y,
                                                         p0.z,
                                                         p1.z,
                                                         p0.x,
                                                         material.clone())));
        Prism { p0,
                p1,
                material,
                hitables }
    }
}

impl Hitable for Prism {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        self.hitables.hit(&ray, position_min, position_max)
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(AABB::new(self.p0, self.p1))
    }
}
