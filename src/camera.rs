use std::f64;

use nalgebra::core::Vector3;
use rand::thread_rng;

use ray::{pick_sphere_point, Ray};


pub struct Camera {
    pub lower_left_corner: Vector3<f64>,
    pub horizontal: Vector3<f64>,
    pub vertical: Vector3<f64>,
    pub origin: Vector3<f64>,
    u: Vector3<f64>,
    v: Vector3<f64>,
    w: Vector3<f64>,
    lens_radius: f64
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
    pub fn new(origin: Vector3<f64>,
               lookat: Vector3<f64>,
               view: Vector3<f64>,
               fov: f64,
               aspect: f64,
               aperture: f64,
               focus_distance: f64) -> Camera {

        let lens_radius: f64 = aperture / 2.0;
        let theta: f64 = fov * f64::consts::PI / 180.0;
        let half_height: f64 = (theta / 2.0).tan();
        let half_width: f64 = aspect * half_height;

        let w: Vector3<f64> = (origin - lookat).normalize();
        let u: Vector3<f64> = view.cross(&w).normalize();
        let v: Vector3<f64> = w.cross(&u);

        let lower_left_corner: Vector3<f64> = origin -
                                              half_width * focus_distance * u -
                                              half_height * focus_distance * v -
                                              focus_distance * w;

        let horizontal: Vector3<f64> = 2.0 * half_width * focus_distance * u;
        let vertical: Vector3<f64> = 2.0 * half_height * focus_distance * v;

        Camera { lower_left_corner, horizontal, vertical, origin, u, v, w, lens_radius }
    }

    /// Get the ray that is coming from the camera into the world
    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let mut rng = thread_rng();
        let radius: Vector3<f64> = self.lens_radius * pick_sphere_point(&mut rng);
        let offset: Vector3<f64> = self.u * radius.x + self.v * radius.y;
        Ray { origin: self.origin + offset,
              direction: self.lower_left_corner + s * self.horizontal + t * self.vertical -
                         self.origin - offset }
    }
}
