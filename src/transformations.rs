use std::f32;

use glam::{Mat4, Vec3};

use aabb::AABB;
use geometry::Geometry;
use hitable::{HitRecord};
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
        let local_bbox = geometry.bounding_box(0.0, 1.0).unwrap();
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

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
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
        
        let local_ray = Ray::new(local_origin, local_direction_normalized, ray.time);
        
        if let Some(mut hit) = self.geometry.hit(&local_ray, local_t_min, local_t_max) {
            // Transform hit point back to world space.
            hit.point = self.forward_transform.transform_point3(hit.point);
            
            // Transform normals using the inverse-transpose, then renormalize.
            hit.shading_normal = self.normal_transform.transform_vector3(hit.shading_normal).normalize();
            hit.geometric_normal = self.normal_transform.transform_vector3(hit.geometric_normal).normalize();
            
            // Convert local-space t to world-space t.
            hit.parameter = hit.parameter / local_dir_length;
            
            Some(hit)
        } else {
            None
        }
    }
    
    pub fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(self.bbox.clone())
    }
}

pub struct Translate {
    offset: Vec3,
    geometry: Geometry,
}

impl Translate {
    pub fn new(offset: Vec3, geometry: Geometry) -> Translate {
        Translate { offset, geometry }
    }

    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray::new(ray.origin - self.offset, ray.direction, ray.time);
        if let Some(mut hit) = self.geometry.hit(&moved_ray, position_min, position_max) {
            hit.point += self.offset;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.geometry.bounding_box(t0, t1) {
            bbox.minimum += self.offset;
            bbox.maximum += self.offset;
            Some(bbox)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Rotate {
    cos_theta_x: f32, sin_theta_x: f32,
    cos_theta_y: f32, sin_theta_y: f32,
    cos_theta_z: f32, sin_theta_z: f32,
    geometry: Geometry,
}

impl Rotate {
    pub fn new(theta_x: f32, theta_y: f32, theta_z: f32, geometry: Geometry) -> Rotate {
        let (tx, ty, tz) = (theta_x.to_radians(), theta_y.to_radians(), theta_z.to_radians());
        Rotate {
            cos_theta_x: tx.cos(), sin_theta_x: tx.sin(),
            cos_theta_y: ty.cos(), sin_theta_y: ty.sin(),
            cos_theta_z: tz.cos(), sin_theta_z: tz.sin(),
            geometry
        }
    }

    /// Forward rotation: applies X, then Y, then Z (extrinsic order)
    fn rotate(&self, v: &Vec3) -> Vec3 {
        // Rotate around X
        let v = Vec3::new(
            v.x,
            self.cos_theta_x * v.y - self.sin_theta_x * v.z,
            self.sin_theta_x * v.y + self.cos_theta_x * v.z,
        );
        // Rotate around Y (matches original Rotate sign convention)
        let v = Vec3::new(
            self.cos_theta_y * v.x - self.sin_theta_y * v.z,
            v.y,
            self.sin_theta_y * v.x + self.cos_theta_y * v.z,
        );
        // Rotate around Z
        Vec3::new(
            self.cos_theta_z * v.x - self.sin_theta_z * v.y,
            self.sin_theta_z * v.x + self.cos_theta_z * v.y,
            v.z,
        )
    }

    /// Inverse rotation: applies Z⁻¹, then Y⁻¹, then X⁻¹
    fn rotate_inv(&self, v: &Vec3) -> Vec3 {
        // Inverse Z
        let v = Vec3::new(
            self.cos_theta_z * v.x + self.sin_theta_z * v.y,
            -self.sin_theta_z * v.x + self.cos_theta_z * v.y,
            v.z,
        );
        // Inverse Y
        let v = Vec3::new(
            self.cos_theta_y * v.x + self.sin_theta_y * v.z,
            v.y,
            -self.sin_theta_y * v.x + self.cos_theta_y * v.z,
        );
        // Inverse X
        Vec3::new(
            v.x,
            self.cos_theta_x * v.y + self.sin_theta_x * v.z,
            -self.sin_theta_x * v.y + self.cos_theta_x * v.z,
        )
    }

    fn hit(&self, ray: &Ray, t0: f32, t1: f32) -> Option<HitRecord> {
        let origin = self.rotate(&ray.origin);
        let direction = self.rotate(&ray.direction);
        let rotated_ray = Ray::new(origin, direction, ray.time);

        if let Some(mut hit) = self.geometry.hit(&rotated_ray, t0, t1) {
            hit.point = self.rotate_inv(&hit.point);
            hit.shading_normal = self.rotate_inv(&hit.shading_normal);
            hit.geometric_normal = self.rotate_inv(&hit.geometric_normal);
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(bbox) = self.geometry.bounding_box(t0, t1) {
            let mut min = Vec3::splat(f32::MAX);
            let mut max = Vec3::splat(f32::MIN);
            for i in 0..8 {
                let x = if i & 1 != 0 { bbox.maximum.x } else { bbox.minimum.x };
                let y = if i & 2 != 0 { bbox.maximum.y } else { bbox.minimum.y };
                let z = if i & 4 != 0 { bbox.maximum.z } else { bbox.minimum.z };
                let corner = self.rotate_inv(&Vec3::new(x, y, z));
                min = min.min(corner);
                max = max.max(corner);
            }
            Some(AABB::from(min, max))
        } else {
            None
        }
    }
}

pub struct Scale {
    scalar: f32,
    geometry: Geometry,
}

impl Scale {
    pub fn new(scalar: f32, geometry: Geometry) -> Scale {
        Scale { scalar, geometry }
    }

    /// Reference: http://woo4.me/raytracer/translations/
    fn hit(&self, ray: &Ray, t0: f32, t1: f32) -> Option<HitRecord> {
        let origin = ray.origin / self.scalar;
        let direction = (ray.direction / self.scalar).normalize();

        let scaled_ray = Ray::new(origin, direction, ray.time);

        // The inner hitable works in scaled-local space. Distances there
        // are 1/scalar times world distances, so scale t bounds accordingly
        // for correct BVH pruning, and scale the returned t back to world
        // space so the outer BVH's depth comparison works correctly.
        let scaled_t0 = t0 / self.scalar;
        let scaled_t1 = t1 / self.scalar;

        if let Some(mut hit) = self.geometry.hit(&scaled_ray, scaled_t0, scaled_t1) {
            hit.point = hit.point * self.scalar;
            hit.shading_normal = (hit.shading_normal / self.scalar).normalize();
            hit.parameter = hit.parameter * self.scalar;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.geometry.bounding_box(t0, t1) {
            bbox.minimum *= self.scalar;
            bbox.maximum *= self.scalar;
            Some(bbox)
        } else {
            None
        }
    }
}
