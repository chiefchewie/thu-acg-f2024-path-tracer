use super::{
    sampling::{to_local, to_world},
    BxDF,
};
use crate::{
    hittable::HitInfo,
    material::Material,
    ray::Ray,
    vec3::Vec3,
};
use rand::{thread_rng, Rng};
use std::f64::consts::PI;

#[derive(Clone)]
pub struct GlassBSDF {
    base_color: Vec3,
    roughness: f64,
    ior: f64,
}

impl GlassBSDF {
    pub fn new(roughness: f64, ior: f64) -> Self {
        Self {
            base_color: Vec3::ONE,
            roughness,
            ior,
        }
    }

    fn F(&self, w: Vec3, h: Vec3, eta_i: f64, eta_o: f64) -> f64 {
        let c = w.dot(h).abs();
        let g_squared = (eta_o / eta_i).powi(2) - 1.0 + c * c;
        if g_squared < 0.0 {
            return 1.0;
        }
        let g = g_squared.sqrt();
        let gmc = g - c;
        let gpc = g + c;
        let x = (c * gpc - 1.0) / (c * gmc + 1.0);
        0.5 * (gmc * gmc) / (gpc * gpc) * (1.0 + x * x)
    }

    fn D(&self, h: Vec3) -> f64 {
        let cos_theta = h.z.max(0.001);
        let alpha2 = (self.roughness * self.roughness).max(0.001);
        let denom = (alpha2 - 1.0) * (cos_theta * cos_theta) + 1.0;
        alpha2 / (PI * denom * denom)
    }

    fn G(&self, v: Vec3, l: Vec3, h: Vec3) -> f64 {
        let g1v = self.G1(v, h);
        let g1l = self.G1(l, h);
        g1v * g1l
    }

    fn G1(&self, w: Vec3, h: Vec3) -> f64 {
        let alpha2 = (self.roughness * self.roughness).max(0.001);
        let cos_theta = w.z.abs();
        2.0 * cos_theta / (cos_theta + (cos_theta * cos_theta * (1.0 - alpha2) + alpha2).sqrt())
    }
}

impl BxDF for GlassBSDF {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);
        let h = sample_microfacet_normal(v, self.roughness);

        let (eta_i, eta_o) = if info.front_face {
            (1.0, self.ior)
        } else {
            (self.ior, 1.0)
        };

        let f = self.F(v, h, eta_i, eta_o);
        if thread_rng().gen::<f64>() < f {
            let r = (-v).reflect(h);
            Some(to_world(info.normal, r))
        } else {
            let mut t = (-v).refract(h, eta_i / eta_o);
            if t == Vec3::ZERO {
                t = (-v).reflect(h);
            }
            Some(to_world(info.normal, t))
        }
    }

    fn pdf(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let v = to_local(info.normal, view_dir);
        let l = to_local(info.normal, light_dir);
        let reflect = l.z * v.z > 0.0;

        let (eta_i, eta_o) = if info.front_face {
            (1.0, self.ior)
        } else {
            (self.ior, 1.0)
        };

        let h = if reflect {
            (l + v).normalize() * v.z.signum()
        } else {
            -(l * eta_o + v * eta_i).normalize()
        };

        let pdf_h = self.G1(v, h) * v.dot(h).abs() * self.D(h) / v.z.abs();

        let f = self.F(v, h, eta_i, eta_o);
        let jacobian = if reflect {
            f * 1.0 / (4.0 * l.dot(h).abs())
        } else {
            let v_dot_h = v.dot(h);
            let l_dot_h = l.dot(h);
            (1.0 - f) * (eta_o * eta_o * l_dot_h.abs())
                / (eta_i * v_dot_h + eta_o * l_dot_h).powi(2)
        };

        pdf_h * jacobian
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let v = to_local(info.normal, view_dir);
        let l = to_local(info.normal, light_dir);
        let reflect = l.z * v.z > 0.0;

        let (eta_i, eta_o) = if info.front_face {
            (1.0, self.ior)
        } else {
            (self.ior, 1.0)
        };

        let h = if reflect {
            (l + v).normalize() * v.z.signum()
        } else {
            -(l * eta_o + v * eta_i).normalize()
        };

        // D term
        let d = self.D(h);

        // G term
        let g = self.G(v, l, h);

        // F term
        let f = self.F(v, h, eta_i, eta_o);
        let result = if reflect {
            let factor = f * g * d / (4.0 * l.z.abs() * v.z.abs());
            Vec3::splat(factor)
        } else {
            let l_dot_h = l.dot(h);
            let v_dot_h = v.dot(h);
            let term1 = ((l_dot_h * v_dot_h) / (l.z * v.z)).abs();
            let term2 = (eta_o * eta_o) / (eta_i * v_dot_h + eta_o * l_dot_h).powi(2);
            let factor = term1 * term2 * (1.0 - f) * g * d;
            Vec3::splat(factor)
        };
        result * l.z.abs()
    }
}

impl Material for GlassBSDF {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let Some(dir) = self.sample(ray, hit_info) else {
            return (self.base_color, None);
        };
        
        // default slow impl
        // let pdf = self.pdf(-ray.direction(), dir, hit_info);
        // let brdf = self.eval(-ray.direction(), dir, hit_info);
        // let brdf_weight = brdf / pdf;

        // simplified faster impl
        let v = to_local(hit_info.normal, -ray.direction());
        let l = to_local(hit_info.normal, dir);
        let reflect = l.z * v.z > 0.0;
        let (eta_i, eta_o) = if hit_info.front_face {
            (1.0, self.ior)
        } else {
            (self.ior, 1.0)
        };
        let h = if reflect {
            (l + v).normalize() * v.z.signum()
        } else {
            -(l * eta_o + v * eta_i).normalize()
        };
        let brdf_weight = self.base_color * self.G1(v, h);

        let eps = 1e-3 * dir.dot(hit_info.normal).signum();
        let next_ray = Ray::new(hit_info.point + hit_info.normal * eps, dir, ray.time());
        (brdf_weight, Some(next_ray))
    }
}

// TODO reorg these in sampling.rs
fn sample_microfacet_normal(v: Vec3, roughness: f64) -> Vec3 {
    sample_ggx_vndf(v, roughness * roughness)
}

fn sample_ggx_vndf(v: Vec3, a2: f64) -> Vec3 {
    // stretch view
    let v = Vec3::new(v.x * a2, v.y * a2, v.z).normalize();

    // orthonormal basis
    let t1 = if v.z < 0.9999 {
        v.cross(Vec3::Z).normalize()
    } else {
        Vec3::X
    };
    let t2 = t1.cross(v);

    // sample
    let e1 = thread_rng().gen::<f64>();
    let e2 = thread_rng().gen::<f64>();
    let a = 1.0 / (1.0 + v.z);
    let r = e1.sqrt();
    let phi = if e2 < a {
        e2 / a * PI
    } else {
        PI + (e2 - a) / (1.0 - a) * PI
    };
    let p1 = r * phi.cos();
    let p2 = r * phi.sin() * if e2 < a { 1.0 } else { v.z };

    let n = p1 * t1 + p2 * t2 + (1.0 - p1 * p1 - p2 * p2).max(0.0).sqrt() * v;
    let unstretched = Vec3::new(a2 * n.x, a2 * n.y, n.z.max(0.0));
    unstretched.normalize()
}
