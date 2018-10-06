use hitable::{Hitable, HitRecord};
use nalgebra::core::Vector3;
use ray::Ray;


pub struct World {
    pub objects: Vec<Box<dyn Hitable>>
}


impl World {
    pub fn new() -> World {
        World { objects: Vec::new() }
    }

    pub fn add(&mut self, object: Box<dyn Hitable>) {
        self.objects.push(object);
    }
}


impl Hitable for World {
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64) -> Option<HitRecord> {
        let mut record = HitRecord::new(0.0, Vector3::zeros(), Vector3::zeros());
        let mut hit_anything: bool = false;
        let mut closed_so_far: f64 = position_max;

        for object in &self.objects {
            match object.hit(ray, position_min, closed_so_far) {
                None => (),
                Some(hit_record) => {
                    hit_anything = true;
                    closed_so_far = hit_record.parameter;
                    record = hit_record;
                }
            }
        }

        if hit_anything { return Some(record); } else { return None; }
    }
}
