use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub power: Vec3,
}

impl PointLight {
    pub fn new(position: Vec3, power: Vec3) -> PointLight {
        PointLight { position, power }
    }
}
