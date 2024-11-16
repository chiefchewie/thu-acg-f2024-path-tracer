use rand::Rng;

use crate::{hit_info::HitInfo, ray::Ray, vec3::Vec3};

// TODO figure out if there's a way to make this abstrat and not have it suck
// pub trait Material {
//     // returns a bool if this material scatters or not
//     // if scatter: then also contains the scattered ray and attenutation vector:w
//     fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (bool, Vec3, Ray);
// }

#[derive(Debug, Clone, Copy, Default)]
pub struct DiffuseMaterial {
    albedo: Vec3,
}

impl DiffuseMaterial {
    pub fn new(r: f64, g: f64, b: f64) -> DiffuseMaterial {
        DiffuseMaterial {
            albedo: Vec3::new(r, g, b),
        }
    }

    pub fn from_rgb(rgb: Vec3) -> DiffuseMaterial {
        DiffuseMaterial { albedo: rgb }
    }

    pub fn scatter(&self, hit_info: &HitInfo) -> (bool, Vec3, Ray) {
        let mut scatter_dir = Vec3::random_dir() + hit_info.normal;
        if scatter_dir.near_zero() {
            scatter_dir = hit_info.normal;
        }

        let eps = 1e-3;
        (
            true,
            self.albedo,
            Ray::new(hit_info.point + hit_info.normal * eps, scatter_dir),
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SpecularMaterial {
    albedo: Vec3,
}

impl SpecularMaterial {
    pub fn new(r: f64, g: f64, b: f64) -> SpecularMaterial {
        SpecularMaterial {
            albedo: Vec3::new(r, g, b),
        }
    }

    pub fn from_rgb(rgb: Vec3) -> SpecularMaterial {
        SpecularMaterial { albedo: rgb }
    }

    pub fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (bool, Vec3, Ray) {
        let refl = ray.direction().reflect(hit_info.normal);
        let eps = 1e-3;
        (
            true,
            self.albedo,
            Ray::new(hit_info.point + hit_info.normal * eps, refl),
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RefractiveMaterial {
    refraction_index: f64,
}

impl RefractiveMaterial {
    pub fn new(refraction_index: f64) -> RefractiveMaterial {
        RefractiveMaterial { refraction_index }
    }

    pub fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (bool, Vec3, Ray) {
        let mut rng = rand::thread_rng();
        let eps = 1e-3;
        let attenuation = Vec3::new(1.0, 1.0, 1.0);
        let ri = if hit_info.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let cos_theta = (-ray.direction()).dot(&hit_info.normal).min(1.0);
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

        let ray = Ray::new(hit_info.point + hit_info.normal * (sign * eps), dir);
        (true, attenuation, ray)
    }

    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 *= r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Material {
    DIFFUSE(DiffuseMaterial),
    SPECULAR(SpecularMaterial),
    REFRACTIVE(RefractiveMaterial),
}

impl Default for Material {
    fn default() -> Self {
        Self::DIFFUSE(DiffuseMaterial {
            albedo: Vec3::zeroes(),
        })
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
