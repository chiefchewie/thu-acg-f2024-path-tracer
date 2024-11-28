use rand::Rng;

pub type Vec3 = glam::DVec3;
pub type Vec2 = glam::DVec2;
pub type Quat = glam::f64::DQuat;

pub fn random_vector_range(min: f64, max: f64) -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(
        rng.gen_range(min..max),
        rng.gen_range(min..max),
        rng.gen_range(min..max),
    )
}

pub fn random_vector() -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(rng.gen(), rng.gen(), rng.gen())
}

/// returns the quaternion that rotates a vector so it is aligned to input as the +z axis
pub fn get_rotation_to_z(input: Vec3) -> Quat {
    if input.z < -0.99999 {
        Quat::from_xyzw(1.0, 0.0, 0.0, 0.0)
    } else {
        Quat::from_xyzw(input.y, -input.x, 0.0, 1.0 + input.z).normalize()
    }
}

pub fn step(edge: Vec3, x: Vec3) -> Vec3 {
    let f = |e: f64, v: f64| if v >= e { 1.0 } else { 0.0 };
    Vec3::new(f(edge.x, x.x), f(edge.y, x.y), f(edge.z, x.z))
}

const L: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);
pub trait Luminance {
    fn luminance(&self) -> f64;
}
impl Luminance for Vec3 {
    fn luminance(&self) -> f64 {
        self.dot(L)
    }
}
