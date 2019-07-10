use rand::Rng;

use aabb::{surrounding_box, AABB};
use hitable::{HitRecord, Hitable};
use ray::Ray;
use world::World;

#[derive(Clone)]
pub struct BVH {
    left: Box<dyn Hitable>,
    right: Box<dyn Hitable>,
    bbox: AABB,
}

impl BVH {
    pub fn new(left: Box<dyn Hitable>,
               right: Box<dyn Hitable>,
               start_time: f32,
               end_time: f32)
               -> BVH {
        let bbox = surrounding_box(&left.bounding_box(start_time, end_time).unwrap(),
                                   &right.bounding_box(start_time, end_time).unwrap());
        BVH { left, right, bbox }
    }

    pub fn from(world: &mut World, start_time: f32, end_time: f32) -> BVH {
        let mut rng = rand::thread_rng();
        let axis: usize = rng.gen_range(0, 3);

        if axis == 0 {
            world.objects.sort_by(|a, b| {
                             a.bounding_box(start_time, end_time)
                              .unwrap()
                              .minimum
                              .x
                              .partial_cmp(&b.bounding_box(start_time, end_time).unwrap().minimum.x)
                              .unwrap()
                         });
        } else if axis == 1 {
            world.objects.sort_by(|a, b| {
                             a.bounding_box(start_time, end_time)
                              .unwrap()
                              .minimum
                              .y
                              .partial_cmp(&b.bounding_box(start_time, end_time).unwrap().minimum.y)
                              .unwrap()
                         });
        } else {
            world.objects.sort_by(|a, b| {
                             a.bounding_box(start_time, end_time)
                              .unwrap()
                              .minimum
                              .z
                              .partial_cmp(&b.bounding_box(start_time, end_time).unwrap().minimum.z)
                              .unwrap()
                         });
        }

        let objects = world.objects.clone();
        let mut left = &objects[0];
        let mut right = &objects[0];

        if world.objects.len() == 1 {
            left = &objects[0];
            right = &objects[0];
        } else if world.objects.len() == 2 {
            left = &objects[0];
            right = &objects[1];
        } else {
            let right_objects = world.objects.split_off(world.objects.len() / 2);
            let left = BVH::from(world, start_time, end_time);
            world.objects = right_objects.clone();
            let right = BVH::from(world, start_time, end_time);
        }

        BVH::new(left.clone(), right.clone(), start_time, end_time)
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
                }

                (Some(left_record), None) => {
                    return Some(left_record);
                }
                (None, Some(right_record)) => {
                    return Some(right_record);
                }
                (None, None) => {
                    return None;
                }
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
