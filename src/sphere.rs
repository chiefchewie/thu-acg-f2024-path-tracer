use core::f64;
use std::f64::consts::PI;

use crate::{hit_info::HitInfo, interval::Interval, material::MaterialType, ray::Ray, vec3::Vec3, Hittable};

#[derive(Clone)]
pub struct Sphere {
    radius: f64,
    center: Vec3,
    material: MaterialType,
}

impl Sphere {
    pub fn new(radius: f64, center: Vec3, material: MaterialType) -> Sphere {
        Sphere {
            radius,
            center,
            material,
        }
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    fn get_uv(p: &Vec3) -> (f64, f64) {
        let theta = (-p.y()).acos();
        let phi = f64::atan2(-p.z(), p.x()) + PI;
        (phi / (2.0 * PI), theta / PI)
    }
}

impl Hittable for Sphere {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let l = self.center - ray.origin();
        let s = Vec3::dot(&l, &ray.direction());
        let l2 = l.length_squared();
        let r2 = self.radius() * self.radius();

        if s < 0.0 && l2 > r2 {
            return None;
        }

        let d2 = l2 - s * s;
        if d2 > r2 {
            return None;
        }

        let q = (r2 - d2).sqrt();
        let intersect = if l2 > r2 { s - q } else { s + q };

        // TODO this condition is sussy
        if intersect <= ray_t.min || intersect >= ray_t.max {
            return None;
        }

        let mut hit_info = HitInfo {
            ..Default::default()
        };
        hit_info.point = ray.at(intersect);
        hit_info.dist = intersect;
        hit_info.mat = self.material.clone();
        let normal = (hit_info.point - self.center).normalized();
        let uv = Self::get_uv(&normal);
        hit_info.u = uv.0;
        hit_info.v = uv.1;
        hit_info.set_face_normal(ray, normal);
        Some(hit_info)
    }
}
