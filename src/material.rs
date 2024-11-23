use std::{f64::consts::PI, rc::Rc};

use rand::{thread_rng, Rng};

use crate::{
    hit_info::HitInfo,
    ray::Ray,
    texture::{SolidColorTexture, Texture},
    vec3::Vec3,
};

const EPS: f64 = 1e-3;

pub trait Material {
    // returns a bool if this material scatters or not
    // if scatter: then also contains the scattered ray and attenutation vector:w
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Option<Ray>, Vec3);
}

#[derive(Clone)]
pub struct Diffuse {
    texture: Rc<dyn Texture>,
}

impl Diffuse {
    pub fn new(texture: Rc<dyn Texture>) -> Diffuse {
        Diffuse { texture }
    }

    pub fn from_rgb(rgb: Vec3) -> Diffuse {
        Diffuse {
            texture: Rc::new(SolidColorTexture::from_vec(rgb)),
        }
    }
}

impl Material for Diffuse {
    /// Lambertian BRDF
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Option<Ray>, Vec3) {
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
            Some(Ray::new(
                hit_info.point + hit_info.normal * EPS,
                scatter_dir,
                ray.time(),
            )),
            self.texture.value(hit_info.u, hit_info.v, &hit_info.point),
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Specular {
    albedo: Vec3,
}

impl Specular {
    pub fn new(r: f64, g: f64, b: f64) -> Specular {
        Specular {
            albedo: Vec3::new(r, g, b),
        }
    }

    pub fn from_rgb(rgb: Vec3) -> Specular {
        Specular { albedo: rgb }
    }
}

impl Material for Specular {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Option<Ray>, Vec3) {
        let refl_dir = ray.direction().reflect(hit_info.normal);
        (
            Some(Ray::new(
                hit_info.point + hit_info.normal * EPS,
                refl_dir,
                ray.time(),
            )),
            self.albedo,
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
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Option<Ray>, Vec3) {
        let mut rng = rand::thread_rng();
        let eps = 1e-3;
        let attenuation = Vec3::new(1.0, 1.0, 1.0);
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
        (Some(ray), attenuation)
    }
}

#[derive(Clone)]
pub enum MaterialType {
    DIFFUSE(Diffuse),
    SPECULAR(Specular),
    REFRACTIVE(Refractive),
}

impl Default for MaterialType {
    fn default() -> Self {
        Self::SPECULAR(Specular { albedo: Vec3::ZERO })
    }
}

/*

#[derive(Debug, Clone, Copy, Default)]
pub enum MaterialType {
    #[default]
    DIFFUSE,
    SPECULAR, //REFRACTIVE
}
#[derive(Debug, Clone, Copy, Default)]
pub struct Material {
    albedo: Vec3,
    mat_type: MaterialType,
}

// TODO smoothness param to blend between diffuse and specular?
impl Material {
    pub fn new(albedo: Vec3, mat_type: MaterialType) -> Material {
        Material { albedo, mat_type }
    }

    pub fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (bool, Vec3, Ray) {
        match self.mat_type {
            MaterialType::DIFFUSE => self.lambertian_scatter(ray, hit_info),
            MaterialType::SPECULAR => self.metal_scatter(ray, hit_info),
            // MaterialType::REFRACTIVE => self.refractive_scatter(ray, hit_info),
        }
    }

    // returns a bool if this material scatters or not
    // if scatter: then also contains attenutation vector, scattered ray
    fn lambertian_scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (bool, Vec3, Ray) {
        let mut scatter_dir = Vec3::random_dir() + hit_info.normal;
        if scatter_dir.near_zero() {
            scatter_dir = hit_info.normal;
        }

        (true, self.albedo, Ray::new(hit_info.point, scatter_dir))
    }

    fn metal_scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (bool, Vec3, Ray) {
        let refl = ray.direction().reflect(hit_info.normal);
        (true, self.albedo, Ray::new(hit_info.point, refl))
    }

    // fn refractive_scatter(&self, ray:&Ray,hit_info:&HitInfo) -> (bool, Vec3, Ray) {

    // }
}

 */
