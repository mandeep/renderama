use std::f32;
use std::sync::Arc;

use glam::Vec3A;
use rand_pcg::Pcg64;
use rand::RngExt;

use aabb::AABB;
use events::HitEvent;
use primitive::Primitive;
use materials::MaterialId;
use ray::Ray;

#[derive(Clone)]
/// The three axes a plane can be created on
pub enum Axis {
    XY,
    YZ,
    XZ,
}

#[derive(Clone, Copy, Debug)]
/// Bounds2D is a type that allows for a more intuitive approach for creating
/// planes. u_min and u_max are the minimum and maximum on the first axis, and
/// v_min and v_max are the minimum and maximum on the second axis of the plane.
pub struct Bounds2D {
    pub u_min: f32,
    pub u_max: f32,
    pub v_min: f32,
    pub v_max: f32,
}

impl Bounds2D {
    /// Create a new Bounds2D from a range.
    ///
    /// # Examples
    ///
    /// Bounds2D::new(0.0..300.0, 0.0..200.0) creates a new bound
    /// with one axis being from 0.0 to 300.0 and the other axis being from
    /// 0.0 to 200.0.
    pub fn new(u: std::ops::Range<f32>, v: std::ops::Range<f32>) -> Self {
        debug_assert!(u.start < u.end, "u range must have start < end, got {:?}", u);
        debug_assert!(v.start < v.end, "v range must have start < end, got {:?}", v);
        Self { u_min: u.start, u_max: u.end, v_min: v.start, v_max: v.end }
    }

    pub fn u_extent(&self) -> f32 { self.u_max - self.u_min }
    pub fn v_extent(&self) -> f32 { self.v_max - self.v_min }
    pub fn area(&self) -> f32 { self.u_extent() * self.v_extent() }
}

#[derive(Clone)]
/// Plane allows for the creation of an axis-aligned plane on the given Axis
/// bounds is a Bounds2D that houses the range of the first and second axes
/// offset is the third axis on which the plane sits
/// material_id is the index to the material in the materials vec
pub struct Plane {
    axis: Axis,
    bounds: Bounds2D,
    offset: f32,
    material_id: MaterialId,
}

impl Plane {
    /// Create a new plane
    ///
    /// # Examples
    ///
    /// Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, mat_idx)
    /// This creates a plane on the YZ axis that sits at 555.0 on the X axis. The first
    /// range shows 0.0 to 555.0 on the Y axis and the second range shows 0.0 to 555.0 on
    /// the Z axis.
    pub fn new(axis: Axis, bounds: Bounds2D, offset: f32, material_id: MaterialId) -> Plane {
        Plane { axis, bounds, offset, material_id }
    }

    /// Convert the Plane into a Primitive for when adding to accelerators
    pub fn into_primitive(self) -> Primitive {
        Primitive::Plane(self)
    }

