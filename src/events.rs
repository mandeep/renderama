use glam::Vec3;

use pdf::PDF;
use ray::Ray;


/// HitEvent contains the elements necessary to render geometry
/// once a ray has hit that geometry.
pub struct HitEvent {
    pub parameter: f32,
    pub u: f32,
    pub v: f32,
    pub point: Vec3,
    pub geometric_normal: Vec3,
    pub shading_normal: Vec3,
    pub material_id: u32,
}

impl HitEvent {
    /// Create a new HitEvent for a given ray-geometry intersection.
    pub fn new(parameter: f32,
               u: f32,
               v: f32,
               point: Vec3,
               geometric_normal: Vec3,
               shading_normal: Vec3,
               material_id: u32)
               -> HitEvent {
        HitEvent { parameter, u, v, point, geometric_normal, shading_normal, material_id }
    }
}
pub struct ScatterEvent<'a> {
    pub specular_ray: Ray,
    pub attenuation: Vec3,
    pub pdf: PDF<'a>,
    pub specular: bool,
}

impl<'a> ScatterEvent<'a> {
    pub fn new(specular_ray: Ray, attenuation: Vec3, pdf: PDF<'a>, specular: bool) -> ScatterEvent<'a> {
        ScatterEvent { specular_ray, attenuation, pdf, specular }
    }
}