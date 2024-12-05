use std::f64::consts::PI;

use glam::FloatExt;
use rand::{thread_rng, Rng};

use crate::{
    hittable::HitInfo,
    material::Material,
    ray::Ray,
    vec3::{get_rotation_to_z, Luminance, Vec3},
};

#[derive(Clone)]
pub struct BRDF {
    pub base_color: Vec3, // TODO replace with texture
    pub metallic: f64,
    pub roughness: f64, // exclusive range (0..1)
    pub subsurface: f64,
    pub spec_trans: f64,
    pub specular_tint: f64,
    pub sheen: f64,
    pub sheen_tint: f64,
    pub clearcoat: f64,
    pub clearcoat_roughness: f64,
    pub ior: f64,
    pub anisotropic: f64,
}

impl BRDF {
    // assume we are already in tangent space
    fn sample(&self, ray: &Ray, info: &HitInfo) -> Option<Vec3> {
        let view_dir = -ray.direction();
        let v_local = to_local(info.normal, view_dir);

        let (_, cspec0, _) = self.tint_colors();

        // weights and probabilities
        let dielectric_wt = 1.0 - self.metallic;
        let metal_wt = self.metallic;

        let schlick_wt = schlick_weight(v_local.z);

        let diffuse_p = dielectric_wt * self.base_color.luminance();
        let dielectric_p = dielectric_wt * cspec0.lerp(Vec3::ONE, schlick_wt).luminance();
        let metal_p = metal_wt * self.base_color.lerp(Vec3::ONE, schlick_wt).luminance();
        let total = diffuse_p + dielectric_p + metal_p;

        let diffuse_p = diffuse_p / total;
        let dielectric_p = dielectric_p / total;
        let metal_p = metal_p / total;

        let rand_choice: f64 = rand::random();

        if rand_choice < diffuse_p {
            let diffuse_dir_local = cosine_sample_hemisphere();
            Some(to_world(info.normal, diffuse_dir_local))
        } else if rand_choice < diffuse_p + dielectric_p + metal_p {
            let aspect = (1.0 - self.anisotropic * 0.9).sqrt();
            let ax = (self.roughness.powi(2) / aspect).max(0.001);
            let ay = (self.roughness.powi(2) * aspect).max(0.001);
            let h_local = {
                let h = sample_ggx_vndf(v_local, ax, ay);
                if h.z < 0.0 {
                    -h
                } else {
                    h
                }
            };
            let specular_dir_local = (-v_local).reflect(h_local);
            let specular_dir = to_world(info.normal, specular_dir_local);
            if specular_dir.dot(info.normal) <= 0.0 {
                None
            } else {
                Some(specular_dir)
            }
        } else {
            None // TODO clearcoat and transmissive
        }
    }

    fn eval(&self, ray: &Ray, light_dir: Vec3, info: &HitInfo) -> (Vec3, f64) {
        let view_dir = -ray.direction();
        let v_local = to_local(info.normal, view_dir);
        let l_local = to_local(info.normal, light_dir);

        let mut pdf = 0.0;
        let mut brdf = Vec3::ZERO;

        let h_local = {
            let h = if l_local.z > 0.0 {
                (v_local + l_local).normalize()
            } else {
                (v_local + l_local * self.ior).normalize()
            };
            if h.z < 0.0 {
                -h
            } else {
                h
            }
        };

        // tint
        let (f0, cspec0, csheen) = self.tint_colors();

        // weights and probabilities
        let dielectric_wt = 1.0 - self.metallic;
        let metal_wt = self.metallic;

        let schlick_wt = schlick_weight(v_local.z);

        let diffuse_p = dielectric_wt * self.base_color.luminance();
        let dielectric_p = dielectric_wt * cspec0.lerp(Vec3::ONE, schlick_wt).luminance();
        let metal_p = metal_wt * self.base_color.lerp(Vec3::ONE, schlick_wt).luminance();
        let total = diffuse_p + dielectric_p + metal_p;

        let diffuse_p = diffuse_p / total;
        let dielectric_p = dielectric_p / total;
        let metal_p = metal_p / total;

        let should_reflect = l_local.z * v_local.z > 0.0;
        let v_dot_h = v_local.dot(h_local).abs();

        if should_reflect {
            // Diffuse
            if diffuse_p > 0.0 {
                let (res_brdf, res_pdf) = self.eval_diffuse(csheen, v_local, l_local, h_local);
                brdf += res_brdf * dielectric_wt;
                pdf += res_pdf * diffuse_p;
            }

            // Specular
            if dielectric_p > 0.0 {
                let dieletric_f = (dielectric_fresnel(v_dot_h, self.ior.recip()) - f0) / (1.0 - f0);
                let dieletric_color = cspec0.lerp(Vec3::ONE, dieletric_f);
                let (res_brdf, res_pdf) =
                    self.eval_microfacet_reflection(v_local, l_local, h_local, dieletric_color);
                brdf += res_brdf * dielectric_wt;
                pdf += res_pdf * dielectric_p;
            }
            if metal_p > 0.0 {
                let metal_f = schlick_weight(v_dot_h);
                let metal_color = self.base_color.lerp(Vec3::ONE, metal_f);
                let (res_brdf, res_pdf) =
                    self.eval_microfacet_reflection(v_local, l_local, h_local, metal_color);
                brdf += res_brdf * metal_wt;
                pdf += res_pdf * metal_p;
            }
        }

        (brdf * l_local.z.abs(), pdf)
    }

