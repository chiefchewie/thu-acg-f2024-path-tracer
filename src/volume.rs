use std::{f64::INFINITY, rc::Rc};

use rand::{thread_rng, Rng};

use crate::{
    hit_info::HitInfo,
    interval::{self, Interval},
    material::MaterialType,
    ray::Ray,
    texture::Texture,
    vec3::Vec3,
    Hittable,
};

pub struct HomogeneousVolume {
    boundary: Rc<dyn Hittable>,
    negative_inv_density: f64,
    phase_function: IsotropicMaterial,
}

impl HomogeneousVolume {
    pub fn from_texture(
        boundary: Rc<dyn Hittable>,
        density: f64,
        texture: Rc<dyn Texture>,
    ) -> Self {
        HomogeneousVolume {
            boundary,
            negative_inv_density: -1.0 / density,
            phase_function: IsotropicMateria::from_texture(texture),
        }
    }

    pub fn from_albedo(boundary: Rc<dyn Hittable>, density: f64, albedo: Vec3) -> Self {
        HomogeneousVolume {
            boundary,
            negative_inv_density: -1.0 / density,
            phase_function: IsotropicMateria::from_albedo(albedo),
        }
    }
}

impl Hittable for HomogeneousVolume {
    fn intersects(&self, _ray: &Ray, _ray_t: Interval) -> Option<HitInfo> {
        todo!()
    }

    fn bounding_box(&self) -> crate::aabb::AABB {
        self.boundary.bounding_box()
    }
}
