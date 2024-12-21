use std::sync::Arc;

use crate::{
    interval::Interval,
    ray::Ray,
    vec3::{Mat4, Quat, Vec3},
};

use super::{HitInfo, Hittable, AABB};

// rotate then translate
pub struct Instance {
    object: Arc<dyn Hittable>,
    bbox: AABB,
    rotation: Quat,
    transform: Mat4,
}

impl Instance {
    pub fn new(object: Arc<dyn Hittable>, axis: Vec3, angle: f64, translation: Vec3) -> Instance {
        let rotation = Quat::from_axis_angle(axis, angle);
        let transform = Mat4::from_rotation_translation(rotation, translation);
        let bbox = object.bounding_box().transform(transform);
        Instance {
            object,
            bbox,
            rotation,
            transform,
        }
    }
}

impl Hittable for Instance {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        // translate ray to local coords
        let local_origin = self.transform.inverse().transform_point3(ray.origin());
        let local_dir = self.transform.inverse().transform_vector3(ray.direction());
        let local_ray = Ray::new(local_origin, local_dir, ray.time());

        // ray collision
        let info = self.object.intersects(&local_ray, ray_t)?;

        // transform hit collision back to world coordinates
        let world_point = self.transform.transform_point3(info.point);
        let normal_mat = Mat4::from_quat(self.rotation).inverse().transpose();
        let world_normal = normal_mat
            .transform_vector3(info.geometric_normal)
            .normalize();
        Some(HitInfo {
            point: world_point,
            geometric_normal: world_normal,
            ..info
        })
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }

    fn material(&self) -> Option<&dyn crate::bsdf::BxDFMaterial> {
        self.object.material()
    }

    fn sample(&self, origin: Vec3, time: f64) -> Option<Vec3> {
        let local_origin = self.transform.inverse().transform_point3(origin);
        let local_dir = self.object.sample(local_origin, time);
        let world_dir = local_dir.map(|dir| self.transform.transform_vector3(dir));
        world_dir
    }

    fn pdf(&self, origin: Vec3, direction: Vec3, time: f64) -> f64 {
        let local_origin = self.transform.inverse().transform_point3(origin);
        let local_dir = self.transform.inverse().transform_vector3(direction);
        self.object.pdf(local_origin, local_dir, time)
    }
}
