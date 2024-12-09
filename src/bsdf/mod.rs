use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};

pub mod clearcoat;
pub mod diffuse;
pub mod glass;
pub mod metal;
pub mod sampling;

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
