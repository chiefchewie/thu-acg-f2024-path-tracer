use std::f64::consts::PI;

use glam::FloatExt;

use crate::{hittable::HitInfo, material::Material, ray::Ray, vec3::Vec3};

use super::{
    fresnel::{self, schlick_weight}, r0,
    sampling::{cosine_sample_hemisphere, ggx, gtr1, to_local, to_world},
    tint, BxDF, EPS,
};

#[derive(Clone)]

/// things to test
/// smooth dielectric
/// rough dielectric
/// smooth metal
/// rough metal
/// smooth glass
/// rough glass
pub struct PrincipledBSDF {
    base_color: Vec3, // TODO replace with texture

    metallic: f64,
    roughness: f64,
    subsurface: f64,

    specular: f64,
    specular_tint: f64,

    ior: f64,
    spec_trans: f64,

    // anisotropic: f64,
    sheen: f64,
    sheen_tint: f64,

    clearcoat: f64,
    clearcoat_gloss: f64,
}

impl PrincipledBSDF {
    pub fn new(
        base_color: Vec3,
        metallic: f64,
        roughness: f64,
        subsurface: f64,
        specular: f64,
        specular_tint: f64,
        ior: f64,
        spec_trans: f64,
        sheen: f64,
        sheen_tint: f64,
        clearcoat: f64,
        clearcoat_gloss: f64,
    ) -> Self {
        // let diffuse_component = DiffuseBRDF::new(base_color, roughness, subsurface);

        // let c_tint = tint(base_color);
        // let r0 = r0(ior);
        // let ks = Vec3::ONE.lerp(c_tint, specular_tint);
        // let c0 = (specular * r0 * ks).lerp(base_color, metallic);
        // let metal_component = MetalBRDF::new(c0, roughness);

        // let glass_component = GlassBSDF::new(base_color, roughness, ior);
        // let clearcoat_component = ClearcoatBRDF::new(clearcoat_gloss);
        // let sheen_component = SheenBRDF::new(base_color, sheen_tint);
        Self {
            base_color,
            metallic,
            roughness,
            subsurface,
            specular,
            specular_tint,
            ior,
            spec_trans,
            sheen,
            sheen_tint,
            clearcoat,
            clearcoat_gloss,
        }
    }

    fn get_alpha_g(&self) -> f64 {
        (1.0 - self.clearcoat_gloss) * 0.1 + self.clearcoat_gloss * 0.001
    }

    fn lobe_weights(&self) -> (f64, f64, f64, f64) {
        let diffuse_wt = (1.0 - self.metallic) * (1.0 - self.spec_trans);
        let specular_wt = 1.0 - self.spec_trans * (1.0 - self.metallic);
        let glass_wt = self.spec_trans * (1.0 - self.metallic);
        let clearcoat_wt = 0.25 * self.clearcoat;
        (diffuse_wt, specular_wt, glass_wt, clearcoat_wt)
    }

    fn lobe_probabilities(
        &self,
        diffuse_wt: f64,
        specular_wt: f64,
        glass_wt: f64,
        clearcoat_wt: f64,
    ) -> (f64, f64, f64, f64) {
        let inv_total = 1.0 / (diffuse_wt + specular_wt + glass_wt + clearcoat_wt);
        let diffuse_p = diffuse_wt * inv_total;
        let specular_p = specular_wt * inv_total;
        let glass_p = glass_wt * inv_total;
        let clearcoat_p = clearcoat_wt * inv_total;
        (diffuse_p, specular_p, glass_p, clearcoat_p)
    }

    fn sample_diffuse(&self, info: &HitInfo) -> Option<Vec3> {
        Some(to_world(info.normal, cosine_sample_hemisphere()))
    }

    fn sample_specular(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);
        let h = ggx::sample_microfacet_normal(v, self.roughness);
        let specular_dir_local = (-v).reflect(h);
        let specular_dir = to_world(info.normal, specular_dir_local);

