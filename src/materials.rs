use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3A;
use rand::RngExt;
use rand::rngs::ThreadRng;

use basis::OrthonormalBasis;
use events::{HitEvent, ScatterEvent};
use ggx::{ggx_distribution, ggx_geometry, ggx_sample_vndf};
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
    Emissive(Emissive),
    Isotropic(Isotropic),
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
    Emissive => Emissive,
    Isotropic => Isotropic,
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
            Material::Emissive(m) => m.scatter(ray, hit, rng),
            Material::Isotropic(m) => m.scatter(ray, hit, rng),
            Material::Plastic(m) => m.scatter(ray, hit, rng),
            Material::Reflective(m) => m.scatter(ray, hit, rng),
            Material::Refractive(m) => m.scatter(ray, hit, rng),
        }
    }

    pub fn emitted(&self, ray: &Ray, hit: &HitEvent) -> Vec3A {
        match self {
            Material::Emissive(m) => m.emitted(ray, hit),
            Material::Diffuse(_)
            | Material::Isotropic(_)
            | Material::Plastic(_)
            | Material::Reflective(_)
            | Material::Refractive(_) => Vec3A::ZERO,
        }
    }

    pub fn scattering_pdf(&self, ray: &Ray, hit: &HitEvent, scattered: &Ray) -> f32 {
        match self {
            Material::Diffuse(m) => m.scattering_pdf(ray, hit, scattered),
            Material::Emissive(_) => 0.0,
            Material::Plastic(m) => m.scattering_pdf(ray, hit, scattered),
            Material::Isotropic(_) => 1.0 / (4.0 * PI),
            Material::Reflective(m) => m.scattering_pdf(ray, hit, scattered),
            Material::Refractive(_) => 0.0,
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
    /// albedo is a Vec3A of the RGB values assigned to the material
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
        let scattered = Ray::new(event.point, ray.direction);
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
fn reflect(incident: Vec3A, normal: Vec3A) -> Vec3A {
    incident - 2.0 * incident.dot(normal) * normal
}

/// Compute the refract vector given the light vector, normal vector, and refractive_index
///
/// In dielectric materials some light is reflected and some refracted. We can use
/// Snell's Law to compute the direction of the refracted light.
///
/// For derivation see Section 10.4.3 in Mathematical and Computer Programming
/// Techniques for Computer Graphics by Peter Comininos.
fn refract(v: Vec3A, n: Vec3A, refractive_index: f32) -> Option<Vec3A> {
    let direction: f32 = v.dot(n);
    let discriminant: f32 =
        1.0 - refractive_index * refractive_index * (1.0 - direction * direction);

    if discriminant > 0.0 {
        Some(refractive_index * (v - n * direction) - n * discriminant.sqrt())
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
    let r0: f32 = ((1.0 - reference_index) / (1.0 + reference_index)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
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
    pub albedo: Vec3A,
    pub fuzz: f32,
}

impl Reflective {
    /// Create a new Reflective material for objects that reflect light only
    ///
    /// albedo is a Vec3A of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. fuzz accounts
    /// for the fuzziness of the reflections due to the size of the sphere.
    /// Generally, the larger the sphere, the fuzzier the reflections will be.
    pub fn new(albedo: Vec3A, fuzz: f32) -> Reflective {
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
    fn scatter(&self, ray: &Ray, event: &HitEvent, _rng: &mut ThreadRng) -> Option<ScatterEvent> {
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

        let offset_point = find_offset_point(event.point, forward_geometric_normal);

        if self.fuzz == 0.0 {
            let reflected = reflect(ray.direction, shading_normal);
            let specular_ray = Ray::new(offset_point, reflected);
            let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&shading_normal) };
            Some(ScatterEvent::new(specular_ray, self.albedo, pdf, true))
        } else {
            let wi = -ray.direction;
            let pdf = MaterialPDF::GGX { wi, normal: shading_normal, alpha: self.fuzz };
            let dummy_ray = Ray::new(offset_point, ray.direction);
            Some(ScatterEvent::new(dummy_ray, self.albedo, pdf, false))
        }
    }

    fn scattering_pdf(&self, ray: &Ray, event: &HitEvent, scattered: &Ray) -> f32 {
        if self.fuzz == 0.0 { return 0.0; }

        let wi = -ray.direction;
        let wo = scattered.direction;
        let n = event.shading_normal;

        let cos_i = n.dot(wi).max(0.0);
        let cos_o = n.dot(wo).max(0.0);

        if cos_i <= 0.0 || cos_o <= 0.0 { return 0.0; }

        let h = (wi + wo).normalize();
        let cos_h = n.dot(h).max(0.0);
        // f·cos_o = D·G/(4·cos_i·cos_o)·cos_o = D·G/(4·cos_i)
        // This makes the VNDF throughput weight G/G1(wi) = G1(wo) ≤ 1, preventing fireflies.
        ggx_distribution(cos_h, self.fuzz) * ggx_geometry(cos_i, cos_o, self.fuzz) / (4.0 * cos_i)
    }
}

#[derive(Clone)]
pub struct Refractive {
    pub refractive_index: f32,
    pub absorption: Vec3A,
}

impl Refractive {
    /// Create a new Refractive material for objects that both reflect and transmit light
    ///
    /// albedo is a Vec3A of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. index determines
    /// how much of the light is refracted when entering the material.
    /// fuzz accounts for the fuzziness of the reflections due to the size of the sphere.
    /// Generally, the larger the sphere, the fuzzier the reflections will be.
    pub fn new(index: f32, albedo: Vec3A) -> Refractive {
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
            Vec3A::ONE
        } else {
            self.absorption
        };

        let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };

        if rng.random::<f32>() < reflect_probability {
            let reflected: Vec3A = reflect(ray.direction, shading_normal);
            let offset_point = find_offset_point(event.point, forward_geometric_normal);
            let specular_ray = Ray::new(offset_point, reflected);
            Some(ScatterEvent::new(specular_ray, attenuation, pdf, true))
        } else {
            let offset_point = find_offset_point(event.point, -forward_geometric_normal);
            let specular_ray = Ray::new(offset_point, refracted.unwrap());
            Some(ScatterEvent::new(specular_ray, attenuation, pdf, true))
        }
    }
}

#[derive(Clone)]
pub struct Emissive {
    pub emit: Arc<Texture>,
}

impl Emissive {
    pub fn new(emit: Texture) -> Emissive {
        let emit = Arc::new(emit);
        Emissive { emit }
    }

    fn scatter(&self, _ray: &Ray, _event: &HitEvent, _rng: &mut ThreadRng) -> Option<ScatterEvent> {
        None
    }

    fn emitted(&self, ray: &Ray, hit: &HitEvent) -> Vec3A {
        if hit.shading_normal.dot(ray.direction) < 0.0 {
            self.emit.value(hit.u, hit.v, &hit.point)
        } else {
            Vec3A::ZERO
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

    fn scatter(&self, _ray: &Ray, event: &HitEvent, rng: &mut ThreadRng) -> Option<ScatterEvent> {
        let scattered = Ray::new(event.point, pick_sphere_point(rng));
        let attenuation = self.albedo.value(event.u, event.v, &event.point);
        let pdf = MaterialPDF::Uniform;
        Some(ScatterEvent::new(scattered, attenuation, pdf, false))
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
        let fresnel = schlick(cos_theta_i, self.ior);  // using schlick for ggx

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

        let offset_point = find_offset_point(event.point, forward_geometric_normal);

        // Probabilistically pick specular or diffuse based on Fresnel
        if rng.random::<f32>() < fresnel {
            // Specular path
            let alpha = self.roughness;
            let microfacet_normal = ggx_sample_vndf(shading_normal, -ray.direction, alpha, rng);
            let reflected = reflect(ray.direction, microfacet_normal);

            if shading_normal.dot(reflected) <= 0.0 {
                return None;
            }

            let specular_ray = Ray::new(offset_point, reflected);
            let pdf = MaterialPDF::GGX { wi: -ray.direction, normal: shading_normal, alpha };

            Some(ScatterEvent::new(specular_ray, Vec3A::ONE, pdf, true))
        } else {
            // Diffuse path
            // even though ray.direction is given, a new ray with offset is generated in the integrator
            let scattered = Ray::new(offset_point, ray.direction);
            let attenuation = self.albedo.value(event.u, event.v, &event.point) * (1.0 - fresnel);
            let pdf = MaterialPDF::Cosine { uvw: OrthonormalBasis::new(&event.shading_normal) };
            Some(ScatterEvent::new(scattered, attenuation, pdf, false))
        }
    }

    fn scattering_pdf(&self, wo: &Ray, event: &HitEvent, wi: &Ray) -> f32 {
        let n = event.shading_normal;

        let cos_o = n.dot(wi.direction).max(0.0);
        if cos_o <= 0.0 {
            return 0.0;
        }

        let cos_theta_i = (-wo.direction).dot(n).max(0.0);
        let fresnel = schlick(cos_theta_i, self.ior);

        let diffuse_pdf = cos_o / PI;

        let alpha = self.roughness * self.roughness;

        let specular_pdf = {
            let wi_local = -wo.direction;
            let wo_local = wi.direction;

            let h = (wi_local + wo_local).normalize();
            let cos_h = n.dot(h).max(0.0);
            let cos_i = n.dot(wi_local).max(0.0);
            let cos_ol = n.dot(wo_local).max(0.0);

            ggx_distribution(cos_h, alpha) * ggx_geometry(cos_i, cos_ol, alpha) / (4.0 * cos_ol)
        };

        // mixture pdf
        fresnel * specular_pdf + (1.0 - fresnel) * diffuse_pdf
    }
}