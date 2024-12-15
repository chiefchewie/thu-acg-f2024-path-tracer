use std::sync::Arc;

use crate::{
    hittable::HitInfo,
    ray::Ray,
    vec3::{Luminance, Vec3},
};

pub mod clearcoat;
pub mod diffuse;
pub mod glass;
pub mod metal;
pub mod mix;
pub mod principled;
pub mod sampling;
pub mod sheen;

pub(crate) const EPS: f64 = 1e-3;

pub trait BxDFMaterial: Send + Sync {
    /// Given the outgoing (view) ray and hit info, sample an incident (light) ray
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3>;

    /// Given an outgoing and incoming ray and hit info, compute the pdf of this incoming (light) ray
    fn pdf(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64;

    /// Given an outgoing and incoming ray and hit info, compute the reflectance
    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3;

    /// returns: attenuation (brdf/pdf), and the scattered ray
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> Option<(Vec3, Ray)> {
        let dir = self.sample(ray, hit_info)?;
        let pdf = self.pdf(-ray.direction(), dir, hit_info);
        let brdf = self.eval(-ray.direction(), dir, hit_info);
        let brdf_weight = brdf / pdf;
        let eps = EPS * dir.dot(hit_info.normal).signum();
        let next_ray = Ray::new(hit_info.point + eps * hit_info.normal, dir, ray.time());
        Some((brdf_weight, next_ray))
    }

    fn emitted(&self, _u: f64, _v: f64, _p: Vec3) -> Vec3 {
        Vec3::ZERO
    }

    fn is_emissive(&self) -> bool {
        false
    }
}

pub type MatPtr = Arc<dyn BxDFMaterial>;

pub fn tint(base_color: Vec3) -> Vec3 {
    // c_tint
    if base_color.luminance() > 0.0 {
        base_color / base_color.luminance()
    } else {
        Vec3::ONE
    }
}

pub fn r0(eta: f64) -> f64 {
    ((eta - 1.0) / (eta + 1.0)).powi(2)
}

pub mod fresnel {
    use crate::vec3::Vec3;

    pub fn dielectric(w: Vec3, h: Vec3, eta_i: f64, eta_o: f64) -> f64 {
        let c = w.dot(h).abs();
        let g_squared = (eta_o / eta_i).powi(2) - 1.0 + c * c;
        if g_squared < 0.0 {
            return 1.0;
        }
        let g = g_squared.sqrt();
        let gmc = g - c;
        let gpc = g + c;
        let x = (c * gpc - 1.0) / (c * gmc + 1.0);
        0.5 * (gmc * gmc) / (gpc * gpc) * (1.0 + x * x)
    }

    pub fn schlick(r0: Vec3, angle: f64) -> Vec3 {
        r0 + (1.0 - r0) * (1.0 - angle).powi(5)
    }

    pub fn schlick_weight(x: f64) -> f64 {
        (1.0 - x).clamp(0.0, 1.0).powi(5)
    }
}
