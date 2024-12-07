use super::PrincipledBSDF;
use std::f64::consts::PI;

use glam::FloatExt;
// use rand::distributions;
use rand::{thread_rng, Rng};

use crate::{
    hittable::HitInfo,
    ray::Ray,
    vec3::{get_rotation_to_z, Luminance, Vec3},
};

// transformations
pub fn to_local(normal: Vec3, input_world: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal);
    rot * input_world
}

pub fn to_world(normal: Vec3, input_local: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal).inverse();
    rot * input_local
}

// sampling functions
fn sample_ggx_vndf(v_local: Vec3, ax: f64, ay: f64) -> Vec3 {
    let mut rng = thread_rng();

    // make orthonormal base
    let v_h = Vec3::new(ax * v_local.x, ay * v_local.y, v_local.z);
    let v1 = Vec3::new(-v_h.y, v_h.x, 0.0)
        .try_normalize()
        .unwrap_or(Vec3::X);
    let v2 = v_h.cross(v1);

    let r1 = rng.gen::<f64>();
    let r = r1.sqrt();
    let phi = rng.gen_range(0.0..2.0 * PI);
    let s = 0.5 * (1.0 + v_h.z);
    let t1 = r * phi.cos();
    let t2 = (1.0 - t1 * t1).sqrt().lerp(r * phi.sin(), s);

    let n_h = t1 * v1 + t2 * v2 + (1.0 - t1 * t1 - t2 * t2).max(0.0).sqrt() * v_h;

    Vec3::new(ax * n_h.x, ay * n_h.y, n_h.z.max(0.0)).normalize()
}

fn sample_microfacet_normal(anisotropic: f64, roughness: f64, v_local: Vec3) -> Vec3 {
    let aspect = (1.0 - anisotropic * 0.9).sqrt();
    let ax = (roughness.powi(2) / aspect).max(0.001);
    let ay = (roughness.powi(2) * aspect).max(0.001);
    let h = sample_ggx_vndf(v_local, ax, ay);
    if h.z < 0.0 {
        -h
    } else {
        h
    }
}

fn cosine_sample_hemisphere() -> Vec3 {
    let mut rng = thread_rng();
    let phi = rng.gen_range(0.0..2.0 * PI);
    let r2 = rng.gen::<f64>();
    let r2s = r2.sqrt();
    Vec3::new(r2s * phi.cos(), r2s * phi.sin(), (1.0 - r2).sqrt())
}

// utils
fn dielectric_fresnel(cos_theta_i: f64, eta: f64) -> f64 {
    let sin_theta_tsq = eta * eta * (1.0 - cos_theta_i * cos_theta_i);

    // Total internal reflection
    if sin_theta_tsq > 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin_theta_tsq).max(0.0).sqrt();

    let rs = (eta * cos_theta_t - cos_theta_i) / (eta * cos_theta_t + cos_theta_i);
    let rp = (eta * cos_theta_i - cos_theta_t) / (eta * cos_theta_i + cos_theta_t);

    0.5 * (rs * rs + rp * rp)
}

fn schlick_weight(cos_theta: f64) -> f64 {
    let m = (1.0 - cos_theta).max(0.0);
    m.powi(5)
}

// BRDF sample
fn sample_diffuse_next_dir(info: &HitInfo) -> Option<Vec3> {
    let diffuse_dir_local = cosine_sample_hemisphere();
    Some(to_world(info.normal, diffuse_dir_local))
}

fn sample_specular_next_dir(v_local: Vec3, mat: &PrincipledBSDF, info: &HitInfo) -> Option<Vec3> {
    let h_local = sample_microfacet_normal(mat.anisotropic, mat.roughness, v_local);
    let specular_dir_local = (-v_local).reflect(h_local);
    let specular_dir = to_world(info.normal, specular_dir_local);
    if specular_dir.dot(info.normal) <= 0.0 {
        None
    } else {
        Some(specular_dir)
    }
}

fn sample_transmissive_next_dir(
    v_local: Vec3,
    mat: &PrincipledBSDF,
    info: &HitInfo,
) -> Option<Vec3> {
    let h_local = sample_microfacet_normal(mat.anisotropic, mat.roughness, v_local);
    let ri = if info.front_face {
        1.0 / mat.ior
    } else {
        mat.ior
    };

    let cos_theta = v_local.dot(h_local).abs().min(1.0);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let r0 = ((1.0 - ri) / (1.0 + ri)).powi(2);
    let reflectance = r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5);

    let cannot_refract = ri * sin_theta > 1.0;

    let rand_choice: f64 = rand::random();
    let dir_local = if cannot_refract || reflectance > rand_choice {
        (-v_local).reflect(h_local)
    } else {
        (-v_local).refract(h_local, ri)
    };
    Some(to_world(info.normal, dir_local))
}

