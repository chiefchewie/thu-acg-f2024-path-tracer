use super::{
    sampling::{to_local, to_world},
    BxDF,
};
use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};
use rand::{thread_rng, Rng};
use std::f64::consts::PI;

#[derive(Clone)]
pub struct DiffuseBRDF {
    base_color: Vec3,
    _roughness: f64,
    _subsurface: f64,
}

impl DiffuseBRDF {
    pub fn rgb(rgb: Vec3) -> Self {
        Self {
            base_color: rgb,
            _roughness: 0.0,
            _subsurface: 0.0,
        }
    }
}

impl BxDF for DiffuseBRDF {
    fn sample(&self, _ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let diffuse_dir_local = cosine_sample_hemisphere();
        Some(to_world(info.normal, diffuse_dir_local))
    }

    fn pdf(&self, _view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let l = to_local(info.normal, light_dir);
        l.z / PI
    }

    fn eval(&self, _view_dir: Vec3, _light_dir: Vec3, _info: &HitInfo) -> Vec3 {
        self.base_color / PI
    }
}

impl Material for DiffuseBRDF {
    /// optimized version combining sample, pdf, and eval
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let Some(dir) = self.sample(ray, hit_info) else {
            return (self.base_color, None);
        };

        let eps = 1e-3;
        let next_ray = Ray::new(hit_info.point + eps * hit_info.normal, dir, ray.time());
        (self.base_color, Some(next_ray))
    }
}

// TODO reorg into sampling.rs
fn cosine_sample_hemisphere() -> Vec3 {
    let mut rng = thread_rng();
    let phi = rng.gen_range(0.0..=2.0 * PI);
    let r2 = rng.gen::<f64>();
    let r2s = r2.sqrt();
    Vec3::new(r2s * phi.cos(), r2s * phi.sin(), (1.0 - r2).sqrt())
}
