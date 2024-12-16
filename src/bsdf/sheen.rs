use std::f64::consts::PI;

use crate::{hittable::HitInfo, ray::Ray, vec3::Vec3};

use super::{
    sampling::{cosine_sample_hemisphere, to_local, to_world},
    tint, BxDFMaterial,
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

impl BxDFMaterial for SheenBRDF {
    fn sample(&self, _ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let dir_local = cosine_sample_hemisphere();
        Some(to_world(info.geometric_normal, dir_local))
    }

    fn pdf(&self, _view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let l = to_local(info.geometric_normal, light_dir);
        l.z.abs() / PI
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let v = to_local(info.geometric_normal, view_dir);
        let l = to_local(info.geometric_normal, light_dir);
        let h = (v + l).normalize();
        let c_tint = tint(self.base_color);
        let c_sheen = Vec3::ONE.lerp(c_tint, self.sheen_tint);
        c_sheen * (1.0 - l.dot(h).abs()).powi(5) * (l.z.abs())
    }
}