impl PrincipledBSDF {
    fn lobe_weights(&self) -> (f64, f64, f64, f64) {
        let diffuse_wt = (1.0 - self.metallic) * (1.0 - self.spec_trans);
        let specular_wt = 1.0 - self.spec_trans * (1.0 - self.metallic);
        // TODO
        let glass_wt = self.spec_trans * (1.0 - self.metallic);
        // let clearcoat_wt = 0.25 * self.clearcoat;
        let clearcoat_wt = 0.0;
        (diffuse_wt, specular_wt, glass_wt, clearcoat_wt)
    }

    fn lobe_probabilities(
        &self,
        diffuse_wt: f64,
        specular_wt: f64,
        glass_wt: f64,
        clearcoat_wt: f64,
    ) -> (f64, f64, f64, f64) {
        let inv_total = (diffuse_wt + specular_wt + glass_wt + clearcoat_wt).recip();
        (
            diffuse_wt * inv_total,
            specular_wt * inv_total,
            glass_wt * inv_total,
            clearcoat_wt * inv_total,
        )
    }

    // assume we are already in tangent space
    pub(super) fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v_local = to_local(info.normal, view_dir);

        // weights and probabilities
        let (diffuse_wt, specular_wt, glass_wt, clearcoat_wt) = self.lobe_weights();
        let (diffuse_p, specular_p, glass_p, _clearcoat_p) =
            self.lobe_probabilities(diffuse_wt, specular_wt, glass_wt, clearcoat_wt);

