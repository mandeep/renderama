use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3A;
use rand::RngExt;
use rand_pcg::Pcg64Mcg;

use basis::OrthonormalBasis;
use results::{HitResult, ScatterResult};
use ggx::{ggx_distribution, ggx_geometry, ggx_sample_vndf};
use pdf::PDF;
use ray::{find_offset_point, Ray};
use sampling::pick_sphere_point;
use texture::Texture;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// MaterialId is used to index into the materials Vec that is instantiated
/// at scene creation.
///
/// Keeping track of the index instead of the actual material
/// saves from allocating memory unnecessarily.
pub struct MaterialId(pub u32);

impl MaterialId {
    /// Create a new MaterialId with material at index
    pub fn new(index: u32) -> MaterialId {
        MaterialId(index)
    }

    /// Retrieve the MaterialId as a usize for indexing purposes
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone)]
/// Material enum allows for quick and easy dispatch of materials
///
/// Used in the integrator to sample materials as well as in scene setup
/// to create the materials Vec.
pub enum Material {
    Diffuse(Diffuse),
    Emissive(Emissive),
    Plastic(Plastic),
    Reflective(Reflective),
    Refractive(Refractive),
    Volumetric(Volumetric),
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
    Plastic => Plastic,
    Reflective => Reflective,
    Refractive => Refractive,
    Volumetric => Volumetric
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
    /// Generate the ScatterResult that tells the integrator how the material responds to sampling
    pub fn generate_response(&self, ray: &Ray, hit: &HitResult, rng: &mut Pcg64Mcg) -> Option<ScatterResult> {
        match self {
            Material::Diffuse(m) => m.generate_response(ray, hit, rng),
            Material::Emissive(m) => m.generate_response(ray, hit, rng),
            Material::Plastic(m) => m.generate_response(ray, hit, rng),
            Material::Reflective(m) => m.generate_response(ray, hit, rng),
            Material::Refractive(m) => m.generate_response(ray, hit, rng),
            Material::Volumetric(m) => m.generate_response(ray, hit, rng),
        }
    }

    /// Evaluate the albedo of the emissive material
    pub fn evaluate_emission(&self, ray: &Ray, hit: &HitResult) -> Vec3A {
        match self {
            Material::Emissive(m) => m.evaluate_emission(ray, hit),
            Material::Diffuse(_)
            | Material::Plastic(_)
            | Material::Reflective(_)
            | Material::Refractive(_)
            | Material::Volumetric(_) => Vec3A::ZERO
        }
    }

    /// Compute the manner in which the material reflects/absorbs light
    pub fn compute_reflectance(&self, ray: &Ray, hit: &HitResult, scattered: &Ray) -> f32 {
        match self {
            Material::Diffuse(m) => m.compute_reflectance(ray, hit, scattered),
            Material::Emissive(_) => 0.0,
            Material::Plastic(m) => m.compute_reflectance(ray, hit, scattered),
            Material::Reflective(m) => m.compute_reflectance(ray, hit, scattered),
            Material::Refractive(_) => 0.0,
            Material::Volumetric(_) => 1.0 / (4.0 * PI),
        }
    }
}


