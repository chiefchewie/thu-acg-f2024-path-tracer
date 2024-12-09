use std::f64::consts::PI;

use rand::{thread_rng, Rng};

use crate::vec3::{get_rotation_to_z, Vec3};

// transformations
pub fn to_local(normal: Vec3, input_world: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal);
    rot * input_world
}

pub fn to_world(normal: Vec3, input_local: Vec3) -> Vec3 {
    let rot = get_rotation_to_z(normal).inverse();
    rot * input_local
}

pub fn cosine_sample_hemisphere() -> Vec3 {
    let mut rng = thread_rng();
    let phi = rng.gen_range(0.0..=2.0 * PI);
    let r2 = rng.gen::<f64>();
    let r2s = r2.sqrt();
    Vec3::new(r2s * phi.cos(), r2s * phi.sin(), (1.0 - r2).sqrt())
}

/// An implementation of ideas in:
/// https://www.graphics.cornell.edu/%7Ebjw/microfacetbsdf.pdf
/// https://inria.hal.science/hal-00996995v1/document
/// https://hal.science/hal-01509746/document
#[allow(non_snake_case)]
pub mod ggx {
    use std::f64::consts::PI;

    use rand::{thread_rng, Rng};

    use crate::vec3::Vec3;

    pub fn D(h: Vec3, roughness: f64) -> f64 {
        let cos_theta = h.z.max(0.001);
        let alpha2 = (roughness * roughness).max(0.001);
        let denom = (alpha2 - 1.0) * (cos_theta * cos_theta) + 1.0;
        alpha2 / (PI * denom * denom)
    }

    pub fn G(v: Vec3, l: Vec3, roughness: f64) -> f64 {
        let g1v = G1(v, roughness);
        let g1l = G1(l, roughness);
        g1v * g1l
    }

    pub fn G1(w: Vec3, roughness: f64) -> f64 {
        let alpha2 = (roughness * roughness).max(0.001);
        let cos_theta = w.z.abs();
        2.0 * cos_theta / (cos_theta + (cos_theta * cos_theta * (1.0 - alpha2) + alpha2).sqrt())
    }

    pub fn sample_microfacet_normal(v: Vec3, roughness: f64) -> Vec3 {
        let h = sample_ggx_vndf(v, roughness * roughness);
        if h.z < 0.0 {
            -h
        } else {
            h
        }
    }

    fn sample_ggx_vndf(v: Vec3, a2: f64) -> Vec3 {
        // stretch view
        let v = Vec3::new(v.x * a2, v.y * a2, v.z).normalize();

        // orthonormal basis
        let t1 = if v.z < 0.9999 {
            v.cross(Vec3::Z).normalize()
        } else {
            Vec3::X
        };
        let t2 = t1.cross(v);

        // sample
        let e1 = thread_rng().gen::<f64>();
        let e2 = thread_rng().gen::<f64>();
        let a = 1.0 / (1.0 + v.z);
        let r = e1.sqrt();
        let phi = if e2 < a {
            e2 / a * PI
        } else {
            PI + (e2 - a) / (1.0 - a) * PI
        };
        let p1 = r * phi.cos();
        let p2 = r * phi.sin() * if e2 < a { 1.0 } else { v.z };

        let n = p1 * t1 + p2 * t2 + (1.0 - p1 * p1 - p2 * p2).max(0.0).sqrt() * v;
        let unstretched = Vec3::new(a2 * n.x, a2 * n.y, n.z.max(0.0));
        unstretched.normalize()
    }

    #[allow(dead_code)]
    // keeping the ndf for reference
    fn sample_ggx(_v: Vec3, a2: f64) -> Vec3 {
        let mut rng = thread_rng();
        let e1: f64 = rng.gen();
        let e2: f64 = rng.gen();

        let theta = ((a2 * e1.sqrt()) / (1.0 - e1).sqrt()).atan();
        let phi = e2 * 2.0 * PI;
        Vec3::new(
            phi.cos() * theta.sin(),
            phi.sin() * theta.sin(),
            theta.cos(),
        )
    }
}

#[allow(non_snake_case)]
pub mod gtr1 {
    use std::f64::consts::PI;

    use rand::{thread_rng, Rng};

    use crate::vec3::Vec3;

    pub fn D(abs_cos_theta: f64, alpha_g: f64) -> f64 {
        let alpha2 = alpha_g * alpha_g;
        let t = 1.0 + (alpha2 - 1.0) * abs_cos_theta * abs_cos_theta;
        (alpha2 - 1.0) / (PI * t * alpha2.log2())
    }

    pub fn sample_microfacet_normal(alpha: f64) -> Vec3 {
        let e1 = thread_rng().gen::<f64>();
        let e2 = thread_rng().gen::<f64>();

        let alpha2 = alpha * alpha;
        let cos_theta = (1.0 - alpha2.powf(1.0 - e1)) / (1.0 - alpha2);
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let phi = 2.0 * PI * e2;

        let h = Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        if h.z < 0.0 {
            -h
        } else {
            h
        }
    }
}
