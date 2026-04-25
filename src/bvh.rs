use std::cmp::Ordering;
use std::sync::Arc;

use glam::Vec3;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use ray::Ray;

#[derive(Clone)]
pub struct BVH {
    left: Arc<dyn Hitable>,
    right: Arc<dyn Hitable>,
    bbox: AABB,
}

const NUM_BUCKETS: usize = 16;

impl BVH {
    /// Construct a new BVH from the objects in the scene.
    ///
    /// An axis is chosen by random and the objects in the scene
    /// are sorted upon that axis. Then, child objects are created
    /// until only leaf nodes exist.

    pub fn new(world: &mut Vec<Arc<dyn Hitable>>, start_time: f32, end_time: f32) -> BVH {
        let n = world.len();

        let mut main_box = world[0].bounding_box(start_time, end_time).unwrap();
        for i in 1..n {
            let new_box = world[i].bounding_box(start_time, end_time).unwrap();
            main_box = main_box.surrounding_box(&new_box);
        }

        if n == 1 {
            return BVH { left: world[0].clone(), right: world[0].clone(), bbox: main_box };
        }
        if n == 2 {
            return BVH { left: world[0].clone(), right: world[1].clone(), bbox: main_box };
        }

        // For small leaves, just split in half on longest axis (avoids SAH overhead)
        if n <= 4 {
            let axis = main_box.longest_axis();
            world.sort_by(|a, b| box_compare(a, b, axis, start_time, end_time));
            let mut right_objects = world.split_off(n / 2);
            let left = Arc::new(BVH::new(world, start_time, end_time));
            let right = Arc::new(BVH::new(&mut right_objects, start_time, end_time));
            return BVH { left, right, bbox: main_box };
        }

        // Compute centroids once
        let centroids: Vec<Vec3> = world.iter().map(|h| {
            let b = h.bounding_box(start_time, end_time).unwrap();
            (b.minimum + b.maximum) * 0.5
        }).collect();

        // Find centroid bounds (the spread of centroids, not the parent box)
        let mut centroid_min = centroids[0];
        let mut centroid_max = centroids[0];
        for c in &centroids[1..] {
            centroid_min = centroid_min.min(*c);
            centroid_max = centroid_max.max(*c);
        }

        let parent_area = main_box.surface_area();
        let mut best_cost = f32::INFINITY;
        let mut best_axis = 0;
        let mut best_bucket = NUM_BUCKETS / 2;

        for axis in 0..3 {
            let (cmin, cmax) = match axis {
                0 => (centroid_min.x(), centroid_max.x()),
                1 => (centroid_min.y(), centroid_max.y()),
                _ => (centroid_min.z(), centroid_max.z()),
            };
            
            // Skip degenerate axes
            if (cmax - cmin) < 1e-6 { continue; }

            // Bin objects into buckets by centroid position
            let mut bucket_counts = [0usize; NUM_BUCKETS];
            let mut bucket_boxes: Vec<Option<AABB>> = vec![None; NUM_BUCKETS];

            for (i, c) in centroids.iter().enumerate() {
                let pos = match axis {
                    0 => c.x(),
                    1 => c.y(),
                    _ => c.z(),
                };
                let mut b = (((pos - cmin) / (cmax - cmin)) * NUM_BUCKETS as f32) as usize;
                if b >= NUM_BUCKETS { b = NUM_BUCKETS - 1; }
                
                bucket_counts[b] += 1;
                let obj_box = world[i].bounding_box(start_time, end_time).unwrap();
                bucket_boxes[b] = Some(match &bucket_boxes[b] {
                    Some(existing) => existing.surrounding_box(&obj_box),
                    None => obj_box,
                });
            }

            // Try each split between buckets
            for split in 1..NUM_BUCKETS {
                let mut left_count = 0;
                let mut right_count = 0;
                let mut left_box: Option<AABB> = None;
                let mut right_box: Option<AABB> = None;

                for b in 0..split {
                    left_count += bucket_counts[b];
                    if let Some(bb) = &bucket_boxes[b] {
                        left_box = Some(match &left_box {
                            Some(existing) => existing.surrounding_box(bb),
                            None => bb.clone(),
                        });
                    }
                }
                for b in split..NUM_BUCKETS {
                    right_count += bucket_counts[b];
                    if let Some(bb) = &bucket_boxes[b] {
                        right_box = Some(match &right_box {
                            Some(existing) => existing.surrounding_box(bb),
                            None => bb.clone(),
                        });
                    }
                }

                if left_count == 0 || right_count == 0 { continue; }

                let left_area = left_box.unwrap().surface_area();
                let right_area = right_box.unwrap().surface_area();

                let cost = 0.125 + (left_area * left_count as f32 + right_area * right_count as f32) / parent_area;

                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_bucket = split;
                }
            }
        }

        // Partition by bucket on best axis
        let (cmin, cmax) = match best_axis {
            0 => (centroid_min.x(), centroid_max.x()),
            1 => (centroid_min.y(), centroid_max.y()),
            _ => (centroid_min.z(), centroid_max.z()),
        };
        
        let split_threshold = best_bucket as f32;
        
        // partition_point trick: sort then find boundary, OR use partition
        let bucket_for = |i: usize| -> usize {
            let pos = match best_axis {
                0 => centroids[i].x(),
                1 => centroids[i].y(),
                _ => centroids[i].z(),
            };
            let mut b = (((pos - cmin) / (cmax - cmin)) * NUM_BUCKETS as f32) as usize;
            if b >= NUM_BUCKETS { b = NUM_BUCKETS - 1; }
            b
        };
        
        // Build indices: items with bucket < best_bucket go left, others go right
        let mut left_items = Vec::new();
        let mut right_items = Vec::new();
        for i in 0..n {
            if (bucket_for(i) as f32) < split_threshold {
                left_items.push(world[i].clone());
            } else {
                right_items.push(world[i].clone());
            }
        }

        // Safety: if partition somehow ends up empty on one side, fall back to median
        if left_items.is_empty() || right_items.is_empty() {
            world.sort_by(|a, b| box_compare(a, b, best_axis, start_time, end_time));
            let mut right_objects = world.split_off(n / 2);
            let left = Arc::new(BVH::new(world, start_time, end_time));
            let right = Arc::new(BVH::new(&mut right_objects, start_time, end_time));
            return BVH { left, right, bbox: main_box };
        }

        let left = Arc::new(BVH::new(&mut left_items, start_time, end_time));
        let right = Arc::new(BVH::new(&mut right_items, start_time, end_time));
        BVH { left, right, bbox: main_box }
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

/// Compare two bounding volumes along the given axis.
///
/// We compare by centroid (midpoint of the AABB on that axis) rather
/// than by the minimum corner. Centroid-based partitioning produces
/// more balanced BVH trees, especially when primitives vary in size.
fn box_compare(a: &Arc<dyn Hitable>,
               b: &Arc<dyn Hitable>,
               axis: usize,
               start_time: f32,
               end_time: f32)
               -> Ordering {
    let box_a = a.bounding_box(start_time, end_time).unwrap();
    let box_b = b.bounding_box(start_time, end_time).unwrap();

    let centroid_a = (box_a.minimum + box_a.maximum) * 0.5;
    let centroid_b = (box_b.minimum + box_b.maximum) * 0.5;

    let (a_val, b_val) = match axis {
        0 => (centroid_a.x(), centroid_b.x()),
        1 => (centroid_a.y(), centroid_b.y()),
        _ => (centroid_a.z(), centroid_b.z()),
    };

    a_val.partial_cmp(&b_val).unwrap()
}
