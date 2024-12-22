use super::{
    sampling::{cosine_sample_hemisphere, to_local, to_world},
    BxDFMaterial, EPS,
};
use crate::{
    hittable::HitInfo,
    ray::Ray,
    texture::{ImageTexture, SolidTexture, Texture},
    vec3::Vec3,
};
use std::{f64::consts::PI, sync::Arc};

#[derive(Clone)]
pub struct DiffuseBRDF {
    base_color: Arc<dyn Texture<Vec3>>,
    normal_map: Option<Arc<ImageTexture>>,
}

// Lambertian diffuse, NOT the one used in PrincipledBSDF
impl DiffuseBRDF {
    pub fn new(base_color: Arc<dyn Texture<Vec3>>) -> Self {
        Self {
            base_color,
            normal_map: None,
        }
    }

    pub fn from_rgb(base_color: Vec3) -> Self {
        Self {
            base_color: Arc::new(SolidTexture::new(base_color)),
            normal_map: None,
        }
    }

    pub fn with_normal(base_color: Vec3, normal_map: ImageTexture) -> Self {
        Self {
            base_color: Arc::new(SolidTexture::new(base_color)),
            normal_map: Some(Arc::new(normal_map)),
        }
    }

    pub fn from_textures(color_texture: Arc<dyn Texture<Vec3>>, normal_map: Option<ImageTexture>) -> Self {
        Self {
            base_color: color_texture,
            normal_map: normal_map.map(Arc::new),
        }
    }
}

impl BxDFMaterial for DiffuseBRDF {
    fn sample(&self, _ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let diffuse_dir_local = cosine_sample_hemisphere();
        Some(to_world(info.shading_normal, diffuse_dir_local))
    }

    fn pdf(&self, _view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let l = to_local(info.shading_normal, light_dir);
        l.z.abs() / PI
    }

    fn eval(&self, _view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let color = self.base_color.value(info.u, info.v, &info.point);
        let l = to_local(info.shading_normal, light_dir);
        l.z.abs() * (color / PI)
    }

    /// optimized version combining sample, pdf, and eval
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> Option<(Vec3, Ray)> {
        let color = self
            .base_color
            .value(hit_info.u, hit_info.v, &hit_info.point);
        let dir = self.sample(ray, hit_info)?;
        let next_ray = Ray::new(
            hit_info.point + EPS * hit_info.geometric_normal,
            dir,
            ray.time(),
        );
        Some((color, next_ray))
    }

    fn normal_map(&self) -> Option<&ImageTexture> {
        self.normal_map.as_deref()
    }
}
