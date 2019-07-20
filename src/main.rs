#![allow(dead_code)]

extern crate image;
extern crate nalgebra;
extern crate rand;
extern crate rayon;

mod aabb;
mod bvh;
mod camera;
mod denoise;
mod hitable;
mod materials;
mod ray;
mod rectangle;
mod scene;
mod sphere;
mod texture;
mod transformations;
mod utils;
mod world;

use std::env;
use std::f32;
use std::time::Instant;

use nalgebra::core::Vector3;
use rand::thread_rng;
use rayon::prelude::*;

use camera::Camera;
use denoise::denoise;

fn main() {
    let mut instant = Instant::now();

    let (width, height): (u32, u32) = (1000, 1000);
    let args: Vec<String> = env::args().collect();
    let samples: u32 = args[1].parse().unwrap();

    let origin = Vector3::new(278.0, 278.0, -800.0);
    let lookat = Vector3::new(278.0, 278.0, 0.0);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;

    let camera = Camera::new(origin, &lookat, &view, fov, aspect_ratio, aperture, focus_distance,
                             time0, time1);

    let world = scene::cornell_box_scene();

    println!("Rendering scene with {} samples at {} x {} dimensions...",
             samples, width, height);

    let mut pixels = vec![image::Rgb([0, 0, 0]); (width * height) as usize];
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                                         let mut coordinate: Vector3<f32> = Vector3::zeros();
                                         let x = i % width as usize;
                                         let y = i / width as usize;

                                         let mut rng = thread_rng();

                                         (0..samples).for_each(|_| {
                                                         let u = (x as f32 + rand::random::<f32>())
                                                                 / width as f32;
                                                         let v = (y as f32 + rand::random::<f32>())
                                                                 / height as f32;
                                                         let ray = camera.get_ray(u, v);
                                                         coordinate +=
                                                             ray::compute_color(&ray, &world, 0,
                                                                                &mut rng);
                                                     });

                                         coordinate /= samples as f32;
                                         (0..3).for_each(|i| {
                                                   coordinate[i] = 255.0 * coordinate[i].sqrt()
                                               });
                                         *pixel = image::Rgb([coordinate.x as u8,
                                                              coordinate.y as u8,
                                                              coordinate.z as u8]);
                                     });

    println!("Finished rendering in {}. Render saved to render.png.",
             utils::format_time(instant.elapsed()));

    let mut buffer = image::ImageBuffer::new(width, height);
    buffer.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
                                     let index = (y * width + x) as usize;
                                     *pixel = pixels[index];
                                 });

    image::ImageRgb8(buffer).flipv().save("render.png").unwrap();

    if args.len() > 2 && args[2] == "--denoise" {
        instant = Instant::now();
        println!("Denoising image...");

        let output_image = denoise(&pixels, width as usize, height as usize);
        image::save_buffer("denoised_render.png",
                           &output_image[..],
                           width as u32,
                           height as u32,
                           image::RGB(8)).expect("Failed to save output image");

        println!("Finished denoising in {}. Render saved to denoised_render.png.",
                 utils::format_time(instant.elapsed()));
    }
}
