use std::{
    f64::consts::PI,
    ops::{Add, Div, Mul, Neg, Sub},
};

use rand::Rng;

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn normalized(&self) -> Vec3 {
        *self / self.length()
    }

    pub fn reflect(self, normal: Vec3) -> Vec3 {
        self - (normal * 2.0 * self.dot(&normal))
    }

    pub fn near_zero(&self) -> bool {
        let eps = 1e-3;
        (self.x.abs() < eps) && (self.y.abs() < eps) && (self.z.abs() < eps)
    }

    pub fn zeroes() -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    pub fn random() -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3::new(rng.gen(), rng.gen(), rng.gen())
    }

    pub fn rand_range(min: f64, max: f64) -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3::new(
            rng.gen_range(min..max),
            rng.gen_range(min..max),
            rng.gen_range(min..max),
        )
    }

    pub fn random_dir() -> Vec3 {
        let normal_distr = || {
            let mut rng = rand::thread_rng();
            let theta = 2.0 * PI * rng.gen::<f64>();
            let rho = (-2.0 * (1.0 - rng.gen::<f64>()).ln()).sqrt();
            rho * theta.cos()
        };

        Vec3::new(normal_distr(), normal_distr(), normal_distr()).normalized()
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, other: f64) -> Self::Output {
        Vec3::new(self.x * other, self.y * other, self.z * other)
    }
}

impl Div<Vec3> for Vec3 {
    type Output = Self;
    fn div(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, other: f64) -> Self::Output {
        Vec3::new(self.x / other, self.y / other, self.z / other)
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Vec3) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}
