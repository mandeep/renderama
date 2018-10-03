use nalgebra::core::Vector3;

use ray::Ray;


pub struct HitRecord {
    pub position: f64,
    pub point: Vector3<f64>,
    pub normal: Vector3<f64>
}


impl HitRecord {
    pub fn new(position: f64) -> HitRecord {
        HitRecord { position: position,
                    point: Vector3::zeros(),
                    normal: Vector3::zeros() }
    }
}


pub trait Hitable {
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64, hit_record: &mut HitRecord)
        -> bool;
}


pub struct HitableList {
    pub list: Vec<Box<dyn Hitable>>
}


impl HitableList {
    pub fn new() -> HitableList {
        HitableList { list: Vec::new() }
    }

    pub fn push(&mut self, hitable: Box<dyn Hitable>) {
        self.list.push(hitable);
    }
}


impl Hitable for HitableList {
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64, hit_record: &mut HitRecord)
            -> bool {

        let mut record = HitRecord::new(position_max);
        let mut hit_anything: bool = false;
        let mut closed_so_far: f64 = position_max;

        for i in 0..self.list.len() {
            if (&self.list[i]).hit(ray, position_min, closed_so_far, &mut record) {
                hit_anything = true;
                closed_so_far = record.position;
                hit_record.position = record.position;
                hit_record.point = record.point;
                hit_record.normal = record.normal;
            }
        }

        hit_anything
    }
}
