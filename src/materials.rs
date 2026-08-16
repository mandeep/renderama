use std::f32::consts::PI;

use glam::Vec3A;
use rand::{Rng, RngExt};

use crate::basis::OrthonormalBasis;
use crate::ggx::{ggx_distribution, ggx_height_correlated_geometry, roughness_to_alpha};
use crate::pdf::{PDF, ScatteringType};
use crate::ray::{find_offset_point, Ray};
use crate::results::{HitResult, ScatterResult};
use crate::sampling::pick_sphere_point;
use crate::texture::{Texture, TextureId};


#[derive(Clone, Copy)]
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

impl Material {
    /// Generate the ScatterResult that tells the integrator how the material responds to sampling
    pub fn generate_response(&self, ray: &Ray, hit: &HitResult, textures: &[Texture], rng: &mut impl Rng) -> Option<ScatterResult> {
        match self {
            Material::Diffuse(m) => m.generate_response(ray, hit, textures),
            Material::Emissive(m) => m.generate_response(ray, hit, rng),
            Material::Plastic(m) => m.generate_response(ray, hit, textures),
            Material::Reflective(m) => m.generate_response(ray, hit, textures),
            Material::Refractive(m) => m.generate_response(ray, hit, textures, rng),
            Material::Volumetric(m) => m.generate_response(ray, hit, textures, rng),
        }
    }

    /// Evaluate the albedo of the emissive material
    pub fn evaluate_emission(&self, ray: &Ray, hit: &HitResult, textures: &[Texture]) -> Vec3A {
        match self {
            Material::Emissive(m) => m.evaluate_emission(ray, hit, textures),
            Material::Diffuse(_)
            | Material::Plastic(_)
            | Material::Reflective(_)
            | Material::Refractive(_)
            | Material::Volumetric(_) => Vec3A::ZERO
        }
    }

    /// Evaluate emitted radiance at the given UV coordinates
    pub fn evaluate_emission_at_uv(&self, u: f32, v: f32, textures: &[Texture]) -> Vec3A {
        match self {
            Material::Emissive(m) => m.evaluate_emission_at_uv(u, v, textures),
            Material::Diffuse(_)
            | Material::Plastic(_)
            | Material::Reflective(_)
            | Material::Refractive(_)
            | Material::Volumetric(_) => Vec3A::ZERO
        }
    }

    /// Compute the manner in which the material reflects/absorbs light
    pub fn compute_reflectance(&self, ray: &Ray, scattered: &Ray, hit: &HitResult, textures: &[Texture], scattering_type: impl Into<Option<ScatteringType>>) -> Vec3A {
        match self {
            Material::Diffuse(m) => m.compute_reflectance(ray, scattered, hit),
            Material::Emissive(_) => Vec3A::ZERO,
            Material::Plastic(m) => m.compute_reflectance(ray, scattered, hit, textures, scattering_type.into()),
            Material::Reflective(m) => m.compute_reflectance(ray, scattered, hit, textures),
            Material::Refractive(_) => Vec3A::ZERO,
            Material::Volumetric(_) => Vec3A::splat(1.0 / (4.0 * PI)),
        }
    }

}

#[derive(Clone, Copy)]
pub struct TextureMap {
    pub color: TextureId,
    pub roughness: Option<TextureId>,
    pub metallic: Option<TextureId>,
    pub normal: Option<TextureId>,
    pub metallic_roughness: Option<TextureId>
}

impl TextureMap {
    pub fn new(color: TextureId) -> Self {
        Self { color, roughness: None, metallic: None, normal: None, metallic_roughness: None}
    }

    pub fn with_color(mut self, color: TextureId) -> Self {
        self.color = color;
        self
    }

    pub fn with_roughness(mut self, roughness: TextureId) -> Self {
        self.roughness = Some(roughness);
        self
    }

    pub fn with_metallic(mut self, metallic: TextureId) -> Self {
        self.metallic = Some(metallic);
        self
    }

    pub fn with_metallic_roughness(mut self, metallic_roughness: TextureId) -> Self {
        self.metallic_roughness = Some(metallic_roughness);
        self
    }

