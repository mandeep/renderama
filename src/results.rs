use glam::Vec3A;

use materials::MaterialId;
use pdf::PDF;
use ray::Ray;


/// HitResult contains all of the information that tells us how a
/// primitive handles ray-primitive intersection.
///
/// parameter is the t value along the ray where the hit occurred at point
/// u, v are the UV coordinates on the surface where the hit occurred
/// point is the world space position where the ray hit the surface
/// geometric_normal is the normal used in physics calculations
/// shading_normal is the normal used in shading calculations
/// material_id is the index to the material in the materials Vec
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

    /// Orients both the geometric and shading normals to face against the incoming ray
    /// depending on which side (inside/outside) of the surface the ray is coming from.
    ///
    /// If the dot product between the ray direction and normals are less than 0,
    /// then the ray hit the surface from the outside, otherwise it hit the
    /// surface from the inside.
    pub fn face_forward_normals(&self, incoming_direction: Vec3A) -> (Vec3A, Vec3A) {
        let geometric_normal = if incoming_direction.dot(self.geometric_normal) < 0.0 {
            // ray hits the surface from outside the primitive
            self.geometric_normal
        } else {
            // ray hits the surface from inside the primitive
            -self.geometric_normal
        };

        let shading_normal = if self.shading_normal.dot(geometric_normal) < 0.0 {
            -self.shading_normal
        } else {
            self.shading_normal
        };

        (geometric_normal, shading_normal)
    }
}

/// ScatterResult contains all the information that tells us how a
/// material responds to a hit.
///
/// scattered_ray is the new ray scattered from the surface given the material's properties
/// contribution is this result's contribution to the ray's throughput
/// sampling_strategy is the pdf required for the specific material
/// specular tells the integrator whether or not this ray comes from a specular reflection
/// pre_weighted tells the integrator whether or not to weight this result's contribution
pub struct ScatterResult {
    pub scattered_ray: Ray,
    pub contribution: Vec3A,
    pub sampling_strategy: PDF,
    pub specular: bool,
    pub pre_weighted: bool,
}

impl ScatterResult {
    /// Create a new ScatterResult from the response of a material.
    pub fn new(scattered_ray: Ray, contribution: Vec3A, sampling_strategy: PDF, specular: bool, pre_weighted: bool) -> ScatterResult {
        ScatterResult { scattered_ray, contribution, sampling_strategy, specular, pre_weighted }
    }
}