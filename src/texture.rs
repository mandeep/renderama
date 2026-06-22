use glam::{Vec2, Vec3A};
use image::DynamicImage;

#[derive(Clone, Copy)]
/// TextureId is used to index into the textures Vec that is instantiated
/// at scene creation.
///
/// Keeping track of the index instead of the actual texture
/// saves from allocating memory unnecessarily.
pub struct TextureId(pub u32);

impl TextureId {
    /// Create a new TextureId with material at index
    pub fn new(index: u32) -> TextureId {
        TextureId(index)
    }

    /// Retrieve the TextureId as a usize for indexing purposes
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[macro_export]
macro_rules! tex {
    ($vec:expr, $texture:expr) => {{
        let id = $crate::texture::TextureId::new($vec.len() as u32);
        $vec.push($texture.into());
        id
    }};
}

#[derive(Clone)]
/// Housing items here such as Color and ImageTexture allows us to pass
/// Texture as the type for the albedo in materials.
pub enum Texture {
    Color(Color),
    ImageTexture(ImageTexture),
    ImageTextureMap(ImageTextureMap)
}

impl Texture {
    pub fn sample_texture(&self, u: f32, v: f32, w: &Vec3A) -> Vec3A {
        match self {
            Texture::Color(texture) => texture.sample_texture(),
            Texture::ImageTexture(texture) => texture.sample_texture(u, v, w),
            Texture::ImageTextureMap(texture) => texture.sample_texture(u, v, w),
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
    ImageTexture => ImageTexture,
    ImageTextureMap => ImageTextureMap
);

#[derive(Copy, Clone)]
/// Color is just a wrapping for a Vec3A of RGB values
pub struct Color {
    color: Vec3A,
}

/// Create a new Color which is used for pure albedo materials
impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { color: Vec3A::new(r, g, b) }
    }

    pub fn splat(value: f32) -> Color {
        Color { color: Vec3A::splat(value) }
    }

    /// Returning the albedo instead of sampling with u and v
    pub fn sample_texture(&self) -> Vec3A {
        self.color
    }
}

#[derive(Clone)]
/// ImageTexture is a struct for textures loaded from file
pub struct ImageTexture {
    im: image::RgbImage,
    scale: Vec2,
}

/// Create a new texture from the given data and image dimensions
impl ImageTexture {
    pub fn new(filename: &str, scale: Vec2) -> ImageTexture {
        let img = image::open(filename).unwrap_or_else(|e| {
            eprintln!("Failed to open texture '{}': {}", filename, e);
            DynamicImage::new_rgb8(1, 1)
        });
        ImageTexture { im: img.to_rgb8(), scale }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn sample_texture(&self, u: f32, v: f32, _p: &Vec3A) -> Vec3A {
        // need to use rem_euclid to properly wrap negative uv coordinates
        // https://doc.rust-lang.org/std/primitive.f32.html#method.rem_euclid
        let inverted_v = 1.0 - v;
        let u_scaled = (u * self.scale.x).rem_euclid(1.0);
        let v_scaled = (inverted_v * self.scale.y).rem_euclid(1.0);

        let i = 0.0f32.max((u_scaled * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v_scaled * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        // need to convert from srgb color space to linear as the
        // renderer computes color in linear color space
        let linear_r = (r as f32 / 255.0).powf(2.2);
        let linear_g = (g as f32 / 255.0).powf(2.2);
        let linear_b = (b as f32 / 255.0).powf(2.2);
        Vec3A::new(linear_r, linear_g, linear_b)

    }
}

#[derive(Clone)]
/// ImageTexture is a struct for textures loaded from file
pub struct ImageTextureMap {
    im: image::RgbImage,
    scale: Vec2,
}

/// Create a new texture from the given data and image dimensions
impl ImageTextureMap {
    pub fn new(filename: &str, scale: Vec2) -> ImageTextureMap {
        let img = image::open(filename).unwrap_or_else(|e| {
            eprintln!("Failed to open texture '{}': {}", filename, e);
            DynamicImage::new_rgb8(1, 1)
        });
        ImageTextureMap { im: img.to_rgb8(), scale }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn sample_texture(&self, u: f32, v: f32, _p: &Vec3A) -> Vec3A {
        // need to use rem_euclid to properly wrap negative uv coordinates
        // https://doc.rust-lang.org/std/primitive.f32.html#method.rem_euclid
        let inverted_v = 1.0 - v;
        let u_scaled = (u * self.scale.x).rem_euclid(1.0);
        let v_scaled = (inverted_v * self.scale.y).rem_euclid(1.0);

        let i = 0.0f32.max((u_scaled * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v_scaled * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        Vec3A::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)

    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splat() {
        let color = Color::splat(0.375);
        let sample = color.sample_texture();

        assert_eq!(sample.x, 0.375);
        assert_eq!(sample.y, 0.375);
        assert_eq!(sample.z, 0.375);
    }
}