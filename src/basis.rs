use nalgebra::Vector3;

pub struct OrthonormalBase {
    axis: Vec<Vector3<f32>>,
}

impl OrthonormalBase {
    pub fn new(normal: Vector3<f32>) -> OrthonormalBase {
        let w = normal.normalize();

        let t = if w.x.abs() > 0.9 {
            Vector3::new(0.0, 1.0, 0.0)
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };

        let v = (w.cross(&t)).normalize();
        let u = w.cross(&v);

        OrthonormalBase { axis: vec![u, v, w] }
    }

    pub fn u(&self) -> Vector3<f32> {
        self.axis[0]
    }

    pub fn v(&self) -> Vector3<f32> {
        self.axis[1]
    }

    pub fn w(&self) -> Vector3<f32> {
        self.axis[2]
    }

    pub fn local(&self, v: Vector3<f32>) -> Vector3<f32> {
        v.x * self.u() + v.y * self.v() + v.z * self.w()
    }
}
