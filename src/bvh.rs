use glam::Vec3A;
use wide::f32x4;

use aabb::AABB;
use events::HitEvent;
use geometry::Geometry;
use ray::Ray;


const NUM_BUCKETS: usize = 16;
const LEAF_FLAG: u32 = 1 << 31;

fn is_leaf(idx: u32) -> bool { idx & LEAF_FLAG != 0 }
fn leaf_index(idx: u32) -> usize { (idx & !LEAF_FLAG) as usize }
fn make_leaf_ref(idx: usize) -> u32 { (idx as u32) | LEAF_FLAG }

/// SOA-layout BVH4 internal node. Stores four child AABBs with each
/// component in its own array so a single f32x4 slab test covers all
/// four children at once.
#[derive(Clone, Default)]
struct InternalNode4 {
    min_x: [f32; 4],
    min_y: [f32; 4],
    min_z: [f32; 4],
    max_x: [f32; 4],
    max_y: [f32; 4],
    max_z: [f32; 4],
    children: [u32; 4],
    count: u32,
}

#[derive(Clone)]
struct LeafNode {
    bbox: AABB,
    geometry: Geometry,  // unboxed is fine here, leaves are rarely touched
}

/// Flattened BVH stored as a Vec, traversed iteratively.
///
/// Children are referenced by index into `nodes` rather than by
/// pointer, eliminating cache misses from chasing Arc pointers.
/// The root is always at index 0.
#[derive(Clone)]
pub struct BVH {
    internals: Vec<InternalNode4>,
    leaves: Vec<LeafNode>,
    root: u32,
    bbox: AABB, // root's bbox
}

