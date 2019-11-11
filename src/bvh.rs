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
    /// Construct a new BVH from the objects in the scene.
    ///
    /// An axis is chosen by random and the objects in the scene
    /// are sorted upon that axis. Then, child objects are created
    /// until only leaf nodes exist.
    pub fn new(mut world: &mut Vec<Arc<dyn Hitable>>, start_time: f32, end_time: f32) -> BVH {
        let mut main_box = world[0].bounding_box(start_time, end_time).unwrap();

        for i in 1..world.len() {
            let new_box = world[i].bounding_box(start_time, end_time).unwrap();
            main_box = main_box.surrounding_box(&new_box);
        }

        let axis = main_box.longest_axis();

        world.sort_by(|a, b| box_compare(a, b, axis, start_time, end_time));

        let mut left = world[0].clone();
        let mut right = world[0].clone();

        if world.len() == 2 {
            left = world[0].clone();
            right = world[1].clone();
        } else if world.len() > 2 {
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
    /// Test whether the ray intersects the bounding volume.
    ///
    /// We check for an intersection with a node in the BVH and
    /// return the node that is hit. If both the left and right
    /// child are hit, then we return the node closest to the ray.
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

/// Compare the coordinates of two bounding volumes.
///
/// We compare two bounding volumes based on their minimum
/// slab and return the order with the volume with the
/// least minimum first.
fn box_compare(a: &Arc<dyn Hitable>,
               b: &Arc<dyn Hitable>,
               axis: usize,
               start_time: f32,
               end_time: f32)
               -> Ordering {
    if axis == 0 {
        a.bounding_box(start_time, end_time)
         .unwrap()
         .minimum
         .x()
         .partial_cmp(&b.bounding_box(start_time, end_time).unwrap().minimum.x())
         .unwrap()
    } else if axis == 1 {
        a.bounding_box(start_time, end_time)
         .unwrap()
         .minimum
         .y()
         .partial_cmp(&b.bounding_box(start_time, end_time).unwrap().minimum.y())
         .unwrap()
    } else {
        a.bounding_box(start_time, end_time)
         .unwrap()
         .minimum
         .z()
         .partial_cmp(&b.bounding_box(start_time, end_time).unwrap().minimum.z())
         .unwrap()
    }
}
