use crate::{
    interval::Interval,
    ray::Ray,
    vec3::Vec3,
};

pub struct AABB {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl AABB {
    pub fn new(x: Interval, y: Interval, z: Interval) -> AABB {
        AABB { x, y, z }
    }

    pub fn from(a: &Vec3, b: &Vec3) -> AABB {
        let x = if a.x < b.x {
            Interval::new(a.x, b.x)
        } else {
            Interval::new(b.x, a.x)
        };

        let y = if a.y < b.y {
            Interval::new(a.y, b.y)
        } else {
            Interval::new(b.y, a.y)
        };

        let z = if a.z < b.z {
            Interval::new(a.z, b.z)
        } else {
            Interval::new(b.z, a.z)
        };

        AABB { x, y, z }
    }

    pub fn intersects(&self, ray: &Ray, ray_t: Interval) -> bool {
        let sizes = Vec3::new(
            self.x.size() / 2.0,
            self.y.size() / 2.0,
            self.z.size() / 2.0,
        );
        let ro = ray.origin() - sizes;
        let rd = ray.direction();

        let m = 1.0 / rd;
        let n = m * ro;
        let k = m.abs() * sizes;

        let t1 = -n - k;
        let t2 = -n + k;
        let t_n = t1.max_element();
        let t_f = t2.min_element();
        if t_n > t_f {
            false
        } else {
            ray_t.surrounds(t_n)
        }
    }
}
