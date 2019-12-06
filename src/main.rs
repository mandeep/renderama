#![allow(dead_code)]

extern crate chrono;
extern crate glam;
extern crate image;
extern crate nalgebra;
extern crate pbr;
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
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use glam::Vec3;
use pbr::ProgressBar;
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


    let mut progress_bar = ProgressBar::new((width * height) as u64);
    progress_bar.show_speed = false;

    let atomic_counter = Arc::new(AtomicU64::new(0));
    let cloned_counter = atomic_counter.clone();

    thread::spawn(move || {
        while cloned_counter.load(Ordering::SeqCst) < (width * height) as u64 {
            let count = cloned_counter.load(Ordering::SeqCst);
            progress_bar.set(count);
            thread::sleep(Duration::from_secs(3));
        }
    });

    let mut pixels = vec![image::Rgb([0, 0, 0]); (width * height) as usize];
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let mut color = Vec3::zero();

        let x = i % width as usize;
        let y = i / width as usize;

        let mut rng = thread_rng();

        (0..samples).for_each(|_| {
            let u = (x as f32 + rand::random::<f32>()) / width as f32;
            let v = (y as f32 + rand::random::<f32>()) / height as f32;
            let ray = camera.get_ray(u, v, &mut rng);
            color += utils::de_nan(&ray::compute_color(ray,
                                                        &world,
                                                        bounces,
                                                        &light_source,
                                                        camera.atmosphere,
                                                        &mut rng));
        });

        color /= samples as f32;

        color.set_x(utils::clamp_rgb(255.0 * utils::gamma_correct(color.x(), 2.2)));
        color.set_y(utils::clamp_rgb(255.0 * utils::gamma_correct(color.y(), 2.2)));
        color.set_z(utils::clamp_rgb(255.0 * utils::gamma_correct(color.z(), 2.2)));

        *pixel = image::Rgb([color.x() as u8, color.y() as u8, color.z() as u8]);

        atomic_counter.fetch_add(1, Ordering::SeqCst);
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
