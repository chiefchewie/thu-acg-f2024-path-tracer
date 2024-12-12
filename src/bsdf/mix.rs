use std::sync::Arc;

use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};

use super::{BxDF, EPS};

#[derive(Clone)]
pub struct MixBxDf {
    t: f64, // 0 = use mat1 entirely, 1 = use mat2 entirely
    bxdf1: Arc<dyn BxDF>,
    bxdf2: Arc<dyn BxDF>,
}

impl MixBxDf {
    pub fn new(t: f64, bxdf1: Arc<dyn BxDF>, bxdf2: Arc<dyn BxDF>) -> MixBxDf {
        Self {
            t: t.clamp(0.0, 1.0),
            bxdf1,
            bxdf2,
        }
    }
}

impl BxDF for MixBxDf {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let p: f64 = rand::random();
        if self.t < p {
            self.bxdf1.sample(ray, info)
        } else {
            self.bxdf2.sample(ray, info)
        }
    }

    fn pdf(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let p1 = (1.0 - self.t) * self.bxdf1.pdf(view_dir, light_dir, info);
        let p2 = self.t * self.bxdf2.pdf(view_dir, light_dir, info);
        p1 + p2
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> crate::vec3::Vec3 {
        let w1 = (1.0 - self.t) * self.bxdf1.eval(view_dir, light_dir, info);
        let w2 = self.t * self.bxdf2.eval(view_dir, light_dir, info);
        w1 + w2
    }
}

impl Material for MixBxDf {
    fn scatter(
        &self,
        ray: &crate::ray::Ray,
        hit_info: &crate::hittable::HitInfo,
    ) -> (Vec3, Option<Ray>) {
        let dir = self.sample(ray, hit_info);
        let Some(dir) = dir else {
            return (crate::vec3::Vec3::ONE, None);
        };
        let pdf = self.pdf(-ray.direction(), dir, hit_info);
        let brdf = self.eval(-ray.direction(), dir, hit_info);
        let brdf_weight = brdf / pdf;

        let eps = EPS * dir.dot(hit_info.normal).signum();
        let next_ray = Ray::new(hit_info.point + eps * hit_info.normal, dir, ray.time());
        (brdf_weight, Some(next_ray))
    }
}
