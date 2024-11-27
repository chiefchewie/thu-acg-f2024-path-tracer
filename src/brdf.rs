use std::f64::consts::PI;

use glam::FloatExt;
use rand::{thread_rng, Rng};

use crate::vec3::{Quat, Vec2, Vec3};

// implementation of https://boksajak.github.io/files/CrashCourseBRDF.pdf
const MIN_DIELECTRICS_F0: f64 = 0.04;

#[derive(Clone)]
pub struct BRDFMaterialProps {
    base_color: Vec3,
    metalness: f64, // a value in 0.0..=1.0

    emission: Vec3,
    roughness: f64, // a value in 0.0..=1.0

    // TODO figure these out
    transmissiveness: f64,
    opacity: f64,
    // texture: Option<Rc<dyn Texture>>,
}

impl BRDFMaterialProps {
    pub fn basic_diffuse(base_color: Vec3) -> Self {
        Self {
            base_color,
            metalness: 0.0,
            emission: Vec3::ZERO,
            roughness: 1.0, // lambertian diffuse brdf doesnt care about roughness
            transmissiveness: 0.0,
            opacity: 1.0,
        }
    }

    pub fn basic_metal(base_color: Vec3, metalness: f64) -> Self {
        Self {
            base_color,
            metalness,
            emission: Vec3::ZERO,
            roughness: 0.0,
            transmissiveness: 0.0,
            opacity: 1.0,
        }
    }

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

    /// return probabilty of selecting SPECULAR vs DIFFUSE based on Fresnel term
    pub fn get_brdf_probability(&self, _view_dir: Vec3, _normal: Vec3) -> f64 {
        // TODO for now just based it off of how "metallic" the material is
        self.metalness
    }

    pub fn base_color(&self) -> Vec3 {
        self.base_color
    }

    pub fn metalness(&self) -> f64 {
        self.metalness
    }

    pub fn emission(&self) -> Vec3 {
        self.emission
    }

    pub fn roughness(&self) -> f64 {
        self.roughness
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
        let v = view_dir.normalize();
        let n = normal.normalize();
        let l = light_dir.normalize();
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

        let specular_f0 = base_color_to_specular_f0(mat.base_color, mat.metalness);
        let diffuse_reflectance = base_color_to_diffuse_reflectance(mat.base_color, mat.metalness);

        let roughness = mat.roughness;
        let alpha = mat.roughness * mat.roughness;
        let alpha_squared = alpha * alpha;

        let fresnel_term = get_fresnel(specular_f0, l_dot_h);
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
        let d = microfacet_d(self.alpha_squared.max(1e-4), self.n_dot_h);
        let g2 = smith_g2_ggx(self.alpha_squared, self.n_dot_l, self.n_dot_v);
        self.fresnel_term * (g2 * d * self.n_dot_v)
    }

    /// Lambertian BRDF
    pub fn eval_diffuse(&self) -> Vec3 {
        self.diffuse_reflectance * (PI.recip() * self.n_dot_l)
    }

    pub fn diffuse_term(&self) -> f64 {
        1.0
    }
}

fn base_color_to_specular_f0(base_color: Vec3, metalness: f64) -> Vec3 {
    Vec3::new(MIN_DIELECTRICS_F0, MIN_DIELECTRICS_F0, MIN_DIELECTRICS_F0)
        .lerp(base_color, metalness)
}

fn base_color_to_diffuse_reflectance(base_color: Vec3, metalness: f64) -> Vec3 {
    base_color * (1.0 - metalness)
}

