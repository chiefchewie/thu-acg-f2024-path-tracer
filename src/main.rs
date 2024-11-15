use core::f64;

use path_tracer::{
    hit_info::HitInfo, light::PointLight, material::Material, ray::Ray, sphere::Sphere, vec3::Vec3,
    Hittable, World,
};

fn trace(ray: &Ray, world: &World) -> Vec3 {
    let t = world.intersects(ray, 0.0, f64::INFINITY);
    if t.did_hit {
        (t.normal + Vec3::new(1.0, 1.0, 1.0)) * 0.5
    } else {
        let a = 0.5 * (ray.direction().y() + 1.0);
        Vec3::new(1.0, 1.0, 1.0) * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
    }
}

fn write_to_ppm(colors: &Vec<Vec3>, width: usize, height: usize) {
    println!("P3\n{} {}\n255\n", width, height);
    for r in 0..height {
        for c in 0..width {
            let color = colors[r * width + c];
            let r = (color.x() * 255.999) as usize;
            let g = (color.y() * 255.999) as usize;
            let b = (color.z() * 255.999) as usize;
            println!("{} {} {}", r, g, b);
        }
    }
}

fn main() {
    let mut world = World::new();
    world.add(Box::new(Sphere::new(0.5, Vec3::new(0.0, 0.0, -1.0))));
    world.add(Box::new(Sphere::new(100.0, Vec3::new(0.0, -100.5, -1.0))));

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as usize;

    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let cam_center = Vec3::new(0.0, 0.0, 0.0);

    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

    let pixel_du = viewport_u / image_width as f64;
    let pixel_dv = viewport_v / image_height as f64;

    let upperleft =
        cam_center - Vec3::new(0.0, 0.0, focal_length) - (viewport_u / 2.0) - (viewport_v / 2.0);
    let pixel00 = upperleft + (pixel_du + pixel_dv) * 0.5;

    let mut pixels = vec![Vec3::new(0.0, 0.0, 0.0); image_width * image_height];
    for r in 0..image_height {
        for c in 0..image_width {
            let pixel_position = pixel00 + (pixel_du * c as f64) + (pixel_dv * r as f64);
            let ray_dir = pixel_position - cam_center;
            let ray = Ray::new(cam_center, ray_dir);
            pixels[r * image_width + c] = trace(&ray, &world);
        }
    }

    write_to_ppm(&pixels, image_width, image_height);
}
