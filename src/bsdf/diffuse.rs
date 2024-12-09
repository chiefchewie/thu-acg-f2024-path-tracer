use super::{
    sampling::{cosine_sample_hemisphere, to_local, to_world},
    BxDF, EPS,
};
use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};
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
        l.z.abs() / PI
    }

    fn eval(&self, _view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let l = to_local(info.normal, light_dir);
        l.z.abs() * (self.base_color / PI)
    }
}

impl Material for DiffuseBRDF {
    /// optimized version combining sample, pdf, and eval
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let Some(dir) = self.sample(ray, hit_info) else {
            return (self.base_color, None);
        };

        let next_ray = Ray::new(hit_info.point + EPS * hit_info.normal, dir, ray.time());
        (self.base_color, Some(next_ray))
    }
}
