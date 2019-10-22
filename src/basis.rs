use nalgebra::Vector3;

pub struct OrthonormalBase {
    axis: Vec<Vector3<f32>>,
}

impl OrthonormalBase {
    /// Create a new orthonormal coordinate frame
    ///
    /// This method implements Listing 3 from the paper listed below:
    ///
    /// Tom Duff, James Burgess, Per Christensen, Christophe Hery, Andrew Kensler,
    /// Max Liani, Ryusuke Villemin: Building an Orthonormal Basis, Revisited,
    /// Journal of Computer Graphics Techniques Vol. 6, No. 1, 2017 http://jcgt.org
    pub fn new(normal: &Vector3<f32>) -> OrthonormalBase {
        let w = normal.normalize();

        let sign = 1.0f32.copysign(w.z);
        let a = -1.0 / (sign + w.z);
        let b = w.x * w.y * a;

        let u = Vector3::new(1.0 + sign * w.x * w.x * a, sign * b, -sign * w.x);
        let v = Vector3::new(b, sign + w.y * w.y * a, -w.y);

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

    pub fn local(&self, v: &Vector3<f32>) -> Vector3<f32> {
        v.x * self.u() + v.y * self.v() + v.z * self.w()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthonormal_frame() {
        use nalgebra::Matrix3;

        let normal = Vector3::new(0.00038527316, 0.00038460016, -0.99999988079);
        let frame = OrthonormalBase::new(&normal);
        let matrix = Matrix3::from_columns(&frame.axis);

        assert_eq!(matrix * matrix.transpose(), Matrix3::identity());
    }
}
