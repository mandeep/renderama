use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::Material;
use texture::Texture;

pub struct Scene {
    pub name: String,
    pub accelerator: BVH,
    pub materials: Vec<Material>,
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub environment: Option<Texture>,
    pub atmosphere: bool,
}

impl Scene {
    pub fn new(name: String,
               accelerator: BVH,
               materials: Vec<Material>,
               camera: Camera,
               lights: Vec<Light>,
               environment: Option<Texture>,
               atmosphere: bool) -> Scene {
        Scene { name, materials, accelerator, camera, lights, environment, atmosphere }
    }
}