/// Temporary tree structure used during construction.
/// Gets collapsed into InternalNode4 / LeafNode at the end.
enum TreeNode {
    Internal {
        bbox: AABB,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
    Leaf {
        bbox: AABB,
        geometry: Geometry,
    },
}

impl TreeNode {
    fn bbox(&self) -> &AABB {
        match self {
            TreeNode::Internal { bbox, .. } => bbox,
            TreeNode::Leaf { bbox, .. } => bbox,
        }
    }
}

/// Recursively builds a TreeNode using binned SAH.
fn build_tree(world: &mut Vec<Geometry>, start_time: f32, end_time: f32) -> TreeNode {
    let n = world.len();

    // compute the bounding box that contains all of the objects in the world and their bounding boxes
    let mut main_box = world[0].bounding_box().unwrap();
    for i in 1..n {
        let new_box = world[i].bounding_box().unwrap();
        main_box = main_box.surrounding_box(&new_box);
    }

    if n == 1 {
        return TreeNode::Leaf {
            bbox: main_box,
            geometry: world[0].clone(),
        };
    }

    if n == 2 {
        let left_box = world[0].bounding_box().unwrap();
        let right_box = world[1].bounding_box().unwrap();
        return TreeNode::Internal {
            bbox: main_box,
            left: Box::new(TreeNode::Leaf { bbox: left_box, geometry: world[0].clone() }),
            right: Box::new(TreeNode::Leaf { bbox: right_box, geometry: world[1].clone() }),
        };
    }

    // For very small node counts, fall back to median split (avoids SAH overhead).
    // 4 objects here is just a randomly picked low number
    if n <= 4 {
        let axis = main_box.longest_axis();
        world.sort_by(|a, b| {
            let centroid_a = centroid(a, axis);
            let centroid_b = centroid(b, axis);
            centroid_a.partial_cmp(&centroid_b).unwrap()
        });

        // split off the world in two halves. `right_objects` takes half of the
        // objects while `world` keeps the other half
        let mut right_objects = world.split_off(n / 2);
        let left = build_tree(world, start_time, end_time);
        let right = build_tree(&mut right_objects, start_time, end_time);

        return TreeNode::Internal {
            bbox: main_box,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    // SAH binned split begins here
    // Compute the centroid of each bounding box
    let centroids: Vec<Vec3A> = world.iter().map(|hit| {
        let bbox = hit.bounding_box().unwrap();
        (bbox.minimum + bbox.maximum) * 0.5
    }).collect();

    // Find the bounds of each centroid so we can later bin them into buckets
    // We bin objects into buckets along each axis based on their centroid
    let mut centroid_min = centroids[0];
    let mut centroid_max = centroids[0];
    for centroid in &centroids[1..] {
        centroid_min = centroid_min.min(*centroid);
        centroid_max = centroid_max.max(*centroid);
    }

    // Beginning of optimization loop for binning. `parent_area` is the
    // surface area bounding box for SAH cost computation
    let parent_area = main_box.surface_area();
    let mut best_cost = f32::INFINITY;
    let mut best_axis = 0;
    let mut best_bucket = NUM_BUCKETS / 2;

    // Find the centroid range along each axis. This will allow us to
    // find which bucket each object goes into
    for axis in 0..3 {
        let (cmin, cmax) = (axis_value(centroid_min, axis), axis_value(centroid_max, axis));
        if (cmax - cmin) < 1e-6 { continue; }

        // For each bucket in NUM_BUCKETS we track how many objects fall into that bin
        // and the bounding box that covers all of the objects in that bin
        let mut bucket_counts = [0usize; NUM_BUCKETS];
        let mut bucket_boxes: Vec<Option<AABB>> = vec![None; NUM_BUCKETS];

        // Compute the bucket each centroid falls into
        for (i, centroid) in centroids.iter().enumerate() {
            let position = axis_value(*centroid, axis);
            let mut bucket = (((position - cmin) / (cmax - cmin)) * NUM_BUCKETS as f32) as usize;
            if bucket >= NUM_BUCKETS { bucket = NUM_BUCKETS - 1; }

            // Update the bucket count for the bin and expand the bucket's bounding box
            // to include this object
            bucket_counts[bucket] += 1;
            let obj_box = world[i].bounding_box().unwrap();
            bucket_boxes[bucket] = Some(match &bucket_boxes[bucket] {
                Some(existing) => existing.surrounding_box(&obj_box),
                None => obj_box,
            });
        }

        // Evaluate the split between the buckets. We try every possible split in this O(n^2) loop
        // and compute the cost of each split. Whichever has the lowest cost is chosen as our split.
        for split in 1..NUM_BUCKETS {
            let mut left_count = 0;
            let mut right_count = 0;
            let mut left_box: Option<AABB> = None;
            let mut right_box: Option<AABB> = None;

            for bucket in 0..split {
                left_count += bucket_counts[bucket];
                if let Some(bbox) = &bucket_boxes[bucket] {
                    left_box = Some(match &left_box {
                        Some(existing) => existing.surrounding_box(bbox),
                        None => *bbox,
                    });
                }
            }

            // Compute the number of right side counts along with the number of bounding boxes on that side
            for bucket in split..NUM_BUCKETS {
                right_count += bucket_counts[bucket];
                if let Some(bbox) = &bucket_boxes[bucket] {
                    right_box = Some(match &right_box {
                        Some(existing) => existing.surrounding_box(bbox),
                        None => *bbox,
                    });
                }
            }

            if left_count == 0 || right_count == 0 { continue; }

            let left_area = left_box.unwrap().surface_area();
            let right_area = right_box.unwrap().surface_area();

            // This is the surface area heuristic (SAH). Compute the probability that a random ray hits each child
            let cost = 0.125 + (left_area * left_count as f32 + right_area * right_count as f32) / parent_area;

            if cost < best_cost {
                best_cost = cost;
                best_axis = axis;
                best_bucket = split;
            }
        }
    }

    let (cmin, cmax) = (axis_value(centroid_min, best_axis), axis_value(centroid_max, best_axis));
    let split_threshold = best_bucket as f32;

    let bucket_for = |position: f32| -> usize {
        let mut bucket = (((position - cmin) / (cmax - cmin)) * NUM_BUCKETS as f32) as usize;
        if bucket >= NUM_BUCKETS { bucket = NUM_BUCKETS - 1; }
        bucket
    };

    let mut left_items = Vec::new();
    let mut right_items = Vec::new();
    for i in 0..n {
        let pos = axis_value(centroids[i], best_axis);
        if (bucket_for(pos) as f32) < split_threshold {
            left_items.push(world[i].clone());
        } else {
            right_items.push(world[i].clone());
        }
    }

    if left_items.is_empty() || right_items.is_empty() {
        world.sort_by(|a, b| {
            let centroid_a = centroid(a, best_axis);
            let centroid_b = centroid(b, best_axis);
            centroid_a.partial_cmp(&centroid_b).unwrap()
        });
        let mut right_objects = world.split_off(n / 2);
        let left = build_tree(world, start_time, end_time);
        let right = build_tree(&mut right_objects, start_time, end_time);
        return TreeNode::Internal {
            bbox: main_box,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    let left = build_tree(&mut left_items, start_time, end_time);
    let right = build_tree(&mut right_items, start_time, end_time);

    TreeNode::Internal {
        bbox: main_box,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Collapse a binary tree node into up to 4 children.
///
/// Starts with the two direct children and repeatedly expands the
/// internal child with the largest surface area until we have 4
/// children or all remaining children are leaves.
fn collect_children(tree: &TreeNode) -> Vec<&TreeNode> {
    let (left, right) = match tree {
        TreeNode::Internal { left, right, .. } => (left.as_ref(), right.as_ref()),
        TreeNode::Leaf { .. } => return vec![tree],
    };
    let mut children: Vec<&TreeNode> = vec![left, right];
    while children.len() < 4 {
        let pos = children.iter().enumerate()
            .filter(|(_, c)| matches!(c, TreeNode::Internal { .. }))
            .max_by(|(_, a), (_, b)| {
                a.bbox().surface_area().partial_cmp(&b.bbox().surface_area()).unwrap()
            })
            .map(|(i, _)| i);
        match pos {
            None => break,
            Some(i) => {
                let child = children.remove(i);
                if let TreeNode::Internal { left, right, .. } = child {
                    children.insert(i, left.as_ref());
                    children.insert(i + 1, right.as_ref());
                }
            }
        }
    }
    children
}

/// Flatten the binary tree into InternalNode4 / LeafNode vecs. Returns the index of the root.
fn flatten4(tree: &TreeNode, internals: &mut Vec<InternalNode4>, leaves: &mut Vec<LeafNode>) -> u32 {
    match tree {
        TreeNode::Leaf { bbox, geometry } => {
            let index = leaves.len();
            leaves.push(LeafNode { bbox: *bbox, geometry: geometry.clone() });
            make_leaf_ref(index)
        }
        TreeNode::Internal { .. } => {
            let index = internals.len();
            internals.push(InternalNode4::default()); // reserve slot before recursing

            let children = collect_children(tree);
            let count = children.len() as u32;

            let mut node = InternalNode4::default();
            node.count = count;

            for (i, child) in children.iter().enumerate() {
                let bbox = child.bbox();
                node.min_x[i] = bbox.minimum.x;
                node.min_y[i] = bbox.minimum.y;
                node.min_z[i] = bbox.minimum.z;
                node.max_x[i] = bbox.maximum.x;
                node.max_y[i] = bbox.maximum.y;
                node.max_z[i] = bbox.maximum.z;
                node.children[i] = flatten4(child, internals, leaves);
            }

            internals[index] = node;
            index as u32
        }
    }
}


/// Index into a vector with the given axis
fn axis_value(vector: Vec3A, axis: usize) -> f32 {
    match axis {
        0 => vector.x,
        1 => vector.y,
        _ => vector.z,
    }
}

/// Compute the centroid of an object's bounding box along the given axis
/// Used for sorting in small lists
fn centroid(hit: &Geometry, axis: usize) -> f32 {
    let bbox = hit.bounding_box().unwrap();
    axis_value((bbox.minimum + bbox.maximum) * 0.5, axis)
}

impl BVH {
    pub fn new(world: &mut Vec<Geometry>, start_time: f32, end_time: f32) -> BVH {
        // Build the tree using SAH, then flatten it.
        let tree = build_tree(world, start_time, end_time);
        let bbox = *tree.bbox();

        let mut internals = Vec::new();
        let mut leaves = Vec::new();
        let root = flatten4(&tree, &mut internals, &mut leaves);

        BVH { internals, leaves, root, bbox }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitEvent> {
        // we iterate the traversal with a depth of 64 which should be okay for
        // millions of objects
        let mut stack: [u32; 64] = [0; 64];
        let mut stack_ptr: usize = 0;

        let mut closest_t = t_max;
        let mut best_hit: Option<HitEvent> = None;

        stack[stack_ptr] = self.root;
        stack_ptr += 1;

        // Splat ray data once — reused for every internal node test.
        let ox = f32x4::splat(ray.origin.x);
        let oy = f32x4::splat(ray.origin.y);
        let oz = f32x4::splat(ray.origin.z);
        let idx = f32x4::splat(ray.inverse_direction.x);
        let idy = f32x4::splat(ray.inverse_direction.y);
        let idz = f32x4::splat(ray.inverse_direction.z);
        let tmin_floor = f32x4::splat(t_min);

        while stack_ptr > 0 {
            stack_ptr -= 1;
            let node_ref = stack[stack_ptr];

            if is_leaf(node_ref) {
                let leaf = &self.leaves[leaf_index(node_ref)];
                if leaf.bbox.hit(ray, t_min, closest_t) {
                    if let Some(hit) = leaf.geometry.hit(ray, t_min, closest_t) {
                        if hit.parameter < closest_t {
                            closest_t = hit.parameter;
                            best_hit = Some(hit);
                        }
                    }
                }
            } else {
                let node = &self.internals[node_ref as usize];

                // Slab test for all 4 children simultaneously using SIMD.
                let t0x = (f32x4::from(node.min_x) - ox) * idx;
                let t1x = (f32x4::from(node.max_x) - ox) * idx;
                let t0y = (f32x4::from(node.min_y) - oy) * idy;
                let t1y = (f32x4::from(node.max_y) - oy) * idy;
                let t0z = (f32x4::from(node.min_z) - oz) * idz;
                let t1z = (f32x4::from(node.max_z) - oz) * idz;

                let tmin4 = t0x.min(t1x).max(t0y.min(t1y)).max(t0z.min(t1z)).max(tmin_floor);
                // Clamp tmax by closest_t so we skip nodes entirely behind a known hit.
                let tmax4 = t0x.max(t1x).min(t0y.max(t1y)).min(t0z.max(t1z))
                    .min(f32x4::splat(closest_t));

                let tmin_arr: [f32; 4] = tmin4.into();
                let tmax_arr: [f32; 4] = tmax4.into();

                // Collect hit children and sort so the nearest is visited first.
                let mut hits: [(f32, u32); 4] = [(f32::MAX, 0); 4];
                let mut hit_count = 0usize;

                for i in 0..node.count as usize {
                    if tmin_arr[i] <= tmax_arr[i] {
                        hits[hit_count] = (tmin_arr[i], node.children[i]);
                        hit_count += 1;
                    }
                }

                // Insertion sort descending by tmin — farthest pushed first so
                // nearest is on top of the stack.
                for i in 1..hit_count {
                    let key = hits[i];
                    let mut j = i;
                    while j > 0 && hits[j - 1].0 < key.0 {
                        hits[j] = hits[j - 1];
                        j -= 1;
                    }
                    hits[j] = key;
                }

                for i in 0..hit_count {
                    stack[stack_ptr] = hits[i].1;
                    stack_ptr += 1;
                }
            }
        }

        best_hit
    }

    pub fn any_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        let mut stack: [u32; 64] = [0; 64];
        let mut stack_ptr: usize = 0;

        stack[stack_ptr] = self.root;
        stack_ptr += 1;

        let ox = f32x4::splat(ray.origin.x);
        let oy = f32x4::splat(ray.origin.y);
        let oz = f32x4::splat(ray.origin.z);
        let idx = f32x4::splat(ray.inverse_direction.x);
        let idy = f32x4::splat(ray.inverse_direction.y);
        let idz = f32x4::splat(ray.inverse_direction.z);
        let tmin_floor = f32x4::splat(t_min);
        let tmax_ceil  = f32x4::splat(t_max);

        while stack_ptr > 0 {
            stack_ptr -= 1;
            let node_ref = stack[stack_ptr];

            if is_leaf(node_ref) {
                let leaf = &self.leaves[leaf_index(node_ref)];
                if leaf.bbox.hit(ray, t_min, t_max) {
                    if leaf.geometry.hit(ray, t_min, t_max).is_some() {
                        return true;
                    }
                }
            } else {
                let node = &self.internals[node_ref as usize];

                let t0x = (f32x4::from(node.min_x) - ox) * idx;
                let t1x = (f32x4::from(node.max_x) - ox) * idx;
                let t0y = (f32x4::from(node.min_y) - oy) * idy;
                let t1y = (f32x4::from(node.max_y) - oy) * idy;
                let t0z = (f32x4::from(node.min_z) - oz) * idz;
                let t1z = (f32x4::from(node.max_z) - oz) * idz;

                let tmin4 = t0x.min(t1x).max(t0y.min(t1y)).max(t0z.min(t1z)).max(tmin_floor);
                let tmax4 = t0x.max(t1x).min(t0y.max(t1y)).min(t0z.max(t1z)).min(tmax_ceil);

                let tmin_arr: [f32; 4] = tmin4.into();
                let tmax_arr: [f32; 4] = tmax4.into();

                for i in 0..node.count as usize {
                    if tmin_arr[i] <= tmax_arr[i] {
                        stack[stack_ptr] = node.children[i];
                        stack_ptr += 1;
                    }
                }
            }
        }

        false
    }

    pub fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(self.bbox)
    }
}
