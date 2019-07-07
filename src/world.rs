use nalgebra::core::Vector3;

use aabb::{AABB, surrounding_box};
use hitable::{Hitable, HitRecord};
use materials::Diffuse;
use ray::Ray;
use texture::ConstantTexture;


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
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let mut record = HitRecord::new(0.0, 0.0, 0.0,
                                        Vector3::zeros(),
                                        Vector3::zeros(),
                                        Box::new(Diffuse::new(ConstantTexture::new(0.0, 0.0, 0.0))));
        let mut hit_anything: bool = false;
        let mut closed_so_far: f32 = position_max;

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

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if self.objects.len() > 0 {
            let temporary_box = AABB::new(Vector3::zeros(), Vector3::zeros());
            if let Some(_) = self.objects.first().unwrap().bounding_box(t0, t1) {
                let mut accumulated_box = AABB::new(Vector3::zeros(), Vector3::zeros());
                for _ in 0..self.objects.len() {
                    if let Some(_) = self.objects.first().unwrap().bounding_box(t0, t1) {
                        accumulated_box = surrounding_box(&accumulated_box, &temporary_box);
                    } else {
                        return None;
                    }
                }
                return Some(accumulated_box);
            } else {
                return None;
            }
        }
        None
    }
}