    pub fn with_normal(mut self, normal: TextureId) -> Self {
        self.normal = Some(normal);
        self
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
    alpha: f32,
    beta: f32,
    texture_map: TextureMap
}

impl Diffuse {
    /// Create a new Diffuse material with the given albedo and sigma (roughness)
    ///
    /// albedo is a Vec3A of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0.
    pub fn new(albedo: TextureId, sigma: f32) -> Diffuse {
        let constant = PI + sigma * (3.0 * PI - 4.0) / 6.0;
        let alpha = 1.0 / constant;
        let beta = sigma / constant;

        let texture_map = TextureMap::new(albedo);

        Diffuse { alpha, beta, texture_map }
    }

    /// The contribution of this Diffuse material is just its albedo. The pdf is
    /// cosine-weighted. While the scattered ray is returned here, the direction is
    /// actually calculated in the integrator by the cosine pdf.
    fn generate_response(&self, ray: &Ray, result: &HitResult, textures: &[Texture]) -> Option<ScatterResult> {
        // ray.direction is passed here because the integrator generates
        // an offset point itself for the Diffuse material
        let scattered = Ray::new(result.point, ray.direction, ray.time);
        let contribution = textures[self.texture_map.color.index()].sample_texture(result.u, result.v);
        let pdf = PDF::Cosine { uvw: OrthonormalBasis::new(&result.shading_normal) };
        Some(ScatterResult::new(scattered, contribution, pdf))
    }

