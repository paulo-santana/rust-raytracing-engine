use std::ops;

#[derive(Debug)]
pub struct Vec3(pub f64, pub f64, pub f64);
pub use Vec3 as Point3;

impl Vec3 {
    pub fn new(r: f64, g: f64, b: f64) -> Vec3 {
        Vec3(r, g, b)
    }
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
    pub fn z(&self) -> f64 {
        self.2
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        (self.0 * other.0) + (self.1 * other.1) + (self.2 * other.2)
    }
}

impl ops::Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3(-self.0, -self.1, -self.2)
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Self::Output {
        Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl ops::Add<Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Self::Output {
        Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl ops::Add<&Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, other: &Vec3) -> Self::Output {
        Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl ops::AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, other: &Vec3) {
        self.0 += other.0;
        self.1 += other.1;
        self.2 += other.2;
    }
}

impl ops::Sub<&Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, other: &Vec3) -> Self::Output {
        Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl ops::Mul<&Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: &Vec3) -> Self::Output {
        Vec3(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, value: f64) -> Self::Output {
        Vec3(self.0 * value, self.1 * value, self.2 * value)
    }
}

impl ops::Mul<&Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, other: &Vec3) -> Self::Output {
        Vec3(self * other.0, self * other.1, self * other.2)
    }
}

impl ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, value: f64) {
        self.0 *= value;
        self.1 *= value;
        self.2 *= value;
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, value: f64) -> Self::Output {
        Vec3(self.0 / value, self.1 / value, self.2 / value)
    }
}

impl ops::DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, value: f64) {
        self.0 /= value;
        self.1 /= value;
        self.2 /= value;
    }
}
