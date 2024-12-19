use rand::{thread_rng, Rng};

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
        self.intersect_objects(&Ray::new(origin, dir, time), Interval::new(1e-3, max_dist))
            .is_none()
    }

    pub fn sample_light(&self, hit_info: &HitInfo, time: f64) -> Option<(Vec3, Vec3, Vec3, f64)> {
        if self.lights.is_empty() {
            return None;
        }
        let i = thread_rng().gen_range(0..self.lights.len());
        let (l_pos, l_nor, l_pdf) = self.lights.get(i).sample_surface(hit_info, time)?;
        if self.shadow_ray(hit_info.point, l_pos, time) {
            let le = self.lights.get(i).material().map_or(Vec3::ZERO, |m| {
                m.emitted(hit_info.u, hit_info.v, hit_info.point)
            });
            Some((l_pos, l_nor, le, l_pdf / self.lights.len() as f64))
        } else {
            None
        }
    }

    /// intersect with t in (t_min, t_max)
    pub fn intersect_objects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        self.objects.intersects(ray, ray_t)
    }

    pub fn intersect_lights(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        self.lights.intersects(ray, ray_t)
    }

    pub fn intersect_all(&self, ray: &Ray, ray_t: Interval) -> Option<(HitInfo, bool)> {
        let light_hit = self.intersect_lights(ray, ray_t);
        let obj_hit = self.intersect_objects(ray, ray_t);
        match (light_hit, obj_hit) {
            (None, None) => None,
            (None, Some(obj)) => Some((obj, false)),
            (Some(light), None) => Some((light, true)),
            (Some(light), Some(obj)) => {
                if light.dist < obj.dist {
                    Some((light, true))
                } else {
                    Some((obj, false))
                }    
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
