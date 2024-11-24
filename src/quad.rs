use crate::{
    aabb::AABB, hit_info::HitInfo, interval::Interval, material::MaterialType, ray::Ray,
    vec3::Vec3, Hittable,
};

pub struct Quad {
    q: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    normal: Vec3,
    d: f64,
    bbox: AABB,
    material: MaterialType,
}

impl Quad {
    pub fn new(q: Vec3, u: Vec3, v: Vec3, material: MaterialType) -> Quad {
        let b1 = AABB::new(q, q + u + v);
        let b2 = AABB::new(q + u, q + v);
        let bbox = b1.union(b2);

        let n = u.cross(v);
        let normal = n.normalize();
        let d = normal.dot(q);
        let w = n / n.length_squared();
        Quad {
            q,
            u,
            v,
            w,
            normal,
            d,
            bbox,
            material,
        }
    }
}

impl Hittable for Quad {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let eps = 1e-8;
        let nd = self.normal.dot(ray.direction());

        if nd.abs() < eps {
            return None;
        }

        let t = (self.d - self.normal.dot(ray.origin())) / nd;
        if !ray_t.contains(t) {
            return None;
        }

        let intersection = ray.at(t);
        let p = intersection - self.q;
        let alpha = self.w.dot(p.cross(self.v));
        let beta = self.w.dot(self.u.cross(p));
        if !(0.0..=1.0).contains(&alpha) || !(0.0..=1.0).contains(&beta) {
            return None;
        }

        Some(HitInfo::new(
            ray,
            ray.at(t),
            self.normal,
            t,
            self.material.clone(),
            alpha,
            beta,
        ))
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
