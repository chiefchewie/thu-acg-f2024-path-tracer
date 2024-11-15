use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    albedo: Vec3,
}

impl Material {
    pub fn new(albedo: Vec3) -> Material {
        Material { albedo }
    }

    pub fn alebdo(&self) -> Vec3 {
        self.albedo
    }
}
