use f32::consts::PI;

use glam::Vec3A;
use image;
use rand::rngs::ThreadRng;
use rand::RngExt;

#[derive(Clone)]
pub enum Texture {
    SolidColor(SolidColor),
    ImageTexture(ImageTexture),
    EnvironmentMap(EnvironmentMap),
}

impl Texture {
    pub fn value(&self, u: f32, v: f32, w: &Vec3A) -> Vec3A {
        match self {
            Texture::SolidColor(texture) => texture.value(u, v, w),
            Texture::ImageTexture(texture) => texture.value(u, v, w),
            Texture::EnvironmentMap(texture) => texture.value(u, v, w),
        }
    }

    /// Returns the solid-angle PDF for sampling the given direction, only if
    /// this texture is an importance-sampled EnvironmentMap.
    pub fn env_pdf_value(&self, direction: &Vec3A) -> Option<f32> {
        match self {
            Texture::EnvironmentMap(env) => Some(env.pdf_value(direction)),
            _ => None
        }
    }

    /// Samples a direction importance-weighted by luminance, only if this
    /// texture is an EnvironmentMap.
    pub fn env_pdf_random(&self, rng: &mut ThreadRng) -> Option<Vec3A> {
        match self {
            Texture::EnvironmentMap(env) => Some(env.pdf_random(rng)),
            _ => None
        }
    }
}

macro_rules! impl_from_texture {
    ($($t:ty => $v:ident),*) => {
        $(
            impl From<$t> for Texture {
                fn from(m: $t) -> Self {
                    Texture::$v(m)
                }
            }
        )*
    };
}

impl_from_texture!(
    SolidColor => SolidColor,
    ImageTexture => ImageTexture,
    EnvironmentMap => EnvironmentMap
);

#[derive(Clone)]
/// SolidColor is just a wrapping for a Vec3A of RGB values
pub struct SolidColor {
    color: Vec3A,
}

/// Create a new SolidColor which is used for pure albedo materials
impl SolidColor {
    pub fn new(r: f32, g: f32, b: f32) -> SolidColor {
        SolidColor { color: Vec3A::new(r, g, b) }
    }

    /// Returning the albedo instead of sampling with u and v
    pub fn value(&self, _u: f32, _v: f32, _p: &Vec3A) -> Vec3A {
        self.color
    }
}

#[derive(Clone)]
/// ImageTexture is a struct for textures loaded from file
pub struct ImageTexture {
    im: image::RgbImage,
    scale: f32,
}

