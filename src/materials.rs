use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3;
use rand::RngExt;
use rand::rngs::ThreadRng;

use basis::OrthonormalBasis;
use events::{HitEvent, ScatterEvent};
use integrator::pick_sphere_point;
use pdf::MaterialPDF;
use ray::{find_offset_point, Ray};
use texture::Texture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialId(pub u32);

impl MaterialId {
    pub fn new(index: u32) -> MaterialId {
        MaterialId(index)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone)]
pub enum Material {
    Diffuse(Diffuse),
    Isotropic(Isotropic),
    Light(Light),
    Plastic(Plastic),
    Reflective(Reflective),
    Refractive(Refractive),
}

macro_rules! impl_from_material {
    ($($t:ty => $v:ident),*) => {
        $(
            impl From<$t> for Material {
                fn from(m: $t) -> Self {
                    Material::$v(m)
                }
            }
        )*
    };
}

impl_from_material!(
    Diffuse => Diffuse,
    Isotropic => Isotropic,
    Light => Light,
    Plastic => Plastic,
    Reflective => Reflective,
    Refractive => Refractive
);

#[macro_export]
macro_rules! mat {
    ($vec:expr, $material:expr) => {{
        let id = $crate::materials::MaterialId::new($vec.len() as u32);
        $vec.push($material.into());
        id
    }};
}

impl Material {
    pub fn scatter(&self, ray: &Ray, hit: &HitEvent, rng: &mut ThreadRng) -> Option<ScatterEvent> {
        match self {
            Material::Diffuse(m) => m.scatter(ray, hit, rng),
            Material::Isotropic(m) => m.scatter(ray, hit, rng),
            Material::Light(m) => m.scatter(ray, hit, rng),
            Material::Plastic(m) => m.scatter(ray, hit, rng),
            Material::Reflective(m) => m.scatter(ray, hit, rng),
            Material::Refractive(m) => m.scatter(ray, hit, rng),
        }
    }

    pub fn emitted(&self, ray: &Ray, hit: &HitEvent) -> Vec3 {
        match self {
            Material::Light(m) => m.emitted(ray, hit),
            Material::Diffuse(_)
            | Material::Isotropic(_)
            | Material::Plastic(_)
            | Material::Reflective(_)
            | Material::Refractive(_) => Vec3::ZERO,
        }
    }

    pub fn scattering_pdf(&self, ray: &Ray, hit: &HitEvent, scattered: &Ray) -> f32 {
        match self {
            Material::Diffuse(m) => m.scattering_pdf(ray, hit, scattered),
            Material::Plastic(m) => m.scattering_pdf(ray, hit, scattered),
            Material::Isotropic(_) => 1.0,
            Material::Reflective(_) => 0.0,
            Material::Refractive(_) => 0.0,
            Material::Light(_) => 0.0,
        }
    }
}

#[derive(Clone)]
pub struct Empty {}

impl Empty {
    pub fn new() -> Empty {
        Empty {}
    }
}

#[derive(Clone)]
pub struct Diffuse {
    pub albedo: Arc<Texture>,
    alpha: f32,
    beta: f32,
}

impl Diffuse {
    /// Create a new Diffuse material with the given albedo and sigma (roughness)
    ///
    /// albedo is a Vec3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0.
    pub fn new(albedo: Texture, sigma: f32) -> Diffuse {
        let albedo = Arc::new(albedo);
        let constant = PI + sigma * (3.0 * PI - 4.0) / 6.0;
        let alpha = 1.0 / constant;
        let beta = sigma / constant;

        Diffuse { albedo,
                  alpha,
                  beta }
    }

