use glam::Vec3A;


/// Construct an orthonormal basis from a given normal
///
/// Orthonormal bases are used to calculate where to scatter the ray
/// when leaving a hit object. ONBs allow for quick and easy computation for
/// such cases as we can use the normal as one of the vectors.
pub struct OrthonormalBasis {
    axis: [Vec3A; 3],
}

impl OrthonormalBasis {
    /// Create a new orthonormal coordinate frame
    ///
    /// This method implements Listing 3 from the paper listed below:
    ///
    /// Tom Duff, James Burgess, Per Christensen, Christophe Hery, Andrew Kensler,
    /// Max Liani, Ryusuke Villemin: Building an Orthonormal Basis, Revisited,
    /// Journal of Computer Graphics Techniques Vol. 6, No. 1, 2017
    /// https://www.jcgt.org/published/0006/01/01/paper-lowres.pdf
    pub fn new(normal: &Vec3A) -> OrthonormalBasis {
        let w = normal.normalize();

        let sign = 1.0f32.copysign(w.z);
        let a = -1.0 / (sign + w.z);
        let b = w.x * w.y * a;

        let u = Vec3A::new(1.0 + sign * w.x * w.x * a, sign * b, -sign * w.x);
        let v = Vec3A::new(b, sign + w.y * w.y * a, -w.y);

        OrthonormalBasis { axis: [u, v, w] }
    }

    pub fn u(&self) -> Vec3A {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3A {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3A {
        self.axis[2]
    }

    /// Convert the given vector from world space into local space
    pub fn local(&self, v: &Vec3A) -> Vec3A {
        v.x * self.u() + v.y * self.v() + v.z * self.w()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthonormal_frame() {
        use glam::Mat3A;

        let normal = Vec3A::new(0.00038527316, 0.00038460016, -0.99999988079);
        let frame = OrthonormalBasis::new(&normal);
        let matrix = Mat3A::from_cols(frame.axis[0], frame.axis[1], frame.axis[2]);

        assert_eq!(matrix * matrix.transpose(), Mat3A::IDENTITY);
    }
}
