// An implementation of https://www.graphics.cornell.edu/%7Ebjw/microfacetbsdf.pdf
// with help from https://schuttejoe.github.io/post/ggximportancesamplingpart1/
// and https://schuttejoe.github.io/post/ggximportancesamplingpart2/

use super::{
    sampling::{to_local, to_world},
    BxDF,
};
use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};
use rand::{thread_rng, Rng};
use std::f64::consts::PI;

#[derive(Clone)]
pub struct MetalBRDF {
    base_color: Vec3,
    roughness: f64,
}

impl MetalBRDF {
    pub fn new(base_color: Vec3, roughness: f64) -> Self {
        Self {
            base_color,
            roughness
        }
    }

    fn D(&self, h: Vec3) -> f64 {
        let cos_theta = h.z.max(0.001);
        let alpha2 = (self.roughness * self.roughness).max(0.001);
        let denom = (alpha2 - 1.0) * (cos_theta * cos_theta) + 1.0;
        alpha2 / (PI * denom * denom)
    }

    fn G(&self, v: Vec3, l: Vec3, h: Vec3) -> f64 {
        let g1v = self.G1(v, h);
        let g1l = self.G1(l, h);
        g1v * g1l
    }

    fn G1(&self, w: Vec3, h: Vec3) -> f64 {
        let alpha2 = (self.roughness * self.roughness).max(0.001);
        let cos_theta = w.z.abs();
        2.0 * cos_theta / (cos_theta + (cos_theta * cos_theta * (1.0 - alpha2) + alpha2).sqrt())
    }
}

impl BxDF for MetalBRDF {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);
        let h = sample_microfacet_normal(v, self.roughness);
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

        // D term
        let d = self.D(h);
        d * (h.z) / (4.0 * v.dot(h).abs())
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let v = to_local(info.normal, view_dir);
        let l = to_local(info.normal, light_dir);
        let h = (v + l).normalize();

        // D term
        let d = self.D(h);

        // G term
        let g = self.G(v, l, h);

        // F term
        let f = schlick_fresnel(self.base_color, l.dot(h));
        l.z.abs() * (f * g * d / (4.0 * l.z.abs() * v.z.abs()))
    }
}

impl Material for MetalBRDF {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        // but here's a more optimized version
        let Some(dir) = self.sample(ray, hit_info) else {
            return (self.base_color, None);
        };

        // default impl
        // let pdf = self.pdf(-ray.direction(), dir, hit_info);
        // let brdf = self.eval(-ray.direction(), dir, hit_info);
        // let brdf_weight = brdf / pdf;

        // simplified faster impl
        let v = to_local(hit_info.normal, -ray.direction());
        let l = to_local(hit_info.normal, dir);
        let h = (v + l).normalize();
        let g = self.G(v, l, h);
        // the simplified result of brdf / pdf
        let brdf_weight = self.base_color * v.dot(h).abs() * g / (v.z.abs() * h.z.abs());

        let eps = 1e-3;
        let next_ray = Ray::new(hit_info.point + hit_info.normal * eps, dir, ray.time());
        (brdf_weight, Some(next_ray))
    }
}

// TODO reorg these in sampling.rs
fn sample_microfacet_normal(v: Vec3, roughness: f64) -> Vec3 {
    let a2 = roughness * roughness;
    let h = sample_ggx(v, a2);
    if h.z < 0.0 {
        -h
    } else {
        h
    }
}

// TODO VNDF, optimizations
fn sample_ggx(_v: Vec3, a2: f64) -> Vec3 {
    let mut rng = thread_rng();
    let e1: f64 = rng.gen();
    let e2: f64 = rng.gen();

    let theta = ((a2 * e1.sqrt()) / (1.0 - e1).sqrt()).atan();
    let phi = e2 * 2.0 * PI;
    Vec3::new(
        phi.cos() * theta.sin(),
        phi.sin() * theta.sin(),
        theta.cos(),
    )
}

fn schlick_fresnel(r0: Vec3, angle: f64) -> Vec3 {
    r0 + (1.0 - r0) * (1.0 - angle).powi(5)
}
