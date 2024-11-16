use std::f64::consts::PI;

use rand::Rng;

pub fn normal_dist() -> f64 {
    let mut rng = rand::thread_rng();
    let theta = 2.0 * PI * rng.gen::<f64>();
    let rho = (-2.0 * (1.0 - rng.gen::<f64>()).ln()).sqrt();
    rho * theta.cos()
}
