use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3;
use rand::rngs::ThreadRng;

use basis::OrthonormalBasis;
use hitable::HitRecord;
use integrator::pick_sphere_point;
use pdf::PDF;
use ray::Ray;
use texture::Texture;

pub struct ScatterRecord<'a> {
    pub specular_ray: Ray,
    pub attenuation: Vec3,
    pub pdf: PDF<'a>,
    pub specular: bool,
}

impl<'a> ScatterRecord<'a> {
    pub fn new(specular_ray: Ray,
               attenuation: Vec3,
               pdf: PDF<'a>,
               specular: bool)
               -> ScatterRecord<'a> {
        ScatterRecord { specular_ray,
                        attenuation,
                        pdf,
                        specular }
    }
}

/// The Material trait is responsible for giving a color to the object implementing the trait
pub trait Material: Send + Sync {
    fn scatter(&self,
               _ray: &Ray,
               _record: &HitRecord,
               _rng: &mut ThreadRng)
               -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, _ray: &Ray, _hit: &HitRecord) -> Vec3 {
        Vec3::zero()
    }

    fn scattering_pdf(&self, _ray: &Ray, _record: &HitRecord, _scattered: &Ray) -> f32 {
        1.0
    }
}

#[derive(Clone)]
pub struct Empty {}

impl Empty {
    pub fn new() -> Empty {
        Empty {}
    }
}
impl Material for Empty {
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord, _rng: &mut ThreadRng) -> Option<ScatterRecord> {
        None
    }
}

#[derive(Clone)]
pub struct Diffuse {
    pub albedo: Arc<dyn Texture>,
    alpha: f32,
    beta: f32,
}

impl Diffuse {
    /// Create a new Diffuse material with the given albedo
    ///
    /// albedo is a Vec3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0.
    pub fn new<T: Texture + 'static>(albedo: T, sigma: f32) -> Diffuse {
        let albedo = Arc::new(albedo);

        let constant = PI + sigma * (3.0 * PI - 4.0) / 6.0;
        let alpha = 1.0 / constant;
        let beta = sigma / constant;

        Diffuse { albedo,
                  alpha,
                  beta }
    }
}

impl Material for Diffuse {
    /// Scatter a new ray from the hit point of the surface
    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               _rng: &mut ThreadRng)
               -> Option<ScatterRecord> {
        let scattered = Ray::new(record.point, ray.direction, ray.time);
        let attenuation = self.albedo.value(record.u, record.v, &record.point);
        let pdf = PDF::CosinePDF { uvw: OrthonormalBasis::new(&record.shading_normal) };
        Some(ScatterRecord::new(scattered, attenuation, pdf, false))
    }

    /// Reflect light according to the Oren-Nayar model
    ///
    /// This method uses the improved Oren-Nayar model as implemented in Cycles:
    ///
    /// Yasuhiro Fujii: A tiny improvement of Oren-Nayar reflectance model
    /// https://mimosa-pudica.net/improved-oren-nayar.html
    ///
    /// https://developer.blender.org/diffusion/C/browse/master/src/kernel/closure/bsdf_oren_nayar.h
    fn scattering_pdf(&self, wo: &Ray, record: &HitRecord, wi: &Ray) -> f32 {
        let l = wi.direction;
        let v = wo.direction;
        let n = record.shading_normal;

        let nl = n.dot(l).max(0.0);
        let nv = n.dot(v).max(0.0);
        let lv = l.dot(v);

        let s = lv - nl * nv;
        let t = if s > 0.0 { nl.max(nv) } else { 1.0 };

        nl * (self.alpha + self.beta * s / t)
    }
}

/// Compute the reflect vector given the light vector and the normal vector of the surface
///
/// The law of reflection tells us that the angle between the indicent ray
/// and the normal vector of the hit point is equal to the angle between
/// the reflected ray and the normal vector of the hit point.
///
/// For derivation see Section 10.4.2 in Mathematical and Computer Programming
/// Techniques for Computer Graphics by Peter Comininos.
fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

/// Compute the refract vector given the light vector, normal vector, and refractive_index
///
/// In dielectric materials some light is reflected and some refracted. We can use
/// Snell's Law to compute the direction of the refracted light.
///
/// For derivation see Section 10.4.3 in Mathematical and Computer Programming
/// Techniques for Computer Graphics by Peter Comininos.
fn refract(v: Vec3, n: Vec3, refractive_index: f32) -> Option<Vec3> {
    let uv: Vec3 = v.normalize();
    let direction: f32 = uv.dot(n);
    let discriminant: f32 =
        1.0 - refractive_index * refractive_index * (1.0 - direction * direction);

    if discriminant > 0.0 {
        Some(refractive_index * (uv - n * direction) - n * discriminant.sqrt())
    } else {
        None
    }
}