    fn eval_microfacet_reflection(
        &self,
        v_local: Vec3,
        l_local: Vec3,
        h_local: Vec3,
        fresnel: Vec3,
    ) -> (Vec3, f64) {
        if l_local.z <= 0.0 {
            return (Vec3::ZERO, 0.0);
        }

        let aspect = (1.0 - self.anisotropic * 0.9).sqrt();
        let ax = (self.roughness.powi(2) / aspect).max(0.001);
        let ay = (self.roughness.powi(2) * aspect).max(0.001);

        // distribution of half-normals
        let d = gtr2_aniso(h_local.z, h_local.x, h_local.y, ax, ay);

        // maksing/geometric factor
        let g1 = smith_g_aniso(v_local.z.abs(), v_local.x, v_local.y, ax, ay);
        let g2 = g1 * smith_g_aniso(l_local.z.abs(), l_local.x, l_local.y, ax, ay);

        let pdf = g1 * d / (4.0 * v_local.z);
        let brdf = fresnel * d * g2 / (4.0 * l_local.z * v_local.z);
        (brdf, pdf)
    }

    fn eval_diffuse(
        &self,
        csheen: Vec3,
        v_local: Vec3,
        l_local: Vec3,
        h_local: Vec3,
    ) -> (Vec3, f64) {
        let l_dot_h = l_local.dot(h_local);

        // diffuse
        let fl = schlick_weight(l_local.z);
        let fv = schlick_weight(v_local.z);

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
        let csheen = Vec3::ONE.lerp(ctint, self.sheen_tint);
        (f0, cspec0, csheen)
    }
}

fn schlick_weight(cos_theta: f64) -> f64 {
    let m = (1.0 - cos_theta).max(0.0);
    m.powi(5)
}

fn cosine_sample_hemisphere() -> Vec3 {
    let mut rng = thread_rng();
    let phi = rng.gen_range(0.0..2.0 * PI);
    let r2 = rng.gen::<f64>();
    let r2s = r2.sqrt();
    Vec3::new(r2s * phi.cos(), r2s * phi.sin(), (1.0 - r2).sqrt())
}

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

impl Material for BRDF {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let eps = 1e-3;

        let sampled_dir = self.sample(ray, hit_info);
        let Some(light_dir) = sampled_dir else {
            return (Vec3::ONE, None);
        };
        let (brdf, pdf) = self.eval(ray, light_dir, hit_info);

        let _t1 = brdf / pdf;
        let _t2 = self.base_color;

        let sign = light_dir.dot(hit_info.normal).signum();
        (
            brdf / pdf,
            Some(Ray::new(
                hit_info.point + hit_info.normal * (eps * sign),
                light_dir,
                ray.time(),
            )),
        )
    }
}

fn to_local(normal: Vec3, input_world: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal);
    rot * input_world
}

fn to_world(normal: Vec3, input_local: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal).inverse();
    rot * input_local
}

fn gtr2_aniso(n_dot_h: f64, h_dot_x: f64, h_dot_y: f64, ax: f64, ay: f64) -> f64 {
    let a = h_dot_x / ax;
    let b = h_dot_y / ay;
    let c = a * a + b * b + n_dot_h * n_dot_h;
    (PI * ax * ay * c * c).recip()
}

fn smith_g_aniso(n_dot_v: f64, v_dot_x: f64, v_dot_y: f64, ax: f64, ay: f64) -> f64 {
    let a = v_dot_x * ax;
    let b = v_dot_y * ay;
    let c = n_dot_v;
    (2.0 * n_dot_v) / (n_dot_v + (a * a + b * b + c * c).sqrt())
}
