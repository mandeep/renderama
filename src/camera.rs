use std::f32::consts::PI;

use glam::Vec3A;
use rand_pcg::Pcg64;

use ray::Ray;
use sampling::pick_sphere_point;

pub struct Camera {
    pub lower_left_corner: Vec3A,
    pub horizontal: Vec3A,
    pub vertical: Vec3A,
    pub origin: Vec3A,
    u: Vec3A,
    v: Vec3A,
    pub lens_radius: f32,
    pub resolution: (f32, f32),
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
    pub fn new(origin: Vec3A,
               lookat: Vec3A,
               view: Vec3A,
               fov: f32,
               aspect: (f32, f32),
               aperture: f32,
               focus_distance: f32,
              )
               -> Camera {
        let lens_radius: f32 = aperture / 2.0;
        let theta: f32 = fov * PI / 180.0;
        let half_height: f32 = (theta / 2.0).tan();
        let half_width: f32 = (aspect.0 / aspect.1) * half_height;

        let w: Vec3A = (origin - lookat).normalize();
        let u: Vec3A = view.cross(w).normalize();
        let v: Vec3A = w.cross(u);

        let lower_left_corner: Vec3A = origin
                                      - half_width * focus_distance * u
                                      - half_height * focus_distance * v
                                      - focus_distance * w;

        let horizontal: Vec3A = 2.0 * half_width * focus_distance * u;
        let vertical: Vec3A = 2.0 * half_height * focus_distance * v;

        Camera { lower_left_corner,
                 horizontal,
                 vertical,
                 origin,
                 u,
                 v,
                 lens_radius,
                 resolution: aspect
                }
    }

    /// Generate the ray that is sent from the camera into the world
    pub fn generate_ray(&self, s: f32, t: f32, rng: &mut Pcg64) -> Ray {
        let radius: Vec3A = self.lens_radius * pick_sphere_point(rng);
        let offset: Vec3A = self.u * radius.x + self.v * radius.y;
        Ray::new(self.origin + offset,
                 self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
                 )
    }
}
