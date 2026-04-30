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
mod geometry;
mod hitable;
mod integrator;
mod materials;
mod pdf;
mod plane;
mod post;
mod ray;
mod rectangle;
mod sampling;
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
use rand::Rng;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use glam::Vec3;
use image::{ImageBuffer, Rgb};
use pbr::ProgressBar;
use rand::thread_rng;
use rayon::prelude::*;

#[cfg(feature = "denoise")]
use denoise::denoise;

fn main() {
    let rendering_time = Instant::now();

    let args: Vec<String> = env::args().collect();
    let samples: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(128);
    let bounces: u32 = 10;
    let (width, height): (usize, usize) = (2048, 2048);

    let scene = scene::cornell_box_object_scene(width, height);

    let render_start_time: DateTime<Local> = Local::now();
    println!("[{}] Rendering '{}' scene with {} samples at {} x {} dimensions...",
             render_start_time.format("%H:%M:%S"),
             &scene.name,
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
            thread::sleep(Duration::from_millis(200));
        }
    });

    let mut pixels = vec![0.0f32; 3 * width * height];
    pixels.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
        let mut color = Vec3::ZERO;

        let x = i % width;
        let y = height - (i / width) - 1;

        let mut rng = thread_rng();

        (0..samples).for_each(|_| {
            let u = (x as f32 + rng.gen::<f32>()) / width as f32;
            let v = (y as f32 + rng.gen::<f32>()) / height as f32;
            let ray = scene.camera.get_ray(u, v, &mut rng);

            // render_normals is used for debugging
            // color += utils::de_nan(&integrator::render_normals(ray, &scene));

            color += utils::de_nan(&integrator::render_path_integrator(ray, &scene, bounces, &mut rng));
        });

        color /= samples as f32;

        pixel[0] = color.x;
        pixel[1] = color.y;
        pixel[2] = color.z;

        atomic_counter.fetch_add(1, Ordering::SeqCst);
    });

    let render_end_time: DateTime<Local> = Local::now();
    println!("[{}] Finished rendering in {}. Render saved to render.exr.",
             render_end_time.format("%H:%M:%S"),
             utils::format_time(rendering_time.elapsed()));

    let buffer: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::from_raw(width as u32, height as u32, pixels.clone()).unwrap();

    buffer.save("render.exr").unwrap();

    #[cfg(feature = "denoise")]
    {
        let denoising_time = Instant::now();
        let denoise_start_time: DateTime<Local> = Local::now();
        println!("[{}] Denoising image...",
                 denoise_start_time.format("%H:%M:%S"));

        let denoised_output = denoise(&pixels, width, height);

        let denoise_end_time: DateTime<Local> = Local::now();
        println!("[{}] Finished denoising in {}. Render saved to denoised_render.exr.",
                 denoise_end_time.format("%H:%M:%S"),
                 utils::format_time(denoising_time.elapsed()));

        let denoised_buffer: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::from_raw(width as u32, height as u32, denoised_output).unwrap();

        denoised_buffer.save("denoised_render.exr").unwrap();
    }
}
