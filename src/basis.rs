use glam::Vec3;

pub struct OrthonormalBasis {
    axis: Vec<Vec3>,
}

impl OrthonormalBasis {
    /// Create a new orthonormal coordinate frame
    ///
    /// This method implements Listing 3 from the paper listed below:
    ///
    /// Tom Duff, James Burgess, Per Christensen, Christophe Hery, Andrew Kensler,
    /// Max Liani, Ryusuke Villemin: Building an Orthonormal Basis, Revisited,
    /// Journal of Computer Graphics Techniques Vol. 6, No. 1, 2017 http://jcgt.org
    pub fn new(normal: &Vec3) -> OrthonormalBasis {
        let w = normal.normalize();

        let sign = 1.0f32.copysign(w.z());
        let a = -1.0 / (sign + w.z());
        let b = w.x() * w.y() * a;

        let u = Vec3::new(1.0 + sign * w.x() * w.x() * a, sign * b, -sign * w.x());
        let v = Vec3::new(b, sign + w.y() * w.y() * a, -w.y());

        OrthonormalBasis { axis: vec![u, v, w] }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn local(&self, v: &Vec3) -> Vec3 {
        v.x() * self.u() + v.y() * self.v() + v.z() * self.w()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthonormal_frame() {
        use glam::Mat3;

        let normal = Vec3::new(0.00038527316, 0.00038460016, -0.99999988079);
        let frame = OrthonormalBasis::new(&normal);
        let matrix = Mat3::from_cols(frame.axis[0], frame.axis[1], frame.axis[2]);

        assert_eq!(matrix * matrix.transpose(), Mat3::identity());
    }
}
