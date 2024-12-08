use crate::vec3::{get_rotation_to_z, Vec3};

// transformations
pub fn to_local(normal: Vec3, input_world: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal);
    rot * input_world
}

pub fn to_world(normal: Vec3, input_local: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal).inverse();
    rot * input_local
}
