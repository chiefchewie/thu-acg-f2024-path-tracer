use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};

pub mod clearcoat;
pub mod diffuse;
pub mod glass;
pub mod metal;
pub mod principled;
pub mod sampling;
pub mod sheen;

pub(crate) const EPS: f64 = 1e-3;

pub trait BxDF: Material {
    /// Given the outgoing (view) ray and hit info, sample an incident (light) ray
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3>;

    /// Given an outgoing and incoming ray and hit info, compute the pdf of this incoming (light) ray
    fn pdf(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64;

    /// Given an outgoing and incoming ray and hit info, compute the reflectance
    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3;
}
