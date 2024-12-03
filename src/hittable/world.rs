use std::sync::Arc;

use crate::{interval::Interval, light::PointLight, ray::Ray, vec3::Vec3};

use super::{BVHNode, HitInfo, Hittable, AABB, BVH};

pub struct World {
    objects: Vec<Arc<dyn Hittable>>,
    lights: Vec<PointLight>, // indices of light sources
    bbox: AABB,
    bvh: Option<BVHNode>,
}

impl World {
    pub fn new() -> World {
        World {
            objects: vec![],
            lights: vec![],
            bbox: AABB::default(),
            bvh: None,
        }
    }

    pub fn add_light(&mut self, light: PointLight) {
        self.lights.push(light);
    }

    pub fn add<T: Hittable + 'static>(&mut self, obj: T) {
        self.bbox = AABB::union(self.bbox, obj.bounding_box());
        let rc = Arc::new(obj);
        self.objects.push(rc.clone());
    }

    pub fn build_bvh(&mut self) {
        self.bvh = Some(BVH::build(self.objects.clone()));
    }

    pub fn shadow_ray(&self, origin: Vec3, light_pos: Vec3, time: f64) -> bool {
        let dir = (light_pos - origin).normalize();
        let max_dist = (light_pos - origin).length();
        self.intersects(&Ray::new(origin, dir, time), Interval::new(1e-3, max_dist))
            .is_none()
    }

    pub fn get_lights(&self) -> &Vec<PointLight> {
        &self.lights
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl Hittable for World {
    /// intersect with t in (t_min, t_max)
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        if let Some(ref bvh) = self.bvh {
            bvh.intersects(ray, ray_t)
        } else {
            let mut closest_hit = ray_t.max;
            let mut hit_info = None;
            for obj in self.objects.iter() {
                if let Some(info) = obj.intersects(ray, Interval::new(ray_t.min, closest_hit)) {
                    closest_hit = info.dist;
                    hit_info = Some(info);
                }
            }

            hit_info
        }
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
