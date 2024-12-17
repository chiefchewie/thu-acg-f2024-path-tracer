use std::ops::Add;

use crate::{
    interval::Interval,
    ray::Ray,
    vec3::{Mat4, Vec3},
};

#[derive(Clone, Copy)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn new(a: Vec3, b: Vec3) -> AABB {
        let delta = Vec3::splat(1e-3); // make sure the bounding box isn't too flat on any of the axis
        let min = a.min(b) - delta;
        let max = a.max(b) + delta;
        AABB { min, max }
    }

    pub fn union(self, other: AABB) -> AABB {
        AABB::new(self.min.min(other.min), self.max.max(other.max))
    }

    pub fn centroid(&self) -> Vec3 {
        0.5 * (self.max + self.max)
    }

    pub fn intersects(&self, ray: &Ray, ray_t: Interval) -> bool {
        let mut t_min = (self.min.x - ray.origin().x) / ray.direction().x;
        let mut t_max = (self.max.x - ray.origin().x) / ray.direction().x;
        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut t_enter = t_min;
        let mut t_exit = t_max;

        for (min, max, origin, direction) in [
            (self.min.y, self.max.y, ray.origin().y, ray.direction().y),
            (self.min.z, self.max.z, ray.origin().z, ray.direction().z),
        ] {
            let mut t_min = (min - origin) / direction;
            let mut t_max = (max - origin) / direction;
            if t_min > t_max {
                std::mem::swap(&mut t_min, &mut t_max);
            }

            t_enter = t_enter.max(t_min);
            t_exit = t_exit.min(t_max);

            if t_enter > t_exit {
                return false; // No intersection
            }
        }

        t_enter <= ray_t.max && t_exit >= ray_t.min
    }

    pub fn extent(&self) -> Vec3 {
        self.max - self.min
    }

    /// technically, half of this AABB's surface area
    pub fn surface_area(&self) -> f64 {
        let e = self.extent();
        e.x * e.y + e.x * e.z + e.y * e.z
    }

    pub fn transform(&self, mat: Mat4) -> AABB {
        let corners = [
            self.min,
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            self.max,
        ];

        let transformed_corners: Vec<Vec3> = corners
            .iter()
            .map(|&corner| mat.transform_point3(corner))
            .collect();
        let mut new_min = Vec3::splat(f64::INFINITY);
        let mut new_max = Vec3::splat(f64::NEG_INFINITY);
        for corner in transformed_corners {
            new_min = new_min.min(corner);
            new_max = new_max.max(corner);
        }

        AABB::new(new_min, new_max)
    }
}

impl Default for AABB {
    fn default() -> AABB {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }
}

impl Add<Vec3> for AABB {
    type Output = AABB;

    fn add(self, rhs: Vec3) -> Self::Output {
        AABB::new(self.min + rhs, self.max + rhs)
    }
}
