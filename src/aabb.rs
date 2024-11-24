use crate::{interval::Interval, ray::Ray, vec3::Vec3};

#[derive(Clone, Copy)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn new(a: Vec3, b: Vec3) -> AABB {
        let min = a.min(b);
        let max = a.max(b);
        AABB { min, max }
    }

    pub fn union(self, other: AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
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
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }
}
