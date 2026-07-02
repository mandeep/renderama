use std::fs;

use glam::{EulerRot, Mat3A, Vec2, Vec3A};
use rand_pcg::Pcg64Mcg;
use rand::RngExt;
use serde::Deserialize;

use crate::ray::Ray;
use crate::sampling::pick_disk_point;

#[derive(Copy, Clone)]
pub enum CameraOrientation {
    LookAt { lookat: Vec3A, view: Vec3A },
    Rotation(Vec3A),
}

impl CameraOrientation {
    pub fn look_at(lookat: Vec3A) -> Self {
        CameraOrientation::LookAt { lookat, view: Vec3A::new(0.0, 1.0, 0.0) }
    }

    pub fn look_at_with_view(lookat: Vec3A, view: Vec3A) -> Self {
        CameraOrientation::LookAt { lookat, view }
    }

    pub fn rotation(rotation: Vec3A) -> Self {
        CameraOrientation::Rotation(rotation)
    }
}

#[derive(Copy, Clone)]
pub enum UpAxis {
    Y,
    Z,
}

#[derive(Copy, Clone)]
pub struct CameraOptions {
    pub origin: Vec3A,
    pub orientation: Option<CameraOrientation>,
    pub focal_length: f32,
    pub f_stop: f32,
    pub sensor_width: f32,
    pub sensor_height: Option<f32>,
    pub focus_distance: Option<f32>,
    pub world_scale: f32,
    pub resolution: (f32, f32),
    pub frame_start_time: f32,
    pub shutter_speed: f32,
    pub up_axis: UpAxis,
}

impl CameraOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_disk(filename: impl Into<String>) -> Self {
        let filename = filename.into();

        let serialized = fs::read_to_string(&filename)
            .expect("Failed to read Camera JSON file.");

        let cameras: Vec<CameraJson> = serde_json::from_str(&serialized)
            .expect("Failed to deserialize Camera JSON file.");

        let camera = cameras.first()
            .expect("Camera JSON file did not contain any cameras.");

        let origin = Vec3A::from_array(camera.location);
        let rotation = Vec3A::from_array(camera.rotation_euler_xyz_degrees);

        let f_stop = if camera.dof.use_dof {
            camera.dof.aperture_fstop
        } else {
            f32::INFINITY
        };

        CameraOptions::new()
            .with_origin(origin)
            .with_rotation(rotation)
            .with_focal_length(camera.lens_mm)
            .with_sensor_width(camera.sensor_width_mm)
            .with_focus_distance(camera.dof.focus_distance)
            .with_fstop(f_stop)
            .with_resolution(camera.render.resolution_x, camera.render.resolution_y)
            .with_up_axis(UpAxis::Z)
    }

    pub fn with_origin(mut self, origin: Vec3A) -> Self {
        self.origin = origin;
        self
    }

    pub fn with_rotation(mut self, rotation: Vec3A) -> Self {
        self.orientation = Some(CameraOrientation::rotation(rotation));
        self
    }

    pub fn with_lookat(mut self, lookat: Vec3A) -> Self {
        self.orientation = Some(CameraOrientation::look_at(lookat));
        self
    }

    pub fn with_lookat_and_view(mut self, lookat: Vec3A, view: Vec3A) -> Self {
        self.orientation = Some(CameraOrientation::look_at_with_view(lookat, view));
        self
    }

    pub fn with_sensor_width(mut self, sensor_width: f32) -> Self {
        self.sensor_width = sensor_width;
        self
    }

    pub fn with_sensor_height(mut self, sensor_height: f32) -> Self {
        self.sensor_height = Some(sensor_height);
        self
    }

    pub fn with_focal_length(mut self, focal_length: f32) -> Self {
        self.focal_length = focal_length;
        self
    }

    pub fn with_fov(mut self, fov: f32) -> Self {
        self.focal_length = (self.sensor_width / 2.0) / (fov.to_radians() / 2.0).tan();
        self
    }

    pub fn with_fstop(mut self, f_stop: f32) -> Self {
        self.f_stop = f_stop;
        self
    }

    pub fn with_focus_distance(mut self, focus_distance: f32) -> Self {
        self.focus_distance = Some(focus_distance);
        self
    }

    pub fn with_frame_duration(mut self, frame_duration: f32) -> Self {
        self.frame_start_time = frame_duration;
        self
    }

    pub fn with_shutter_speed(mut self, shutter_speed: f32) -> Self {
        self.shutter_speed = shutter_speed;
        self
    }

    pub fn with_world_scale(mut self, world_scale: f32) -> Self {
        self.world_scale = world_scale;
        self
    }

    pub fn with_resolution(mut self, width: usize, height: usize) -> Self {
        self.resolution = (width as f32, height as f32);
        self
    }

    pub fn with_up_axis(mut self, axis: UpAxis) -> Self {
        self.up_axis = axis;
        self
    }
}

impl Default for CameraOptions {
    fn default() -> Self {
        let origin = Vec3A::ZERO;

        CameraOptions {
            origin,
            orientation: None,
            focal_length: 50.0,
            f_stop: f32::INFINITY,
            sensor_width: 36.0,
            sensor_height: None,
            focus_distance: None,
            world_scale: 0.001,
            resolution: (1920.0, 1080.0),
            frame_start_time: 0.0,
            shutter_speed: 0.0,
            up_axis: UpAxis::Y
        }
    }
}

pub struct Camera {
    pub top_left_corner: Vec3A,
    pub horizontal: Vec3A,
    pub vertical: Vec3A,
    pub origin: Vec3A,
    u: Vec3A,
    v: Vec3A,
    pub lens_radius: f32,
    pub resolution: (f32, f32),
    pub start_time: f32,
    pub end_time: f32,
}

