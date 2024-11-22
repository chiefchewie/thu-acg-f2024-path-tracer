use std::ops::{Add, Div, Mul, Neg, Sub};

use rand::Rng;

use crate::utils::normal_dist;

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    v: [f64; 3]
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 {v:[x,y,z]}
    }

    pub fn x(&self) -> f64 {
        self.v[0]
    }

    pub fn y(&self) -> f64 {
        self.v[1]
    }

    pub fn z(&self) -> f64 {
        self.v[2]
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.v[0] * self.v[0] + self.v[1] * self.v[1]+ self.v[2] * self.v[2]
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.v[0] * other.v[0] + self.v[1] * other.v[1] + self.v[2] * other.v[2]
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.v[1] * other.v[2] - self.v[2] * other.v[1],
            self.v[2] * other.v[0] - self.v[0] * other.v[2],
            self.v[0] * other.v[1] - self.v[1] * other.v[0],
        )
    }

    pub fn normalized(&self) -> Vec3 {
        *self / self.length()
    }

    pub fn reflect(self, normal: Vec3) -> Vec3 {
        self - (normal * 2.0 * self.dot(&normal))
    }

    pub fn refract(self, normal: Vec3, eta_ratio: f64) -> Vec3 {
        let cos_theta = -self.dot(&normal).min(1.0);
        let out_perp = (self + normal * cos_theta) * eta_ratio;
        let out_parallel = normal * -(1.0 - out_perp.length_squared()).abs().sqrt();
        out_perp + out_parallel
    }

    pub fn near_zero(&self) -> bool {
        let eps = 1e-3;
        (self.v[0].abs() < eps) && (self.v[1].abs() < eps) && (self.v[2].abs() < eps)
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
        Vec3::new(normal_dist(), normal_dist(), normal_dist()).normalized()
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x() + other.x(), self.y() + other.y(), self.z() + other.z())
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x() - other.x(), self.y() - other.y(), self.z() - other.z())
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vec3::new(-self.x(), -self.y(), -self.z())
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x() * other.x(), self.y() * other.y(), self.z() * other.z())
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, other: f64) -> Self::Output {
        Vec3::new(self.x() * other, self.y() * other, self.z() * other)
    }
}

impl Div<Vec3> for Vec3 {
    type Output = Self;
    fn div(self, other: Vec3) -> Self::Output {
        Vec3::new(self.x() / other.x(), self.y() / other.y(), self.z() / other.z())
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, other: f64) -> Self::Output {
        Vec3::new(self.x() / other, self.y() / other, self.z() / other)
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Vec3) -> bool {
        self.x() == other.x() && self.y() == other.y() && self.z() == other.z()
    }
}
