use glam::Vec3;

use aabb::AABB;
use geometry::Geometry;
use hitable::{HitRecord};
use ray::Ray;

#[derive(Clone)]
/// The World struct holds all of the objects in the scene
pub struct World {
    pub objects: Vec<Geometry>,
}

impl World {
    /// Create a new World to hold all of the objects in the scene
    pub fn new() -> World {
        World { objects: Vec::new() }
    }

    /// Add objects to the instantiated world
    ///
    /// We use a 'static lifetime so that we can Arc
    /// object inside the function rather than having to
    /// pass object as an Arced object as an input parameter.
    pub fn add(&mut self, object: Geometry) {
        self.objects.push(object);
    }

    /// Determine if the given ray has hit any of the objects in the world
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let mut record =
            HitRecord::new(0.0,
                           0.0,
                           0.0,
                           Vec3::ZERO,
                           Vec3::ZERO,
                           Vec3::ZERO,
                           0);
        let mut hit_anything: bool = false;
        let mut closest_so_far: f32 = position_max;

        for object in &self.objects {
            match object.hit(ray, position_min, closest_so_far) {
                None => (),
                Some(hit_record) => {
                    hit_anything = true;
                    closest_so_far = hit_record.parameter;
                    record = hit_record;
                }
            }
        }

        if hit_anything { Some(record) } else { None }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }

        let mut accumulated_box = self.objects.first().unwrap().bounding_box(t0, t1)?;
        for object in self.objects.iter().skip(1) {
            let new_box = object.bounding_box(t0, t1)?;
            accumulated_box = accumulated_box.surrounding_box(&new_box);
        }
        Some(accumulated_box)
    }
}
