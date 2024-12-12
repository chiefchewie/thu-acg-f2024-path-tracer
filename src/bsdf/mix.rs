use std::sync::Arc;

use crate::{hittable::HitInfo, ray::Ray, vec3::Vec3};

use super::BxDFMaterial;

#[derive(Clone)]
pub struct MixBxDf {
    t: f64, // 0 = use mat1 entirely, 1 = use mat2 entirely
    bxdf1: Arc<dyn BxDFMaterial>,
    bxdf2: Arc<dyn BxDFMaterial>,
}

impl MixBxDf {
    pub fn new(t: f64, bxdf1: Arc<dyn BxDFMaterial>, bxdf2: Arc<dyn BxDFMaterial>) -> MixBxDf {
        Self {
            t: t.clamp(0.0, 1.0),
            bxdf1,
            bxdf2,
        }
    }
}

impl BxDFMaterial for MixBxDf {
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
