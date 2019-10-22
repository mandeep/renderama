#![allow(dead_code)]

extern crate chrono;
extern crate image;
extern crate nalgebra;
extern crate rand;
extern crate rand_distr;
extern crate rayon;
extern crate tobj;

mod aabb;
mod basis;
mod bvh;
mod camera;
mod denoise;
mod hitable;
mod materials;
mod pdf;
mod plane;
mod ray;
mod rectangle;
mod scene;
mod sphere;
mod texture;
mod tone;
mod transformations;
mod triangle;
mod utils;
mod volume;
mod world;

use std::env;
use std::f32;
use std::time::Instant;

use chrono::{DateTime, Local};
use nalgebra::core::Vector3;
use rand::thread_rng;
use rayon::prelude::*;

#[cfg(feature = "denoise")]
use denoise::denoise;

fn main() {
    let rendering_time = Instant::now();

    let (width, height): (u32, u32) = (2048, 2048);
    let args: Vec<String> = env::args().collect();
    let samples: u32 = args[1].parse().unwrap();
    let bounces: u32 = 10;

    let (name, camera, world, light_source) = scene::cornell_box_scene(width, height);

    let render_start_time: DateTime<Local> = Local::now();
    println!("[{}] Rendering '{}' scene with {} samples at {} x {} dimensions...",
             render_start_time.format("%H:%M:%S"),
             name,
             samples,
             width,
             height);

    let mut pixels = vec![image::Rgb([0, 0, 0]); (width * height) as usize];
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                                         let mut color: Vector3<f32> = Vector3::zeros();

                                         let x = i % width as usize;
                                         let y = i / width as usize;

                                         let mut rng = thread_rng();

                                         (0..samples).for_each(|_| {
                                                         let u = (x as f32 + rand::random::<f32>())
                                                                 / width as f32;
                                                         let v = (y as f32 + rand::random::<f32>())
                                                                 / height as f32;
                                                         let ray = camera.get_ray(u, v, &mut rng);
                                                         color +=
                                                             utils::de_nan(&ray::compute_color(ray,
                                                                                &world,
                                                                                bounces,
                                                                                &light_source,
                                                                                camera.atmosphere,
                                                                                &mut rng));
                                                     });

                                         color /= samples as f32;
                                         (0..3).for_each(|j| {
                                                   color[j] =
                                                 utils::clamp(255.0
                                                              * utils::gamma_correct(color[j],
                                                                                     2.2));
                                               });
                                         *pixel = image::Rgb([color.x as u8,
                                                              color.y as u8,
                                                              color.z as u8]);
                                     });

    let render_end_time: DateTime<Local> = Local::now();
    println!("[{}] Finished rendering in {}. Render saved to render.png.",
             render_end_time.format("%H:%M:%S"),
             utils::format_time(rendering_time.elapsed()));

    let mut buffer = image::ImageBuffer::new(width, height);
    buffer.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
                                     let index = (y * width + x) as usize;
                                     *pixel = pixels[index];
                                 });

    image::ImageRgb8(buffer).flipv().save("render.png").unwrap();

    #[cfg(feature = "denoise")]
    {
        let denoising_time = Instant::now();
        let denoise_start_time: DateTime<Local> = Local::now();
        println!("[{}] Denoising image...",
                 denoise_start_time.format("%H:%M:%S"));

        let output_image = denoise(&pixels, width as usize, height as usize);

        image::save_buffer("denoised_render.png",
                           &output_image[..],
                           width as u32,
                           height as u32,
                           image::RGB(8)).expect("Failed to save output image");

        let denoise_end_time: DateTime<Local> = Local::now();
        println!("[{}] Finished denoising in {}. Render saved to denoised_render.png.",
                 denoise_end_time.format("%H:%M:%S"),
                 utils::format_time(denoising_time.elapsed()));
    }
}