#[derive(Clone)]
/// Diffuse material as specified by the Oren-Nayar model
///
/// Reference:
/// Generalization of Lambert's reflectance model
/// Michael Oren, Shree K. Nayaer
/// https://dl.acm.org/doi/abs/10.1145/192161.192213
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

    /// The contribution of this Diffuse material is just its albedo. The pdf is
    /// cosine-weighted. While the scattered ray is returned here, the direction is
    /// actually calculated in the integrator by the cosine pdf.
    fn generate_response(&self, ray: &Ray, result: &HitResult, _rng: &mut Pcg64Mcg) -> Option<ScatterResult> {
        // ray.direction is passed here because the integrator generates
        // an offset point itself for the Diffuse material
        let scattered = Ray::new(result.point, ray.direction);
        let contribution = self.albedo.sample_texture(result.u, result.v, &result.point);
        let pdf = PDF::Cosine { uvw: OrthonormalBasis::new(&result.shading_normal) };
        Some(ScatterResult::new(scattered, contribution, pdf, false, false))
    }

    /// Reflect light according to the Oren-Nayar model
    ///
    /// This method uses the improved Oren-Nayar model as implemented in Cycles:
    ///
    /// Yasuhiro Fujii: A tiny improvement of Oren-Nayar reflectance model
    /// https://mimosa-pudica.net/improved-oren-nayar.html
    ///
    /// https://developer.blender.org/diffusion/C/browse/master/src/kernel/closure/bsdf_oren_nayar.h
    fn compute_reflectance(&self, wo: &Ray, result: &HitResult, wi: &Ray) -> f32 {
        let l = wi.direction;
        let v = wo.direction;
        let n = result.shading_normal;

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

/// Fresnel equations are used to compute physically accurate transmission.
///
/// These equations tell us how much of the light is reflected and
/// how much is refracted (transmitted).
///
/// Reference:
/// https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/reflection-refraction-fresnel.html
fn fresnel_coefficient(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
    let cos_theta_i = cos_theta_i.max(0.0);

    let eta = eta_i / eta_t;
    let sin2_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
    let sin2_theta_t = eta * eta * sin2_theta_i;

    // if sin2_theta is greater than 1 then we have total internal reflection
    // and there is no need to compute fresnel equations as all light is reflected.
    if sin2_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).sqrt();

    // light is composed of two perpendicular waves treated as parallel
    // and perpendicular polarised light
    let r_parallel = ((eta_t * cos_theta_i - eta_i * cos_theta_t)
        / (eta_t * cos_theta_i + eta_i * cos_theta_t)).powi(2);

    let r_perp = ((eta_i * cos_theta_i - eta_t * cos_theta_t)
        / (eta_i * cos_theta_i + eta_t * cos_theta_t)).powi(2);

    // ratio of reflected light for the two waves is obtained by taking the average
    0.5 * (r_parallel + r_perp)
}

/// Reflective material is a purely specular material with only a
/// specular lobe.
///
/// albedo is the color of the material and roughness is the amount of fuzziness.
#[derive(Clone)]
pub struct Reflective {
    pub albedo: Vec3A,
    pub roughness: f32,
}

impl Reflective {
    /// Create a new Reflective material for objects that reflect light only
    ///
    /// albedo is a Vec3A of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. roughness accounts
    /// for the fuzziness of the reflections due to the size of the primitive.
    ///
    /// Generally, the larger the primitive, the fuzzier the reflections will be.
    pub fn new(albedo: Vec3A, roughness: f32) -> Reflective {
        Reflective { albedo, roughness: roughness * roughness }
    }
}

impl Reflective {
    /// Use the incoming ray and geometric/shading normal at the hit point to
    /// generate the response of the material.
    fn generate_response(&self, ray: &Ray, result: &HitResult, _rng: &mut Pcg64Mcg) -> Option<ScatterResult> {
        // if the dot product between the ray direction and normals are less than 0
        // then the ray hit the surface from the outside, otherwise it hit the
        // surface from the inside
        let geometric_normal = if ray.direction.dot(result.geometric_normal) < 0.0 {
            // ray hits the surface from outside the primitive
            result.geometric_normal
        } else {
            // ray hits the surface from inside the primitive
            -result.geometric_normal
        };

        let shading_normal = if result.shading_normal.dot(geometric_normal) < 0.0 {
            -result.shading_normal
        } else {
            result.shading_normal
        };

        let offset_point = find_offset_point(result.point, geometric_normal);
        let reflected = reflect(ray.direction, shading_normal);
        let scattered_ray = Ray::new(offset_point, reflected);

        if self.roughness == 0.0 {
            // if roughness is set to 0.0, then handle this as a pre-weighted
            // specular material and skip NEE
            let pdf = PDF::Cosine { uvw: OrthonormalBasis::new(&shading_normal) };
            Some(ScatterResult::new(scattered_ray, self.albedo, pdf, true, true))
        } else {
            let pdf = PDF::GGX { wi: -ray.direction, normal: shading_normal, alpha: self.roughness };
            Some(ScatterResult::new(scattered_ray, self.albedo, pdf, false, true))
        }
    }

