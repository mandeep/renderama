extern crate chrono;
extern crate clap;
extern crate glam;
extern crate image;
extern crate pbr;
extern crate rand;
extern crate rand_distr;
extern crate rand_pcg;
extern crate rayon;
extern crate tobj;
extern crate wide;

mod aabb;
mod atmosphere;
mod basis;
mod bvh;
mod camera;
mod denoise;
mod environment;
mod events;
mod ggx;
mod integrator;
mod lights;
mod materials;
mod pdf;
mod plane;
mod primitive;
mod ray;
mod rectangle;
mod sampling;
mod scene;
mod scenes;
mod sphere;
mod texture;
mod transformations;
mod triangle;
mod utils;
mod volume;

use std::f32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use clap::Parser;
use glam::Vec3A;
use image::{ImageBuffer, Rgb};
use pbr::ProgressBar;
use rand::{rng, RngExt, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

use integrator::{Integrator, render_beauty, render_normals};
use scenes::Scenes;
use scene::Scene;

#[cfg(feature = "denoise")]
use denoise::denoise;

#[derive(Parser)]
#[command(name = "Renderama", version, about)]
struct Args {
    #[arg(long, default_value = "cornell_box_objects")]
    scene: Scenes,

    #[arg(long)]
    samples: Option<usize>,

    #[arg(long)]
    width: Option<usize>,

    #[arg(long)]
    height: Option<usize>,

    #[arg(long)]
    output: Option<String>,

    #[arg(long, default_value = "beauty")]
    integrator: Integrator,
}

fn main() {
    let rendering_time = Instant::now();

    let args = Args::parse();

    let integrator: Integrator = args.integrator;
    let samples: usize = args.samples.unwrap_or(integrator.default_samples());
    let scene: Scene = args.scene.load(args.width, args.height);
    let (width, height) = (scene.camera.resolution.0 as usize, scene.camera.resolution.1 as usize);
    let output_path = args.output;

    println!("[{}] Rendering '{}' scene with {} samples at {} x {} dimensions...",
             Local::now().format("%H:%M:%S"),
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

        // for testing purposes seed the rng from a u64
        // need to seed per pixel otherwise it will create the same RNG for every pixel and ray
        let mut rng = if cfg!(feature = "tests") {
            let seed = (y * width + x) as u64;
            Pcg64Mcg::seed_from_u64(seed)
        } else {
            Pcg64Mcg::from_rng(&mut rng())
        };

        let samples_sqrt = samples.isqrt();
        let step = 1.0 / samples_sqrt as f32;

        (0..samples_sqrt).for_each(|i| {
            (0..samples_sqrt).for_each(|j| {
                let u = (x as f32 + (i as f32 + rng.random::<f32>()) * step) / width as f32;
                let v = (y as f32 + (j as f32 + rng.random::<f32>()) * step) / height as f32;

                let ray = scene.camera.generate_ray(u, v, &mut rng);

                match integrator {
                    Integrator::Beauty => {
                        let (color_sample, albedo_sample, normal_sample) =
                            render_beauty(ray, &scene, &mut rng);

                        color += utils::de_nan(&color_sample);
                        albedo += albedo_sample;
                        normal += normal_sample;
                    },
                    Integrator::Normals => {
                        // render_normals is used for debugging
                        color += utils::de_nan(&render_normals(ray, &scene));
                    }
                }
        })});

        color /= (samples_sqrt * samples_sqrt) as f32;
        albedo /= (samples_sqrt * samples_sqrt) as f32;
        normal /= (samples_sqrt * samples_sqrt) as f32;

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

    let buffer: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::from_raw(width as u32, height as u32, pixels.clone()).unwrap();

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let filepath = output_path.unwrap_or_else(|| format!("render_{}.exr", timestamp));

    buffer.save(&filepath).unwrap();

    println!("[{}] Finished rendering in {}. Render saved to {}.",
            Local::now().format("%H:%M:%S"),
            utils::format_time(rendering_time.elapsed()),
            &filepath,
            );

    #[cfg(feature = "denoise")]
    {
        let denoising_time = Instant::now();

        println!("[{}] Denoising image...",
                 Local::now().format("%H:%M:%S"));

        let denoised_output = denoise(&pixels, &albedo_pixels, &normal_pixels, width, height);
        let denoised_buffer: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::from_raw(width as u32, height as u32, denoised_output).unwrap();

        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let filepath = format!("denoised_render_{}.exr", timestamp);

        denoised_buffer.save(&filepath).unwrap();

        println!("[{}] Finished denoising in {}. Render saved to {}.",
                 Local::now().format("%H:%M:%S"),
                 utils::format_time(denoising_time.elapsed()),
                 &filepath,
                );
    }
}
