use crate::{hittable::Hittable, vec3::Vec3};

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

impl Hittable for PointLight {
    fn intersects(
        &self,
        _ray: &crate::ray::Ray,
        _ray_t: crate::interval::Interval,
    ) -> Option<crate::hittable::HitInfo> {
        None
    }

    fn bounding_box(&self) -> crate::hittable::AABB {
        crate::hittable::AABB::default()
    }

    fn material(&self) -> Option<&dyn crate::bsdf::BxDFMaterial> {
        None
    }

    fn sample(&self, origin: Vec3, _time: f64) -> Option<Vec3> {
        Some((self.position - origin).normalize())
    }

    fn pdf(&self, origin: Vec3, direction: Vec3, time: f64) -> f64 {
        let _ = time;
        let _ = direction;
        let _ = origin;
        todo!()
    }
}