    /// Convert the Plane into a Plane with its normal flipped so that
    /// the plane can be used in the opposite orientation
    pub fn into_reversed(self) -> Primitive {
        Primitive::ReverseOrientation(Arc::new(Primitive::Plane(self)))
    }

    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitEvent> {
        match self.axis {
            Axis::XY => {
                let t = (self.offset - ray.origin.z) / ray.direction.z;

                if t < position_min || t > position_max {
                    return None;
                }

                let x = ray.origin.x + t * ray.direction.x;
                let y = ray.origin.y + t * ray.direction.y;

                if x < self.bounds.u_min || x > self.bounds.u_max || y < self.bounds.v_min || y > self.bounds.v_max {
                    return None;
                }

                let normal = Vec3A::new(0.0, 0.0, 1.0);

                let event = HitEvent::new(t,
                                            (x - self.bounds.u_min) / (self.bounds.u_max - self.bounds.u_min),
                                            (y - self.bounds.v_min) / (self.bounds.v_max - self.bounds.v_min),
                                            ray.point_at_parameter(t),
                                            normal,
                                            normal,
                                            self.material_id);

                Some(event)
            }
            Axis::YZ => {
                let t = (self.offset - ray.origin.x) / ray.direction.x;

                if t < position_min || t > position_max {
                    return None;
                }

                let y = ray.origin.y + t * ray.direction.y;
                let z = ray.origin.z + t * ray.direction.z;

                if y < self.bounds.u_min || y > self.bounds.u_max || z < self.bounds.v_min || z > self.bounds.v_max {
                    return None;
                }

                let normal = Vec3A::new(1.0, 0.0, 0.0);

                let event = HitEvent::new(t,
                                            (y - self.bounds.u_min) / (self.bounds.u_max - self.bounds.u_min),
                                            (z - self.bounds.v_min) / (self.bounds.v_max - self.bounds.v_min),
                                            ray.point_at_parameter(t),
                                            normal,
                                            normal,
                                            self.material_id);

                Some(event)
            }
            Axis::XZ => {
                let t = (self.offset - ray.origin.y) / ray.direction.y;

                if t < position_min || t > position_max {
                    return None;
                }

                let x = ray.origin.x + t * ray.direction.x;
                let z = ray.origin.z + t * ray.direction.z;

                if x < self.bounds.u_min || x > self.bounds.u_max || z < self.bounds.v_min || z > self.bounds.v_max {
                    return None;
                }

                let normal = Vec3A::new(0.0, 1.0, 0.0);

                let event = HitEvent::new(t,
                                            (x - self.bounds.u_min) / (self.bounds.u_max - self.bounds.u_min),
                                            (z - self.bounds.v_min) / (self.bounds.v_max - self.bounds.v_min),
                                            ray.point_at_parameter(t),
                                            normal,
                                            normal,
                                            self.material_id);

                Some(event)
            }
        }
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        match self.axis {
            Axis::XY => {
                let minimum = Vec3A::new(self.bounds.u_min, self.bounds.v_min, self.offset - 0.0001);
                let maximum = Vec3A::new(self.bounds.u_max, self.bounds.v_max, self.offset + 0.0001);
                Some(AABB::from(minimum, maximum))
            }
            Axis::YZ => {
                let minimum = Vec3A::new(self.offset - 0.0001, self.bounds.u_min, self.bounds.v_min);
                let maximum = Vec3A::new(self.offset + 0.0001, self.bounds.u_max, self.bounds.v_max);
                Some(AABB::from(minimum, maximum))
            }
            Axis::XZ => {
                let minimum = Vec3A::new(self.bounds.u_min, self.offset - 0.0001, self.bounds.v_min);
                let maximum = Vec3A::new(self.bounds.u_max, self.offset + 0.0001, self.bounds.v_max);
                Some(AABB::from(minimum, maximum))
            }
        }
    }

    pub fn evaluate_sampling_weight(&self, origin: Vec3A, direction: Vec3A) -> f32 {
        // originally epsilon was 1e-2 but updated here to match value elsewhere
        if let Some(hit) = self.hit(&Ray::new(origin, direction), 1e-4, f32::MAX) {
            let distance_squared = hit.parameter * hit.parameter * direction.length_squared();
            let cosine = direction.dot(hit.shading_normal).abs() / direction.length();
            distance_squared / (cosine * self.bounds.area())
        } else {
            0.0
        }
    }

    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut Pcg64) -> Vec3A {
        let u = self.bounds.u_min + rng.random::<f32>() * (self.bounds.u_max - self.bounds.u_min);
        let v = self.bounds.v_min + rng.random::<f32>() * (self.bounds.v_max - self.bounds.v_min);

        let random_point = match self.axis {
            Axis::XY => Vec3A::new(u, v, self.offset),
            Axis::YZ => Vec3A::new(self.offset, u, v),
            Axis::XZ => Vec3A::new(u, self.offset, v),
        };

        random_point - origin
    }
}
