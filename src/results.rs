use glam::Vec3A;

use materials::MaterialId;
use pdf::MaterialPDF;
use ray::Ray;


/// HitResult contains the elements necessary to render primitives
/// once a ray has hit that primitive.
pub struct HitResult {
    pub parameter: f32,
    pub u: f32,
    pub v: f32,
    pub point: Vec3A,
    pub geometric_normal: Vec3A,
    pub shading_normal: Vec3A,
    pub material_id: MaterialId,
}

impl HitResult {
    /// Create a new HitResult for a given ray-primitive intersection.
    pub fn new(parameter: f32,
               u: f32,
               v: f32,
               point: Vec3A,
               geometric_normal: Vec3A,
               shading_normal: Vec3A,
               material_id: MaterialId)
               -> HitResult {
        HitResult { parameter, u, v, point, geometric_normal, shading_normal, material_id }
    }
}
pub struct ScatterResult {
    pub specular_ray: Ray,
    pub attenuation: Vec3A,
    pub sampling_strategy: MaterialPDF,
    pub specular: bool,
    pub pre_weighted: bool,
}

impl ScatterResult {
    pub fn new(specular_ray: Ray, attenuation: Vec3A, sampling_strategy: MaterialPDF, specular: bool, pre_weighted: bool) -> ScatterResult {
        ScatterResult { specular_ray, attenuation, sampling_strategy, specular, pre_weighted }
    }
}