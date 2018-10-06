use hitable::HitRecord;
use nalgebra::core::Vector3;
use rand;
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

    fn scatter(&self, _ray: &Ray, record: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let target: Vector3<f64> = record.point + record.normal + random_point_in_sphere();
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
pub struct Metal {
    pub albedo: Vector3<f64>,
    pub fuzz: f64
}


impl Metal {
    pub fn new(albedo: Vector3<f64>, fuzz: f64) -> Metal {
        Metal { albedo: albedo, fuzz: fuzz }
    }

}


impl Material for Metal {
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let reflected: Vector3<f64> = reflect(&ray.direction.normalize(), &record.normal);
        let scattered = Ray::new(record.point, reflected + self.fuzz * random_point_in_sphere());

        if scattered.direction.dot(&record.normal) > 0.0 {
            return Some((self.albedo, scattered));
        }
        None
    }
}


#[derive(Clone)]
pub struct Dielectric {
    pub refractive_index: f64
}


impl Dielectric {
    pub fn new(index: f64) -> Dielectric {
        Dielectric { refractive_index: index }
    }
}


impl Material for Dielectric {
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let reflected: Vector3<f64> = reflect(&ray.direction, &record.normal);
        let attenuation = Vector3::new(1.0, 1.0, 1.0);
        let incident: f64 = ray.direction.dot(&record.normal);

        let outward_normal = if incident > 0.0 {
               -record.normal
            } else {
                record.normal
            };

        let refractive_index = if incident > 0.0 {
                self.refractive_index
            } else {
                1.0 / self.refractive_index
            };

        let cosine = if incident > 0.0 {
                self.refractive_index * ray.direction.dot(&record.normal) / ray.direction.norm()
            } else {
                -ray.direction.dot(&record.normal) / ray.direction.norm()
            };

        let refracted = refract(&ray.direction, &outward_normal, refractive_index);
        let reflect_probability = match refracted {
            Some(_) => schlick(cosine, self.refractive_index),
            None => 1.0
        };

        if rand::random::<f64>() < reflect_probability {
            return Some((attenuation, Ray::new(record.point, reflected)));
        } else {
            return Some((attenuation, Ray::new(record.point, refracted.unwrap())));
        }
    }
}
