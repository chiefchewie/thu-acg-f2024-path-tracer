use crate::{interval::Interval, ray::Ray, vec3::Vec3};

use super::{HitInfo, Hittable, HittableList};

pub struct World {
    objects: HittableList,
    lights: HittableList,
}

impl World {
    pub fn new() -> World {
        World {
            objects: HittableList::new(),
            lights: HittableList::new(),
        }
    }

    pub fn add_light<T: Hittable + 'static>(&mut self, light: T) {
        self.lights.add(light);
    }

    pub fn add_object<T: Hittable + 'static>(&mut self, object: T) {
        self.objects.add(object);
    }

    pub fn build_bvh(&mut self) {
        self.objects.build_bvh();
        self.lights.build_bvh();
    }

    pub fn shadow_ray(&self, origin: Vec3, light_pos: Vec3, time: f64) -> bool {
        let dir = (light_pos - origin).normalize();
        let max_dist = (light_pos - origin).length();
        self.intersects(&Ray::new(origin, dir, time), Interval::new(1e-3, max_dist))
            .is_none()
    }

    pub fn get_lights(&self) -> &HittableList {
        &self.lights
    }

    /// intersect with t in (t_min, t_max)
    pub fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        self.objects.intersects(ray, ray_t)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
