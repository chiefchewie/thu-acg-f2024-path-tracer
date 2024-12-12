use crate::{hittable::HitInfo, ray::Ray, vec3::Vec3};

use super::{
    r0,
    sampling::{ggx, gtr1, to_local, to_world},
    BxDFMaterial,
};

#[derive(Clone)]
pub struct ClearcoatBRDF {
    alpha_g: f64,
}

impl ClearcoatBRDF {
    pub fn new(clearcoat_gloss: f64) -> Self {
        Self {
            alpha_g: (1.0 - clearcoat_gloss) * 0.1 + clearcoat_gloss * 0.001,
        }
    }
}

impl BxDFMaterial for ClearcoatBRDF {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);

        let h = gtr1::sample_microfacet_normal(0.25);
        let specular_dir_local = (-v).reflect(h);
        let specular_dir = to_world(info.normal, specular_dir_local);
        if specular_dir.dot(info.normal) <= 0.0 {
            None
        } else {
            Some(specular_dir)
        }
    }

    fn pdf(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let v = to_local(info.normal, view_dir);
        let l = to_local(info.normal, light_dir);
        let h = (v + l).normalize();
        let pdf_h =
            ggx::G1(v, 0.25) * v.dot(h).abs() * gtr1::D(l.dot(h).abs(), self.alpha_g) / v.z.abs();
        let jacobian = 1.0 / (4.0 * l.dot(h).abs());
        pdf_h * jacobian
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let v = to_local(info.normal, view_dir);
        let l = to_local(info.normal, light_dir);
        let h = (v + l).normalize();

        let d = gtr1::D(l.dot(h).abs(), self.alpha_g);

        let g = ggx::G(v, l, 0.25);

        let eta = 1.5;
        let r0 = Vec3::splat(r0(eta));
        let f = schlick_fresnel(r0, l.dot(h));

        l.z.abs() * (f * d * g / (4.0 * l.z.abs() * v.z.abs()))
    }
}

fn schlick_fresnel(r0: Vec3, angle: f64) -> Vec3 {
    r0 + (1.0 - r0) * (1.0 - angle).powi(5)
}
