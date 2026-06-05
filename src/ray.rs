use glam::{IVec3, Vec3A};

pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
    pub inverse_direction: Vec3A,
    pub time: f32,
}

impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(origin: Vec3A, direction: Vec3A, time: f32) -> Ray {
        let direction = direction.normalize();
        Ray { origin, direction, inverse_direction: direction.recip(), time }
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

    let offset_int = IVec3::new(
        (int_scale * geometric_normal.x) as i32,
        (int_scale * geometric_normal.y) as i32,
        (int_scale * geometric_normal.z) as i32,
    );

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
        new_offset.x = point.x + float_scale * geometric_normal.x;
    }
    if point.y.abs() < origin {
        new_offset.y = point.y + float_scale * geometric_normal.y;
    }
    if point.z.abs() < origin {
        new_offset.z = point.z + float_scale * geometric_normal.z;
    }

    new_offset
}


#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;

    #[test]
    fn test_ray_new_normalization_and_recip() {
        let origin = Vec3A::ZERO;
        let direction = Vec3A::new(0.0, 3.0, 4.0);
        let ray = Ray::new(origin, direction, 0.0);

        let expected_direction = Vec3A::new(0.0, 0.6, 0.8);
        assert_eq!(ray.direction, expected_direction);

        let expected_inverse_direction = Vec3A::new(f32::INFINITY, 1.0 / 0.6, 1.0 / 0.8);
        assert_eq!(ray.inverse_direction, expected_inverse_direction);
    }

    #[test]
    fn test_point_at_parameter() {
        let ray = Ray::new(Vec3A::new(1.0, 1.0, 1.0), Vec3A::new(0.0, 1.0, 0.0), 0.0);
        let point = ray.point_at_parameter(5.0);
        assert_eq!(point, Vec3A::new(1.0, 6.0, 1.0));
    }

    #[test]
    fn test_find_offset_point_far_from_origin() {
        let point = Vec3A::new(100.0, -100.0, 50.0);
        let normal = Vec3A::new(0.0, 1.0, 0.0);

        let offset_point = find_offset_point(point, normal);

        assert!(offset_point.y > point.y);

        assert_eq!(offset_point.x, point.x);
        assert_eq!(offset_point.z, point.z);
    }

    #[test]
    fn test_find_offset_point_near_origin_fallback() {
        let point = Vec3A::new(0.0, 0.0, 0.0);
        let normal = Vec3A::new(1.0, 0.0, -1.0).normalize();

        let offset_point = find_offset_point(point, normal);

        let float_scale: f32 = 1.0 / 65536.0;
        let expected_x = float_scale * normal.x;
        let expected_z = float_scale * normal.z;

        assert_eq!(offset_point.x, expected_x);
        assert_eq!(offset_point.y, 0.0);
        assert_eq!(offset_point.z, expected_z);
    }
}