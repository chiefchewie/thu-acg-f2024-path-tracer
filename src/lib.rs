use hit_info::HitInfo;
use ray::Ray;

pub mod camera;
pub mod hit_info;
pub mod light;
pub mod material;
pub mod ray;
pub mod sphere;
pub mod vec3;

pub trait Hittable {
    fn intersects(&self, ray: &Ray, t_min: f64, t_max: f64) -> HitInfo;
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
    fn intersects(&self, ray: &Ray, t_min: f64, t_max: f64) -> HitInfo {
        let mut closest_hit = HitInfo {
            dist: t_max,
            ..Default::default()
        };
        for obj in self.objects.iter() {
            let info = obj.intersects(ray, t_min, closest_hit.dist);
            if info.did_hit {
                closest_hit = info;
            }
        }

        closest_hit
    }
}