        if specular_dir.dot(info.normal) <= 0.0 {
            None
        } else {
            Some(specular_dir)
        }
    }

    fn sample_glass(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);
        let h = ggx::sample_microfacet_normal(v, self.roughness);

        let (eta_i, eta_o) = if info.front_face {
            (1.0, self.ior)
        } else {
            (self.ior, 1.0)
        };

        let f = fresnel::dielectric(v, h, eta_i, eta_o);
        if rand::random::<f64>() < f {
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

    fn sample_clearcoat(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v = to_local(info.normal, view_dir);
        let h = gtr1::sample_microfacet_normal(0.25);
        let specular_dir_local = (-v).reflect(h);
        let specular_dir = to_world(info.normal, specular_dir_local);
        if specular_dir.dot(info.normal) <= 0.0 {
            None
        } else {
            Some(specular_dir)
        }
    }

    fn diffuse_pdf(&self, l: Vec3) -> f64 {
        l.z.abs() / PI
    }

    fn specular_pdf(&self, v: Vec3, l: Vec3, h: Vec3) -> f64 {
        let pdf_h =
            ggx::G1(v, self.roughness) * v.dot(h).abs() * ggx::D(h, self.roughness) / v.z.abs();

        let jacobian = 1.0 / (4.0 * l.dot(h).abs());

        pdf_h * jacobian
    }

    fn glass_pdf(&self, v: Vec3, l: Vec3, h: Vec3, eta_i: f64, eta_o: f64, reflect: bool) -> f64 {
        let pdf_h =
            ggx::G1(v, self.roughness) * v.dot(h).abs() * ggx::D(h, self.roughness) / v.z.abs();

        let f = fresnel::dielectric(v, h, eta_i, eta_o);
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

    fn clearcoat_pdf(&self, v: Vec3, l: Vec3, h: Vec3) -> f64 {
        let pdf_h = ggx::G1(v, 0.25) * v.dot(h).abs() * gtr1::D(l.dot(h).abs(), self.get_alpha_g())
            / v.z.abs();
        let jacobian = 1.0 / (4.0 * l.dot(h).abs());
        pdf_h * jacobian
    }

    // note that the evals here do not include the cosine, only the final public one in
    // PrincipledBSFD::eval does
    fn eval_diffuse(&self, v: Vec3, l: Vec3, h: Vec3) -> Vec3 {
        let l_dot_h = l.dot(h);
        let rr = 2.0 * self.roughness * l_dot_h * l_dot_h;

        let fl = fresnel::schlick_weight(l.z);
        let fv = fresnel::schlick_weight(v.z);

        // diffuse
        let f_retro = rr * (fl + fv + fl * fv * (rr - 1.0));
        let f_d = (1.0 - 0.5 * fl) * (1.0 - 0.5 * fv);

        // approximate subsurface
        let fss90 = 0.5 * rr;
        let f_ss = 1.0.lerp(fss90, fl) * 1.0.lerp(fss90, fv);
        let ss = 1.25 * (f_ss * (1.0 / (l.z + v.z) - 0.5) + 0.5);

        self.base_color / PI * (f_d + f_retro).lerp(ss, self.subsurface)
    }

    fn eval_specular(&self, fresnel: Vec3, v: Vec3, l: Vec3, h: Vec3) -> Vec3 {
        // D term
        let d = ggx::D(h, self.roughness);

        // G term
        let g = ggx::G(v, l, self.roughness);

        // F term
        fresnel * g * d / (4.0 * l.z.abs() * v.z.abs())
    }

    fn eval_glass(&self, v: Vec3, l: Vec3, h: Vec3, eta_i: f64, eta_o: f64, reflect: bool) -> Vec3 {
        // D term
        let d = ggx::D(h, self.roughness);

        // G term
        let g = ggx::G(v, l, self.roughness);

        // F term
        let f = fresnel::dielectric(v, h, eta_i, eta_o);
        if reflect {
            let factor = f * g * d / (4.0 * l.z.abs() * v.z.abs());
            Vec3::splat(factor)
        } else {
            let l_dot_h = l.dot(h);
            let v_dot_h = v.dot(h);
            let term1 = ((l_dot_h * v_dot_h) / (l.z * v.z)).abs();
            let term2 = (eta_o * eta_o) / (eta_i * v_dot_h + eta_o * l_dot_h).powi(2);
            let factor = term1 * term2 * (1.0 - f) * g * d;
            Vec3::splat(factor)
        }
    }

    fn eval_clearcoat(&self, v: Vec3, l: Vec3, h: Vec3) -> Vec3 {
        let d = gtr1::D(l.dot(h).abs(), self.get_alpha_g());

        let g = ggx::G(v, l, 0.25);

        let eta = 1.5;
        let r0 = Vec3::splat(r0(eta));
        let f = fresnel::schlick(r0, l.dot(h));

        l.z.abs() * (f * d * g / (4.0 * l.z.abs() * v.z.abs()))
    }
}

impl BxDF for PrincipledBSDF {
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let (diffuse_wt, specular_wt, glass_wt, clearcoat_wt) = self.lobe_weights();
        let (diffuse_p, specular_p, glass_p, _) =
            self.lobe_probabilities(diffuse_wt, specular_wt, glass_wt, clearcoat_wt);

        let r = rand::random::<f64>();
        if r < diffuse_p {
            self.sample_diffuse(info)
        } else if r < diffuse_p + specular_p {
            self.sample_specular(ray, info)
        } else if r < diffuse_p + specular_p + glass_p {
            self.sample_glass(ray, info)
        } else {
            self.sample_clearcoat(ray, info)
        }
    }

    fn pdf(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> f64 {
        let (diffuse_wt, specular_wt, glass_wt, clearcoat_wt) = self.lobe_weights();
        let (diffuse_p, specular_p, glass_p, clearcoat_p) =
            self.lobe_probabilities(diffuse_wt, specular_wt, glass_wt, clearcoat_wt);

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

        let mut pdf = 0.0;
        if diffuse_p > 0.0 && reflect {
            pdf += diffuse_p * self.diffuse_pdf(l)
        }
        if specular_p > 0.0 && reflect {
            pdf += specular_p * self.specular_pdf(v, l, h)
        }
        if glass_p > 0.0 {
            pdf += glass_p * self.glass_pdf(v, l, h, eta_i, eta_o, reflect)
        }
        if clearcoat_p > 0.0 && reflect {
            pdf += clearcoat_p * self.clearcoat_pdf(v, l, h)
        }

        pdf
    }

    fn eval(&self, view_dir: Vec3, light_dir: Vec3, info: &HitInfo) -> Vec3 {
        let (diffuse_wt, specular_wt, glass_wt, clearcoat_wt) = self.lobe_weights();
        let (diffuse_p, specular_p, glass_p, clearcoat_p) =
            self.lobe_probabilities(diffuse_wt, specular_wt, glass_wt, clearcoat_wt);

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

        let mut brdf = Vec3::ZERO;
        if diffuse_p > 0.0 && reflect {
            let c_tint = tint(self.base_color);
            let c_sheen = Vec3::ONE.lerp(c_tint, self.sheen_tint);
            let sheen_term = self.sheen * c_sheen * schlick_weight(l.dot(h).abs());
            let diffuse_term = self.eval_diffuse(v, l, h);
            brdf += diffuse_wt * (diffuse_term + sheen_term)
        }
        if specular_p > 0.0 && reflect {
            let c_tint = tint(self.base_color);
            let ks = Vec3::ONE.lerp(c_tint, self.specular_tint);
            let c0 = (self.specular * r0(eta_i / eta_o) * ks).lerp(self.base_color, self.metallic);

            let metallic_fresnel = fresnel::schlick(c0, l.dot(h));
            let dielectric_fresnel = Vec3::splat(fresnel::dielectric(v, h, eta_i, eta_o));
            let fresnel = dielectric_fresnel.lerp(metallic_fresnel, self.metallic);

            brdf += specular_wt * self.eval_specular(fresnel, v, l, h)
        }
        if glass_p > 0.0 {
            brdf += glass_wt * self.eval_glass(v, l, h, eta_i, eta_o, reflect)
        }
        if clearcoat_p > 0.0 && reflect {
            brdf += clearcoat_wt * self.eval_clearcoat(v, l, h)
        }
        
        brdf * l.z.abs()
    }
}

impl Material for PrincipledBSDF {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let Some(dir) = self.sample(ray, hit_info) else {
            return (Vec3::ONE, None);
        };

        let pdf = self.pdf(-ray.direction(), dir, hit_info);
        let brdf = self.eval(-ray.direction(), dir, hit_info);
        let brdf_weight = brdf / pdf;

        let eps = EPS * dir.dot(hit_info.normal).signum();
        let next_ray = Ray::new(hit_info.point + eps * hit_info.normal, dir, ray.time());
        (brdf_weight, Some(next_ray))
    }
}
