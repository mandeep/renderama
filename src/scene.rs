use crate::atmosphere::Atmosphere;
use crate::bvh::BVH;
use crate::camera::Camera;
use crate::environment::EnvironmentMap;
use crate::lights::Light;
use crate::materials::Material;
use crate::texture::Texture;

/// Scene contains all the items necessary for the integrator to start a render
pub struct Scene {
    pub name: String,
    pub accelerator: BVH,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub environment: Option<EnvironmentMap>,
    pub atmosphere: Option<Atmosphere>,
}

/// SceneBuilder constructs a Scene with the given build items
pub struct SceneBuilder {
    name: String,
    accelerator: Option<BVH>,
    materials: Vec<Material>,
    textures: Vec<Texture>,
    camera: Option<Camera>,
    lights: Vec<Light>,
    environment: Option<EnvironmentMap>,
    atmosphere: Option<Atmosphere>,
}

#[derive(Debug)]
pub enum SceneBuildError {
    MissingCamera,
    MissingBVH,
}

impl SceneBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        SceneBuilder {
            name: name.into(),
            accelerator: None,
            materials: Vec::new(),
            textures: Vec::new(),
            camera: None,
            lights: Vec::new(),
            environment: None,
            atmosphere: None
        }
    }

    pub fn with_camera(mut self, camera: Camera) -> Self {
        self.camera = Some(camera);
        self
    }

    pub fn with_accelerator(mut self, accelerator: BVH) -> Self {
        self.accelerator = Some(accelerator);
        self
    }

    pub fn with_materials(mut self, materials: Vec<Material>, textures: Vec<Texture>) -> Self {
        self.materials = materials;
        self.textures = textures;
        self
    }

    pub fn with_lights(mut self, lights: Vec<Light>) -> Self {
        self.lights = lights;
        self
    }

    pub fn with_environment(mut self, environment: EnvironmentMap) -> Self {
        self.environment = Some(environment);
        self
    }

    pub fn with_atmosphere(mut self, atmosphere: Atmosphere) -> Self {
        self.atmosphere = Some(atmosphere);
        self
    }

    pub fn build(self) -> Result<Scene, SceneBuildError> {
        let camera = self.camera.ok_or(SceneBuildError::MissingCamera)?;
        let accelerator = self.accelerator.ok_or(SceneBuildError::MissingBVH)?;

        let scene = Scene {
            name: self.name,
            accelerator: accelerator,
            materials: self.materials,
            textures: self.textures,
            camera: camera,
            lights: self.lights,
            environment: self.environment,
            atmosphere: self.atmosphere
        };

        Ok(scene)
    }
}