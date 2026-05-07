use bvh::BVH;
use camera::Camera;
use light_source::LightSource;
use materials::Material;
use texture::Texture;

pub struct Scene {
    pub name: String,
    pub accelerator: BVH,
    pub materials: Vec<Material>,
    pub camera: Camera,
    pub light_source: Option<LightSource>,
    pub environment: Option<Texture>,
    pub atmosphere: bool,
}

impl Scene {
    pub fn new(name: String,
               accelerator: BVH,
               materials: Vec<Material>,
               camera: Camera,
               light_source: Option<LightSource>,
               environment: Option<Texture>,
               atmosphere: bool) -> Scene {
        Scene { name, materials, accelerator, camera, light_source, environment, atmosphere }
    }
}
