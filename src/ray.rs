use std::f32;

use glam::Vec3;
use nalgebra::Vector3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub time: f32,
    pub inverse_direction: Vec3,
}

impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(origin: Vec3, direction: Vec3, time: f32) -> Ray {
        Ray { origin: origin,
              direction: direction.normalize(),
              time: time,
              inverse_direction: direction.reciprocal() }
    }

    /// Find the point on the ray given the parameter of the direction vector
    pub fn point_at_parameter(&self, parameter: f32) -> Vec3 {
        self.origin + parameter * self.direction
    }
}

/// Find the offset ray given the ray origin and geometric normal of the shape
///
/// Reference:
/// Carsten Wächter, Nikolaus Binder
/// A Fast and Robust Method for Avoiding Self-Intersection
/// Ray Tracing Gems, Chapter 6
pub fn find_offset_point(point: Vec3, geometric_normal: Vec3) -> Vec3 {
    let origin: f32 = 1.0 / 32.0;
    let float_scale: f32 = 1.0 / 65536.0;
    let int_scale: f32 = 256.0;

    let offset_int: Vector3<u32> = Vector3::new((int_scale * geometric_normal.x()) as u32,
                                                (int_scale * geometric_normal.y()) as u32,
                                                (int_scale * geometric_normal.z()) as u32);

    let mut point_int = Vec3::zero();

    if point.x() < 0.0 {
        point_int.set_x(f32::from_bits(f32::to_bits(point.x()).wrapping_sub(offset_int.x)));
    } else {
        point_int.set_x(f32::from_bits(f32::to_bits(point.x()).wrapping_add(offset_int.x)));
    }
    if point.y() < 0.0 {
        point_int.set_y(f32::from_bits(f32::to_bits(point.y()).wrapping_sub(offset_int.y)));
    } else {
        point_int.set_y(f32::from_bits(f32::to_bits(point.y()).wrapping_add(offset_int.y)));
    }

    if point.z() < 0.0 {
        point_int.set_z(f32::from_bits(f32::to_bits(point.z()).wrapping_sub(offset_int.z)));
    } else {
        point_int.set_z(f32::from_bits(f32::to_bits(point.z()).wrapping_add(offset_int.z)));
    }

    let mut new_offset: Vec3 = point_int.clone();

    if point.x().abs() < origin {
        new_offset.set_x(point_int.x() + float_scale * geometric_normal.x());
    }
    if point.y().abs() < origin {
        new_offset.set_y(point_int.y() + float_scale * geometric_normal.y());
    }
    if point.z().abs() < origin {
        new_offset.set_z(point_int.z() + float_scale * geometric_normal.z());
    }

    new_offset
}
