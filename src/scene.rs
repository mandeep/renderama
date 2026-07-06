use crate::atmosphere::Atmosphere;
use crate::bvh::BVH;
use crate::camera::Camera;
use crate::environment::EnvironmentMap;
use crate::lights::Light;
use crate::materials::{Material, MaterialId};
use crate::texture::{Texture, TextureId};

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
    context: Option<SceneContext>,
    camera: Option<Camera>,
    environment: Option<EnvironmentMap>,
    atmosphere: Option<Atmosphere>,
}

#[derive(Debug)]
pub enum SceneBuildError {
    MissingCamera(String),
    MissingBVH(String),
    MissingLights(String),
    MissingSceneContext(String),
}

impl SceneBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        SceneBuilder {
            name: name.into(),
            accelerator: None,
            context: None,
            camera: None,
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

    pub fn with_context(mut self, context: SceneContext) -> Self {
        self.context = Some(context);
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
        let camera = self.camera.ok_or(
            SceneBuildError::MissingCamera("Scene requires a camera.".to_string())
        )?;
        let accelerator = self.accelerator.ok_or(
            SceneBuildError::MissingBVH("Scene is missing objects to render.".to_string())
        )?;

        let context = self.context.ok_or(
            SceneBuildError::MissingSceneContext("Scene is missing a SceneContext.".to_string())
        )?;

        if self.environment.is_none() && context.lights.is_empty() && self.atmosphere.is_none() {
            return Err(
                SceneBuildError::MissingLights("Scene requires an atmosphere, environment map, or at least one light.".to_string())
            );
        }

        let scene = Scene {
            name: self.name,
            accelerator: accelerator,
            materials: context.materials,
            textures: context.textures,
            camera: camera,
            lights: context.lights,
            environment: self.environment,
            atmosphere: self.atmosphere
        };

        Ok(scene)
    }
}

pub struct SceneContext {
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub lights: Vec<Light>,
}

impl SceneContext {
    pub fn new() -> SceneContext {
        SceneContext { materials: Vec::new(), textures: Vec::new(), lights: Vec::new() }
    }
    pub fn add_texture(&mut self, texture: impl Into<Texture>) -> TextureId {
        let id = TextureId(self.textures.len() as u32);
        self.textures.push(texture.into());
        id
    }

    pub fn add_material(&mut self, material: impl Into<Material>) -> MaterialId {
        let id = MaterialId(self.materials.len() as u32);
        self.materials.push(material.into());
        id
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }
}



#[cfg(test)]
mod tests {
    use glam::Vec3A;

    use super::*;
    use crate::camera::CameraOptions;
    use crate::extensions::PushInto;
    use crate::materials::Diffuse;
    use crate::sphere::Sphere;
    use crate::texture::Color;

    #[test]
    fn test_scene_missing_camera() {
        let mut objects = Vec::new();
        let mut scene_context = SceneContext::new();

        let floor_id = scene_context.add_texture(Color::new(0.5, 0.5, 0.52));
        let floor_idx = scene_context.add_material(Diffuse::new(floor_id, 0.0));
        objects.push_into(Sphere::new(Vec3A::new(0.0, -100.5, -1.0), 100.0, floor_idx));

        let bvh = BVH::new(&mut objects);

        let environment = EnvironmentMap::new("extras/textures/dusk_1_puresky.exr", 1.0);

        let builder = SceneBuilder::new("Three Spheres")
            .with_accelerator(bvh)
            .with_context(scene_context)
            .with_environment(environment);

        let result = builder.build();

        assert!(matches!(result, Err(SceneBuildError::MissingCamera(_))));
    }

    #[test]
    fn test_scene_missing_accelerator() {
        let origin = Vec3A::new(478.0, 278.0, -600.0);
        let lookat = Vec3A::new(278.0, 278.0, 0.0);
        let fov = 40.0;
        let focus_distance = 10.0;
        let world_scale = 1.0;
        let fps = 24.0;
        let frame_duration = 1.0 / fps;
        let shutter_speed = 1.0;

        let camera_options = CameraOptions::new()
            .with_origin(origin)
            .with_lookat(lookat)
            .with_fov(fov)
            .with_focus_distance(focus_distance)
            .with_world_scale(world_scale)
            .with_frame_duration(frame_duration)
            .with_shutter_speed(shutter_speed)
            .with_resolution(512, 512);
        let camera = Camera::new(&camera_options);

        let context = SceneContext::new();

        let environment = EnvironmentMap::new("extras/textures/dusk_1_puresky.exr", 1.0);

        let builder = SceneBuilder::new("Three Spheres")
            .with_camera(camera)
            .with_context(context)
            .with_environment(environment);

        let result = builder.build();

        assert!(matches!(result, Err(SceneBuildError::MissingBVH(_))));
    }

    #[test]
    fn test_scene_missing_lights() {
        let origin = Vec3A::new(478.0, 278.0, -600.0);
        let lookat = Vec3A::new(278.0, 278.0, 0.0);
        let fov = 40.0;
        let focus_distance = 10.0;
        let world_scale = 1.0;
        let fps = 24.0;
        let frame_duration = 1.0 / fps;
        let shutter_speed = 1.0;

        let camera_options = CameraOptions::new()
            .with_origin(origin)
            .with_lookat(lookat)
            .with_fov(fov)
            .with_focus_distance(focus_distance)
            .with_world_scale(world_scale)
            .with_frame_duration(frame_duration)
            .with_shutter_speed(shutter_speed)
            .with_resolution(512, 512);
        let camera = Camera::new(&camera_options);

        let mut objects = Vec::new();
        let mut context = SceneContext::new();

        let floor_id = context.add_texture(Color::new(0.5, 0.5, 0.52));
        let floor_idx = context.add_material(Diffuse::new(floor_id, 0.0));
        objects.push_into(Sphere::new(Vec3A::new(0.0, -100.5, -1.0), 100.0, floor_idx));

        let bvh = BVH::new(&mut objects);

        let builder = SceneBuilder::new("Three Spheres")
            .with_camera(camera)
            .with_accelerator(bvh)
            .with_context(context);

        let result = builder.build();

        assert!(matches!(result, Err(SceneBuildError::MissingLights(_))));

    }
}