//! Contains all of the necessary structures for building a flattened
//! 4-wide BVH.
//!
//! Further reading:
//! https://medium.com/@bromanz/how-to-create-awesome-accelerators-the-surface-area-heuristic-e14b5dec6160
//! https://www.sci.utah.edu/~wald/Publications/2007/ParallelBVHBuild/fastbuild.pdf
//! https://research.nvidia.com/sites/default/files/pubs/2013-09_On-Quality-Metrics/aila2013hpg_paper.pdf
//! https://pbr-book.org/3ed-2018/Primitives_and_Intersection_Acceleration/Bounding_Volume_Hierarchies
use glam::Vec3A;
use rand::Rng;
use wide::f32x4;

use crate::aabb::AABB;
use crate::primitive::Primitive;
use crate::ray::Ray;
use crate::results::HitResult;


// 16 is used as the number of buckets as it's a common number in BVH builds
const NUM_BUCKETS: usize = 16;
// leaf flag is used so that we don't have extra overhead.
// if bit 31 is set then it's a leaf, otherwise it's an index into internals[].
// this keeps branching at a minimum
const LEAF_FLAG: u32 = 1 << 31;

fn is_leaf(idx: u32) -> bool { idx & LEAF_FLAG != 0 }
fn leaf_index(idx: u32) -> usize { (idx & !LEAF_FLAG) as usize }
fn make_leaf_ref(idx: usize) -> u32 { (idx as u32) | LEAF_FLAG }


/// Store child bounding boxes in a Structure-of-Arrays (SoA) format.
/// This allows slab testing in a single instruction with SIMD.
/// min_x, min_y, etc. are the values of each of the four children.
/// children contains the index into internals[] or leaves[] for
/// the count members of the node.
#[derive(Clone, Default)]
#[repr(C, align(16))]
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
/// Store the AABB and index into BVH::primitives for the primitive at
/// the leaf node. Used once we determine that BVH traversal has resulted
/// in a hit.
struct LeafNode {
    bbox: AABB,
    primitive_index: u32,
}


#[derive(Clone)]
/// Flattened BVH structure used as the finalized BVH. This representation
/// avoids the overhead of recursion and is cache friendly. Instead of storing
/// nodes on the heap, having a contiguous block of memory avoids the pain
/// of a cache miss.
pub struct BVH {
    internals: Vec<InternalNode4>,
    leaves: Vec<LeafNode>,
    primitives: Vec<Primitive>,
    root: u32,
    bbox: AABB, //root's bbox
}