    /// Compute the reflective material's reflectance using ggx.
    ///
    /// References:
    /// https://learnopengl.com/PBR/Theory
    fn compute_reflectance(&self, ray: &Ray, result: &HitResult, scattered: &Ray) -> f32 {
        if self.roughness == 0.0 { return 0.0; }

        let wi = -ray.direction;
        let wo = scattered.direction;
        let n = result.shading_normal;

        let cos_i = n.dot(wi);
        let cos_o = n.dot(wo);

        // if the dot product is less than 0 then the rays are below the horizon
        // of the surface and don't contribute to reflectance
        if cos_i <= 0.0 || cos_o <= 0.0 {
            return 0.0;
        }

        let h = (wi + wo).normalize();
        let cos_h = n.dot(h);

        // a safety check for the half-way vector that may be unnecessary
        // since we checked cos_i and cos_o beforehand
        if cos_h <= 0.0 {
            return 0.0;
        }

        // currently there is intentionally no fresnel term in the numerator
        // as Reflective is a singular specular lobe. Once a diffuse lobe is added
        // we can add in the fresnel term in this Cook-Torrance calculation.
        ggx_distribution(cos_h, self.roughness) * ggx_geometry(cos_i, cos_o, self.roughness) / (4.0 * cos_i)
    }
}

/// Refractive material to simulate items such as glass
#[derive(Clone)]
pub struct Refractive {
    pub refractive_index: f32,
    pub absorption: Vec3A,
}

impl Refractive {
    /// Create a new Refractive material for objects that both reflect and transmit light.
    ///
    /// albedo is a Vec3A of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0.
    /// index determines how much of the light is refracted when entering the material.
    pub fn new(index: f32, albedo: Vec3A) -> Refractive {
        Refractive { refractive_index: index, absorption: albedo }
    }

    /// Generate the ScatterResult that tells the integrator how the Refractive
    /// material responds to a surface hit.
    ///
    /// References:
    /// See Peter Shirley's Ray Tracing in One Weekend for an overview of refractive
    /// scattering and Section 10.3.2 in Mathematical and Computer Programming
    /// Techniques for Computer Graphics by Peter Comininos.
    fn generate_response(&self, ray: &Ray, result: &HitResult, rng: &mut Pcg64Mcg) -> Option<ScatterResult> {
        // cache this result since it's used many times in this method
        let geometric_incident: f32 = ray.direction.dot(result.geometric_normal);
        let entering = geometric_incident < 0.0;

        // if the dot product between the ray direction and normals are less than 0
        // then the ray hit the surface from the outside, otherwise it hit the
        // surface from the inside
        let geometric_normal = if ray.direction.dot(result.geometric_normal) < 0.0 {
            result.geometric_normal
        } else {
            -result.geometric_normal
        };

        let shading_normal = if result.shading_normal.dot(geometric_normal) < 0.0 {
            -result.shading_normal
        } else {
            result.shading_normal
        };

        let (eta_i, eta_t) = if entering {
            (1.0, self.refractive_index)
        } else {
            (self.refractive_index, 1.0)
        };

        let cos_theta_i = -ray.direction.dot(shading_normal);

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

        let pdf = PDF::Cosine { uvw: OrthonormalBasis::new(&result.shading_normal) };

        if rng.random::<f32>() < reflect_probability {
            let reflected: Vec3A = reflect(ray.direction, shading_normal);
            let offset_point = find_offset_point(result.point, geometric_normal);
            let scattered_ray = Ray::new(offset_point, reflected);
            Some(ScatterResult::new(scattered_ray, attenuation, pdf, true, true))
        } else {
            let offset_point = find_offset_point(result.point, -geometric_normal);
            let scattered_ray = Ray::new(offset_point, refracted.unwrap());
            Some(ScatterResult::new(scattered_ray, attenuation, pdf, true, true))
        }
    }
}

/// Emissive material is used to determine albedo of light sources.
/// It does not control the light source's intensity.
#[derive(Clone)]
pub struct Emissive {
    pub emissive_texture: Arc<Texture>,
}

impl Emissive {
    /// Create a new Emissive material with the given Texture.
    pub fn new(emissive_texture: Texture) -> Emissive {
        let emissive_texture = Arc::new(emissive_texture);
        Emissive { emissive_texture }
    }

    /// The Light type and primitives handle actual light physics so this material
    /// will not generate a response.
    fn generate_response(&self, _ray: &Ray, _result: &HitResult, _rng: &mut Pcg64Mcg) -> Option<ScatterResult> {
        None
    }

    /// Sample the texture of the material if the ray hits the surface
    /// from the front.
    fn evaluate_emission(&self, ray: &Ray, hit: &HitResult) -> Vec3A {
        if hit.shading_normal.dot(ray.direction) < 0.0 {
            self.emissive_texture.sample_texture(hit.u, hit.v, &hit.point)
        } else {
            Vec3A::ZERO
        }
    }
}

