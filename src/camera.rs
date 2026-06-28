use glam::{EulerRot, Mat3A, Vec2, Vec3A};
use rand_pcg::Pcg64Mcg;
use rand::RngExt;

use crate::ray::Ray;
use crate::sampling::pick_disk_point;

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
    pub fn new(
        origin: Vec3A,
        lookat: Vec3A,
        view: Vec3A,
        focal_length: f32,
        f_stop: f32,
        sensor_height: f32,
        focus_distance: f32,
        world_scale: f32,
        resolution: (f32, f32),
        frame_start_time: f32,
        shutter_speed: f32,
        ) -> Camera {
            let w: Vec3A = (origin - lookat).normalize(); // points away from scene
            let u: Vec3A = view.cross(w).normalize(); // points to the right
            let v: Vec3A = w.cross(u); // points up

            Camera::from_basis(
                origin, u, v, w,
                focal_length, f_stop, sensor_height, focus_distance,
                world_scale, resolution, frame_start_time, shutter_speed,
            )
    }

    /// Create a new camera with which to see the world!
    ///
    /// This differs from the other new method in that it is built using
    /// a rotation vector (Euler angles in degrees) rather than a lookat
    /// vector. This approach is more intuitive when rotating a camera
    /// to achieve final frame.
    pub fn new_from_rotation(
        location: Vec3A,
        rotation: Vec3A,
        focal_length: f32,
        f_stop: f32,
        sensor_height: f32,
        focus_distance: f32,
        world_scale: f32,
        resolution: (f32, f32),
        frame_start_time: f32,
        shutter_speed: f32,
    ) -> Camera {
        let rotation_matrix = Mat3A::from_euler(
            EulerRot::XYZEx,
            rotation.x.to_radians(),
            rotation.y.to_radians(),
            rotation.z.to_radians(),
        );

        let u = rotation_matrix.x_axis;
        let v = rotation_matrix.y_axis;
        let w = rotation_matrix.z_axis;

        Camera::from_basis(
            location, u, v, w,
            focal_length, f_stop, sensor_height, focus_distance,
            world_scale, resolution, frame_start_time, shutter_speed,
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
        sensor_height: f32,
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
        let sensor_width = sensor_height * (resolution.0 / resolution.1);
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
