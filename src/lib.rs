use std::rc::Rc;

use aabb::AABB;
use bvh::{BVHNode, BVH};
use hit_info::HitInfo;
use interval::Interval;
use ray::Ray;

pub mod aabb;
pub mod bvh;
pub mod camera;
pub mod hit_info;
pub mod interval;
pub mod light;
pub mod material;
pub mod ray;
pub mod sphere;
pub mod texture;
pub mod utils;
pub mod vec3;

pub trait Hittable {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo>;
    fn bounding_box(&self) -> AABB;
}

pub struct World {
    objects: Vec<Rc<dyn Hittable>>,
    bbox: AABB,
    bvh: Option<BVHNode>,
}

impl World {
    pub fn new() -> World {
        World {
            objects: vec![],
            bbox: AABB::default(),
            bvh: None,
        }
    }

    pub fn add<T: Hittable + 'static>(&mut self, obj: T) {
        self.bbox = AABB::union(self.bbox, obj.bounding_box());
        self.objects.push(Rc::new(obj));
    }

    pub fn build_bvh(&mut self) {
        self.bvh = Some(BVH::build(self.objects.clone()));
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
