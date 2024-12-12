// An implementation of https://www.graphics.cornell.edu/%7Ebjw/microfacetbsdf.pdf
// with help from https://schuttejoe.github.io/post/ggximportancesamplingpart1/
// and https://schuttejoe.github.io/post/ggximportancesamplingpart2/

use std::sync::Arc;

use super::{
    sampling::{ggx, to_local, to_world},
    BxDFMaterial, EPS,
};
use crate::{
    hittable::HitInfo,
    ray::Ray,
    texture::{SolidTexture, Texture},
    vec3::Vec3,
};
use rand::{thread_rng, Rng};

#[derive(Clone)]
pub struct GlassBSDF {
    base_color: Arc<dyn Texture<Vec3>>,
    roughness: Arc<dyn Texture<f64>>,
    _anisotropic: f64,
    ior: f64,
}

impl GlassBSDF {
    pub fn new(
        base_color: Arc<dyn Texture<Vec3>>,
        roughness: Arc<dyn Texture<f64>>,
        anisotropic: f64,
        ior: f64,
    ) -> Self {
        Self {
            base_color,
            roughness,
            _anisotropic: anisotropic,
            ior,
        }
    }

    pub fn basic(ior: f64) -> Self {
        Self {
            base_color: Arc::new(SolidTexture::new(Vec3::ONE)),
            roughness: Arc::new(SolidTexture::new(0.001)),
            _anisotropic: 0.0,
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

impl BxDFMaterial for GlassBSDF {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);

        let roughness = self.roughness.value(info.u, info.v, &info.point);
        let h = ggx::sample_microfacet_normal(v, roughness);

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

        let roughness = self.roughness.value(info.u, info.v, &info.point);
        let pdf_h = ggx::G1(v, roughness) * v.dot(h).abs() * ggx::D(h, roughness) / v.z.abs();

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
        let roughness = self.roughness.value(info.u, info.v, &info.point);
        let d = ggx::D(h, roughness);

        // G term
        let g = ggx::G(v, l, roughness);

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

    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> Option<(Vec3, Ray)> {
        let dir = self.sample(ray, hit_info)?;

        // simplified faster impl
        let v = to_local(hit_info.normal, -ray.direction());

        let base_color = self
            .base_color
            .value(hit_info.u, hit_info.v, &hit_info.point);
        let roughness = self
            .roughness
            .value(hit_info.u, hit_info.v, &hit_info.point);
        let brdf_weight = base_color * ggx::G1(v, roughness);

        let eps = EPS * dir.dot(hit_info.normal).signum();
        let next_ray = Ray::new(hit_info.point + eps * hit_info.normal, dir, ray.time());
        Some((brdf_weight, next_ray))
    }
}
