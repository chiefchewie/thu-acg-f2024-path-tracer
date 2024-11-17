use crate::{material::Material, ray::Ray, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct HitInfo {
    pub point: Vec3,
    pub normal: Vec3,
    pub dist: f64,
    pub front_face: bool,
    pub mat: Material,
}

impl HitInfo {
    pub fn set_face_normal(&mut self, ray: &Ray, normal: Vec3) {
        self.front_face = Vec3::dot(&ray.direction(), &normal) < 0.0;
        self.normal = if self.front_face { normal } else { -normal };
    }
}

impl Default for HitInfo {
    fn default() -> HitInfo {
        HitInfo {
            point: Default::default(),
            normal: Default::default(),
            dist: f64::INFINITY,
            front_face: false,
            mat: Default::default(),
        }
    }
}
