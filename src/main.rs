#![allow(dead_code)]

extern crate chrono;
extern crate glam;
extern crate image;
extern crate pbr;
extern crate rand;
extern crate rand_distr;
extern crate rayon;
extern crate tobj;
extern crate wide;

mod aabb;
mod basis;
mod bvh;
mod camera;
mod denoise;
mod events;
mod geometry;
mod ggx;
mod integrator;
mod lights;
mod materials;
mod pdf;
mod plane;
mod post;
mod ray;
mod rectangle;
mod sampling;
mod scenarios;
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
use rand::RngExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use glam::Vec3A;
use image::{ImageBuffer, Rgb};
use pbr::ProgressBar;
use rand::rng;
use rayon::prelude::*;

#[cfg(feature = "denoise")]
use denoise::denoise;

fn main() {
    let rendering_time = Instant::now();

    let args: Vec<String> = env::args().collect();
    let samples: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(128);
    let bounces: u32 = 10;
    let (width, height): (usize, usize) = (2048, 2048);

    let scene = scenarios::cornell_box_object_scene(width, height);

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

    // compute a vec for albedo and normal buffers so that we can pass them to OIDN
    // 9 floats per pixel: [color RGB | albedo RGB | normal RGB]
    let mut combined = vec![0.0f32; 9 * width * height];
    combined.par_chunks_mut(9).enumerate().for_each(|(i, chunk)| {
        let mut color = Vec3A::ZERO;
        let mut albedo = Vec3A::ZERO;
        let mut normal = Vec3A::ZERO;

        let x = i % width;
        let y = height - (i / width) - 1;

        let mut rng = rng();
        let samples_sqrt = samples.isqrt();
        let step = 1.0 / samples_sqrt as f32;

        (0..samples_sqrt).for_each(|i| {
            (0..samples_sqrt).for_each(|j| {
                let u = (x as f32 + (i as f32 + rng.random::<f32>()) * step) / width as f32;
                let v = (y as f32 + (j as f32 + rng.random::<f32>()) * step) / height as f32;

                let ray = scene.camera.get_ray(u, v, &mut rng);

                // render_normals is used for debugging
                // color += utils::de_nan(&integrator::render_normals(ray, &scene));

                // old pure path tracer with hybrid pdf
                // color += utils::de_nan(&integrator::render_path_integrator(ray, &scene, bounces, &mut rng));

                let (c, a, n) = integrator::render_nee_integrator(ray, &scene, bounces, &mut rng);
                color += utils::de_nan(&c);
                albedo += a;
                normal += n;
        })});

        color /= samples as f32;
        albedo /= samples as f32;
        normal /= samples as f32;

        chunk[0] = color.x;  chunk[1] = color.y;  chunk[2] = color.z;
        chunk[3] = albedo.x; chunk[4] = albedo.y; chunk[5] = albedo.z;
        chunk[6] = normal.x; chunk[7] = normal.y; chunk[8] = normal.z;

        atomic_counter.fetch_add(1, Ordering::SeqCst);
    });

    let mut pixels = vec![0.0f32; 3 * width * height];
    let mut albedo_pixels = vec![0.0f32; 3 * width * height];
    let mut normal_pixels = vec![0.0f32; 3 * width * height];
    for (i, chunk) in combined.chunks(9).enumerate() {
        pixels[3*i..3*i+3].copy_from_slice(&chunk[0..3]);
        albedo_pixels[3*i..3*i+3].copy_from_slice(&chunk[3..6]);
        normal_pixels[3*i..3*i+3].copy_from_slice(&chunk[6..9]);
    }

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

        let denoised_output = denoise(&pixels, &albedo_pixels, &normal_pixels, width, height);

        let denoise_end_time: DateTime<Local> = Local::now();
        println!("[{}] Finished denoising in {}. Render saved to denoised_render.exr.",
                 denoise_end_time.format("%H:%M:%S"),
                 utils::format_time(denoising_time.elapsed()));

        let denoised_buffer: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::from_raw(width as u32, height as u32, denoised_output).unwrap();

        denoised_buffer.save("denoised_render.exr").unwrap();
    }
}