/// Determine the reflectivity amount based on the angle
///
/// In objects like glass, reflectivity varies with the view angle. Schlick's
/// approximation is used to compute the Fresnel factor in the specular reflection.
///
/// For derivation see Section 10.10.3 in Mathematical and Computer Programming
/// Techniques for Computer Graphics by Peter Comininos and
/// https://en.wikipedia.org/wiki/Schlick's_approximation.
fn schlick(cosine: f32, reference_index: f32) -> f32 {
    let r0: f32 = (1.0 - reference_index) / (1.0 + reference_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

#[derive(Clone)]
pub struct Reflective {
    pub albedo: Vec3,
    pub fuzz: f32,
}

impl Reflective {
    /// Create a new Reflective material for objects that reflect light only
    ///
    /// albedo is a Vec3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. fuzz accounts
    /// for the fuzziness of the reflections due to the size of the sphere.
    /// Generally, the larger the sphere, the fuzzier the reflections will be.
    pub fn new(albedo: Vec3, fuzz: f32) -> Reflective {
        Reflective { albedo: albedo,
                     fuzz: fuzz }
    }
}

impl Material for Reflective {
    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the record.point
    /// plus the record.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. A fuzziness
    /// factor is also added in to account for the reflection fuzz due to
    /// the size of the sphere. The target minus the record.point is used
    /// to determine the ray that is being reflected from the surface of the material.
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut ThreadRng) -> Option<ScatterRecord> {
        let reflected: Vec3 = reflect(ray.direction, record.shading_normal);
        let specular_ray = Ray::new(record.point,
                                    reflected + self.fuzz * pick_sphere_point(rng),
                                    ray.time);
        let pdf = PDF::CosinePDF { uvw: OrthonormalBasis::new(&record.shading_normal) };
        Some(ScatterRecord::new(specular_ray, self.albedo, pdf, true))
    }
}

#[derive(Clone)]
pub struct Refractive {
    pub refractive_index: f32,
}

impl Refractive {
    /// Create a new Refractive material for objects that both reflect and transmit light
    ///
    /// albedo is a Vec3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. index determines
    /// how much of the light is refracted when entering the material.
    /// fuzz accounts for the fuzziness of the reflections due to the size of the sphere.
    /// Generally, the larger the sphere, the fuzzier the reflections will be.
    pub fn new(index: f32) -> Refractive {
        Refractive { refractive_index: index }
    }
}

impl Material for Refractive {
    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the record.point
    /// plus the record.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. A fuzziness
    /// factor is also added in to account for the reflection fuzz due to
    /// the size of the sphere. The target minus the record.point is used
    /// to determine the ray that is being reflected from the surface of the material.
    ///
    /// See Peter Shirley's Ray Tracing in One Weekend for an overview of refractive
    /// scattering and Section 10.3.2 in Mathematical and Computer Programming
    /// Techniques for Computer Graphics by Peter Comininos.
    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               _rng: &mut ThreadRng)
               -> Option<ScatterRecord> {
        let reflected: Vec3 = reflect(ray.direction, record.shading_normal);
        let incident: f32 = ray.direction.dot(record.shading_normal);

        let (outward_normal, refractive_index, cosine) = if incident > 0.0 {
            (-record.shading_normal,
             self.refractive_index,
             self.refractive_index * ray.direction.dot(record.shading_normal)
             / ray.direction.length())
        } else {
            (record.shading_normal,
             1.0 / self.refractive_index,
             -ray.direction.dot(record.shading_normal) / ray.direction.length())
        };

        let refracted = refract(ray.direction, outward_normal, refractive_index);
        let reflect_probability = match refracted {
            Some(_) => schlick(cosine, self.refractive_index),
            None => 1.0,
        };

        let attenuation = Vec3::new(1.0, 1.0, 1.0);
        let pdf = PDF::CosinePDF { uvw: OrthonormalBasis::new(&record.shading_normal) };

        if rand::random::<f32>() < reflect_probability {
            let specular_ray = Ray::new(record.point, reflected, ray.time);
            Some(ScatterRecord::new(specular_ray, attenuation, pdf, true))
        } else {
            let specular_ray = Ray::new(record.point, refracted.unwrap(), ray.time);
            Some(ScatterRecord::new(specular_ray, attenuation, pdf, true))
        }
    }
}

#[derive(Clone)]
pub struct Light {
    pub emit: Arc<dyn Texture>,
}

impl Light {
    pub fn new<T: Texture + 'static>(emit: T) -> Light {
        let emit = Arc::new(emit);
        Light { emit: emit }
    }
}

impl Material for Light {
    fn scatter(&self,
               _ray: &Ray,
               _record: &HitRecord,
               _rng: &mut ThreadRng)
               -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, ray: &Ray, hit: &HitRecord) -> Vec3 {
        if hit.shading_normal.dot(ray.direction) < 0.0 {
            self.emit.value(hit.u, hit.v, &hit.point)
        } else {
            Vec3::zero()
        }
    }
}

#[derive(Clone)]
pub struct Isotropic {
    pub albedo: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn new<T: Texture + 'static>(albedo: T) -> Isotropic {
        let albedo = Arc::new(albedo);
        Isotropic { albedo }
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut ThreadRng) -> Option<ScatterRecord> {
        let scattered = Ray::new(record.point, pick_sphere_point(rng), ray.time);
        let attenuation = self.albedo.value(record.u, record.v, &record.point);
        let pdf = PDF::CosinePDF { uvw: OrthonormalBasis::new(&record.shading_normal) };
        Some(ScatterRecord::new(scattered, attenuation, pdf, true))
    }
}
