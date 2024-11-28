use std::{f64::consts::PI, sync::Arc};

use glam::FloatExt;
use rand::{thread_rng, Rng};

use crate::{
    hit_info::HitInfo,
    ray::Ray,
    texture::{SolidColorTexture, Texture},
    vec3::{get_rotation_to_z, Vec2, Vec3},
};

const EPS: f64 = 1e-3;

pub trait Material {
    /// returns: attenuation (brdf/pdf), and optionally the scattered ray
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>);
    fn emitted(&self, _u: f64, _v: f64, _p: Vec3) -> Vec3 {
        Vec3::ZERO
    }
}

#[derive(Clone)]
pub struct Diffuse {
    texture: Arc<dyn Texture>,
}

impl Diffuse {
    pub fn new(texture: Arc<dyn Texture>) -> Diffuse {
        Diffuse { texture }
    }

    pub fn from_rgb(rgb: Vec3) -> Diffuse {
        Diffuse {
            texture: Arc::new(SolidColorTexture::from_vec(rgb)),
        }
    }
}

impl Material for Diffuse {
    /// Lambertian BRDF
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let mut rng = thread_rng();
        let r1 = rng.gen_range(0.0..2.0 * PI);
        let r2 = rng.gen::<f64>();
        let r2s = r2.sqrt();
        let w = hit_info.normal;
        let u = if w.x.abs() > 0.1 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        }
        .cross(w)
        .normalize();
        let v = w.cross(u);
        let scatter_dir =
            (u * r1.cos() * r2s + v * r1.sin() * r2s + w * ((1.0 - r2).sqrt())).normalize();
        (
            self.texture.value(hit_info.u, hit_info.v, &hit_info.point),
            Some(Ray::new(
                hit_info.point + hit_info.normal * EPS,
                scatter_dir,
                ray.time(),
            )),
        )
    }
}

#[derive(Clone)]
pub struct Specular {
    texture: Arc<dyn Texture>,
    roughness: f64,
}

impl Specular {
    pub fn new(texture: Arc<dyn Texture>, roughness: f64) -> Specular {
        Specular { texture, roughness }
    }

    pub fn from_rgb(rgb: Vec3, roughness: f64) -> Specular {
        Specular {
            texture: Arc::new(SolidColorTexture::from_vec(rgb)),
            roughness,
        }
    }

    fn f0(&self, hit_info: &HitInfo) -> Vec3 {
        self.texture.value(hit_info.u, hit_info.v, &hit_info.point)
    }

    fn sample_specular(
        v_local: Vec3,
        alpha: f64,
        alpha_squared: f64,
        specular_f0: Vec3,
    ) -> (Vec3, Vec3) {
        let n_local = Vec3::Z;

        // H is a microfact normal
        let h_local = if alpha == 0.0 {
            // no roughness -> no microfacets
            n_local
        } else {
            // sample from the half-vector distribution
            Self::sample_microfacet_normal(v_local, Vec2::new(alpha, alpha))
        };

        // reflect the view direction
        let l_local = (-v_local).reflect(h_local);

        let h_dot_l = h_local.dot(l_local).clamp(EPS, 1.0);
        let n_dot_l = n_local.dot(l_local).clamp(EPS, 1.0);
        let n_dot_v = n_local.dot(v_local).clamp(EPS, 1.0);
        let fresnel = Self::eval_fresnel(specular_f0, h_dot_l);

        let weight = fresnel * Self::sample_specular_weight(alpha_squared, n_dot_l, n_dot_v);
        (weight, l_local)
    }

    /// view_dir: the view direction
    /// alpha2d: the roughness params for x- and y- axis
    /// returns: a sampled microfacet half normal on the microfacet distribution (GGX)
    fn sample_microfacet_normal(view_dir: Vec3, alpha2d: Vec2) -> Vec3 {
        let mut rng = thread_rng();

        // make the orthonormal base v_h, t1, t2
        let v_h = Vec3::new(alpha2d.x * view_dir.x, alpha2d.y * view_dir.y, view_dir.z).normalize();
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

        Vec3::new(alpha2d.x * n_h.x, alpha2d.y * n_h.y, n_h.z.max(0.0)).normalize()
    }
    fn sample_specular_weight(alpha_squared: f64, n_dot_l: f64, n_dot_v: f64) -> f64 {
        let g1v = Self::smith_g1(alpha_squared, n_dot_v * n_dot_v);
        let g1l = Self::smith_g1(alpha_squared, n_dot_l * n_dot_l);
        g1l / (g1v + g1l - g1v * g1l)
    }
    fn smith_g1(alpha_squared: f64, n_dot_s_sqrd: f64) -> f64 {
        2.0 / ((((alpha_squared * (1.0 - n_dot_s_sqrd)) + n_dot_s_sqrd) / n_dot_s_sqrd).sqrt()
            + 1.0)
    }
    fn eval_fresnel(f0: Vec3, cosine: f64) -> Vec3 {
        f0 + (1.0 - f0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Specular {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let view_dir = -ray.direction();

        // transform to tangent space
        let rotation_to_z = get_rotation_to_z(hit_info.normal);
        let v_local = rotation_to_z * view_dir;
        let alpha = self.roughness * self.roughness * self.roughness;
        let alpha_squared = alpha * alpha;

        let (attenuation, dir_local) =
            Self::sample_specular(v_local, alpha, alpha_squared, self.f0(hit_info));

        let scatter_dir = rotation_to_z.inverse() * dir_local;
        (
            attenuation,
            Some(Ray::new(
                hit_info.point + hit_info.normal * EPS,
                scatter_dir,
                ray.time(),
            )),
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Refractive {
    refraction_index: f64,
}

impl Refractive {
    pub fn new(refraction_index: f64) -> Refractive {
        Refractive { refraction_index }
    }

    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 *= r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Refractive {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let mut rng = rand::thread_rng();
        let eps = 1e-3;
        let attenuation = Vec3::ONE;
        let ri = if hit_info.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let cos_theta = (-ray.direction()).dot(hit_info.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let sign: f64; // if refracting, need to negate normal when calculating the shadow acne offset
        let cannot_refract = ri * sin_theta > 1.0;
        let dir = if cannot_refract || Self::reflectance(cos_theta, ri) > rng.gen::<f64>() {
            sign = 1.0;
            Vec3::reflect(ray.direction(), hit_info.normal)
        } else {
            sign = -1.0;
            Vec3::refract(ray.direction(), hit_info.normal, ri)
        };

        let ray = Ray::new(
            hit_info.point + hit_info.normal * (sign * eps),
            dir,
            ray.time(),
        );
        (attenuation, Some(ray))
    }
}

#[derive(Clone)]
pub struct DiffuseLight {
    texture: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    pub fn from_rgb(rgb: Vec3) -> Self {
        Self {
            texture: Arc::new(SolidColorTexture::from_vec(rgb)),
        }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
        self.texture.value(u, v, &p)
    }

    fn scatter(&self, _ray: &Ray, _hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        (Vec3::ZERO, None)
    }
}

#[derive(Clone)]
pub enum MaterialType {
    DIFFUSE(Diffuse),
    SPECULAR(Specular),
    REFRACTIVE(Refractive),
    LIGHT(DiffuseLight),
}

impl Default for MaterialType {
    fn default() -> Self {
        // Self::SPECULAR(Specular { albedo: Vec3::ZERO })
        Self::LIGHT(DiffuseLight::from_rgb(Vec3::ZERO))
    }
}
