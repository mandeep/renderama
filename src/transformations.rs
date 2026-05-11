use std::f32;

use glam::{Mat4, Vec3};

use aabb::AABB;
use events::HitEvent;
use geometry::Geometry;
use ray::Ray;

/// A mesh with combined translation, rotation (XYZ Euler), and uniform scale,
/// stored as precomputed forward and inverse matrices for fast ray transforms.
#[derive(Clone)]
pub struct TransformedMesh {
    /// Inverse of the transform: world-to-local. Used to transform rays.
    inv_transform: Mat4,
    /// Forward transform: local-to-world. Used to transform hits back.
    forward_transform: Mat4,
    /// Inverse-transpose for normal transformation (handles non-uniform scale,
    /// though we use uniform scale so it could be simpler).
    normal_transform: Mat4,
    /// World-space bounding box, computed once at construction.
    bbox: AABB,
    /// Uniform scale factor — needed to convert ray parameter t back to world space.
    scale: f32,
    geometry: Box<Geometry>,
}

fn transform_aabb(bbox: &AABB, transform: &Mat4) -> AABB {
    // Transform all 8 corners and take their bounds.
    let min = bbox.minimum;
    let max = bbox.maximum;
    
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ];
    
    let first = transform.transform_point3(corners[0]);
    let mut new_min = first;
    let mut new_max = first;
    
    for i in 1..8 {
        let p = transform.transform_point3(corners[i]);
        new_min = new_min.min(p);
        new_max = new_max.max(p);
    }
    
    AABB::from(new_min, new_max)
}

impl TransformedMesh {
    pub fn new(
        translate: Vec3,
        rotate_xyz_degrees: Vec3,
        scale: f32,
        geometry: Geometry,
    ) -> TransformedMesh {
        // Build the forward transform: local → world.
        // Order: Scale, then Rotate (X then Y then Z), then Translate.
        // This matches your existing Translate(Rotate(Scale(...))) composition.
        
        let scale_mat = Mat4::from_scale(Vec3::new(scale, scale, scale));
        
        let rx = rotate_xyz_degrees.x.to_radians();
        let ry = rotate_xyz_degrees.y.to_radians();
        let rz = rotate_xyz_degrees.z.to_radians();
        
        // Match your Rotate's sign convention (Y rotation matches original)
        let rot_x = Mat4::from_rotation_x(rx);
        let rot_y = Mat4::from_rotation_y(ry);
        let rot_z = Mat4::from_rotation_z(rz);
        
        let translate_mat = Mat4::from_translation(translate);
        
        let forward = translate_mat * rot_z * rot_y * rot_x * scale_mat;
        let inv = forward.inverse();
        
        // Compute world-space bounding box by transforming local bbox corners.
        let local_bbox = geometry.bounding_box().unwrap();
        let bbox = transform_aabb(&local_bbox, &forward);

        let geometry = Box::new(geometry);
        
        TransformedMesh {
            inv_transform: inv,
            forward_transform: forward,
            normal_transform: inv.transpose(),
            bbox,
            scale,
            geometry,
        }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitEvent> {
        // Transform ray into local space using the inverse matrix.
        let local_origin = self.inv_transform.transform_point3(ray.origin);
        let local_direction = self.inv_transform.transform_vector3(ray.direction);
        
        // The local direction may not be unit-length if there's scaling.
        // We need to handle the t-parameter rescaling carefully.
        let local_dir_length = local_direction.length();
        let local_direction_normalized = local_direction / local_dir_length;
        
        // The t-parameter in local space differs from world space by 1/local_dir_length
        // (since we normalized). Convert t bounds:
        let local_t_min = t_min * local_dir_length;
        let local_t_max = t_max * local_dir_length;
        
        let local_ray = Ray::new(local_origin, local_direction_normalized);
        
        if let Some(mut hit) = self.geometry.hit(&local_ray, local_t_min, local_t_max) {
            // Transform hit point back to world space.
            hit.point = self.forward_transform.transform_point3(hit.point);
            
            // Transform normals using the inverse-transpose, then renormalize.
            hit.shading_normal = self.normal_transform.transform_vector3(hit.shading_normal).normalize();
            hit.geometric_normal = self.normal_transform.transform_vector3(hit.geometric_normal).normalize();
            
            // Convert local-space t to world-space t.
            hit.parameter /= local_dir_length;
            
            Some(hit)
        } else {
            None
        }
    }
    
    pub fn bounding_box(&self) -> Option<AABB> {
        Some(self.bbox)
    }
}