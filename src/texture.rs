use glam::Vec3A;
use image;


#[derive(Clone)]
/// Housing items here such as Color and ImageTexture allows us to pass
/// Texture as the type for the albedo in materials.
pub enum Texture {
    Color(Color),
    ImageTexture(ImageTexture),
}

impl Texture {
    pub fn sample_texture(&self, u: f32, v: f32, w: &Vec3A) -> Vec3A {
        match self {
            Texture::Color(texture) => texture.sample_texture(u, v, w),
            Texture::ImageTexture(texture) => texture.sample_texture(u, v, w),
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
    Color => Color,
    ImageTexture => ImageTexture
);

#[derive(Clone)]
/// Color is just a wrapping for a Vec3A of RGB values
pub struct Color {
    color: Vec3A,
}

/// Create a new Color which is used for pure albedo materials
impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { color: Vec3A::new(r, g, b) }
    }

    /// Returning the albedo instead of sampling with u and v
    pub fn sample_texture(&self, _u: f32, _v: f32, _p: &Vec3A) -> Vec3A {
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
    pub fn sample_texture(&self, u: f32, v: f32, _p: &Vec3A) -> Vec3A {
        let u_scaled = (u * self.scale) % 1.0;
        let v_scaled = (v * self.scale) % 1.0;

        let i = 0.0f32.max((u_scaled * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v_scaled * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        Vec3A::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}
