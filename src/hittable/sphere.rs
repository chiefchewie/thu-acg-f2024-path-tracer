use std::f64::consts::PI;

use crate::bsdf::MatPtr;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::Vec3;

use super::hit_info::HitInfo;
use super::Hittable;
use super::AABB;

#[derive(Clone)]
pub struct Sphere {
    radius: f64,
    position1: Vec3,
    position2: Vec3,
    material: MatPtr,
    bbox: AABB,
}

impl Sphere {
    pub fn new_still(radius: f64, position: Vec3, material: MatPtr) -> Sphere {
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

    pub fn new_moving(radius: f64, position1: Vec3, position2: Vec3, material: MatPtr) -> Sphere {
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

        let point = ray.at(intersect);
        let normal = (point - current_center).normalize();
        let (u, v) = Self::get_uv(&normal);
        Some(HitInfo::new(
            ray,
            point,
            normal,
            intersect,
            self.material.clone(),
            u,
            v,
        ))
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }

    fn material(&self) -> Option<&dyn crate::bsdf::BxDFMaterial> {
        Some(self.material.as_ref())
    }

    fn sample(&self, origin: Vec3, time: f64) -> Option<Vec3> {
        let u: f64 = rand::random();
        let v: f64 = rand::random();
        let theta = 2.0 * PI * u;
        let phi = f64::acos(2.0 * v - 1.0);
        let x = phi.sin() * theta.cos();
        let y = phi.sin() * theta.sin();
        let z = phi.cos();
        let point = self.get_position(time) + Vec3::new(x, y, z) * self.radius;
        let dir = (point - origin).normalize();
        Some(dir)
    }

    fn pdf(&self, origin: Vec3, direction: Vec3, time: f64) -> f64 {
        if let Some(_hit) = self.intersects(
            &Ray::new(origin, direction, time),
            Interval::new(0.0, f64::INFINITY),
        ) {
            let r2 = self.radius * self.radius;
            let solid_angle =
                2.0 * PI * (1.0 - r2 / (self.get_position(time) - origin).length_squared()).sqrt();
            1.0 / solid_angle
        } else {
            0.0
        }
    }
}
