use glam::Vec3A;

use materials::MaterialId;
use pdf::MaterialPDF;
use ray::Ray;


/// HitEvent contains the elements necessary to render geometry
/// once a ray has hit that geometry.
pub struct HitEvent {
    pub parameter: f32,
    pub u: f32,
    pub v: f32,
    pub point: Vec3A,
    pub geometric_normal: Vec3A,
    pub shading_normal: Vec3A,
    pub material_id: MaterialId,
}

impl HitEvent {
    /// Create a new HitEvent for a given ray-geometry intersection.
    pub fn new(parameter: f32,
               u: f32,
               v: f32,
               point: Vec3A,
               geometric_normal: Vec3A,
               shading_normal: Vec3A,
               material_id: MaterialId)
               -> HitEvent {
        HitEvent { parameter, u, v, point, geometric_normal, shading_normal, material_id }
    }
}
pub struct ScatterEvent {
    pub specular_ray: Ray,
    pub attenuation: Vec3A,
    pub pdf: MaterialPDF,
    pub specular: bool,
}

impl ScatterEvent {
    pub fn new(specular_ray: Ray, attenuation: Vec3A, pdf: MaterialPDF, specular: bool) -> ScatterEvent {
        ScatterEvent { specular_ray, attenuation, pdf, specular }
    }
}