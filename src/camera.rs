use std::f32;

use nalgebra::core::Vector3;
use rand::{thread_rng, Rng};

use ray::{pick_sphere_point, Ray};

pub struct Camera {
    pub lower_left_corner: Vector3<f32>,
    pub horizontal: Vector3<f32>,
    pub vertical: Vector3<f32>,
    pub origin: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    w: Vector3<f32>,
    lens_radius: f32,
    start_time: f32,
    end_time: f32,
}

impl Camera {
    /// Create a new camera with which to see the world!
    ///
    /// The origin determines where the eye is placed on the camera.
    /// The lookat variable determines where in the world the eye is looking.
    /// The view vector is responsible for determining the tilt of the camera.
    /// FOV is the angle at which the eye is looking through the camera.
    /// The aspect ratio is the proportial difference between the width and height.
    /// aperture controls how big the lens of the camera is and focus distance
    /// controls the shortest distance that the camera can focus.
    pub fn new(origin: Vector3<f32>,
               lookat: Vector3<f32>,
               view: Vector3<f32>,
               fov: f32,
               aspect: f32,
               aperture: f32,
               focus_distance: f32,
               start_time: f32,
               end_time: f32)
               -> Camera {
        let lens_radius: f32 = aperture / 2.0;
        let theta: f32 = fov * f32::consts::PI / 180.0;
        let half_height: f32 = (theta / 2.0).tan();
        let half_width: f32 = aspect * half_height;

        let w: Vector3<f32> = (origin - lookat).normalize();
        let u: Vector3<f32> = view.cross(&w).normalize();
        let v: Vector3<f32> = w.cross(&u);

        let lower_left_corner: Vector3<f32> = origin
                                              - half_width * focus_distance * u
                                              - half_height * focus_distance * v
                                              - focus_distance * w;

        let horizontal: Vector3<f32> = 2.0 * half_width * focus_distance * u;
        let vertical: Vector3<f32> = 2.0 * half_height * focus_distance * v;

        Camera { lower_left_corner,
                 horizontal,
                 vertical,
                 origin,
                 u,
                 v,
                 w,
                 lens_radius,
                 start_time,
                 end_time }
    }

    /// Get the ray that is coming from the camera into the world
    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        let mut rng = thread_rng();
        let radius: Vector3<f32> = self.lens_radius * pick_sphere_point(&mut rng);
        let offset: Vector3<f32> = self.u * radius.x + self.v * radius.y;
        let time = self.start_time + rng.gen::<f32>() * (self.end_time - self.start_time);
        Ray::new(self.origin + offset,
                 self.lower_left_corner + s * self.horizontal + t * self.vertical
                            - self.origin
                            - offset,
                 time)
    }
}
