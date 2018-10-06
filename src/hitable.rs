use materials::Material;
use nalgebra::core::Vector3;
use ray::Ray;


pub struct HitRecord {
    pub parameter: f64,
    pub point: Vector3<f64>,
    pub normal: Vector3<f64>,
    pub material: Box<dyn Material>
}


impl HitRecord {
    pub fn new(parameter: f64,
               point: Vector3<f64>,
               normal: Vector3<f64>,
               material: Box<dyn Material>) -> HitRecord {
        HitRecord { parameter: parameter, point: point, normal: normal, material: material }
    }
}


pub trait Hitable: Send + Sync {
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64) -> Option<HitRecord>;
}
