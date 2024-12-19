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

    fn sample_surface(&self, hit_info: &HitInfo, time: f64) -> Option<(Vec3, Vec3, f64)> {
        let current_center = self.get_position(time);
        // Create coordinate system aligned with hit normal
        let w = hit_info.geometric_normal;
        let a = if w.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let v = w.cross(a).normalize();
        let u = w.cross(v);

        // Sample random point on hemisphere
        let r1 = rand::random::<f64>();
        let r2 = rand::random::<f64>();
        let cos_theta = r1.sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * std::f64::consts::PI * r2;

        // Convert to cartesian coordinates
        let direction = Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta);

        // Transform to world space
        let world_direction = u * direction.x + v * direction.y + w * direction.z;

        // Get point on sphere
        let point = current_center + world_direction * self.radius;
        let normal = (point - current_center).normalize();

        // PDF is 1/(2*PI) for hemisphere sampling
        let pdf = 1.0 / (2.0 * std::f64::consts::PI);

        Some((point, normal, pdf))
    }
}