    /// Reflect light according to the Oren-Nayar model
    ///
    /// This method uses the improved Oren-Nayar model as implemented in Cycles:
    ///
    /// Yasuhiro Fujii: A tiny improvement of Oren-Nayar reflectance model
    /// https://mimosa-pudica.net/improved-oren-nayar.html
    ///
    /// https://developer.blender.org/diffusion/C/browse/master/src/kernel/closure/bsdf_oren_nayar.h
    fn compute_reflectance(&self, ray: &Ray, scattered: &Ray, result: &HitResult) -> Vec3A {
        let wi = -ray.direction;
        let wo = scattered.direction;
        let n = result.shading_normal;

        let nl = n.dot(wo).max(0.0);
        let nv = n.dot(wi).max(0.0);
        let lv = wo.dot(wi);

        let s = lv - nl * nv;
        let t = if s > 0.0 { nl.max(nv) } else { 1.0 };

        let diffuse = nl * (self.alpha + self.beta * s / t);
        Vec3A::splat(diffuse)
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
pub fn reflect(incident: Vec3A, normal: Vec3A) -> Vec3A {
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
fn schlick_from_ior(cosine: f32, reference_index: f32) -> f32 {
    let r = (1.0 - reference_index) / (1.0 + reference_index);
    let r0 = r.powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

/// Calculate the Schlick approximation for conductors.
///
/// For conductors, like metal, the Schlick term is
/// approximated from the base color of the object's
/// material, f0 in this case.
fn schlick_from_f0(cosine: f32, f0: Vec3A) -> Vec3A {
    f0 + (Vec3A::ONE - f0) * (1.0 - cosine).powi(5)
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
    pub albedo: TextureId,
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
    pub fn new(albedo: TextureId, roughness: f32) -> Reflective {
        Reflective { albedo, roughness }
    }

    fn is_perfect_mirror(&self) -> bool {
        self.roughness == 0.0
    }

    /// Use the incoming ray and geometric/shading normal at the hit point to
    /// generate the response of the material.
    fn generate_response(&self, ray: &Ray, result: &HitResult, textures: &[Texture]) -> Option<ScatterResult> {
        // if the dot product between the ray direction and normals are less than 0
        // then the ray hit the surface from the outside, otherwise it hit the
        // surface from the inside
        let (geometric_normal, shading_normal) = result.face_forward_normals(&ray.direction);

        let offset_point = find_offset_point(result.point, geometric_normal);
        let reflected = reflect(ray.direction, shading_normal);
        let scattered_ray = Ray::new(offset_point, reflected, ray.time);

        let f0 = textures[self.albedo.index()].sample_texture(result.u, result.v);
        let cos_theta_i = (-ray.direction).dot(shading_normal).max(0.0);
        let fresnel = schlick_from_f0(cos_theta_i, f0);

        if self.is_perfect_mirror() {
            // handle this as a pre-weighted specular material and skip NEE.
            let pdf = PDF::Delta;
            Some(ScatterResult::new(scattered_ray, fresnel, pdf))
        } else {
            // for roughness in [0.0, 0.05], fireflies will appear in high
            // luminance scenes, however these can be denoised just fine.
            let alpha = roughness_to_alpha(self.roughness);
            let pdf = PDF::GGX { wi: -ray.direction, normal: shading_normal, alpha };
            Some(ScatterResult::new(scattered_ray, Vec3A::ONE, pdf))
        }
    }

    /// Compute the reflective material's reflectance using ggx.
    ///
    /// References:
    /// https://learnopengl.com/PBR/Theory
    fn compute_reflectance(&self, ray: &Ray, scattered: &Ray, result: &HitResult, textures: &[Texture]) -> Vec3A {
        if self.is_perfect_mirror() {
            return Vec3A::ZERO;
        }

        let (_, shading_normal) = result.face_forward_normals(&ray.direction);
        let wi = -ray.direction;
        let wo = scattered.direction;
        let n = shading_normal;

        let cos_i = n.dot(wi);
        let cos_o = n.dot(wo);

        // if the dot product is less than 0 then the rays are below the horizon
        // of the surface and don't contribute to reflectance
        if cos_i <= 0.0 || cos_o <= 0.0 {
            return Vec3A::ZERO;
        }

        let h = (wi + wo).normalize();
        let cos_h = n.dot(h);

        // a safety check for the half-way vector that may be unnecessary
        // since we checked cos_i and cos_o beforehand
        if cos_h <= 0.0 {
            return Vec3A::ZERO;
        }

        let f0 = textures[self.albedo.index()].sample_texture(result.u, result.v);

        let v_dot_h = wi.dot(h);
        let l_dot_h = wo.dot(h);

        // performing this check so that we don't end up with fireflies
        if v_dot_h <= 0.0 || l_dot_h <= 0.0 {
            return Vec3A::ZERO;
        }

        let alpha = roughness_to_alpha(self.roughness);

        let f = schlick_from_f0(v_dot_h, f0);
        let d = ggx_distribution(cos_h, alpha);
        let g = ggx_height_correlated_geometry(cos_i, cos_o, alpha);

        (f * d * g) / (4.0 * cos_i)
    }
}

/// Refractive material to simulate items such as glass
#[derive(Clone)]
pub struct Refractive {
    pub refractive_index: f32,
    pub texture_map: TextureMap
}

impl Refractive {
    /// Create a new Refractive material for objects that both reflect and transmit light.
    ///
    /// albedo is a Vec3A of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0.
    /// index determines how much of the light is refracted when entering the material.
    pub fn new(absorption: TextureId, index: f32) -> Refractive {
        let texture_map = TextureMap::new(absorption);
        Refractive { refractive_index: index, texture_map }
    }

    /// Generate the ScatterResult that tells the integrator how the Refractive
    /// material responds to a surface hit.
    ///
    /// References:
    /// See Peter Shirley's Ray Tracing in One Weekend for an overview of refractive
    /// scattering and Section 10.3.2 in Mathematical and Computer Programming
    /// Techniques for Computer Graphics by Peter Comininos.
    fn generate_response(&self, ray: &Ray, result: &HitResult, textures: &[Texture], rng: &mut impl Rng) -> Option<ScatterResult> {
        // cache this result since it's used many times in this method
        let geometric_incident: f32 = ray.direction.dot(result.geometric_normal);
        let entering = geometric_incident < 0.0;

        let (geometric_normal, shading_normal) = result.face_forward_normals(&ray.direction);

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
            textures[self.texture_map.color.index()].sample_texture(result.u, result.v)
        };

        let pdf = PDF::Delta;

        if rng.random::<f32>() < reflect_probability {
            let reflected: Vec3A = reflect(ray.direction, shading_normal);
            let offset_point = find_offset_point(result.point, geometric_normal);
            let scattered_ray = Ray::new(offset_point, reflected, ray.time);
            Some(ScatterResult::new(scattered_ray, attenuation, pdf))
        } else {
            let offset_point = find_offset_point(result.point, -geometric_normal);
            let scattered_ray = Ray::new(offset_point, refracted.unwrap(), ray.time);
            Some(ScatterResult::new(scattered_ray, attenuation, pdf))
        }
    }
}

/// Emissive material controls the albedo and light intensity of primitives
/// that require emission.
#[derive(Clone)]
pub struct Emissive {
    pub emissive_color: TextureId,
    pub intensity: Option<f32>,
}

impl Emissive {
    /// Create a new Emissive material with the given Texture.
    pub fn new(emissive_color: TextureId) -> Emissive {
        Emissive {emissive_color, intensity: None }
    }

    pub fn with_intensity(mut self, intensity: impl Into<Option<f32>>) -> Self {
        self.intensity = intensity.into();
        self
    }

    /// The Light type and primitives handle actual light physics so this material
    /// will not generate a response.
    fn generate_response(&self, _ray: &Ray, _result: &HitResult, _rng: &mut impl Rng) -> Option<ScatterResult> {
        None
    }

    /// Sample the texture of the material if the ray hits the surface
    /// from the front.
    fn evaluate_emission(&self, ray: &Ray, hit: &HitResult, textures: &[Texture]) -> Vec3A {
        if hit.shading_normal.dot(ray.direction) >= 0.0 {
            return Vec3A::ZERO;
        }

        self.evaluate_emission_at_uv(hit.u, hit.v, textures)
    }

    /// Evaluate the emission of the material at the given UV coordinates.
    fn evaluate_emission_at_uv(&self, u: f32, v: f32, textures: &[Texture]) -> Vec3A {
        let emission = textures[self.emissive_color.index()].sample_texture(u, v);
        self.intensity.map_or(emission, |intensity| emission * intensity)
    }
}

/// Volumetric materials are to be used with the Volume type to create
/// homogenous participating media such as fog and mist. This material
/// can also be used to simulate subsurface scattering.
#[derive(Clone)]
pub struct Volumetric {
    albedo: TextureId
}

impl Volumetric {
    /// Create a new Volumetric material with the given Texture
    pub fn new(albedo: TextureId) -> Volumetric {
        Volumetric { albedo }
    }

    /// Volumetric materials generate a uniform response when hit no matter the ray's direction
    fn generate_response(&self, ray: &Ray, result: &HitResult, textures: &[Texture], rng: &mut impl Rng) -> Option<ScatterResult> {
        let scattered = Ray::new(result.point, pick_sphere_point(rng), ray.time);
        let contribution = textures[self.albedo.index()].sample_texture(result.u, result.v);
        let pdf = PDF::Uniform;
        Some(ScatterResult::new(scattered, contribution, pdf))
    }
}

/// Plastic materials simulate both a specular lobe and a diffuse lobe with only one lobe
/// returned based on random probability.
#[derive(Clone)]
pub struct Plastic {
    pub roughness: f32,
    pub ior: f32,
    pub subsurface: f32,
    pub diffuse_transmission: f32,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub texture_map: TextureMap
}

impl Plastic {
    /// Create a new Plastic material.
    ///
    /// albedo is the Texture to use for the material.
    /// roughness handles the amount of microfacets on the surface
    /// ior determines the refractiveness of the material
    pub fn new(albedo: TextureId, roughness: f32, ior: f32) -> Plastic {
        let texture_map = TextureMap::new(albedo);
        let subsurface = 0.0;
        let diffuse_transmission = 0.0;
        let clearcoat = 0.0;
        let clearcoat_roughness = 0.025;
        Plastic { roughness, ior, subsurface, diffuse_transmission, clearcoat, clearcoat_roughness, texture_map }
    }

    pub fn with_subsurface(mut self, subsurface: f32) -> Self {
        self.subsurface = subsurface;
        self
    }

    pub fn with_diffuse_transmission(mut self, diffuse_transmission: f32) -> Self {
        self.diffuse_transmission = diffuse_transmission.clamp(0.0, 1.0);
        self
    }

    pub fn with_clearcoat(mut self, clearcoat: f32, clearcoat_roughness: f32) -> Self {
        self.clearcoat = clearcoat;
        self.clearcoat_roughness = clearcoat_roughness;
        self
    }

    pub fn with_textures(mut self, texture_map: TextureMap) -> Self {
        self.texture_map = texture_map;
        self
    }

    /// Evaluates the normal map to perturb the shading normal.
    ///
    /// This might work for now, but later will need to add tangent
    /// and bitangent vectors to primitives' hit results.
    fn get_mapped_normal(&self, result: &HitResult, shading_normal: Vec3A, textures: &[Texture]) -> Vec3A {
        if let Some(normal_map) = self.texture_map.normal {
            let normal_map_texture = &textures[normal_map.index()];
            let normap_map_value = normal_map_texture.sample_texture(result.u, result.v);
            let tangent_normal = normap_map_value * 2.0 - Vec3A::ONE;

            let uvw = OrthonormalBasis::new(&result.shading_normal);
            uvw.local(&tangent_normal).normalize()
        } else {
            shading_normal
        }
    }

    fn sample_roughness(&self, result: &HitResult, textures: &[Texture]) -> f32 {
        if let Some(roughness_id) = self.texture_map.roughness {
            let roughness_map_texture = &textures[roughness_id.index()];
            let roughness_map_value = roughness_map_texture.sample_texture(result.u, result.v);
            roughness_map_value.x
        } else if let Some(roughness_id) = self.texture_map.metallic_roughness {
            let roughness_map_texture = &textures[roughness_id.index()];
            let roughness_map_value = roughness_map_texture.sample_texture(result.u, result.v);
            roughness_map_value.y
        } else {
            self.roughness
        }
    }

    /// Generate either a specular or diffuse response to a surface hit, dependent
    /// on a random reflect probability.
    fn generate_response(&self, ray: &Ray, result: &HitResult, textures: &[Texture]) -> Option<ScatterResult> {
        let (geometric_normal, base_shading_normal) = result.face_forward_normals(&ray.direction);
        let shading_normal = self.get_mapped_normal(result, base_shading_normal, textures);

        let roughness = self.sample_roughness(result, textures);
        let alpha = roughness_to_alpha(roughness);

        let cos_i = (-ray.direction).dot(shading_normal).max(0.0);
        let r = (1.0 - self.ior) / (1.0 + self.ior);
        let f0 = r * r;
        let fresnel = f0 + (1.0 - f0) * (1.0 - cos_i).powi(5);

        let clearcoat_weight = self.clearcoat * fresnel;
        let clearcoat_alpha = roughness_to_alpha(self.clearcoat_roughness);

        let remaining = 1.0 - clearcoat_weight;
        let specular_weight = remaining * fresnel;
        let diffuse_weight = (remaining - specular_weight).max(0.0);
        let transmission_weight = diffuse_weight * self.diffuse_transmission;

        let offset_point = find_offset_point(result.point, geometric_normal);
        let scattered_ray = Ray::new(offset_point, ray.direction, ray.time);

        let pdf = PDF::Composite {
            uvw: OrthonormalBasis::new(&shading_normal),
            wi: -ray.direction,
            normal: shading_normal,
            alpha,
            clearcoat_alpha,
            specular_weight,
            clearcoat_weight,
            transmission_weight,
        };

        Some(ScatterResult::new(scattered_ray, Vec3A::ONE, pdf))
    }

    /// Compute how the Plastic material handles reflectance.
    fn compute_reflectance(&self, ray: &Ray, scattered: &Ray, result: &HitResult, textures: &[Texture], scattering_type: Option<ScatteringType>) -> Vec3A {
        let wi = -ray.direction;
        let wo = scattered.direction;

        let (geometric_normal, shading_normal) = result.face_forward_normals(&ray.direction);
        if geometric_normal.dot(wi) <= 0.0 {
            return Vec3A::ZERO;
        }

        let n = self.get_mapped_normal(result, shading_normal, textures);

        let cos_i = n.dot(wi);
        if cos_i <= 0.0 {
            return Vec3A::ZERO;
        }

        let transmitted = match scattering_type {
            Some(ScatteringType::Transmission) => true,
            Some(ScatteringType::Reflection | ScatteringType::Volume) => false,
            None => geometric_normal.dot(wo) <= 0.0,
        };

        if transmitted {
            let cos_o = -n.dot(wo);
            if cos_o <= 0.0 {
                return Vec3A::ZERO;
            }

            let albedo = textures[self.texture_map.color.index()].sample_texture(result.u, result.v);
            let clearcoat_fresnel = schlick_from_ior(cos_i, self.ior);
            let coat_transmittance = 1.0 - self.clearcoat * clearcoat_fresnel;
            return albedo * (self.diffuse_transmission * cos_o / PI) * coat_transmittance;
        }

        if geometric_normal.dot(wo) <= 0.0 {
            return Vec3A::ZERO;
        }

        let cos_o = n.dot(wo);
        if cos_o <= 0.0 {
            return Vec3A::ZERO;
        }

        let h = (wi + wo).normalize();
        let cos_h = n.dot(h);

        if cos_h <= 0.0 {
            return Vec3A::ZERO;
        }

        let v_dot_h = wi.dot(h).max(0.0);
        let micro_fresnel = schlick_from_ior(v_dot_h, self.ior);

        let roughness = self.sample_roughness(result, textures);
        let alpha = roughness_to_alpha(roughness);

        let d = ggx_distribution(cos_h, alpha);
        let g = ggx_height_correlated_geometry(cos_i, cos_o, alpha);
        let specular = Vec3A::splat(micro_fresnel * d * g / (4.0 * cos_i));

        let (clearcoat, coat_transmittance) = if self.clearcoat > 0.0 {
            let clearcoat_fresnel = schlick_from_ior(v_dot_h, self.ior);
            let clearcoat_alpha = roughness_to_alpha(self.clearcoat_roughness);
            let clearcoat_d = ggx_distribution(cos_h, clearcoat_alpha);
            let clearcoat_g = ggx_height_correlated_geometry(cos_i, cos_o, clearcoat_alpha);
            let clearcoat_term = Vec3A::splat(self.clearcoat * clearcoat_fresnel * clearcoat_d * clearcoat_g / (4.0 * cos_i));

            let blocked = self.clearcoat * clearcoat_fresnel;
            (clearcoat_term, 1.0 - blocked)
        } else {
            (Vec3A::ZERO, 1.0)
        };

        let specular = specular * coat_transmittance;

        // the macro fresnel term creates too dark shadows at grazing angles so
        // we use Burley's roughness model to account for that with a highlight.
        // Reference: pg 14 of Burley's 2012 Disney BRDF paper
        // https://media.disneyanimation.com/uploads/production/publication_asset/48/asset/s2012_pbs_disney_brdf_notes_v3.pdf
        let f_d90 = 0.5 + 2.0 * alpha * v_dot_h * v_dot_h;

        let f_i = 1.0 + (f_d90 - 1.0) * (1.0 - cos_i).powi(5);
        let f_o = 1.0 + (f_d90 - 1.0) * (1.0 - cos_o).powi(5);
        let base_diffuse = f_i * f_o;

        // since we added the Disney BRDF diffuse term might as well
        // add the subsurface approximation term as well.
        // Later will need to implement a proper subsurface scattering random walk.
        // Reference: https://cseweb.ucsd.edu/~tzli/cse272/wi2023/homework1.pdf
        let f_ss90 = alpha * v_dot_h * v_dot_h;
        let ss_i = 1.0 + (f_ss90 - 1.0) * (1.0 - cos_i).powi(5);
        let ss_o = 1.0 + (f_ss90 - 1.0) * (1.0 - cos_o).powi(5);
        let subsurface_approximation = 1.25 * (ss_i * ss_o * (1.0 / (cos_i + cos_o) - 0.5) + 0.5);

        let blended_diffuse = (1.0 - self.subsurface) * base_diffuse + self.subsurface * subsurface_approximation;

        let albedo = textures[self.texture_map.color.index()].sample_texture(result.u, result.v);
        let diffuse = albedo
            * ((1.0 - self.diffuse_transmission) * blended_diffuse * cos_o / PI)
            * coat_transmittance;

        specular + diffuse + clearcoat
    }
}