/// Create a new texture from the given data and image dimensions
impl ImageTexture {
    pub fn new(filename: &str, scale: f32) -> ImageTexture {
        ImageTexture { im: image::open(filename).unwrap().flipv().to_rgb8(), scale }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn value(&self, u: f32, v: f32, _p: &Vec3A) -> Vec3A {
        let u_scaled = (u * self.scale) % 1.0;
        let v_scaled = (v * self.scale) % 1.0;

        let i = 0.0f32.max((u_scaled * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v_scaled * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        Vec3A::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}

fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Binary search a normalized CDF (cdf[0] = 0, cdf[n] = 1) for uniform sample u.
/// Returns the bin index k such that cdf[k] <= u < cdf[k+1].
fn sample_cdf(cdf: &[f32], u: f32) -> usize {
    let mut lo = 0usize;
    let mut hi = cdf.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if cdf[mid] <= u { lo = mid + 1; } else { hi = mid; }
    }
    lo.saturating_sub(1).min(cdf.len().saturating_sub(2))
}

/// EnvironmentMap is a struct for loading an HDR environment map from an EXR file.
///
/// At construction time a 2-D luminance CDF is built over the image pixels
/// (weighted by sin(theta) to correct for equirectangular pole distortion).
/// This enables O(log n) importance sampling that directs samples toward
/// bright regions like the sun, eliminating the fireflies that appear when
/// a diffuse path happens to scatter toward a 100 000-nit pixel.
#[derive(Clone)]
pub struct EnvironmentMap {
    im: image::Rgb32FImage,
    width: usize,
    height: usize,
    /// Row CDF — marginal distribution p(v). Length = height + 1.
    marginal_cdf: Vec<f32>,
    /// Per-row column CDFs — conditional distribution p(u|v).
    /// Row j occupies indices [j*(width+1) .. j*(width+1)+width+1].
    conditional_cdf: Vec<f32>,
    /// Raw sum Σ luminance*sin_theta used in the solid-angle PDF formula.
    total_weight: f32,
}

impl EnvironmentMap {
    pub fn new(filename: &str) -> EnvironmentMap {
        let im = image::open(filename).unwrap().to_rgb32f();
        let width = im.width() as usize;
        let height = im.height() as usize;

        let mut conditional_cdf = vec![0.0f32; height * (width + 1)];
        let mut marginal_weights = vec![0.0f32; height];

        for j in 0..height {
            // sin_theta corrects for the shrinking pixel area toward the poles
            let sin_theta = (PI * (j as f32 + 0.5) / height as f32).sin();
            let row = j * (width + 1);
            conditional_cdf[row] = 0.0;
            for i in 0..width {
                let pixel = im.get_pixel(i as u32, j as u32);
                let w = luminance(pixel[0], pixel[1], pixel[2]) * sin_theta;
                conditional_cdf[row + i + 1] = conditional_cdf[row + i] + w;
            }
            marginal_weights[j] = conditional_cdf[row + width];
            let row_sum = marginal_weights[j];
            if row_sum > 0.0 {
                for i in 1..=width {
                    conditional_cdf[row + i] /= row_sum;
                }
            }
        }

        let mut marginal_cdf = vec![0.0f32; height + 1];
        for j in 0..height {
            marginal_cdf[j + 1] = marginal_cdf[j] + marginal_weights[j];
        }
        let total_weight = marginal_cdf[height];
        if total_weight > 0.0 {
            for j in 1..=height {
                marginal_cdf[j] /= total_weight;
            }
        }

        EnvironmentMap { im, width, height, marginal_cdf, conditional_cdf, total_weight }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn value(&self, _u: f32, _v: f32, direction: &Vec3A) -> Vec3A {
        let u = 0.5 + direction.z.atan2(direction.x) / (2.0 * PI);
        let v = 0.5 - direction.y.asin() / PI;

        let i = 0.0f32.max((u * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        Vec3A::new(r, g, b)
    }

    /// Solid-angle PDF for the given direction.
    ///
    /// Derivation: p(ω) = L * W * H / (total_weight * 2π²).
    /// The sin_theta factor from the area element cancels with the sin_theta
    /// in the CDF weight, leaving only the raw luminance scaled by resolution.
    pub fn pdf_value(&self, direction: &Vec3A) -> f32 {
        if self.total_weight <= 0.0 { return 0.0; }

        let u = 0.5 + direction.z.atan2(direction.x) / (2.0 * PI);
        let v = 0.5 - direction.y.asin() / PI;

        let i = ((u * self.width as f32) as usize).min(self.width - 1);
        let j = ((v * self.height as f32) as usize).min(self.height - 1);

        let pixel = self.im.get_pixel(i as u32, j as u32);
        let lum = luminance(pixel[0], pixel[1], pixel[2]);

        lum * self.width as f32 * self.height as f32 / (self.total_weight * 2.0 * PI * PI)
    }

    /// Sample a direction from the environment map proportional to luminance.
    pub fn pdf_random(&self, rng: &mut ThreadRng) -> Vec3A {
        let u1 = rng.random::<f32>();
        let u2 = rng.random::<f32>();

        let j = sample_cdf(&self.marginal_cdf, u1);
        let row = j * (self.width + 1);
        let i = sample_cdf(&self.conditional_cdf[row..row + self.width + 1], u2);

        // Convert pixel center to spherical direction (inverse of value() mapping)
        let u = (i as f32 + 0.5) / self.width as f32;
        let v = (j as f32 + 0.5) / self.height as f32;

        let phi = (u - 0.5) * 2.0 * PI;       // azimuth: atan2(z, x)
        let elevation = (0.5 - v) * PI;         // elevation: asin(y)
        let (sin_el, cos_el) = elevation.sin_cos();
        let (sin_phi, cos_phi) = phi.sin_cos();
        Vec3A::new(cos_el * cos_phi, sin_el, cos_el * sin_phi)
    }
}
