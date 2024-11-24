use rand::Rng;

pub type Vec3 = glam::DVec3;
pub type Vec2 = glam::DVec2;

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

pub fn step(edge: Vec3, x: Vec3) -> Vec3 {
    let f = |e: f64, v: f64| if v >= e { 1.0 } else { 0.0 };
    Vec3::new(f(edge.x, x.x), f(edge.y, x.y), f(edge.z, x.z))
}
