use glam::Vec3A;

/// A simple atmosphere that can be used in scenes to fill the background
/// when the ray misses an object.
///
/// Was originally handled by the integrator but moving it to its own type
/// will help when we want to create more physically correct atmospheres.
///
/// References for future use:
/// Preetham Analytical Model
/// https://dl.acm.org/doi/pdf/10.1145/311535.311545
/// Hosek-Wilkie Analytical Model
/// https://dl.acm.org/doi/abs/10.1145/2185520.2185591
/// Sebastian Hillaire Sky and Atmosphere Technique
/// https://sebh.github.io/publications/egsr2020.pdf
pub struct Atmosphere {
    background: Vec3A,
    lerp: bool,
}

impl Atmosphere {
    pub fn new(background: Vec3A, lerp: bool) -> Atmosphere {
        Atmosphere { background, lerp }
    }

    pub fn compute_atmosphere_color(&self, direction: &Vec3A) -> Vec3A {
        if self.lerp {
            // a normalized ray has direction [-1, 1] so we add 1 and multiply
            // by 1/2 so that the we can lerp between [0, 1]
            let point: f32 = 0.5 * (direction.y + 1.0);
            // linearly blend between white at 0.0 and self.background at 1.0
            let lerp = (1.0 - point) * Vec3A::splat(1.0) + point * self.background;
            return lerp;
        }

        self.background
    }
}

impl Default for Atmosphere {
    fn default() -> Atmosphere {
        Atmosphere {
            // first implementation used (0.5, 0.7, 1.0) as background color
            background: Vec3A::new(0.5, 0.7, 1.0),
            lerp: true,
        }
    }
}