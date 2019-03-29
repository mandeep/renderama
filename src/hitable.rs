use nalgebra::core::Vector3;

use materials::Material;
use ray::Ray;


/// HitRecord contains the elements necessary to render geometry
/// once a ray has hit that geometry.
pub struct HitRecord {
    pub parameter: f64,
    pub point: Vector3<f64>,
    pub normal: Vector3<f64>,
    pub material: Box<dyn Material>,
}


impl HitRecord {
    /// Create a new HitRecord for a given ray-geometry intersection.
    pub fn new(parameter: f64,
               point: Vector3<f64>,
               normal: Vector3<f64>,
               material: Box<dyn Material>) -> HitRecord {
        HitRecord { parameter: parameter, point: point, normal: normal, material: material }
    }
}


/// The Hitable trait is a trait that all hitable objects will implement.
/// This way we can easily add different types of geometry to the renderer/
pub trait Hitable: Send + Sync {
    /// Determine if the ray records a hit.
    ///
    /// We use position_min and position_max to omit points on the ray
    /// near zero. This helps in reducing noise.
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64) -> Option<HitRecord>;
}
