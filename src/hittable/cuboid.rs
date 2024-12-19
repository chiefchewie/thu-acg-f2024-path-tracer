use crate::{bsdf::MatPtr, vec3::Vec3};

use super::{Hittable, HittableList, Quad};

pub struct Cuboid {
    sides: HittableList,
    material: MatPtr,
}

impl Cuboid {
    pub fn new(a: Vec3, b: Vec3, mat: MatPtr) -> Cuboid {
        let mut sides = HittableList::new();
        let min = a.min(b);
        let max = a.max(b);
        let dx = Vec3::ZERO.with_x(max.x - min.x);
        let dy = Vec3::ZERO.with_y(max.y - min.y);
        let dz = Vec3::ZERO.with_z(max.z - min.z);
        sides.add(Quad::new(
            Vec3::new(min.x, min.y, max.z),
            dx,
            dy,
            mat.clone(),
        )); // front
        sides.add(Quad::new(
            Vec3::new(max.x, min.y, max.z),
            -dz,
            dy,
            mat.clone(),
        )); // right
        sides.add(Quad::new(
            Vec3::new(max.x, min.y, min.z),
            -dx,
            dy,
            mat.clone(),
        )); // back
        sides.add(Quad::new(
            Vec3::new(min.x, min.y, min.z),
            dz,
            dy,
            mat.clone(),
        )); // left
        sides.add(Quad::new(
            Vec3::new(min.x, max.y, max.z),
            dx,
            -dz,
            mat.clone(),
        )); // top
        sides.add(Quad::new(
            Vec3::new(min.x, min.y, min.z),
            dx,
            dz,
            mat.clone(),
        )); // bottom
        Cuboid {
            sides,
            material: mat,
        }
    }
}

impl Hittable for Cuboid {
    fn intersects(
        &self,
        ray: &crate::ray::Ray,
        ray_t: crate::interval::Interval,
    ) -> Option<super::HitInfo> {
        self.sides.intersects(ray, ray_t)
    }

    fn bounding_box(&self) -> super::AABB {
        self.sides.bounding_box()
    }

    fn material(&self) -> Option<&dyn crate::bsdf::BxDFMaterial> {
        Some(self.material.as_ref())
    }

    fn sample_surface(&self, hit_info: &super::HitInfo, time: f64) -> Option<(Vec3, Vec3, f64)> {
        self.sides.sample_surface(hit_info, time)
    }
}
