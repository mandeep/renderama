use atmosphere::Atmosphere;
use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use lights::Light;
use materials::Material;

/// Scene contains all the items necessary for the integrator to start a render
pub struct Scene {
    pub name: String,
    pub accelerator: BVH,
    pub materials: Vec<Material>,
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

    pub fn with_materials(mut self, materials: Vec<Material>) -> Self {
        self.materials = materials;
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
            camera: camera,
            lights: self.lights,
            environment: self.environment,
            atmosphere: self.atmosphere
        };

        Ok(scene)
    }
}