use std::ops;
#[cfg(feature = "portable_simd")]
use std::simd::{f64x4, SimdFloat};

#[derive(Debug, Clone, Copy)]
pub struct Vec3(pub [f64; 4]);
pub use Vec3 as Point3;

impl Vec3 {
    pub fn unit_vector(v: &Vec3) -> Vec3 {
        v.clone() / v.length()
    }
    pub fn new(r: f64, g: f64, b: f64) -> Vec3 {
        Vec3([r, g, b, 0.0])
    }
    pub fn x(&self) -> f64 {
        self.0[0]
    }
    pub fn y(&self) -> f64 {
        self.0[1]
    }
    pub fn z(&self) -> f64 {
        self.0[2]
    }

    pub fn length(&self) -> f64 {
        self.lenght_squared().sqrt()
    }

    pub fn lenght_squared(&self) -> f64 {
        self.dot(self)
    }
}

#[cfg(feature = "portable_simd")]
impl Vec3 {
    pub fn dot(&self, other: &Vec3) -> f64 {
        let me = f64x4::from_array(self.to_array());
        let other = f64x4::from_array(other.to_array());
        (me * other).reduce_sum()
        // (self.0 * other.0) + (self.1 * other.1) + (self.2 * other.2)
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        let a_left = f64x4::from_array([self.0[1], self.0[2], self.0[0], 0.0]);
        let a_right = f64x4::from_array([other.0[2], other.0[0], other.0[1], 0.0]);
        let b_left = f64x4::from_array([self.0[2], self.0[0], self.0[1], 0.0]);
        let b_right = f64x4::from_array([other.0[1], other.0[2], other.0[0], 0.0]);
        let a = a_left * a_right;
        let b = b_left * b_right;
        Vec3::from_array((a - b).as_array())
    }

    pub fn from_array(array: &[f64; 4]) -> Vec3 {
        Vec3(*array)
    }

    pub fn to_array(&self) -> [f64; 4] {
        self.0
    }

    pub(crate) fn as_simd(&self) -> f64x4 {
        f64x4::from_array(self.0)
    }
}

#[cfg(not(feature = "portable_simd"))]
impl Vec3 {
    pub fn dot(&self, other: &Vec3) -> f64 {
        (self.0[0] * other.0[0]) + (self.0[1] * other.0[1]) + (self.0[2] * other.0[2])
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3([
            self.0[1] * other.0[2] - self.0[2] * other.0[1],
            self.0[2] * other.0[0] - self.0[0] * other.0[2],
            self.0[0] * other.0[1] - self.0[1] * other.0[0],
            0.0,
        ])
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    #[cfg(feature = "portable_simd")]
    fn neg(self) -> Self::Output {
        let simd = self.as_simd();
        Vec3::from_array((-simd).as_array())
    }

    #[cfg(not(feature = "portable_simd"))]
    fn neg(self) -> Self::Output {
        Vec3([-self.0[0], -self.0[1], -self.0[2], 0.0])
    }
}

// impl ops::Neg for &Vec3 {
//     type Output = Vec3;
//
//     fn neg(self) -> Self::Output {
//         Vec3(-self.0, -self.1, -self.2)
//     }
// }

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    #[cfg(feature = "portable_simd")]
    fn add(self, other: Vec3) -> Self::Output {
        let a = self.as_simd();
        let b = other.as_simd();
        Vec3::from_array((a + b).as_array())
    }

    #[cfg(not(feature = "portable_simd"))]
    fn add(self, other: Vec3) -> Self::Output {
        Vec3([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            0.0,
        ])
    }
}

impl ops::Add<f64> for Vec3 {
    type Output = Vec3;

    #[cfg(feature = "portable_simd")]
    fn add(self, value: f64) -> Self::Output {
        let a = self.as_simd();
        let b = f64x4::splat(value);
        // b[3] = 0.0;
        Vec3::from_array((a + b).as_array())
    }

    #[cfg(not(feature = "portable_simd"))]
    fn add(self, value: f64) -> Self::Output {
        Vec3([self.0[0] + value, self.0[1] + value, self.0[2] + value, 0.0])
    }
}

// impl ops::Add<Vec3> for &Vec3 {
//     type Output = Vec3;
//
//     fn add(self, other: Vec3) -> Self::Output {
//         Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
//     }
// }