    fn scatter(&self, ray: &Ray, event: &HitEvent, _rng: &mut ThreadRng) -> Option<ScatterEvent> {
        // ray.direction is passed here because the integrator generates
        // an offset point itself for diffuse materials
        let scattered = Ray::new(event.point, ray.direction, ray.time);
        let attenuation = self.albedo.value(event.u, event.v, &event.point);
        let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };
        Some(ScatterEvent::new(scattered, attenuation, pdf, false))
    }

    /// Reflect light according to the Oren-Nayar model
    ///
    /// This method uses the improved Oren-Nayar model as implemented in Cycles:
    ///
    /// Yasuhiro Fujii: A tiny improvement of Oren-Nayar reflectance model
    /// https://mimosa-pudica.net/improved-oren-nayar.html
    ///
    /// https://developer.blender.org/diffusion/C/browse/master/src/kernel/closure/bsdf_oren_nayar.h
    fn scattering_pdf(&self, wo: &Ray, event: &HitEvent, wi: &Ray) -> f32 {
        let l = wi.direction;
        let v = wo.direction;
        let n = event.shading_normal;

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
fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
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
    let r0: f32 = ((1.0 - reference_index) / (1.0 + reference_index)).powf(2.0);
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

/// Fresnel equations are used to compute physically accurate transmission
/// For more information see the following resources:
/// https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/reflection-refraction-fresnel.html
fn fresnel_coefficient(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
    // cos_theta_i is the incident ray. clamped for safety
    let cos_theta_i = cos_theta_i.max(0.0);

    // Snell's law calculation
    let eta = eta_i / eta_t; // eta is the refraction index from the scene
    let sin2_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
    let sin2_theta_t = eta * eta * sin2_theta_i;

    // Total internal reflection
    if sin2_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).sqrt();

    let r_parallel = ((eta_t * cos_theta_i - eta_i * cos_theta_t)
        / (eta_t * cos_theta_i + eta_i * cos_theta_t)).powi(2);

    let r_perp = ((eta_i * cos_theta_i - eta_t * cos_theta_t)
        / (eta_i * cos_theta_i + eta_t * cos_theta_t)).powi(2);

    0.5 * (r_parallel + r_perp)
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
        Reflective { albedo, fuzz }
    }
}

impl Reflective {
    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the event.point
    /// plus the event.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. A fuzziness
    /// factor is also added in to account for the reflection fuzz due to
    /// the size of the sphere. The target minus the event.point is used
    /// to determine the ray that is being reflected from the surface of the material.
    fn scatter(&self, ray: &Ray, event: &HitEvent, rng: &mut ThreadRng) -> Option<ScatterEvent> {
        let forward_geometric_normal = if ray.direction.dot(event.geometric_normal) < 0.0 {
            event.geometric_normal
        } else {
            -event.geometric_normal
        };

        let shading_normal = if event.shading_normal.dot(forward_geometric_normal) < 0.0 {
            -event.shading_normal
        } else {
            event.shading_normal
        };

        let reflected: Vec3 = reflect(ray.direction, shading_normal);
        let scattered = reflected + self.fuzz * pick_sphere_point(rng);
        let offset_point = find_offset_point(event.point, forward_geometric_normal);
        let specular_ray = Ray::new(offset_point, scattered, ray.time);

        let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };
        Some(ScatterEvent::new(specular_ray, self.albedo, pdf, true))
    }
}

#[derive(Clone)]
pub struct Refractive {
    pub refractive_index: f32,
    pub absorption: Vec3,
}

impl Refractive {
    /// Create a new Refractive material for objects that both reflect and transmit light
    ///
    /// albedo is a Vec3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. index determines
    /// how much of the light is refracted when entering the material.
    /// fuzz accounts for the fuzziness of the reflections due to the size of the sphere.
    /// Generally, the larger the sphere, the fuzzier the reflections will be.
    pub fn new(index: f32, albedo: Vec3) -> Refractive {
        Refractive { refractive_index: index, absorption: albedo }
    }

    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the event.point
    /// plus the event.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. A fuzziness
    /// factor is also added in to account for the reflection fuzz due to
    /// the size of the sphere. The target minus the event.point is used
    /// to determine the ray that is being reflected from the surface of the material.
    ///
    /// See Peter Shirley's Ray Tracing in One Weekend for an overview of refractive
    /// scattering and Section 10.3.2 in Mathematical and Computer Programming
    /// Techniques for Computer Graphics by Peter Comininos.
    fn scatter(&self, ray: &Ray, event: &HitEvent, rng: &mut ThreadRng) -> Option<ScatterEvent> {
        let geometric_incident: f32 = ray.direction.dot(event.geometric_normal);
        let entering = geometric_incident < 0.0;

        // Pick a forward-facing geometric normal (points back toward the ray origin).
        let forward_geometric_normal = if entering {
            event.geometric_normal
        } else {
            -event.geometric_normal
        };

       // For the shading side of refraction (Fresnel/Snell), use the shading normal,
        // but make sure it agrees with the forward-facing geometric normal.
        let shading_normal = if event.shading_normal.dot(forward_geometric_normal) < 0.0 {
            -event.shading_normal
        } else {
            event.shading_normal
        };

        let (eta_i, eta_t) = if entering {
            (1.0, self.refractive_index)
        } else {
            (self.refractive_index, 1.0)
        };

        let cos_theta_i = -ray.direction.dot(shading_normal); // positive

        let refracted = refract(ray.direction, shading_normal, eta_i / eta_t);

        let reflect_probability = match refracted {
            Some(_) => fresnel_coefficient(cos_theta_i, eta_i, eta_t),
            None => 1.0,
        };

        let attenuation = if entering {
            Vec3::ONE
        } else {
            self.absorption
        };

        let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };

        if rng.random::<f32>() < reflect_probability {
            let reflected: Vec3 = reflect(ray.direction, shading_normal);
            let offset_point = find_offset_point(event.point, forward_geometric_normal);
            let specular_ray = Ray::new(offset_point, reflected, ray.time);
            Some(ScatterEvent::new(specular_ray, attenuation, pdf, true))
        } else {
            let offset_point = find_offset_point(event.point, -forward_geometric_normal);
            let specular_ray = Ray::new(offset_point, refracted.unwrap(), ray.time);
            Some(ScatterEvent::new(specular_ray, attenuation, pdf, true))
        }
    }
}

#[derive(Clone)]
pub struct Light {
    pub emit: Arc<Texture>,
}

impl Light {
    pub fn new(emit: Texture) -> Light {
        let emit = Arc::new(emit);
        Light { emit }
    }

    fn scatter(&self, _ray: &Ray, _event: &HitEvent, _rng: &mut ThreadRng) -> Option<ScatterEvent> {
        None
    }

    fn emitted(&self, ray: &Ray, hit: &HitEvent) -> Vec3 {
        if hit.shading_normal.dot(ray.direction) < 0.0 {
            self.emit.value(hit.u, hit.v, &hit.point)
        } else {
            Vec3::ZERO
        }
    }
}

#[derive(Clone)]
pub struct Isotropic {
    pub albedo: Arc<Texture>,
}

impl Isotropic {
    pub fn new(albedo: Texture) -> Isotropic {
        let albedo = Arc::new(albedo);
        Isotropic { albedo }
    }

    fn scatter(&self, ray: &Ray, event: &HitEvent, rng: &mut ThreadRng) -> Option<ScatterEvent> {
        let scattered = Ray::new(event.point, pick_sphere_point(rng), ray.time);
        let attenuation = self.albedo.value(event.u, event.v, &event.point);
        let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };
        Some(ScatterEvent::new(scattered, attenuation, pdf, true))
    }
}

#[derive(Clone)]
pub struct Plastic {
    pub albedo: Arc<Texture>,
    pub roughness: f32,  // 0 = mirror smooth, 1 = very rough
    pub ior: f32,        // typically 1.5 for plastic/ceramic
}

impl Plastic {
    pub fn new(albedo: Texture, roughness: f32, ior: f32) -> Plastic {
        let albedo = Arc::new(albedo);
        Plastic { albedo, roughness: roughness.max(0.0), ior }
    }

    fn scatter(&self, ray: &Ray, event: &HitEvent, rng: &mut ThreadRng) -> Option<ScatterEvent> {
        let cos_theta_i = (-ray.direction).dot(event.shading_normal).max(0.0);
        let fresnel = fresnel_coefficient(cos_theta_i, 1.0, self.ior);

        // Probabilistically pick specular or diffuse based on Fresnel
        if rng.random::<f32>() < fresnel {
            // Specular path
            let reflected = reflect(ray.direction, event.shading_normal);
            let perturbed = reflected + self.roughness * pick_sphere_point(rng);
            let specular_ray = Ray::new(event.point, perturbed, ray.time);
            let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };
            Some(ScatterEvent::new(specular_ray, Vec3::ONE, pdf, true))
        } else {
            // Diffuse path
            let scattered = Ray::new(event.point, ray.direction, ray.time);
            let attenuation = self.albedo.value(event.u, event.v, &event.point);
            let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };
            Some(ScatterEvent::new(scattered, attenuation, pdf, false))
        }
    }

    fn scattering_pdf(&self, _wo: &Ray, event: &HitEvent, wi: &Ray) -> f32 {
        let cosine = event.shading_normal.dot(wi.direction.normalize()).max(0.0);
        cosine / std::f32::consts::PI
    }
}