use std::sync::Arc;

use nalgebra::core::Vector3;

use aabb::AABB;
use materials::Material;
use ray::Ray;

/// HitRecord contains the elements necessary to render geometry
/// once a ray has hit that geometry.
pub struct HitRecord {
    pub parameter: f32,
    pub u: f32,
    pub v: f32,
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub material: Arc<dyn Material>,
}

impl HitRecord {
    /// Create a new HitRecord for a given ray-geometry intersection.
    pub fn new(parameter: f32,
               u: f32,
               v: f32,
               point: Vector3<f32>,
               normal: Vector3<f32>,
               material: Arc<dyn Material>)
               -> HitRecord {
        HitRecord { parameter: parameter,
                    u: u,
                    v: v,
                    point: point,
                    normal: normal,
                    material: material }
    }
}

/// The Hitable trait is a trait that all hitable objects will implement.
/// This way we can easily add different types of geometry to the renderer/
pub trait Hitable: Send + Sync {
    /// Determine if the ray records a hit.
    ///
    /// We use position_min and position_max to omit points on the ray
    /// near zero. This helps in reducing noise.
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord>;

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB>;
}

pub struct FlipNormals {
    hitable: Arc<dyn Hitable>,
}

impl FlipNormals {
    pub fn of<H: Hitable + 'static>(hitable: H) -> FlipNormals {
        let hitable = Arc::new(hitable);
        FlipNormals { hitable }
    }
}

impl Hitable for FlipNormals {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        if let Some(mut hit) = self.hitable.hit(&ray, position_min, position_max) {
            hit.normal = -hit.normal;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        self.hitable.bounding_box(t0, t1)
    }
}
