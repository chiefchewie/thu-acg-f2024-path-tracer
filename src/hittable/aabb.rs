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
        0.5 * (self.min + self.max)
    }

    pub fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<f64> {
        let m = ray.direction().recip();
        let t1 = (self.min - ray.origin()) * m;
        let t2 = (self.max - ray.origin()) * m;
        let t_near = t1.min(t2).max_element();
        let t_far = t1.max(t2).min_element();
        if t_near <= t_far && t_far >= ray_t.min && t_near <= ray_t.max {
            Some(t_near.max(ray_t.min))
        } else {
            None
        }
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
            min: Vec3::INFINITY,
            max: Vec3::NEG_INFINITY,
        }
    }
}

impl Add<Vec3> for AABB {
    type Output = AABB;

    fn add(self, rhs: Vec3) -> Self::Output {
        AABB::new(self.min + rhs, self.max + rhs)
    }
}
