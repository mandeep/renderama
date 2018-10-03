use nalgebra::core::Vector3;

use ray::Ray;


pub struct HitRecord {
    pub parameter: f64,
    pub point: Vector3<f64>,
    pub normal: Vector3<f64>
}


impl HitRecord {
    pub fn new(parameter: f64, point: Vector3<f64>, normal: Vector3<f64>) -> HitRecord {
        HitRecord { parameter: parameter, point: point, normal: normal }
    }
}


pub trait Hitable: Send + Sync {
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64) -> Option<HitRecord>;
}
