// An implementation of https://www.graphics.cornell.edu/%7Ebjw/microfacetbsdf.pdf
// with help from https://schuttejoe.github.io/post/ggximportancesamplingpart1/
// and https://schuttejoe.github.io/post/ggximportancesamplingpart2/

use std::sync::Arc;

use super::sampling::ggx;
use super::EPS;
use super::{
    sampling::{to_local, to_world},
    BxDF,
};
use crate::texture::{SolidTexture, Texture};
use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};

#[derive(Clone)]
pub struct MetalBRDF {
    base_color: Arc<dyn Texture<Vec3>>,
    roughness: Arc<dyn Texture<f64>>,
}

impl MetalBRDF {
    pub fn new(base_color: Arc<dyn Texture<Vec3>>, roughness: Arc<dyn Texture<f64>>) -> Self {
        Self {
            base_color,
            roughness,
        }
    }

    pub fn from_rgb(base_color: Vec3, roughness: f64) -> Self {
        Self {
            base_color: Arc::new(SolidTexture::new(base_color)),
            roughness: Arc::new(SolidTexture::new(roughness)),
        }
    }
}

impl BxDF for MetalBRDF {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);

        let roughness = self.roughness.value(info.u, info.v, &info.point);
        let h = ggx::sample_microfacet_normal(v, roughness);

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

        let roughness = self.roughness.value(info.u, info.v, &info.point);
        let pdf_h = ggx::G1(v, roughness) * v.dot(h).abs() * ggx::D(h, roughness) / v.z.abs();

        let jacobian = 1.0 / (4.0 * l.dot(h).abs());

        pdf_h * jacobian
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let v = to_local(info.normal, view_dir);
        let l = to_local(info.normal, light_dir);
        let h = (v + l).normalize();

        let roughness = self.roughness.value(info.u, info.v, &info.point);
        let base_color = self.base_color.value(info.u, info.v, &info.point);
        let d = ggx::D(h, roughness);
        let g = ggx::G(v, l, roughness);
        let f = schlick_fresnel(base_color, l.dot(h));
        l.z.abs() * (f * g * d / (4.0 * l.z.abs() * v.z.abs()))
    }
}

impl Material for MetalBRDF {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let roughness = self
            .roughness
            .value(hit_info.u, hit_info.v, &hit_info.point);
        let base_color = self
            .base_color
            .value(hit_info.u, hit_info.v, &hit_info.point);

        // but here's a more optimized version
        let Some(dir) = self.sample(ray, hit_info) else {
            return (base_color, None);
        };

        // default impl
        // let pdf = self.pdf(-ray.direction(), dir, hit_info);
        // let brdf = self.eval(-ray.direction(), dir, hit_info);
        // let brdf_weight = brdf / pdf;

        // simplified faster impl
        let v = to_local(hit_info.normal, -ray.direction());
        let l = to_local(hit_info.normal, dir);
        let h = (v + l).normalize();
        let g = ggx::G(v, l, roughness);

        // the simplified result of brdf / pdf
        // note that f is not cancelled out like in glass.rs because it's not present in the pdf
        let f = schlick_fresnel(base_color, l.dot(h));
        let brdf_weight = f * v.dot(h).abs() * g / (v.z.abs() * h.z.abs());

        let next_ray = Ray::new(hit_info.point + EPS * hit_info.normal, dir, ray.time());
        (brdf_weight, Some(next_ray))
    }
}

fn schlick_fresnel(r0: Vec3, angle: f64) -> Vec3 {
    r0 + (1.0 - r0) * (1.0 - angle).powi(5)
}
