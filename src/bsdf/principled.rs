use crate::vec3::Vec3;

#[derive(Clone)]
pub struct PrincipledBSDF {
    base_color: Vec3, // TODO replace with texture
    spec_trans: f64,
    metallic: f64,
    // subsurface: f64,
    specular: f64,
    specular_tint: f64,
    roughness: f64,
    // anisotropic: f64,
    sheen: f64,
    sheen_tint: f64,
    clearcoat: f64,
    clearcoat_gloss: f64,
}