/// Recursive binary tree used as an intermediate step to build the binary tree.
/// Using Arc/Box to store TreeNode's on the heap is okay here since this tree
/// isn't used in traversal. Leaves reference primitives by index into the
/// original world slice rather than cloning them as we did in the past.
enum TreeNode {
    Internal {
        bbox: AABB,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
    Leaf {
        bbox: AABB,
        index: u32,
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

/// Build a BVH using the binary tree node representation using the surface area
/// heuristic (SAH). Fall back methods are provided if the number of primitives
/// in the scene don't pass the threshold where the cost of computing SAH is
/// necessary. Once built, this tree is then flattened for use with SIMD.
///
/// indices contains the index of the Primitive in the world allowing
/// us to use less memory when building the tree.
fn build_tree(world: &[Primitive], indices: &mut [u32]) -> TreeNode {
    let n = indices.len();

    // compute the bounding box that contains all of the objects in the world
    // and their bounding boxes
    let mut main_box = world[indices[0] as usize].bounding_box().unwrap();
    for i in 1..n {
        let new_box = world[indices[i] as usize].bounding_box().unwrap();
        main_box = main_box.surrounding_box(&new_box);
    }

    if n == 1 {
        return TreeNode::Leaf {
            bbox: main_box,
            index: indices[0],
        };
    }

    if n == 2 {
        let left_box = world[indices[0] as usize].bounding_box().unwrap();
        let right_box = world[indices[1] as usize].bounding_box().unwrap();
        return TreeNode::Internal {
            bbox: main_box,
            left: Box::new(TreeNode::Leaf { bbox: left_box, index: indices[0] }),
            right: Box::new(TreeNode::Leaf { bbox: right_box, index: indices[1] }),
        };
    }

    // fall back to median split when not enough objects in scene.
    // 4 objects here is just a randomly picked low number.
    // this is the BVH leftover from Shirley's Ray Tracing series.
    if n <= 4 {
        let axis = main_box.longest_axis();
        indices.sort_by(|&a, &b| {
            let centroid_a = centroid(&world[a as usize], axis);
            let centroid_b = centroid(&world[b as usize], axis);
            centroid_a.partial_cmp(&centroid_b).unwrap()
        });

        // split the indices in two halves. right_indices takes half of the
        // indices while indices keeps the other half
        let (left_indices, right_indices) = indices.split_at_mut(n / 2);
        let left = build_tree(world, left_indices);
        let right = build_tree(world, right_indices);

        return TreeNode::Internal {
            bbox: main_box,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    // SAH binned split begins here

    // compute the centroid of each bounding box.
    // using centroid's makes binning more stable
    let centroids: Vec<Vec3A> = indices.iter().map(|&i| {
        let bbox = world[i as usize].bounding_box().unwrap();
        (bbox.minimum + bbox.maximum) * 0.5
    }).collect();

    // find the bounds of each centroid so we can later bin them into buckets.
    // we bin objects into buckets along each axis based on their centroid
    let mut centroid_min = centroids[0];
    let mut centroid_max = centroids[0];
    for centroid in &centroids[1..] {
        centroid_min = centroid_min.min(*centroid);
        centroid_max = centroid_max.max(*centroid);
    }

    // beginning of optimization loop for binning. parent_area is the
    // surface area bounding box for SAH cost computation
    let parent_area = main_box.surface_area();
    let mut best_cost = f32::INFINITY;
    let mut best_axis = 0;
    let mut best_bucket = NUM_BUCKETS / 2;

    // find the centroid range along each axis. this will allow us to
    // find which bucket each object goes into. the buckets are then used
    // to find the best split.
    for axis in 0..3 {
        let (cmin, cmax) = (axis_value(centroid_min, axis), axis_value(centroid_max, axis));
        if (cmax - cmin) < 1e-6 { continue; }

        // for each bucket in NUM_BUCKETS we track how many objects fall into that bin
        // and the bounding box that covers all of the objects in that bin
        let mut bucket_counts = [0usize; NUM_BUCKETS];
        let mut bucket_boxes: Vec<Option<AABB>> = vec![None; NUM_BUCKETS];

        // compute the bucket each centroid falls into
        for (i, centroid) in centroids.iter().enumerate() {
            let position = axis_value(*centroid, axis);
            // each primitive is mapped to a bucket baised on its centroid
            let mut bucket = (((position - cmin) / (cmax - cmin)) * NUM_BUCKETS as f32) as usize;
            if bucket >= NUM_BUCKETS { bucket = NUM_BUCKETS - 1; }

            // update the bucket count for the bin and expand the bucket's
            // bounding box to include this object
            bucket_counts[bucket] += 1;
            let object_box = world[indices[i] as usize].bounding_box().unwrap();
            bucket_boxes[bucket] = Some(match &bucket_boxes[bucket] {
                Some(existing_box) => existing_box.surrounding_box(&object_box),
                None => object_box,
            });
        }

        // evaluate the split between the buckets.
        // we try every possible split in this O(n^2) loop
        // and compute the cost of each split.
        // whichever has the lowest cost is chosen as our split.
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

            // compute the number of right side counts along with the number
            // of bounding boxes on that side
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

            // this is the surface area heuristic (SAH).
            // compute the probability that a random ray hits each child
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

    // now that we've calculated the best split we can subdivide the indices.
    let mut left_indices = Vec::with_capacity(n);
    let mut right_indices = Vec::with_capacity(n);
    for i in 0..n {
        let position = axis_value(centroids[i], best_axis);
        if (bucket_for(position) as f32) < split_threshold {
            left_indices.push(indices[i]);
        } else {
            right_indices.push(indices[i]);
        }
    }

    // fallback to median split if everything ends up on one side of the split
    if left_indices.is_empty() || right_indices.is_empty() {
        indices.sort_by(|&a, &b| {
            let centroid_a = centroid(&world[a as usize], best_axis);
            let centroid_b = centroid(&world[b as usize], best_axis);
            centroid_a.partial_cmp(&centroid_b).unwrap()
        });
        let (left_indices, right_indices) = indices.split_at_mut(n / 2);
        let left = build_tree(world, left_indices);
        let right = build_tree(world, right_indices);
        return TreeNode::Internal {
            bbox: main_box,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    let split = left_indices.len();
    indices[..split].copy_from_slice(&left_indices);
    indices[split..].copy_from_slice(&right_indices);
    let (left_indices, right_indices) = indices.split_at_mut(split);

    let left = build_tree(world, left_indices);
    let right = build_tree(world, right_indices);

    TreeNode::Internal {
        bbox: main_box,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Collapse the children in the BVH.
///
/// Used to convert children in a binary tree
/// into a collection of 4 children for our flattened BVH. Children are
/// collapsed from largest surface area to smallest.
fn collect_children(tree: &TreeNode) -> Vec<&TreeNode> {
    let (left, right) = match tree {
        TreeNode::Internal { left, right, .. } => (left.as_ref(), right.as_ref()),
        TreeNode::Leaf { .. } => return vec![tree],
    };

    let mut children: Vec<&TreeNode> = vec![left, right];

    while children.len() < 4 {
        let position = children.iter().enumerate()
            .filter(|(_, c)| matches!(c, TreeNode::Internal { .. }))
            .max_by(|(_, a), (_, b)| {
                a.bbox().surface_area().partial_cmp(&b.bbox().surface_area()).unwrap()
            })
            .map(|(i, _)| i);
        match position {
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

/// Flatten the binary tree BVH into a Vec for traversal purposes
fn flatten4(tree: &TreeNode, internals: &mut Vec<InternalNode4>, leaves: &mut Vec<LeafNode>) -> u32 {
    match tree {
        // push a leaf into the leaves vec and encode the leaf as a u32 index.
        // this index is stored in the children array
        TreeNode::Leaf { bbox, index } => {
            let leaf_index = leaves.len();
            leaves.push(LeafNode { bbox: *bbox, primitive_index: *index });
            make_leaf_ref(leaf_index)
        }

        // collect all children of the binary tree into the internals vec.
        // this is the step that converts from binary to 4-wide. the u32
        // index returned is the index into the internals vec.
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
fn centroid(hit: &Primitive, axis: usize) -> f32 {
    let bbox = hit.bounding_box().unwrap();
    axis_value((bbox.minimum + bbox.maximum) * 0.5, axis)
}


impl BVH {
    pub fn new(world: Vec<Primitive>) -> BVH {
        if world.is_empty() {
            eprintln!("BVH contains no objects. Stopping render.");
            std::process::exit(0);
        }

        let mut indices: Vec<u32> = (0..world.len() as u32).collect();
        let tree = build_tree(&world, &mut indices);
        let bbox = *tree.bbox();

        let mut internals = Vec::new();
        let mut leaves = Vec::new();
        let root = flatten4(&tree, &mut internals, &mut leaves);

        BVH { internals, leaves, primitives: world, root, bbox }
    }

    /// Traverse the BVH and return the closest hit.
    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut impl Rng) -> Option<HitResult> {
        // we iterate the traversal with a depth of 64 which should be okay for
        // millions of objects
        let mut stack: [u32; 256] = [0; 256];
        let mut stack_ptr: usize = 0;

        let mut closest_distance = end_distance;
        let mut best_hit: Option<HitResult> = None;

        stack[stack_ptr] = self.root;
        stack_ptr += 1;

        // splat the ray into SIMD registers for quicker computations
        let ox = f32x4::splat(ray.origin.x);
        let oy = f32x4::splat(ray.origin.y);
        let oz = f32x4::splat(ray.origin.z);
        let idx = f32x4::splat(ray.inverse_direction.x);
        let idy = f32x4::splat(ray.inverse_direction.y);
        let idz = f32x4::splat(ray.inverse_direction.z);
        let start_distance_floor = f32x4::splat(start_distance);

        while stack_ptr > 0 {
            stack_ptr -= 1;
            let node_ref = stack[stack_ptr];

            if is_leaf(node_ref) {
                let leaf = &self.leaves[leaf_index(node_ref)];
                if leaf.bbox.hit(ray, start_distance, closest_distance) {
                    let leaf_primitive = &self.primitives[leaf.primitive_index as usize];
                    if let Some(hit) = leaf_primitive.hit(ray, start_distance, closest_distance, rng) {
                        if hit.parameter < closest_distance {
                            closest_distance = hit.parameter;
                            best_hit = Some(hit);
                        }
                    }
                }
            } else {
                let node = &self.internals[node_ref as usize];

                // perform a SIMD slab test on internal nodes (children)
                let t0x = (f32x4::from(node.min_x) - ox) * idx;
                let t1x = (f32x4::from(node.max_x) - ox) * idx;
                let t0y = (f32x4::from(node.min_y) - oy) * idy;
                let t1y = (f32x4::from(node.max_y) - oy) * idy;
                let t0z = (f32x4::from(node.min_z) - oz) * idz;
                let t1z = (f32x4::from(node.max_z) - oz) * idz;

                let start_distance4 = t0x.min(t1x).max(t0y.min(t1y)).max(t0z.min(t1z)).max(start_distance_floor);

                let end_distance4 = t0x.max(t1x).min(t0y.max(t1y)).min(t0z.max(t1z))
                    .min(f32x4::splat(closest_distance));

                let start_distance_arr: [f32; 4] = start_distance4.into();
                let end_distance_arr: [f32; 4] = end_distance4.into();

                let mut hits: [(f32, u32); 4] = [(f32::MAX, 0); 4];
                let mut hit_count = 0usize;

                // collect the hit children with their entry distances
                for i in 0..node.count as usize {
                    if start_distance_arr[i] <= end_distance_arr[i] {
                        hits[hit_count] = (start_distance_arr[i], node.children[i]);
                        hit_count += 1;
                    }
                }

                // sort the hit children in descending order based on entry distance.
                // since there are only 4 children, insertion sort works fine here.
                for i in 1..hit_count {
                    let key = hits[i];
                    let mut j = i;
                    while j > 0 && hits[j - 1].0 < key.0 {
                        hits[j] = hits[j - 1];
                        j -= 1;
                    }
                    hits[j] = key;
                }

                // push hit children onto the stack
                for i in 0..hit_count {
                    stack[stack_ptr] = hits[i].1;
                    stack_ptr += 1;
                }
            }
        }

        best_hit
    }

    /// Test if a shadow ray hits anything on its path to the light source
    pub fn hits_anything(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut impl Rng) -> bool {
        let mut stack: [u32; 256] = [0; 256];
        let mut stack_ptr: usize = 0;

        stack[stack_ptr] = self.root;
        stack_ptr += 1;

        let ox = f32x4::splat(ray.origin.x);
        let oy = f32x4::splat(ray.origin.y);
        let oz = f32x4::splat(ray.origin.z);
        let idx = f32x4::splat(ray.inverse_direction.x);
        let idy = f32x4::splat(ray.inverse_direction.y);
        let idz = f32x4::splat(ray.inverse_direction.z);
        let start_distance_floor = f32x4::splat(start_distance);
        let end_distance_ceil  = f32x4::splat(end_distance);

        while stack_ptr > 0 {
            stack_ptr -= 1;
            let node_ref = stack[stack_ptr];

            if is_leaf(node_ref) {
                let leaf = &self.leaves[leaf_index(node_ref)];
                if leaf.bbox.hit(ray, start_distance, end_distance) {
                    let leaf_primitive = &self.primitives[leaf.primitive_index as usize];
                    if leaf_primitive.hits_anything(ray, start_distance, end_distance, rng) {
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

                let start_distance4 = t0x.min(t1x).max(t0y.min(t1y)).max(t0z.min(t1z)).max(start_distance_floor);
                let end_distance4 = t0x.max(t1x).min(t0y.max(t1y)).min(t0z.max(t1z)).min(end_distance_ceil);

                let start_distance_arr: [f32; 4] = start_distance4.into();
                let end_distance_arr: [f32; 4] = end_distance4.into();

                for i in 0..node.count as usize {
                    if start_distance_arr[i] <= end_distance_arr[i] {
                        stack[stack_ptr] = node.children[i];
                        stack_ptr += 1;
                    }
                }
            }
        }

        false
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        Some(self.bbox)
    }

    pub fn primitive(&self, index: usize) -> &Primitive {
        &self.primitives[index]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;
    use rand_pcg::Pcg64Mcg;
    use rand::SeedableRng;

    use crate::materials::MaterialId;
    use crate::ray::Ray;
    use crate::sphere::Sphere;

    fn get_rng() -> Pcg64Mcg {
        Pcg64Mcg::seed_from_u64(0)
    }

    fn get_mat() -> MaterialId {
        MaterialId(0)
    }

    #[test]
    fn test_bvh_single_object_hit() {
        let mut rng = get_rng();
        let sphere = Sphere::new(Vec3A::new(0.0, 0.0, -10.0), 2.0, get_mat());
        let world = vec![sphere.into()];
        let bvh = BVH::new(world);

        let ray = Ray::new(Vec3A::ZERO, Vec3A::new(0.0, 0.0, -1.0), 0.0);
        let hit = bvh.hit(&ray, 0.001, f32::MAX, &mut rng);

        assert!(hit.is_some());
        assert!((hit.unwrap().parameter - 8.0).abs() < 1e-4);
    }

    #[test]
    fn test_bvh_single_object_miss() {
        let mut rng = get_rng();
        let sphere = Sphere::new(Vec3A::new(0.0, 0.0, -10.0), 2.0, get_mat());
        let world = vec![sphere.into()];
        let bvh = BVH::new(world);

        let ray = Ray::new(Vec3A::ZERO, Vec3A::new(0.0, 1.0, 0.0), 0.0);
        let hit = bvh.hit(&ray, 0.001, f32::MAX, &mut rng);

        assert!(hit.is_none());
    }

    #[test]
    fn test_bvh_closest_hit_priority() {
        let mut rng = get_rng();

        let sphere_front = Sphere::new(Vec3A::new(0.0, 0.0, -10.0), 2.0, get_mat());
        let sphere_back = Sphere::new(Vec3A::new(0.0, 0.0, -15.0), 4.0, get_mat());

        let world = vec![sphere_back.into(), sphere_front.into()];
        let bvh = BVH::new(world);

        let ray = Ray::new(Vec3A::ZERO, Vec3A::new(0.0, 0.0, -1.0), 0.0);
        let hit = bvh.hit(&ray, 0.001, f32::MAX, &mut rng);

        assert!(hit.is_some());
        assert!((hit.unwrap().parameter - 8.0).abs() < 1e-4);
    }

    #[test]
    fn test_bvh_hits_anything_early_exit() {
        let mut rng = get_rng();
        let sphere1 = Sphere::new(Vec3A::new(-10.0, 0.0, -10.0), 2.0, get_mat());
        let sphere2 = Sphere::new(Vec3A::new(10.0, 0.0, -10.0), 2.0, get_mat());
        let world = vec![sphere1.into(), sphere2.into()];
        let bvh = BVH::new(world);

        let hit_ray = Ray::new(Vec3A::ZERO, Vec3A::new(10.0, 0.0, -10.0), 0.0);
        assert!(bvh.hits_anything(&hit_ray, 0.001, f32::MAX, &mut rng));

        let miss_ray = Ray::new(Vec3A::ZERO, Vec3A::new(0.0, 0.0, -1.0), 0.0);
        assert!(!bvh.hits_anything(&miss_ray, 0.001, f32::MAX, &mut rng));
    }

    #[test]
    fn test_bvh_distance_bounds() {
        let mut rng = get_rng();
        let sphere = Sphere::new(Vec3A::new(0.0, 0.0, 10.0), 1.0, get_mat());
        let world = vec![sphere.into()];
        let bvh = BVH::new(world);

        let ray = Ray::new(Vec3A::ZERO, Vec3A::new(0.0, 0.0, 1.0), 0.0);

        let too_short_hit = bvh.hit(&ray, 0.001, 5.0, &mut rng);
        assert!(too_short_hit.is_none());

        let too_far_hit = bvh.hit(&ray, 15.0, f32::MAX, &mut rng);
        assert!(too_far_hit.is_none());
    }
}