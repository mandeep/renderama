use rand::Rng;
use std::cmp::Ordering;
use std::sync::Arc;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use ray::Ray;

#[derive(Clone)]
pub struct BVH {
    left: Arc<dyn Hitable>,
    right: Arc<dyn Hitable>,
    bbox: AABB,
}

impl BVH {
    pub fn new(mut world: &mut Vec<Arc<dyn Hitable>>, start_time: f32, end_time: f32) -> BVH {
        let mut rng = rand::thread_rng();
        let axis: usize = rng.gen_range(0, 3);

        world.sort_by(|a, b| box_compare(a, b, axis, start_time, end_time));

        let left: Arc<dyn Hitable>;
        let right: Arc<dyn Hitable>;

        if world.len() == 1 {
            left = world[0].clone();
            right = world[0].clone();
        } else if world.len() == 2 {
            left = world[0].clone();
            right = world[1].clone();
        } else {
            let mut right_objects = world.split_off(world.len() / 2);
            left = Arc::new(BVH::new(&mut world, start_time, end_time));
            right = Arc::new(BVH::new(&mut right_objects, start_time, end_time));
        }

        let bbox = left.bounding_box(start_time, end_time)
                       .unwrap()
                       .surrounding_box(&right.bounding_box(start_time, end_time).unwrap());

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
}

fn box_compare(a: &Arc<dyn Hitable>,
               b: &Arc<dyn Hitable>,
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