impl Camera {
    /// Create a new camera with which to see the world!
    ///
    /// origin: determines where the eye is placed on the camera
    /// lookat: determines the point in the world the eye is looking
    /// view: vector responsible for determining the rotation of the camera
    /// focal_length: focal length of the lens in millimeters (35mm, 50mm, 85mm, etc.)
    /// f_stop: the size of the aperture of the lens
    /// sensor_height: height of the camera sensor in millimeters (24.0 for full frame)
    /// focus_distance: the distance of the plane of perfect focus (in world units)
    /// world_scale: factor to convert from millimeters to world units(0.001 if 1.0 world unit = 1 meter)
    /// resolution: the resolution in pixels for the rendered image
    ///
    /// References:
    /// https://pbr-book.org/4ed/Cameras_and_Film
    /// https://en.wikipedia.org/wiki/F-number
    pub fn new(options: &CameraOptions) -> Camera {
        let orientation = options.orientation.as_ref()
            .expect("CameraOptions::orientation must be set before building a Camera");

        let change_of_basis = Mat3A::from_cols(Vec3A::X, -Vec3A::Z, Vec3A::Y);

        let origin = match options.up_axis {
            UpAxis::Y => options.origin,
            UpAxis::Z => change_of_basis * options.origin,
        };

        let (u, v, w) = match orientation {
            CameraOrientation::LookAt { lookat, view } => {
                let (lookat, view) = match options.up_axis {
                    UpAxis::Y => (*lookat, *view),
                    UpAxis::Z => (change_of_basis * lookat, change_of_basis * view)
                };

                let w: Vec3A = (origin - lookat).normalize(); // points away from scene
                let u: Vec3A = view.cross(w).normalize(); // points to the right
                let v: Vec3A = w.cross(u); // points up

                (u, v, w)
            },
            CameraOrientation::Rotation(rotation) => {
                let rotation = rotation.map(|angle| angle.to_radians());
                let rotation_matrix = Mat3A::from_euler(EulerRot::XYZEx, rotation.x, rotation.y, rotation.z);

                let basis = match options.up_axis {
                    UpAxis::Y => rotation_matrix,
                    UpAxis::Z => change_of_basis * rotation_matrix,
                };

                let u = basis.x_axis;
                let v = basis.y_axis;
                let w = basis.z_axis;

                (u, v, w)
            },
        };

        let focus_distance = options.focus_distance.unwrap_or_else(|| {
            match orientation {
                CameraOrientation::LookAt { lookat, .. } => (lookat - options.origin).length(),
                CameraOrientation::Rotation(_) => panic!("focus_distance must be set when using the Rotation orientation."),
            }
        });

        Camera::from_basis(
            origin, u, v, w,
            options.focal_length, options.f_stop, options.sensor_width, options.sensor_height, focus_distance,
            options.world_scale, options.resolution, options.frame_start_time, options.shutter_speed,
        )
    }

    /// Build a camera basis given origin, u, v, and w.
    ///
    /// The code in this method was once housed in the new()
    /// method, however it was moved into a separate private method
    /// so that any additional methods can use it.
    fn from_basis(
        origin: Vec3A,
        u: Vec3A,
        v: Vec3A,
        w: Vec3A,
        focal_length: f32,
        f_stop: f32,
        sensor_width: f32,
        sensor_height: Option<f32>,
        focus_distance: f32,
        world_scale: f32,
        resolution: (f32, f32),
        frame_start_time: f32,
        shutter_speed: f32,
    ) -> Camera {
        let lens_diameter = focal_length / f_stop;
        let lens_radius = (lens_diameter * world_scale) / 2.0;

        // full frame sensor width is 36.0, calculating it here
        // but can move it to the constructor if necessary later on
        let sensor_height = sensor_height.unwrap_or(sensor_width * (resolution.1 / resolution.0));
        let half_height = (sensor_height / 2.0) / focal_length;
        let half_width = (sensor_width / 2.0) / focal_length;

        // right-handed coordinate system where +X is to the right,
        // +Y is up, and +Z travels out of the screen
        let top_left_corner: Vec3A = origin
                                      - half_width * focus_distance * u
                                      + half_height * focus_distance * v
                                      - focus_distance * w;

        let horizontal: Vec3A = 2.0 * half_width * focus_distance * u;
        let vertical: Vec3A = 2.0 * half_height * focus_distance * v;

        let start_time = frame_start_time;
        let end_time = start_time + shutter_speed;

        Camera { top_left_corner,
                 horizontal,
                 vertical,
                 origin,
                 u,
                 v,
                 lens_radius,
                 resolution,
                 start_time,
                 end_time,
                }
    }

    /// Generate the ray that is sent from the camera into the world
    pub fn generate_ray(&self, s: f32, t: f32, rng: &mut Pcg64Mcg) -> Ray {
        let radius: Vec2 = self.lens_radius * pick_disk_point(rng);
        let offset: Vec3A = self.u * radius.x + self.v * radius.y;
        let time = if self.start_time == self.end_time {
            self.start_time
        } else {
            self.start_time + rng.random::<f32>() * (self.end_time - self.start_time)
        };

        Ray::new(
            self.origin + offset,
            self.top_left_corner + s * self.horizontal - t * self.vertical - self.origin - offset,
            time
        )
    }
}

#[derive(Deserialize)]
struct CameraJson {
    location: [f32; 3],
    rotation_euler_xyz_degrees: [f32; 3],

    lens_mm: f32,
    sensor_width_mm: f32,

    dof: CameraDofJson,
    render: CameraRenderJson,
}

#[derive(Deserialize)]
struct CameraDofJson {
    use_dof: bool,
    focus_distance: f32,
    aperture_fstop: f32,
}

#[derive(Deserialize)]
struct CameraRenderJson {
    resolution_x: usize,
    resolution_y: usize,
}