use super::{
    sampling::{cosine_sample_hemisphere, to_local, to_world},
    BxDF, EPS,
};
use crate::{
    hittable::HitInfo,
    material::Material,
    ray::Ray,
    texture::{SolidTexture, Texture},
    vec3::Vec3,
};
use std::{f64::consts::PI, sync::Arc};

#[derive(Clone)]
pub struct DiffuseBRDF {
    base_color: Arc<dyn Texture<Vec3>>,
}

// Lambertian diffuse, NOT the one used in PrincipledBSDF
impl DiffuseBRDF {
    pub fn new(base_color: Arc<dyn Texture<Vec3>>) -> Self {
        Self { base_color }
    }

    pub fn from_rgb(base_color: Vec3) -> Self {
        Self {
            base_color: Arc::new(SolidTexture::new(base_color)),
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
        let color = self.base_color.value(info.u, info.v, &info.point);
        let l = to_local(info.normal, light_dir);
        l.z.abs() * (color / PI)
    }
}

impl Material for DiffuseBRDF {
    /// optimized version combining sample, pdf, and eval
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let color = self
            .base_color
            .value(hit_info.u, hit_info.v, &hit_info.point);
        let Some(dir) = self.sample(ray, hit_info) else {
            return (Vec3::ONE, None);
        };

        let next_ray = Ray::new(hit_info.point + EPS * hit_info.normal, dir, ray.time());
        (color, Some(next_ray))
    }
}
