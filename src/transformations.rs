use std::f32;
use std::f32::consts::PI;
use std::sync::Arc;

use glam::{Mat4, Vec3};

use aabb::AABB;
use hitable::{HitRecord, Hitable};
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
    hitable: Arc<dyn Hitable>,
}

impl TransformedMesh {
    pub fn new<H: Hitable + 'static>(
        translate: Vec3,
        rotate_xyz_degrees: Vec3,
        scale: f32,
        hitable: H,
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
        let rot_y = Mat4::from_rotation_y(-ry);  // verify sign matches your Rotate
        let rot_z = Mat4::from_rotation_z(rz);
        
        let translate_mat = Mat4::from_translation(translate);
        
        let forward = translate_mat * rot_z * rot_y * rot_x * scale_mat;
        let inv = forward.inverse();
        
        // Compute world-space bounding box by transforming local bbox corners.
        let local_bbox = hitable.bounding_box(0.0, 1.0).unwrap();
        let bbox = transform_aabb(&local_bbox, &forward);
        
        TransformedMesh {
            inv_transform: inv,
            forward_transform: forward,
            normal_transform: inv.transpose(),
            bbox,
            scale,
            hitable: Arc::new(hitable),
        }
    }
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

impl Hitable for TransformedMesh {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
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
        
        if let Some(mut hit) = self.hitable.hit(&local_ray, local_t_min, local_t_max) {
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
    
    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(self.bbox.clone())
    }
}

pub struct Translate {
    offset: Vec3,
    hitable: Arc<dyn Hitable>,
}

impl Translate {
    pub fn new<H: Hitable + 'static>(offset: Vec3, hitable: H) -> Translate {
        let hitable = Arc::new(hitable);
        Translate { offset, hitable }
    }
}

impl Hitable for Translate {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray::new(ray.origin - self.offset, ray.direction, ray.time);
        if let Some(mut hit) = self.hitable.hit(&moved_ray, position_min, position_max) {
            hit.point += self.offset;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.hitable.bounding_box(t0, t1) {
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
    sin_theta: f32,
    cos_theta: f32,
    hitable: Arc<dyn Hitable>,
}

impl Rotate {
    pub fn new<H: Hitable + 'static>(angle: f32, hitable: H) -> Rotate {
        let hitable = Arc::new(hitable);
        let radians = (PI / 180.0) * angle;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        Rotate { sin_theta,
                 cos_theta,
                 hitable }
    }

    pub fn rotate(&self, vector: &Vec3) -> Vec3 {
        Vec3::new(self.cos_theta * vector.x - self.sin_theta * vector.z,
                  vector.y,
                  self.sin_theta * vector.x + self.cos_theta * vector.z)
    }

    pub fn rotate_inv(&self, vector: &Vec3) -> Vec3 {
        Vec3::new(self.cos_theta * vector.x + self.sin_theta * vector.z,
                  vector.y,
                  -self.sin_theta * vector.x + self.cos_theta * vector.z)
    }
}

impl Hitable for Rotate {
    fn hit(&self, ray: &Ray, t0: f32, t1: f32) -> Option<HitRecord> {
        let origin = self.rotate(&ray.origin);
        let direction = self.rotate(&ray.direction);

        let rotated_ray = Ray::new(origin, direction, ray.time);

        if let Some(mut hit) = self.hitable.hit(&rotated_ray, t0, t1) {
            hit.point = self.rotate_inv(&hit.point);
            hit.shading_normal = self.rotate_inv(&hit.shading_normal);
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.hitable.bounding_box(t0, t1) {
            let mut minimum = Vec3::splat(f32::MAX);
            let mut maximum = Vec3::splat(f32::MIN);
            (0..2).for_each(|i| {
                      (0..2).for_each(|j| {
                                (0..2).for_each(|k| {
                                          let x = i as f32 * bbox.maximum.x
                                                  + (1 - i) as f32 * bbox.minimum.x;
                                          let y = j as f32 * bbox.maximum.y
                                                  + (1 - j) as f32 * bbox.minimum.y;
                                          let z = k as f32 * bbox.maximum.z
                                                  + (1 - k) as f32 * bbox.minimum.z;
                                          let newx = self.cos_theta * x + self.sin_theta * z;
                                          let newz = -self.sin_theta * x + self.cos_theta * z;
                                          let rotation = Vec3::new(newx, y, newz);
                                          maximum = maximum.max(rotation);
                                          minimum = minimum.min(rotation);
                                      });
                            });
                  });

            bbox.minimum = minimum;
            bbox.maximum = maximum;
            Some(bbox)
        } else {
            None
        }
    }
}

pub struct Scale {
    scalar: f32,
    hitable: Arc<dyn Hitable>,
}

impl Scale {
    pub fn new<H: Hitable + 'static>(scalar: f32, hitable: H) -> Scale {
        let hitable = Arc::new(hitable);
        Scale { scalar, hitable }
    }
}

impl Hitable for Scale {
    /// Reference: http://woo4.me/raytracer/translations/
    fn hit(&self, ray: &Ray, t0: f32, t1: f32) -> Option<HitRecord> {
        let origin = ray.origin / self.scalar;
        let direction = (ray.direction / self.scalar).normalize();

        let scaled_ray = Ray::new(origin, direction, ray.time);

        if let Some(mut hit) = self.hitable.hit(&scaled_ray, t0, t1) {
            hit.point = hit.point * self.scalar;
            hit.shading_normal = (hit.shading_normal / self.scalar).normalize();
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.hitable.bounding_box(t0, t1) {
            bbox.minimum *= self.scalar;
            bbox.maximum *= self.scalar;
            Some(bbox)
        } else {
            None
        }
    }
}
