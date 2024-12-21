use std::sync::Arc;

use rand::{thread_rng, Rng};

use crate::{interval::Interval, vec3::Vec3};

use super::{BVHNode, Hittable, AABB, BVH};

pub struct HittableList {
    objects: Vec<Arc<dyn Hittable>>,
    bbox: AABB,
    bvh: Option<BVHNode>,
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList {
            objects: vec![],
            bbox: AABB::default(),
            bvh: None,
        }
    }

    pub fn add<T: Hittable + 'static>(&mut self, object: T) {
        self.bbox = AABB::union(self.bbox, object.bounding_box());
        self.objects.push(Arc::new(object));
    }

    pub fn build_bvh(&mut self) {
        if !self.objects.is_empty() {
            self.bvh = Some(BVH::build(self.objects.clone()));
        }
    }

    pub fn get(&self, i: usize) -> &Arc<dyn Hittable> {
        &self.objects[i]
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

impl Hittable for HittableList {
    fn intersects(
        &self,
        ray: &crate::ray::Ray,
        ray_t: crate::interval::Interval,
    ) -> Option<super::HitInfo> {
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

    fn material(&self) -> Option<&dyn crate::bsdf::BxDFMaterial> {
        None
    }

    fn sample(&self, origin: Vec3, time: f64) -> Option<crate::vec3::Vec3> {
        let i = thread_rng().gen_range(0..self.objects.len());
        self.objects[i].sample(origin,time)
    }

    fn pdf(&self, origin: Vec3, direction: Vec3, time: f64) -> f64 {
        let pdf = self
            .objects
            .iter()
            .map(|obj| obj.pdf(origin, direction, time))
            .sum::<f64>();
        pdf / self.objects.len() as f64
    }
}

impl Default for HittableList {
    fn default() -> Self {
        Self::new()
    }
}
