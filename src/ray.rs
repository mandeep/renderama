use std::f32;

use glam::{IVec3, Vec3A};

pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
    pub inverse_direction: Vec3A,
}

impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(origin: Vec3A, direction: Vec3A) -> Ray {
        Ray { origin, direction: direction.normalize(), inverse_direction: direction.recip() }
    }

    /// Find the point on the ray given the parameter of the direction vector
    pub fn point_at_parameter(&self, parameter: f32) -> Vec3A {
        self.origin + parameter * self.direction
    }
}

/// Find the offset ray given the ray origin and geometric normal of the shape
/// 
/// This particular function discovers the closest point to the hit point that
/// will result in no self-intersection.
///
/// Reference:
/// Carsten Wächter, Nikolaus Binder
/// A Fast and Robust Method for Avoiding Self-Intersection
/// Ray Tracing Gems, Chapter 6
pub fn find_offset_point(point: Vec3A, geometric_normal: Vec3A) -> Vec3A {
    let origin: f32 = 1.0 / 32.0;
    let float_scale: f32 = 1.0 / 65536.0;
    let int_scale: f32 = 256.0;

    let offset_int = IVec3::new((int_scale * geometric_normal.x) as i32,
                                                (int_scale * geometric_normal.y) as i32,
                                                (int_scale * geometric_normal.z) as i32);

    let mut point_int = Vec3A::ZERO;

    if point.x < 0.0 {
        point_int.x = f32::from_bits(f32::to_bits(point.x).wrapping_sub(offset_int.x as u32));
    } else {
        point_int.x = f32::from_bits(f32::to_bits(point.x).wrapping_add(offset_int.x as u32));
    }
    if point.y < 0.0 {
        point_int.y = f32::from_bits(f32::to_bits(point.y).wrapping_sub(offset_int.y as u32));
    } else {
        point_int.y = f32::from_bits(f32::to_bits(point.y).wrapping_add(offset_int.y as u32));
    }

    if point.z < 0.0 {
        point_int.z = f32::from_bits(f32::to_bits(point.z).wrapping_sub(offset_int.z as u32));
    } else {
        point_int.z = f32::from_bits(f32::to_bits(point.z).wrapping_add(offset_int.z as u32));
    }

    let mut new_offset: Vec3A = point_int;

    if point.x.abs() < origin {
        new_offset.x = point_int.x + float_scale * geometric_normal.x;
    }
    if point.y.abs() < origin {
        new_offset.y = point_int.y + float_scale * geometric_normal.y;
    }
    if point.z.abs() < origin {
        new_offset.z = point_int.z + float_scale * geometric_normal.z;
    }

    new_offset
}
