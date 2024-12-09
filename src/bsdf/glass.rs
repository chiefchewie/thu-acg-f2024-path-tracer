// An implementation of https://www.graphics.cornell.edu/%7Ebjw/microfacetbsdf.pdf
// with help from https://schuttejoe.github.io/post/ggximportancesamplingpart1/
// and https://schuttejoe.github.io/post/ggximportancesamplingpart2/

use super::{
    sampling::{ggx, to_local, to_world},
    BxDF,
};
use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};
use rand::{thread_rng, Rng};

#[derive(Clone)]
pub struct GlassBSDF {
    base_color: Vec3,
    roughness: f64,
    ior: f64,
}

impl GlassBSDF {
    pub fn new(roughness: f64, ior: f64) -> Self {
        Self {
            base_color: Vec3::new(0.87, 0.2, 0.1),
            roughness,
            ior,
        }
    }

    fn dielectric_fresnel(&self, w: Vec3, h: Vec3, eta_i: f64, eta_o: f64) -> f64 {
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
}

impl BxDF for GlassBSDF {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);
        let h = ggx::sample_microfacet_normal(v, self.roughness);

        let (eta_i, eta_o) = if info.front_face {
            (1.0, self.ior)
        } else {
            (self.ior, 1.0)
        };

        let f = self.dielectric_fresnel(v, h, eta_i, eta_o);
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

        let pdf_h =
            ggx::G1(v, self.roughness) * v.dot(h).abs() * ggx::D(h, self.roughness) / v.z.abs();

        let f = self.dielectric_fresnel(v, h, eta_i, eta_o);
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
        let d = ggx::D(h, self.roughness);

        // G term
        let g = ggx::G(v, l, self.roughness);

        // F term
        let f = self.dielectric_fresnel(v, h, eta_i, eta_o);
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
        let brdf_weight = self.base_color * ggx::G1(v, self.roughness);

        let eps = 1e-3 * dir.dot(hit_info.normal).signum();
        let next_ray = Ray::new(hit_info.point + hit_info.normal * eps, dir, ray.time());
        (brdf_weight, Some(next_ray))
    }
}
