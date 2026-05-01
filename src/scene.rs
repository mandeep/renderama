use bvh::BVH;
use camera::Camera;
use materials::Material;
use plane::Plane;
use texture::Texture;

pub struct Scene {
    pub name: String,
    pub accelerator: BVH,
    pub materials: Vec<Material>,
    pub camera: Camera,
    pub light_source: Option<Plane>,
    pub environment: Option<Texture>,
}

impl Scene {
    pub fn new(name: String, accelerator: BVH, materials: Vec<Material>, camera: Camera, light_source: Option<Plane>, environment: Option<Texture>) -> Scene {
        Scene { name, materials, accelerator, camera, light_source, environment }
    }
}