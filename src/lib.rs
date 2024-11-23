use hit_info::HitInfo;
use interval::Interval;
use ray::Ray;

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
}

pub struct World {
    objects: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> World {
        World { objects: vec![] }
    }

    pub fn add(&mut self, obj: Box<dyn Hittable>) {
        self.objects.push(obj);
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