        let rand_choice: f64 = rand::random();
        if rand_choice < diffuse_p {
            sample_diffuse_next_dir(info)
        } else if rand_choice < diffuse_p + specular_p {
            sample_specular_next_dir(v_local, self, info)
        } else if rand_choice < diffuse_p + specular_p + glass_p {
            sample_transmissive_next_dir(v_local, self, info)
        } else {
            None // TODO clearcoat
        }
    }

    pub(super) fn eval(&self, ray: &Ray, light_dir: Vec3, info: &HitInfo) -> (Vec3, f64) {
        // return (Vec3::ONE, 1.0);

        let view_dir = -ray.direction();
        let v_local = to_local(info.normal, view_dir);
        let l_local = to_local(info.normal, light_dir);

        let mut pdf = 0.0;
        let mut brdf = Vec3::ZERO;

        let ri = if info.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };

        let h_local = {
            let h = if l_local.z > 0.0 {
                (v_local + l_local).normalize()
            } else {
                (v_local + l_local * ri).normalize()
            };
            if h.z < 0.0 {
                -h
            } else {
                h
            }
        };

        // weights and probabilities
        let (diffuse_wt, specular_wt, glass_wt, clearcoat_wt) = self.lobe_weights();
        let (diffuse_p, specular_p, glass_p, clearcoat_p) =
            self.lobe_probabilities(diffuse_wt, specular_wt, glass_wt, clearcoat_wt);

        let should_reflect = l_local.z > 0.0 && v_local.z > 0.0;

        // Diffuse
        if diffuse_p > 0.0 && should_reflect {
            let (res_brdf, res_pdf) = self.eval_diffuse(v_local, l_local, h_local);
            brdf += diffuse_wt * res_brdf;
            pdf += diffuse_p * res_pdf;
        }

        // Specular
        if specular_p > 0.0 && should_reflect {
            let (res_brdf, res_pdf) = self.eval_microfacet_reflection(v_local, l_local, h_local);
            brdf += specular_wt * res_brdf;
            pdf += specular_p * res_pdf;
        }

        // Glass
        if glass_p > 0.0 {
            let (res_brdf, res_pdf) = if should_reflect {
                self.eval_microfacet_reflection(v_local, l_local, h_local)
            } else {
                self.eval_microfacet_refraction(ri, v_local, h_local, l_local)
            };
            brdf += glass_wt * res_brdf;
            pdf += glass_p * res_pdf;
        }

        if clearcoat_p > 0.0 && should_reflect {
            // TODO
        }

        (brdf * l_local.z.abs(), pdf)
    }

    fn eval_diffuse(&self, v_local: Vec3, l_local: Vec3, h_local: Vec3) -> (Vec3, f64) {
        let (_, _, csheen) = self.tint_colors();
        let l_dot_h = l_local.dot(h_local);

        // diffuse
        let fl = schlick_weight(l_local.z.abs());
        let fv = schlick_weight(v_local.z.abs());

        let rr = 2.0 * self.roughness * l_dot_h * l_dot_h;

        let f_lambert = 1.0;
        let f_retro = rr * (fl + fv + fl * fv * (rr - 1.0));

        let subsurface_approx = f_lambert; // TODO thin surfaces???

        // sheen
        let f_h = schlick_weight(l_dot_h);
        let sheen = self.sheen * csheen * f_h;

        let brdf = self.base_color
            * PI.recip()
            * (f_retro + subsurface_approx * (1.0 - 0.5 * fl) * (1.0 - 0.5 * fv))
            + sheen;
        let pdf = l_local.z * PI.recip();
        (brdf, pdf)
    }

    fn eval_microfacet_reflection(
        &self,
        v_local: Vec3,
        l_local: Vec3,
        h_local: Vec3,
    ) -> (Vec3, f64) {
        if l_local.z <= 0.0 {
            return (Vec3::ZERO, 0.0);
        }

        let (f0, cspec0, _) = self.tint_colors();

        // I'm not quite sure this is the right way to get the right fresnel
        // but this looks okay
        let schlick_wt = schlick_weight(v_local.dot(h_local).abs());
        let metal_fresnel = Vec3::ONE.lerp(self.base_color, schlick_wt);
        let dieletric_f =
            (dielectric_fresnel(v_local.dot(h_local).abs(), self.ior.recip()) - f0) / (1.0 - f0);
        let dieletric_fresnel = cspec0.lerp(Vec3::ONE, dieletric_f);
        let fresnel = dieletric_fresnel.lerp(metal_fresnel, self.metallic);

        // let aspect = (1.0 - self.anisotropic * 0.9).sqrt();
        // let ax = (self.roughness.powi(2) / aspect).max(0.001);
        // let ay = (self.roughness.powi(2) * aspect).max(0.001);

        let alpha = self.roughness;
        let a2 = alpha * alpha;

        // distribution of half-normals
        let t = h_local.z * (a2 - 1.0) + 1.0;
        let d = a2 / (PI * t * t);

        // maksing/geometric factor
        let gv = v_local.z * (a2 + (1.0 - a2) * l_local.z.powi(2)).sqrt();
        let gl = l_local.z * (a2 + (1.0 - a2) * v_local.z.powi(2)).sqrt();
        let g2 = 2.0 * (l_local.z) * (v_local.z) / (gv + gl);

        let brdf = fresnel * d * g2 / (4.0 * l_local.z.abs() * v_local.z.abs());
        let pdf = d * h_local.z / (4.0 * v_local.dot(h_local).abs());

        (brdf, pdf)
    }

    fn eval_microfacet_refraction(
        &self,
        ri: f64,
        v_local: Vec3,
        h_local: Vec3,
        l_local: Vec3,
    ) -> (Vec3, f64) {
        return (Vec3::ONE, 1.0);

        let ri2 = ri * ri;
        let n_dot_l = l_local.z.abs();
        let n_dot_v = v_local.z.abs();

        let h_dot_l = h_local.dot(l_local).max(0.0);
        let h_dot_v = h_local.dot(v_local).max(0.0);

        let alpha = self.roughness;
        let a2 = alpha * alpha;

        // distribution of half-normals
        let t = h_local.z * (a2 - 1.0) + 1.0;
        let d = a2 / (PI * t * t);

        // maksing/geometric factor
        let gv = v_local.z * (a2 + (1.0 - a2) * l_local.z.powi(2)).sqrt();
        let gl = l_local.z * (a2 + (1.0 - a2) * v_local.z.powi(2)).sqrt();
        let g2 = 2.0 * (l_local.z) * (v_local.z) / (gv + gl);

        let denom = h_dot_l + h_dot_v * ri;
        let jacobian = h_dot_l / (denom * denom);

        let f = dielectric_fresnel(h_dot_v, ri);
        let brdf =
            self.base_color * h_dot_v * ri2 * (1.0 - f) * g2 * d * jacobian / (n_dot_l * n_dot_v);
        let pdf = gv * h_dot_v * d * jacobian / v_local.z;
        (brdf, pdf)
    }

    fn tint_colors(&self) -> (f64, Vec3, Vec3) {
        let eta = self.ior;
        let lum = self.base_color.luminance();
        let ctint = if lum > 0.0 {
            self.base_color / lum
        } else {
            Vec3::ONE
        };

        let f0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
        let cspec0 = f0 * ctint.lerp(Vec3::ONE, self.specular_tint);
        let csheen = Vec3::ONE.lerp(ctint, self.specular_tint);
        (f0, cspec0, csheen)
    }
}
