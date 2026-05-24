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
pub mod ggx;
mod integrator;
mod lights;
pub mod materials;
pub mod pdf;
mod plane;
mod primitive;
mod ray;
mod rectangle;
mod results;
pub mod sampling;
mod scene;
pub mod scenes;
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

use integrator::Integrator;
use scenes::Scenes;
use scene::Scene;
use utils::de_nan;

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

    let mut scene_rng = if cfg!(feature = "tests") {
        Pcg64Mcg::seed_from_u64(0)
    } else {
        Pcg64Mcg::from_rng(&mut rng())
    };

    let integrator: Integrator = args.integrator;
    let samples: usize = args.samples.unwrap_or(integrator.default_samples());
    let scene: Scene = args.scene.load(args.width, args.height, &mut scene_rng);
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

    let mut pixels = vec![0.0f32; 3 * width * height];

    pixels.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
        let mut color = Vec3A::ZERO;

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

                color += de_nan(&integrator.render_scene(ray, &scene, &mut rng));

        })});

        color /= (samples_sqrt * samples_sqrt) as f32;

        pixel[0] = color.x;
        pixel[1] = color.y;
        pixel[2] = color.z;

        atomic_counter.fetch_add(1, Ordering::SeqCst);
    });

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

        let denoised_output = denoise(&pixels, width, height);
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
