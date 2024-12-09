use std::f64::consts::PI;

use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};

use super::{
    sampling::{cosine_sample_hemisphere, to_local, to_world},
    tint, BxDF, EPS,
};

#[derive(Clone)]
pub struct SheenBRDF {
    base_color: Vec3,
    sheen_tint: f64,
}

impl SheenBRDF {
    pub fn new(base_color: Vec3, sheen_tint: f64) -> Self {
        Self {
            base_color,
            sheen_tint,
        }
    }
}

impl BxDF for SheenBRDF {
    fn sample(&self, _ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let dir_local = cosine_sample_hemisphere();
        Some(to_world(info.normal, dir_local))
    }

    fn pdf(&self, _view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let l = to_local(info.normal, light_dir);
        l.z.abs() / PI
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let v = to_local(info.normal, view_dir);
        let l = to_local(info.normal, light_dir);
        let h = (v + l).normalize();
        let c_tint = tint(self.base_color);
        let c_sheen = Vec3::ONE.lerp(c_tint, self.sheen_tint);
        c_sheen * (1.0 - l.dot(h).abs()).powi(5) * (l.z.abs())
    }
}

impl Material for SheenBRDF {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let Some(dir) = self.sample(ray, hit_info) else {
            return (Vec3::ONE, None);
        };

        let pdf = self.pdf(-ray.direction(), dir, hit_info);
        let brdf = self.eval(-ray.direction(), dir, hit_info);
        let brdf_weight = brdf / pdf;

        let eps = EPS * dir.dot(hit_info.normal).signum();
        let next_ray = Ray::new(hit_info.point + eps * hit_info.normal, dir, ray.time());
        (brdf_weight, Some(next_ray))
    }
}
