use std::{
    f64::{consts::PI, MIN},
    rc::Rc,
};

use rand::{thread_rng, Rng};

use crate::{
    texture::Texture,
    vec3::{Quat, Vec2, Vec3},
};

// implementation of https://boksajak.github.io/files/CrashCourseBRDF.pdf
const MIN_DIELECTRICS_F0: f64 = 0.04;

#[derive(Clone)]
pub struct BRDFMaterialProps {
    base_color: Vec3,
    metalness: f64,

    emission: Vec3,
    roughness: f64,

    transmissiveness: f64,
    opacity: f64,
    // texture: Option<Rc<dyn Texture>>,
}

impl BRDFMaterialProps {
    pub fn new(
        base_color: Vec3,
        metalness: f64,

        emission: Vec3,
        roughness: f64,

        transmissiveness: f64,
        opacity: f64,
    ) -> Self {
        BRDFMaterialProps {
            base_color,
            metalness,
            emission,
            roughness,
            transmissiveness,
            opacity,
        }
    }
}

pub enum BRDFType {
    DIFFUSE,
    SPECULAR,
}

pub struct BRDFData {
    specular_f0: Vec3,
    diffuse_reflectance: Vec3,

    roughness: f64,
    alpha: f64,
    alpha_squared: f64,

    fresnel_term: Vec3,

    v: Vec3,
    n: Vec3,
    l: Vec3,
    h: Vec3,

    n_dot_l: f64,
    n_dot_v: f64,

    l_dot_h: f64,
    n_dot_h: f64,
    v_dot_h: f64,

    v_backfacing: bool,
    l_backfacing: bool,
}

impl BRDFData {
    pub fn new(normal: Vec3, light_dir: Vec3, view_dir: Vec3, mat: &BRDFMaterialProps) -> Self {
        let v = view_dir;
        let n = normal;
        let l = light_dir;
        let h = (l + v).normalize();
        let n_dot_l = n.dot(l);
        let n_dot_v = n.dot(v);
        let v_backfacing = n_dot_v < 0.0;
        let l_backfacing = n_dot_l < 0.0;

        let n_dot_l = n_dot_l.clamp(1e-4, 1.0);
        let n_dot_v = n_dot_v.clamp(1e-4, 1.0);

        let l_dot_h = l.dot(h).clamp(0.0, 1.0);
        let n_dot_h = n.dot(h).clamp(0.0, 1.0);
        let v_dot_h = v.dot(h).clamp(0.0, 1.0);

        let specular_f0 = Self::base_color_to_specular_f0(mat.base_color, mat.metalness);
        let diffuse_reflectance =
            Self::base_color_to_diffuse_reflectance(mat.base_color, mat.metalness);

        let roughness = mat.roughness;
        let alpha = mat.roughness * mat.roughness;
        let alpha_squared = alpha * alpha;

        let fresnel_term = Self::get_fresnel(specular_f0, l_dot_h);
        Self {
            specular_f0,
            diffuse_reflectance,
            roughness,
            alpha,
            alpha_squared,
            fresnel_term,
            v,
            n,
            l,
            h,
            n_dot_l,
            n_dot_v,
            l_dot_h,
            n_dot_h,
            v_dot_h,
            v_backfacing,
            l_backfacing,
        }
    }

    /// Microfacet specular
    pub fn eval_specular(&self) -> Vec3 {
        let d = Self::microfacet_d(self.alpha_squared.max(1e-4), self.n_dot_h);
        let g2 = Self::smith_g2_ggx(self.alpha_squared, self.n_dot_l, self.n_dot_v);
        self.fresnel_term * (g2 * d * self.n_dot_v)
    }

    /// Lambertian BRDF
    pub fn eval_diffuse(&self) -> Vec3 {
        self.diffuse_reflectance * (PI.recip() * self.n_dot_l)
    }

    pub fn diffuse_term(&self) -> f64 {
        1.0
    }

    fn base_color_to_specular_f0(base_color: Vec3, metalness: f64) -> Vec3 {
        Vec3::new(MIN_DIELECTRICS_F0, MIN_DIELECTRICS_F0, MIN_DIELECTRICS_F0)
            .lerp(base_color, metalness)
    }

    fn base_color_to_diffuse_reflectance(base_color: Vec3, metalness: f64) -> Vec3 {
        base_color * (1.0 - metalness)
    }

    pub fn get_fresnel(f0: Vec3, n_dot_s: f64) -> Vec3 {
        f0 + (1.0 - f0) * (1.0 - n_dot_s).powi(5)
    }

    fn microfacet_d(alpha_squared: f64, n_dot_h: f64) -> f64 {
        let b = (alpha_squared - 1.0) * n_dot_h * n_dot_h + 1.0;
        alpha_squared / (PI * b * b)
    }

    fn smith_g2_ggx(alpha_squared: f64, n_dot_l: f64, n_dot_v: f64) -> f64 {
        let a = n_dot_v * (alpha_squared + n_dot_l * (n_dot_l - alpha_squared * n_dot_l)).sqrt();
        let b = n_dot_l * (alpha_squared + n_dot_v * (n_dot_v - alpha_squared * n_dot_v)).sqrt();
        0.5 / (a + b)
    }
}

pub fn eval_direct_lighting(
    normal: Vec3,
    light_dir: Vec3,
    view_dir: Vec3,
    mat: &BRDFMaterialProps,
) -> Vec3 {
    let data = BRDFData::new(normal, light_dir, view_dir, mat);
    if data.v_backfacing || data.l_backfacing {
        return Vec3::ZERO;
    }

    let specular = data.eval_specular();
    let diffuse = data.eval_diffuse();

    (Vec3::ONE - data.fresnel_term) * diffuse + specular
}

pub fn eval_scatter(
    normal: Vec3,
    view_dir: Vec3,
    mat: &BRDFMaterialProps,
    brdf_type: BRDFType,
) -> Option<(Vec3, Vec3)> {
    if normal.dot(view_dir) < 0.0 {
        return None;
    }

    let rotation_to_z = get_rotation_to_z(normal);
    let v_local = rotation_to_z * view_dir;
    let n_local = Vec3::Z;

    let ray_dir_local: Vec3;
    let sample_weight: Vec3;

    match brdf_type {
        BRDFType::DIFFUSE => {
            ray_dir_local = sample_hemisphere();
            let data = BRDFData::new(n_local, ray_dir_local, v_local, &mat);
            sample_weight = data.diffuse_reflectance * data.diffuse_term();
        }
        BRDFType::SPECULAR => {
            let data = BRDFData::new(n_local, Vec3::Z, v_local, &mat);
            let (dir, sample) =
                sampple_specular(v_local, data.alpha, data.alpha_squared, data.specular_f0);
            ray_dir_local = dir;
            sample_weight = sample;
        }
    }

    let ray_direction = rotation_to_z.inverse() * ray_dir_local;

    Some((ray_direction, sample_weight))
}

fn get_rotation_to_z(input: Vec3) -> Quat {
    if input.z < -0.99999 {
        Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)
    } else {
        Quat::from_xyzw(input.y, -input.x, 0.0, 1.0 + input.z).normalize()
    }
}

fn sample_hemisphere() -> Vec3 {
    let mut rng = thread_rng();
    let r1 = rng.gen_range(0.0..2.0 * PI);
    let r2 = rng.gen::<f64>();
    let r2s = r2.sqrt();
    Vec3::new(r2s * r1.cos(), r2s * r1.sin(), (1.0 - r2).sqrt())
}
