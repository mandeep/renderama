use rand::Rng;

use aabb::{AABB, surrounding_box};
use hitable::{Hitable, HitRecord};
use ray::Ray;
use world::World;

#[derive(Clone)]
pub struct BVH {
    left: Box<dyn Hitable>,
    right: Box<dyn Hitable>,
    bbox: AABB
}

impl BVH {

    pub fn new(left: Box<dyn Hitable>, right: Box<dyn Hitable>, start_time: f32, end_time: f32) -> BVH {
        let bbox = surrounding_box(&left.bounding_box(start_time, end_time).unwrap(),
                                   &right.bounding_box(start_time, end_time).unwrap());
        BVH { left, right, bbox }
   }

    pub fn from(world: &mut World, start_time: f32, end_time: f32) -> BVH {
        let mut rng = rand::thread_rng();
        let axis: usize = rng.gen_range(1, 4);

        if axis == 0 {

        }

    }
}

impl Hitable for BVH {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        if self.bbox.hit(ray, position_min, position_max) {
            let left_record = self.left.hit(ray, position_min, position_max);
            let right_record = self.right.hit(ray, position_min, position_max);

            match (left_record, right_record) {
                (Some(left_record), Some(right_record)) => {
                    if left_record.point < right_record.point {
                        return Some(left_record);
                    } else {
                        return Some(right_record);
                    }
                },

                (Some(left_record), None) => { return Some(left_record); },
                (None, Some(right_record)) => { return Some(right_record); },
                (None, None) => { return None; },
            }
        }
        None
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        Some(self.bbox.clone())
    }

    fn box_clone(&self) -> Box<dyn Hitable> {
        Box::new(self.clone())
    }
}
