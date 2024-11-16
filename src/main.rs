use std::time::Instant;

use path_tracer::{
    camera::Camera,
    material::{DiffuseMaterial, Material, RefractiveMaterial, SpecularMaterial},
    sphere::Sphere,
    vec3::Vec3,
    World,
};

fn gamma_correct(x: f64) -> f64 {
    if x > 0.0 {
        x.sqrt()
    } else {
        0.0
    }
}

fn write_to_ppm(colors: &[Vec3], width: usize, height: usize) {
    println!("P3\n{} {}\n255\n", width, height);
    for r in 0..height {
        for c in 0..width {
            let color = colors[r * width + c];
            let r = gamma_correct(color.x()).clamp(0.0, 0.999);
            let g = gamma_correct(color.y()).clamp(0.0, 0.999);
            let b = gamma_correct(color.z()).clamp(0.0, 0.999);
            println!(
                "{} {} {}",
                (r * 256.0) as usize,
                (g * 256.0) as usize,
                (b * 256.0) as usize,
            );
        }
    }
}

fn main() {
    let mut world = World::new();
    let mat_ground = Material::DIFFUSE(DiffuseMaterial::new(0.8, 0.8, 0.0));
    let mat_center = Material::DIFFUSE(DiffuseMaterial::new(0.1, 0.2, 0.5));
    let mat_left = Material::REFRACTIVE(RefractiveMaterial::new(1.5));
    let mat_bubble = Material::REFRACTIVE(RefractiveMaterial::new(1.0 / 1.5));
    let mat_right = Material::SPECULAR(SpecularMaterial::new(0.8, 0.6, 0.2));

    world.add(Box::new(Sphere::new(
        100.0,
        Vec3::new(0.0, -100.5, -1.0),
        mat_ground,
    )));

    world.add(Box::new(Sphere::new(
        0.5,
        Vec3::new(0.0, 0.0, -1.2),
        mat_center,
    )));

    world.add(Box::new(Sphere::new(
        0.5,
        Vec3::new(-1.0, 0.0, -1.0),
        mat_left,
    )));

    world.add(Box::new(Sphere::new(
        0.4,
        Vec3::new(-1.0, 0.0, -1.0),
        mat_bubble,
    )));

    world.add(Box::new(Sphere::new(
        0.5,
        Vec3::new(1.0, 0.0, -1.0),
        mat_right,
    )));

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 10;

    camera.vfov = 20.0;
    camera.look_from = Vec3::zeroes();
    camera.look_from = Vec3::new(-2., 2., 1.);
    camera.look_at = Vec3::new(0., 0., -1.);
    camera.vup = Vec3::new(0., 1., 0.);

    camera.blur_strength = 0.5;

    camera.init();

    let mut pixels = vec![Vec3::new(0.0, 0.0, 0.0); camera.width() * camera.height()];

    let mut start = Instant::now();
    dbg!("start render", start);
    camera.render(&world, &mut pixels);
    dbg!("finished render, took {:?} seconds", start.elapsed());

    start = Instant::now();
    dbg!("start write", start);
    write_to_ppm(&pixels, camera.width(), camera.height());
    dbg!("finished write, took {:?} seconds", start.elapsed());
}
