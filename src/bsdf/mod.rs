use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};

pub mod diffuse;
pub mod metal;
mod sampling;

// TODO: consider merging two of these to be faster
pub trait BxDF: Material {
    /// Given the outgoing (view) ray and hit info, sample an incident (light) ray
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3>;

    /// Given an outgoing and incoming ray and hit info, compute the pdf of this incoming (light) ray
    fn pdf(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64;

    /// Given an outgoing and incoming ray and hit info, compute the reflectance
    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3;
}

#[derive(Clone)]
pub struct PrincipledBSDF {
    pub base_color: Vec3, // TODO replace with texture
    pub metallic: f64,
    pub roughness: f64, // exclusive range (0..1)
    pub subsurface: f64,
    pub spec_trans: f64,
    pub specular_tint: f64,
    pub sheen: f64,
    pub sheen_tint: f64,
    pub clearcoat: f64,
    pub clearcoat_roughness: f64,
    pub ior: f64,
    pub anisotropic: f64,
}

impl Material for PrincipledBSDF {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let eps = 1e-3;

        let sampled_dir = self.sample(ray, hit_info);
        let Some(light_dir) = sampled_dir else {
            return (Vec3::ONE, None);
        };
        let (brdf, pdf) = self.eval(ray, light_dir, hit_info);

        let _t1 = brdf / pdf;
        let _t2 = self.base_color;

        let sign = light_dir.dot(hit_info.normal).signum();
        (
            brdf / pdf,
            Some(Ray::new(
                hit_info.point + hit_info.normal * (eps * sign),
                light_dir,
                ray.time(),
            )),
        )
    }
}