// impl ops::Add<&Vec3> for Vec3 {
//     type Output = Vec3;
//
//     fn add(self, other: &Vec3) -> Self::Output {
//         Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
//     }
// }

// impl ops::AddAssign<&Vec3> for Vec3 {
//     fn add_assign(&mut self, other: &Vec3) {
//         self.0 += other.0;
//         self.1 += other.1;
//         self.2 += other.2;
//     }
// }

// impl ops::Sub<Vec3> for &Vec3 {
//     type Output = Vec3;
//
//     fn sub(self, other: Vec3) -> Self::Output {
//         Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2)
//     }
// }

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    #[cfg(feature = "portable_simd")]
    fn sub(self, other: Vec3) -> Self::Output {
        let a = self.as_simd();
        let b = other.as_simd();
        Vec3::from_array((a - b).as_array())
    }

    #[cfg(not(feature = "portable_simd"))]
    fn sub(self, other: Vec3) -> Self::Output {
        Vec3([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            0.0,
        ])
    }
}

// impl ops::Sub<&Vec3> for Vec3 {
//     type Output = Vec3;
//
//     fn sub(self, other: &Vec3) -> Self::Output {
//         Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2)
//     }
// }

// impl ops::Mul<&Vec3> for Vec3 {
//     type Output = Vec3;
//
//     fn mul(self, other: &Vec3) -> Self::Output {
//         Vec3(self.0 * other.0, self.1 * other.1, self.2 * other.2)
//     }
// }

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    #[cfg(feature = "portable_simd")]
    fn mul(self, value: f64) -> Self::Output {
        let a = self.as_simd();
        let b = f64x4::splat(value);
        Vec3::from_array((a * b).as_array())
    }

    #[cfg(not(feature = "portable_simd"))]
    fn mul(self, value: f64) -> Self::Output {
        Vec3([self.0[0] * value, self.0[1] * value, self.0[2] * value, 0.0])
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    #[cfg(feature = "portable_simd")]
    fn mul(self, vec: Vec3) -> Self::Output {
        let a = f64x4::splat(self);
        let b = vec.as_simd();
        Vec3::from_array((a * b).as_array())
    }

    #[cfg(not(feature = "portable_simd"))]
    fn mul(self, vec: Vec3) -> Self::Output {
        Vec3([self * vec.0[0], self * vec.0[1], self * vec.0[2], 0.0])
    }
}

// impl ops::Mul<&Vec3> for f64 {
//     type Output = Vec3;
//
//     fn mul(self, vec: &Vec3) -> Self::Output {
//         Vec3(self * vec.0, self * vec.1, self * vec.2)
//     }
// }

impl ops::MulAssign<f64> for Vec3 {
    #[cfg(feature = "portable_simd")]
    fn mul_assign(&mut self, value: f64) {
        let a = self.as_simd();
        let b = f64x4::splat(value);
        self.0 = (a * b).to_array();
    }

    #[cfg(not(feature = "portable_simd"))]
    fn mul_assign(&mut self, value: f64) {
        self.0[0] *= value;
        self.0[1] *= value;
        self.0[2] *= value;
    }
}

// impl ops::Div<f64> for &Vec3 {
//     type Output = Vec3;
//
//     fn div(self, value: f64) -> Self::Output {
//         Vec3(self.0 / value, self.1 / value, self.2 / value)
//     }
// }

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;
    #[cfg(feature = "portable_simd")]
    fn div(self, value: f64) -> Self::Output {
        let a = self.as_simd();
        let b = f64x4::splat(value);
        Vec3::from_array((a / b).as_array())
        // Vec3(self.0 / value, self.1 / value, self.2 / value)
    }

    #[cfg(not(feature = "portable_simd"))]
    fn div(self, value: f64) -> Self::Output {
        Vec3([self.0[0] / value, self.0[1] / value, self.0[2] / value, 0.0])
    }
}

impl ops::DivAssign<f64> for Vec3 {
    #[cfg(feature = "portable_simd")]
    fn div_assign(&mut self, value: f64) {
        let a = self.as_simd();
        let b = f64x4::splat(value);
        self.0 = (a / b).to_array();
    }

    #[cfg(not(feature = "portable_simd"))]
    fn div_assign(&mut self, value: f64) {
        self.0[0] /= value;
        self.0[1] /= value;
        self.0[2] /= value;
    }
}
