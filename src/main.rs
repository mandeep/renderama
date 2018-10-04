extern crate image;
extern crate nalgebra;
extern crate rand;
extern crate rayon;

mod camera;
mod hitable;
mod ray;
mod sphere;
mod world;

use camera::Camera;
use nalgebra::core::Vector3;
use rayon::prelude::*;
use sphere::Sphere;
use std::f64;
use world::World;


fn main() {
    let (width, height): (u32, u32) = (1920, 960);
    let samples: u32 = 100;

    let mut buffer = image::ImageBuffer::new(width, height);
    let camera = Camera::new(Vector3::new(-2.0, -1.0, -1.0),
                             Vector3::new(4.0, 0.0, 0.0),
                             Vector3::new(0.0, 2.0, 0.0),
                             Vector3::new(0.0, 0.0, 0.0));

    let mut world = World::new();
    world.objects.push(Box::new(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)));
    world.objects.push(Box::new(Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0)));

    let mut pixels = vec![image::Rgb([0, 0, 0]); (width * height) as usize];
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let mut coordinate: Vector3<f64> = Vector3::zeros();
        let x = i % width as usize;
        let y = i / width as usize;

        (0..samples).for_each(|_| {
            let u = (x as f64 + rand::random::<f64>()) / width as f64;
            let v = (y as f64 + rand::random::<f64>()) / height as f64;
            let ray = camera.get_ray(u, v);
            coordinate += ray::compute_color(&ray, &world);
        });

        coordinate /= samples as f64;
        (0..3).for_each(|i| coordinate[i] = 255.0 * coordinate[i].sqrt());
        *pixel = image::Rgb([coordinate.x as u8, coordinate.y as u8, coordinate.z as u8]);
    });


    buffer.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        let index = (y * width + x) as usize;
        *pixel = pixels[index];
    });

    image::ImageRgb8(buffer).flipv().save("render.png").unwrap();
}
