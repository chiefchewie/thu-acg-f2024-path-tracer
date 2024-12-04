use std::{f64::consts::PI, sync::Arc};

use rand::{thread_rng, Rng};

use crate::{
    bsdf::BRDF, hittable::hit_info::HitInfo, ray::Ray, texture::{SolidColorTexture, Texture}, vec3::{random_vector, Vec3}
};

const EPS: f64 = 1e-3;

pub trait Material: Send + Sync {
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
}

impl Material for Specular {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let attenuation = self.texture.value(hit_info.u, hit_info.v, &hit_info.point);
        let reflect_dir = ray.direction().reflect(hit_info.normal);
        let scatter_dir = reflect_dir + (self.roughness * random_vector());
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
pub struct MixMaterial {
    t: f64, // 0 = use mat1 entirely, 1 = use mat2 entirely
    mat1: Arc<dyn Material>,
    mat2: Arc<dyn Material>,
}

impl MixMaterial {
    pub fn new(t: f64, mat1: Arc<dyn Material>, mat2: Arc<dyn Material>) -> MixMaterial {
        Self {
            t: t.clamp(0.0, 1.0),
            mat1,
            mat2,
        }
    }
}

impl Material for MixMaterial {
    fn scatter(&self, ray: &Ray, hit_info: &HitInfo) -> (Vec3, Option<Ray>) {
        let p: f64 = thread_rng().gen();
        if self.t < p {
            self.mat1.scatter(ray, hit_info)
        } else {
            self.mat2.scatter(ray, hit_info)
        }
    }
}

#[derive(Clone)]
pub enum MaterialType {
    DIFFUSE(Diffuse),
    SPECULAR(Specular),
    REFRACTIVE(Refractive),
    LIGHT(DiffuseLight),
    MIX(MixMaterial),
    BRDFMat(BRDF)
}

impl Default for MaterialType {
    fn default() -> Self {
        Self::DIFFUSE(Diffuse::from_rgb(Vec3::ONE))
    }
}
