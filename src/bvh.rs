use aabb::AABB;
use hitable::HitRecord;
use geometry::Geometry;
use ray::Ray;
use glam::Vec3;

const NUM_BUCKETS: usize = 16;
const LEAF_FLAG: u32 = 1 << 31;

fn is_leaf(idx: u32) -> bool { idx & LEAF_FLAG != 0 }
fn leaf_index(idx: u32) -> usize { (idx & !LEAF_FLAG) as usize }
fn make_leaf_ref(idx: usize) -> u32 { (idx as u32) | LEAF_FLAG }

/// A node in the flattened BVH array.
///
/// Internal nodes store the indices of their two children.
/// Leaf nodes store an Arc to the actual hitable object.
/// If a ray misses a child, then the entire subtree can be skipped.
/// Once we hit a leaf, then we perform ray-object intersection.
#[derive(Clone)]
enum BVHNode {
    Internal {
        bbox: AABB,
        left: u32,
        right: u32,
    },
    Leaf {
        bbox: AABB,
        geometry: Box<Geometry>,
    },
}

#[derive(Clone, Copy)]
struct InternalNode {
    bbox: AABB,        // 24 bytes
    left: u32,
    right: u32,
}  // 32 bytes — exactly half a cache line, two per line

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
    internals: Vec<InternalNode>,
    leaves: Vec<LeafNode>,
    root: u32,
    bbox: AABB, // root's bbox
}

/// Temporary tree structure used during construction.
/// Gets flattened into the Vec<BVHNode> at the end.
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
    let mut main_box = world[0].bounding_box(start_time, end_time).unwrap();
    for i in 1..n {
        let new_box = world[i].bounding_box(start_time, end_time).unwrap();
        main_box = main_box.surrounding_box(&new_box);
    }

    if n == 1 {
        return TreeNode::Leaf {
            bbox: main_box,
            geometry: world[0].clone(),
        };
    }

    if n == 2 {
        let left_box = world[0].bounding_box(start_time, end_time).unwrap();
        let right_box = world[1].bounding_box(start_time, end_time).unwrap();
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
            let centroid_a = centroid(a, axis, start_time, end_time);
            let centroid_b = centroid(b, axis, start_time, end_time);
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
    let centroids: Vec<Vec3> = world.iter().map(|hit| {
        let bbox = hit.bounding_box(start_time, end_time).unwrap();
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
            let obj_box = world[i].bounding_box(start_time, end_time).unwrap();
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
            
            // At this point we have how many objects fall in each bucket in bucket_counts
            // and the bounding box covering each object in bucket_boxes

            // Compute the number of left side counts along with number of bounding boxes on that side
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
    
    // We've found the split with the lowest cost so it's time to actually perform the split
    // Recompute the bucket index for each object using the best axis we found
    let (cmin, cmax) = (axis_value(centroid_min, best_axis), axis_value(centroid_max, best_axis));
    let split_threshold = best_bucket as f32;
    
    let bucket_for = |position: f32| -> usize {
        let mut bucket = (((position - cmin) / (cmax - cmin)) * NUM_BUCKETS as f32) as usize;
        if bucket >= NUM_BUCKETS { bucket = NUM_BUCKETS - 1; }
        bucket
    };
    
    // Traverse the list of centroids and categorize them as left side or right side
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
    
    // Safety fallback: if SAH partition was degenerate, use median.
    if left_items.is_empty() || right_items.is_empty() {
        world.sort_by(|a, b| {
            let centroid_a = centroid(a, best_axis, start_time, end_time);
            let centroid_b = centroid(b, best_axis, start_time, end_time);
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
    
    // Recurse into each side and wrap the result into the parent's Internal node
    let left = build_tree(&mut left_items, start_time, end_time);
    let right = build_tree(&mut right_items, start_time, end_time);
    
    TreeNode::Internal {
        bbox: main_box,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Flatten the tree into a Vec<BVHNode>. Returns the index of the root.
fn flatten(tree: &TreeNode, internals: &mut Vec<InternalNode>, leaves: &mut Vec<LeafNode>) -> u32 { 
    match tree {
        TreeNode::Leaf { bbox, geometry } => {
            let index = leaves.len();
            leaves.push(LeafNode {
                bbox: *bbox,
                geometry: geometry.clone(),
            });

            make_leaf_ref(index)
        }
        TreeNode::Internal { bbox, left, right } => {
            let index = internals.len();
            internals.push(InternalNode {
                bbox: *bbox,
                left: 0,  // placeholder
                right: 0, // placeholder
            });
            
            let left_idx = flatten(left, internals, leaves);
            let right_idx = flatten(right, internals, leaves);
            
            internals[index].left = left_idx;
            internals[index].right = right_idx;

            index as u32
        }
    }
}


/// Index into a vector with the given axis
fn axis_value(vector: Vec3, axis: usize) -> f32 {
    match axis {
        0 => vector.x,
        1 => vector.y,
        _ => vector.z,
    }
}

/// Compute the centroid of an object's bounding box along the given axis
/// Used for sorting in small lists
fn centroid(hit: &Geometry, axis: usize, t0: f32, t1: f32) -> f32 {
    let bbox = hit.bounding_box(t0, t1).unwrap();
    axis_value((bbox.minimum + bbox.maximum) * 0.5, axis)
}


impl BVH {
    pub fn new(world: &mut Vec<Geometry>, start_time: f32, end_time: f32) -> BVH {
        // Build the tree using SAH, then flatten it.
        let tree = build_tree(world, start_time, end_time);
        let bbox = *tree.bbox();
        
        let mut internals = Vec::new();
        let mut leaves = Vec::new();

        let root = flatten(&tree, &mut internals, &mut leaves);
        
        BVH { internals, leaves, root, bbox }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // Iterative traversal with explicit stack.
        // Stack size 64 supports trees up to depth 64, which covers
        // millions of objects with safety margin.
        let mut stack: [u32; 64] = [0; 64];
        let mut stack_pointer: usize = 0;
        
        let mut closest_t = t_max;
        let mut best_hit: Option<HitRecord> = None;
        
        stack[stack_pointer] = self.root;  // root
        stack_pointer += 1;
        
        while stack_pointer > 0 {
            stack_pointer -= 1;
            let node_ref = stack[stack_pointer];

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
                if node.bbox.hit(ray, t_min, closest_t) {
                    stack[stack_pointer] = node.left;
                    stack_pointer += 1;
                    stack[stack_pointer] = node.right;
                    stack_pointer += 1;
                }
            }
        }
        
        best_hit
    }
    
    pub fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(self.bbox)
    }
}
