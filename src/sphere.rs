use core::f64;
use std::f64::consts::PI;

use crate::{
    aabb::AABB, hit_info::HitInfo, interval::Interval, material::MaterialType, ray::Ray,
    vec3::Vec3, Hittable,
};

#[derive(Clone)]
pub struct Sphere {
    radius: f64,
    position1: Vec3,
    position2: Vec3,
    material: MaterialType,
    bbox: AABB,
}

impl Sphere {
    pub fn new_still(radius: f64, position: Vec3, material: MaterialType) -> Sphere {
        let rvec = Vec3::new(radius, radius, radius);
        let bbox = AABB::new(position - rvec, position + rvec);
        Sphere {
            radius: radius.max(0.0),
            position1: position,
            position2: position,
            material,
            bbox,
        }
    }

    pub fn new_moving(
        radius: f64,
        position1: Vec3,
        position2: Vec3,
        material: MaterialType,
    ) -> Sphere {
        let rvec = Vec3::new(radius, radius, radius);
        let box1 = AABB::new(position1 - rvec, position1 + rvec);
        let box2 = AABB::new(position2 - rvec, position2 + rvec);
        let bbox = AABB::union(box1, box2);
        Sphere {
            radius: radius.max(0.0),
            position1,
            position2,
            material,
            bbox,
        }
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    fn get_uv(p: &Vec3) -> (f64, f64) {
        let theta = (-p.y).acos();
        let phi = f64::atan2(-p.z, p.x) + PI;
        (phi / (2.0 * PI), theta / PI)
    }

    fn get_position(&self, t: f64) -> Vec3 {
        self.position1 + (self.position2 - self.position1) * t
    }
}

impl Hittable for Sphere {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let current_center = self.get_position(ray.time());
        let l = current_center - ray.origin();
        let s = Vec3::dot(l, ray.direction());
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
        let normal = (hit_info.point - current_center).normalize();
        let uv = Self::get_uv(&normal);
        hit_info.u = uv.0;
        hit_info.v = uv.1;
        hit_info.set_face_normal(ray, normal);
        Some(hit_info)
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
