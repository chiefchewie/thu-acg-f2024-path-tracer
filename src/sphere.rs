use core::f64;

use crate::{hit_info::HitInfo, material::Material, ray::Ray, vec3::Vec3, Hittable};

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    radius: f64,
    center: Vec3,
    material: Material,
}

impl Sphere {
    pub fn new(radius: f64, center: Vec3, material: Material) -> Sphere {
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
}

impl Hittable for Sphere {
    fn intersects(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitInfo> {
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
        if intersect <= t_min || intersect >= t_max {
            return None;
        }

        let mut hit_info = HitInfo {
            ..Default::default()
        };
        hit_info.point = ray.at(intersect);
        hit_info.dist = intersect;
        hit_info.mat = self.material;

        let normal = (hit_info.point - self.center).normalized();
        hit_info.set_face_normal(ray, normal);
        Some(hit_info)
    }
}
