use path_tracer::{
    camera::Camera,
    sphere::Sphere,
    vec3::Vec3,
    World,
};

fn write_to_ppm(colors: &Vec<Vec3>, width: usize, height: usize) {
    println!("P3\n{} {}\n255\n", width, height);
    for r in 0..height {
        for c in 0..width {
            let color = colors[r * width + c];
            let r = (color.x().clamp(0.0, 0.999) * 256.0) as usize;
            let g = (color.y().clamp(0.0, 0.999) * 256.0) as usize;
            let b = (color.z().clamp(0.0, 0.999) * 256.0) as usize;
            println!("{} {} {}", r, g, b);
        }
    }
}

fn main() {
    let mut world = World::new();
    world.add(Box::new(Sphere::new(0.5, Vec3::new(0.0, 0.0, -1.0))));
    world.add(Box::new(Sphere::new(100.0, Vec3::new(0.0, -100.5, -1.0))));

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 10;
    camera.init();

    let mut pixels = vec![Vec3::new(0.0, 0.0, 0.0); camera.width() * camera.height()];
    camera.render(&world, &mut pixels);

    write_to_ppm(&pixels, camera.width(), camera.height());
}
