use crate::{bsdf::MatPtr, ray::Ray, texture::Texture, vec3::Vec3};

#[derive(Clone)]
pub struct HitInfo {
    pub point: Vec3,
    pub geometric_normal: Vec3,
    pub shading_normal: Vec3,
    pub dist: f64,
    pub front_face: bool,
    pub mat: MatPtr,
    pub u: f64,
    pub v: f64,
}

impl HitInfo {
    pub fn new(
        ray: &Ray,
        point: Vec3,
        geometric_normal: Vec3,
        dist: f64,
        mat: MatPtr,
        u: f64,
        v: f64,
    ) -> HitInfo {
        let front_face = ray.direction().dot(geometric_normal) < 0.0;
        let geometric_normal = if front_face {
            geometric_normal.normalize()
        } else {
            -geometric_normal.normalize()
        };

        // normal and bump mapping
        let shading_normal = if let Some(normal_map) = mat.normal_map() {
            let Vec3 { x, y, z } = normal_map.value(u, v, &point);
            let mapped_normal = 2.0 * Vec3::new(x, y, z) - Vec3::ONE;
            let (tangent, bitangent) = get_tangent_basis(geometric_normal);
            (mapped_normal.x * tangent
                + mapped_normal.y * bitangent
                + mapped_normal.z * geometric_normal)
                .normalize()
        } else {
            geometric_normal
        };

        HitInfo {
            point,
            geometric_normal,
            shading_normal,
            dist,
            front_face,
            mat,
            u,
            v,
        }
    }
}

fn get_tangent_basis(normal: Vec3) -> (Vec3, Vec3) {
    let a = if normal.x.abs() > 0.9 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let tangent = normal.cross(a).normalize();
    let bitangent = normal.cross(tangent);
    (tangent, bitangent)
}
