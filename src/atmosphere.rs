use glam::Vec3A;

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
            let point: f32 = 0.5 * (direction.y + 1.0);
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