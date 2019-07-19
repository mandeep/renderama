use rand::Rng;
use std::cmp::Ordering;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use ray::Ray;

#[derive(Clone)]
pub struct BVH {
    left: Box<dyn Hitable>,
    right: Box<dyn Hitable>,
    bbox: AABB,
}

impl BVH {
    pub fn new(world: &Vec<Box<dyn Hitable>>, start_time: f32, end_time: f32) -> BVH {
        let mut objects = world.clone();
        let mut rng = rand::thread_rng();
        let axis: usize = rng.gen_range(0, 3);

        objects.sort_by(|a, b| box_compare(a, b, axis, start_time, end_time));

        let left: Box<dyn Hitable>;
        let right: Box<dyn Hitable>;

        if objects.len() == 1 {
            left = objects[0].clone();
            right = objects[0].clone();
        } else if objects.len() == 2 {
            left = objects[0].clone();
            right = objects[1].clone();
        } else {
            let mut right_objects = objects.split_off(world.len() / 2);
            left = Box::new(BVH::new(&mut objects, start_time, end_time));
            right = Box::new(BVH::new(&mut right_objects, start_time, end_time));
        }

        let bbox = left.bounding_box(start_time, end_time)
            .unwrap().
            surrounding_box(&right.bounding_box(start_time, end_time).unwrap());

        BVH { left, right, bbox }
    }
}

impl Hitable for BVH {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if self.bbox.hit(&ray, t_min, t_max) {
            let left = self.left.hit(&ray, t_min, t_max);
            let right = self.right.hit(&ray, t_min, t_max);
            match (left, right) {
                (Some(left), Some(right)) => {
                    if left.parameter < right.parameter {
                        Some(left)
                    } else {
                        Some(right)
                    }
                }
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                _ => None,
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(self.bbox.clone())
    }

    fn box_clone(&self) -> Box<dyn Hitable> {
        Box::new(self.clone())
    }
}

fn box_compare(a: &Box<dyn Hitable>,
               b: &Box<dyn Hitable>,
               axis: usize,
               start_time: f32,
               end_time: f32)
               -> Ordering {
    a.bounding_box(start_time, end_time)
      .unwrap()
      .minimum[axis]
      .partial_cmp(&b.bounding_box(start_time, end_time).unwrap().minimum[axis])
      .unwrap()
}
