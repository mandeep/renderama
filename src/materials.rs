use nalgebra::core::Vector3;
use rand;

use hitable::HitRecord;
use ray::{pick_sphere_point, Ray};


/// The Material trait is responsible for giving a color to the object implementing the trait
pub trait Material: Send + Sync {
    fn box_clone(&self) -> Box<Material>;

    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::ThreadRng) -> Option<(Vector3<f64>, Ray)>;
}


#[derive(Clone)]
pub struct Diffuse {
    pub albedo: Vector3<f64>
}



impl Diffuse {
    /// Create a new Diffuse material with the given albedo
    ///
    /// albedo is a Vector3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0.
    pub fn new(albedo: Vector3<f64>) -> Diffuse {
        Diffuse { albedo: albedo }
    }
}


impl Material for Diffuse {
    /// Create a new Diffuse Material on the heap
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }
a
    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the record.point
    /// plus the record.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. The target minus
    /// the record.point is used to determine the ray that is being reflected
    /// from the surface of the material.
    fn scatter(&self,
               _ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::ThreadRng) -> Option<(Vector3<f64>, Ray)> {

        let target: Vector3<f64> = record.point + record.normal + pick_sphere_point(rng);
        Some((self.albedo, Ray::new(record.point, target - record.point)))
    }
}


fn reflect(v: &Vector3<f64>, n: &Vector3<f64>) -> Vector3<f64> {
    v - 2.0 * v.dot(&n) * n
}


fn refract(v: &Vector3<f64>, n: &Vector3<f64>, refractive_index: f64) -> Option<Vector3<f64>> {
    let uv: Vector3<f64> = v.normalize();
    let direction: f64 = uv.dot(&n);
    let discriminant: f64 = 1.0 - refractive_index * refractive_index * (1.0 - direction * direction);

    if discriminant > 0.0 {
        return Some(refractive_index * (uv - n * direction) - n * discriminant.sqrt());
    }
    None
}


fn schlick(cosine: f64, reference_index: f64) -> f64 {
    let r0: f64 = (1.0 - reference_index) / (1.0 + reference_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}


#[derive(Clone)]
pub struct Reflective {
    pub albedo: Vector3<f64>,
    pub fuzz: f64
}


impl Reflective {
    pub fn new(albedo: Vector3<f64>, fuzz: f64) -> Reflective {
        Reflective { albedo: albedo, fuzz: fuzz }
    }

}


impl Material for Reflective {
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::ThreadRng) -> Option<(Vector3<f64>, Ray)> {

        let reflected: Vector3<f64> = reflect(&ray.direction.normalize(), &record.normal);
        let scattered = Ray::new(record.point, reflected + self.fuzz * pick_sphere_point(rng));

        if scattered.direction.dot(&record.normal) > 0.0 {
            return Some((self.albedo, scattered));
        }
        None
    }
}


#[derive(Clone)]
pub struct Refractive {
    pub albedo: Vector3<f64>,
    pub refractive_index: f64,
    pub fuzz: f64
}


impl Refractive {
    pub fn new(albedo: Vector3<f64>, index: f64, fuzz: f64) -> Refractive {
        Refractive { albedo: albedo, refractive_index: index, fuzz: fuzz }
    }
}


impl Material for Refractive {
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::ThreadRng) -> Option<(Vector3<f64>, Ray)> {

        let reflected: Vector3<f64> = reflect(&ray.direction.normalize(), &record.normal);
        let incident: f64 = ray.direction.dot(&record.normal);

        let (outward_normal, refractive_index, cosine) = if incident > 0.0 {
               (-record.normal,
                self.refractive_index,
                self.refractive_index * ray.direction.dot(&record.normal) / ray.direction.norm())
            } else {
               (record.normal,
                1.0 / self.refractive_index,
                -ray.direction.dot(&record.normal) / ray.direction.norm())

            };

        let refracted = refract(&ray.direction, &outward_normal, refractive_index);
        let reflect_probability = match refracted {
            Some(_) => schlick(cosine, self.refractive_index),
            None => 1.0
        };

        if rand::random::<f64>() < reflect_probability {
            return Some((self.albedo,
                         Ray::new(record.point, reflected + self.fuzz * pick_sphere_point(rng))
                         ));
        } else {
            return Some((self.albedo, Ray::new(record.point,
                                               refracted.unwrap() +
                                                   self.fuzz * pick_sphere_point(rng))
                         ));
        }
    }
}
