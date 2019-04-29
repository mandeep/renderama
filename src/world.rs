use nalgebra::core::Vector3;

use hitable::{Hitable, HitRecord};
use materials::Diffuse;
use ray::Ray;


/// The World struct holds all of the objects in the scene
pub struct World {
    pub objects: Vec<Box<dyn Hitable>>,
}


impl World {
    /// Create a new World to hold all of the objects in the scene
    pub fn new() -> World {
        World { objects: Vec::new() }
    }

    /// Add objects to the instantiated world
    ///
    /// We use a 'static lifetime so that we can Box
    /// object inside the function rather than having to
    /// pass object as a Boxed object as an input parameter.
    pub fn add<H: Hitable + 'static>(&mut self, object: H) {
        let object = Box::new(object);
        self.objects.push(object);
    }
}


impl Hitable for World {
    /// Determine if the given ray has hit any of the objects in the world
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64) -> Option<HitRecord> {
        let mut record = HitRecord::new(0.0,
                                        Vector3::zeros(),
                                        Vector3::zeros(),
                                        Box::new(Diffuse::new(Vector3::zeros())));
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

        return if hit_anything { Some(record) } else { None }
    }
}