fn get_fresnel(f0: Vec3, n_dot_s: f64) -> Vec3 {
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
) -> (Vec3, Option<Vec3>) {
    if normal.dot(view_dir) < 0.0 {
        return (Vec3::ONE, None);
    }

    let rotation_to_z = get_rotation_to_z(normal);
    let v_local = rotation_to_z * view_dir;
    let n_local = Vec3::Z;

    let (sample_weight, ray_dir_local) = match brdf_type {
        BRDFType::DIFFUSE => {
            let ray_dir_local = sample_hemisphere();
            let data = BRDFData::new(n_local, ray_dir_local, v_local, &mat);
            let sample_weight = data.diffuse_reflectance * data.diffuse_term();
            (sample_weight, ray_dir_local)
        }
        BRDFType::SPECULAR => {
            let data = BRDFData::new(n_local, Vec3::Z, v_local, &mat);
            //rayDirectionLocal = sampleSpecular(Vlocal, data.alpha, data.alphaSquared, data.specularF0, u, sampleWeight);
            sample_specular(v_local, data.alpha, data.alpha_squared, data.specular_f0)
        }
    };

    let ray_dir = rotation_to_z.inverse() * ray_dir_local;

    (sample_weight, Some(ray_dir))
}

/// returns the quaternion that rotates a vector so it is aligned to input as the +z axis
fn get_rotation_to_z(input: Vec3) -> Quat {
    if input.z < -0.99999 {
        Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)
    } else {
        Quat::from_xyzw(input.y, -input.x, 0.0, 1.0 + input.z).normalize()
    }
}

/// cosine-weighted distribution oriented along +z axis
fn sample_hemisphere() -> Vec3 {
    let mut rng = thread_rng();
    let r1 = rng.gen_range(0.0..2.0 * PI);
    let r2 = rng.gen::<f64>();
    let r2s = r2.sqrt();
    Vec3::new(r2s * r1.cos(), r2s * r1.sin(), (1.0 - r2).sqrt())
}

/// return the (brdf_weight, scatter_dir) of a speclar surface
fn sample_specular(
    v_local: Vec3,
    alpha: f64,
    alpha_squared: f64,
    specular_f0: Vec3,
) -> (Vec3, Vec3) {
    let eps = 1e-4;
    let n_local = Vec3::Z;

    // H is a microfact normal
    let h_local = if alpha == 0.0 {
        // no roughness -> no microfacets
        n_local
    } else {
        // sample from the half-vector distribution
        sample_specular_half_vector(v_local, Vec2::new(alpha, alpha))
    };

    // reflect the view direction
    let l_local = (-v_local).reflect(h_local);

    let h_dot_l = h_local.dot(l_local).clamp(eps, 1.0);
    let n_dot_l = n_local.dot(l_local).clamp(eps, 1.0);
    let n_dot_v = n_local.dot(v_local).clamp(eps, 1.0);
    let n_dot_h = n_local.dot(h_local).clamp(eps, 1.0);
    let fresnel = get_fresnel(specular_f0, h_dot_l);

    let weight = fresnel * sample_specular_weight(alpha_squared, n_dot_l, n_dot_v);
    (weight, l_local)
}

/// view_dir: the view direction
/// alpha2d: the roughness params for x- and y- axis
/// returns: a sampled half-vector on the microfacet distribution
fn sample_specular_half_vector(view_dir: Vec3, alpha2d: Vec2) -> Vec3 {
    let mut rng = thread_rng();
    // make the orthonormal base v_h, t1, t2
    let v_h = Vec3::new(alpha2d.x * view_dir.x, alpha2d.y * view_dir.y, view_dir.z).normalize();

    let lensq = v_h.x * v_h.x + v_h.y * v_h.y;
    let v1 = if lensq > 0.0 {
        Vec3::new(-v_h.y, v_h.x, 0.0) / lensq
    } else {
        Vec3::X
    };
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
    let g1v = smith_g1(alpha_squared, n_dot_v * n_dot_v);
    let g1l = smith_g1(alpha_squared, n_dot_l * n_dot_l);
    return g1l / (g1v + g1l - g1v * g1l);
}

fn smith_g1(alpha_squared: f64, n_dot_s_sqrd: f64) -> f64 {
    return 2.0
        / ((((alpha_squared * (1.0 - n_dot_s_sqrd)) + n_dot_s_sqrd) / n_dot_s_sqrd).sqrt() + 1.0);
}
