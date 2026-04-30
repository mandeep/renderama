use glam::Vec3;


/// HitRecord contains the elements necessary to render geometry
/// once a ray has hit that geometry.
pub struct HitRecord {
    pub parameter: f32,
    pub u: f32,
    pub v: f32,
    pub point: Vec3,
    pub geometric_normal: Vec3,
    pub shading_normal: Vec3,
    pub material_id: u32,
}

impl HitRecord {
    /// Create a new HitRecord for a given ray-geometry intersection.
    pub fn new(parameter: f32,
               u: f32,
               v: f32,
               point: Vec3,
               geometric_normal: Vec3,
               shading_normal: Vec3,
               material_id: u32)
               -> HitRecord {
        HitRecord { parameter: parameter,
                    u: u,
                    v: v,
                    point: point,
                    geometric_normal: geometric_normal,
                    shading_normal: shading_normal,
                    material_id: material_id }
    }
}