/// Volumetric materials are to be used with the Volume type to create
/// homogenous participating media such as fog and mist. This material
/// can also be used to simulate subsurface scattering.
#[derive(Clone)]
pub struct Volumetric {
    pub albedo: Arc<Texture>,
}

impl Volumetric {
    /// Create a new Volumetric material with the given Texture
    pub fn new(albedo: Texture) -> Volumetric {
        let albedo = Arc::new(albedo);
        Volumetric { albedo }
    }

    /// Volumetric materials generate a uniform response when hit no matter the ray's direction
    fn generate_response(&self, _ray: &Ray, result: &HitResult, rng: &mut Pcg64Mcg) -> Option<ScatterResult> {
        let scattered = Ray::new(result.point, pick_sphere_point(rng));
        let contribution = self.albedo.sample_texture(result.u, result.v, &result.point);
        let pdf = PDF::Uniform;
        Some(ScatterResult::new(scattered, contribution, pdf, false, false))
    }
}

/// Plastic materials simulate both a specular lobe and a diffuse lobe with only one lobe
/// returned based on random probability.
#[derive(Clone)]
pub struct Plastic {
    pub albedo: Arc<Texture>,
    pub roughness: f32,
    pub ior: f32,
}

impl Plastic {
    /// Create a new Plastic material.
    ///
    /// albedo is the Texture to use for the material.
    /// roughness handles the amount of microfacets on the surface
    /// ior determines the refractiveness of the material
    pub fn new(albedo: Texture, roughness: f32, ior: f32) -> Plastic {
        let albedo = Arc::new(albedo);
        Plastic { albedo, roughness: roughness * roughness, ior }
    }

    /// Generate either a specular or diffuse response to a surface hit, dependent
    /// on a random reflect probability.
    fn generate_response(&self, ray: &Ray, result: &HitResult, rng: &mut Pcg64Mcg) -> Option<ScatterResult> {
        let cos_theta_i = (-ray.direction).dot(result.shading_normal).max(0.0);
        let fresnel = schlick(cos_theta_i, self.ior);

        let geometric_normal = if ray.direction.dot(result.geometric_normal) < 0.0 {
            result.geometric_normal
        } else {
            -result.geometric_normal
        };

        let shading_normal = if result.shading_normal.dot(geometric_normal) < 0.0 {
            -result.shading_normal
        } else {
            result.shading_normal
        };

        let offset_point = find_offset_point(result.point, geometric_normal);

        // probabilistically pick specular or diffuse based on Fresnel.
        // this material has two lobes, specular and diffuse. in the future
        // we need to work on returning both lobes rather than picking one randomly.
        if rng.random::<f32>() < fresnel {
            // this is the specular path
            let alpha = self.roughness;
            let microfacet_normal = ggx_sample_vndf(shading_normal, -ray.direction, alpha, rng);
            let reflected = reflect(ray.direction, microfacet_normal);

            if shading_normal.dot(reflected) <= 0.0 {
                return None;
            }

            let specular_ray = Ray::new(offset_point, reflected);
            let pdf = PDF::GGX { wi: -ray.direction, normal: shading_normal, alpha };

            Some(ScatterResult::new(specular_ray, Vec3A::ONE, pdf, false, true))
        } else {
            // this is the diffuse path
            // even though ray.direction is given, a new ray with offset is generated in the integrator
            let scattered_ray = Ray::new(offset_point, ray.direction);
            let contribution = self.albedo.sample_texture(result.u, result.v, &result.point) * (1.0 - fresnel);
            let pdf = PDF::Cosine { uvw: OrthonormalBasis::new(&result.shading_normal) };
            Some(ScatterResult::new(scattered_ray, contribution, pdf, false, false))
        }
    }

    /// Compute how the Plastic material handles reflectance.
    fn compute_reflectance(&self, wo: &Ray, result: &HitResult, wi: &Ray) -> f32 {
        // we only need to compute the reflectance for the diffuse branch since
        // the specular branch is pre_weighted and this method is only called
        // for non-pre_weighted branches
        let n = result.shading_normal;

        let cos_o = n.dot(wi.direction);
        let cos_theta_i = (-wo.direction).dot(n);

        if cos_o <= 0.0 || cos_theta_i < 0.0 {
            return 0.0;
        }

        let fresnel = schlick(cos_theta_i, self.ior);

        (1.0 - fresnel) * cos_o / PI
    }
}