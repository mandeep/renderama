use hitable::HitRecord;
use nalgebra::core::Vector3;
use ray::{random_point_in_sphere, Ray};


pub trait Material: Send + Sync {
    fn box_clone(&self) -> Box<Material>;

    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vector3<f64>, Ray)>;
}


#[derive(Clone)]
pub struct Lambertian {
    pub albedo: Vector3<f64>
}



impl Lambertian {
    pub fn new(albedo: Vector3<f64>) -> Lambertian {
        Lambertian { albedo: albedo }
    }
}


impl Material for Lambertian {
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let target: Vector3<f64> = record.point + record.normal + random_point_in_sphere();
        Some((self.albedo, Ray::new(record.point, target - record.point)))
    }
}


#[derive(Clone)]
pub struct Metal {
    pub albedo: Vector3<f64>,
    pub fuzz: f64
}


impl Metal {
    pub fn new(albedo: Vector3<f64>, fuzz: f64) -> Metal {
        Metal { albedo: albedo, fuzz: fuzz }
    }

    fn reflect(&self, v: &Vector3<f64>, n: &Vector3<f64>) -> Vector3<f64> {
        v - 2.0 * v.dot(&n) * n
    }
}


impl Material for Metal {
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let reflected: Vector3<f64> = self.reflect(&ray.direction.normalize(), &record.normal);
        let scattered = Ray::new(record.point, reflected + self.fuzz * random_point_in_sphere());

        if scattered.direction.dot(&record.normal) > 0.0 {
            return Some((self.albedo, scattered));
        }
        None
    }
}
