use std::f32::consts::PI;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::Rng;

use integrator::pick_sphere_point;
use ray::Ray;

pub struct Camera {
    pub lower_left_corner: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub origin: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    pub lens_radius: f32,
    pub start_time: f32,
    pub end_time: f32,
    pub atmosphere: bool,
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
    pub fn new(origin: Vec3,
               lookat: Vec3,
               view: Vec3,
               fov: f32,
               aspect: f32,
               aperture: f32,
               focus_distance: f32,
               start_time: f32,
               end_time: f32,
               atmosphere: bool)
               -> Camera {
        let lens_radius: f32 = aperture / 2.0;
        let theta: f32 = fov * PI / 180.0;
        let half_height: f32 = (theta / 2.0).tan();
        let half_width: f32 = aspect * half_height;

        let w: Vec3 = (origin - lookat).normalize();
        let u: Vec3 = view.cross(w).normalize();
        let v: Vec3 = w.cross(u);

        let lower_left_corner: Vec3 = origin
                                      - half_width * focus_distance * u
                                      - half_height * focus_distance * v
                                      - focus_distance * w;

        let horizontal: Vec3 = 2.0 * half_width * focus_distance * u;
        let vertical: Vec3 = 2.0 * half_height * focus_distance * v;

        Camera { lower_left_corner,
                 horizontal,
                 vertical,
                 origin,
                 u,
                 v,
                 w,
                 lens_radius,
                 start_time,
                 end_time,
                 atmosphere }
    }

    /// Get the ray that is coming from the camera into the world
    pub fn get_ray(&self, s: f32, t: f32, mut rng: &mut ThreadRng) -> Ray {
        let radius: Vec3 = self.lens_radius * pick_sphere_point(&mut rng);
        let offset: Vec3 = self.u * radius.x() + self.v * radius.y();
        let time = self.start_time + rng.gen::<f32>() * (self.end_time - self.start_time);
        Ray::new(self.origin + offset,
                 self.lower_left_corner + s * self.horizontal + t * self.vertical
                 - self.origin
                 - offset,
                 time)
    }
}
