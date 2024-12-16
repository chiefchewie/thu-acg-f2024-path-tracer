use crate::{bsdf::MatPtr, ray::Ray, vec3::Vec3};

#[derive(Clone)]
pub struct HitInfo {
    pub point: Vec3,
    pub geometric_normal: Vec3,
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
        let normal = if front_face {
            geometric_normal.normalize()
        } else {
            -geometric_normal.normalize()
        };
        HitInfo {
            point,
            geometric_normal: normal,
            dist,
            front_face,
            mat,
            u,
            v,
        }
    }
}
