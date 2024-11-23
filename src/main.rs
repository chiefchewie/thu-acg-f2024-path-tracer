use std::rc::Rc;

use path_tracer::{
    camera::Camera,
    material::{Diffuse, MaterialType, Refractive, Specular},
    sphere::Sphere,
    texture::{CheckerTexture, ImageTexture, Texture},
    vec3::Vec3,
    World,
};
use rand::{thread_rng, Rng};

fn balls_scene() {
    let mut world = World::new();

    let checker_tex =
        CheckerTexture::from_colors(0.32, Vec3::new(0.2, 0.3, 0.1), Vec3::new(0.9, 0.9, 0.9));

    let mat_ground = MaterialType::DIFFUSE(Diffuse::new(Rc::new(checker_tex)));

    world.add(Box::new(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        mat_ground,
    )));

    let mat1 = MaterialType::REFRACTIVE(Refractive::new(1.5));
    world.add(Box::new(Sphere::new_still(
        1.0,
        Vec3::new(0.0, 1.0, 0.0),
        mat1,
    )));

    let mat2 = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new_still(
        1.0,
        Vec3::new(-4.0, 1.0, 0.0),
        mat2,
    )));

    let mat3 = MaterialType::SPECULAR(Specular::new(0.7, 0.6, 0.5));
    world.add(Box::new(Sphere::new_still(
        1.0,
        Vec3::new(4.0, 1.0, 0.0),
        mat3,
    )));

    let mut rng = rand::thread_rng();
    for a in (-11..11).map(|x| x as f64) {
        for b in (-11..11).map(|x| x as f64) {
            let choose_mat = rng.gen::<f64>();
            let center = Vec3::new(a + 0.9 * rng.gen::<f64>(), 0.2, b + 0.9 * rng.gen::<f64>());
            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_material = if choose_mat < 0.8 {
                    let albedo = Vec3::random() * Vec3::random();
                    MaterialType::DIFFUSE(Diffuse::from_rgb(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Vec3::rand_range(0.5, 1.0);
                    MaterialType::SPECULAR(Specular::from_rgb(albedo))
                } else {
                    MaterialType::REFRACTIVE(Refractive::new(1.5))
                };
                if let MaterialType::DIFFUSE(_) = sphere_material {
                    let pos2 = center + Vec3::new(0.0, thread_rng().gen_range(0.0..0.5), 0.0);
                    world.add(Box::new(Sphere::new_moving(
                        0.2,
                        center,
                        pos2,
                        sphere_material,
                    )));
                } else {
                    world.add(Box::new(Sphere::new_still(0.2, center, sphere_material)));
                }
            }
        }
    }

    ImageTexture::new("image.png").value(0.0, 0.0, &Vec3::zeroes());

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 10;

    camera.vfov = 20.0;
    camera.look_from = Vec3::new(13.0, 2.0, 3.0);
    camera.look_at = Vec3::new(0.0, 0.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.6;

    camera.init();
    camera.render(&world);
}

fn earth_scene() {
    let earth_texture = ImageTexture::new("earthmap.jpg");
    let earth_surface = MaterialType::DIFFUSE(Diffuse::new(Rc::new(earth_texture)));

    let mut world = World::new();
    world.add(Box::new(Sphere::new_still(
        2.0,
        Vec3::zeroes(),
        earth_surface,
    )));

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 10;
    camera.max_depth = 10;

    camera.vfov = 20.0;
    camera.look_from = Vec3::new(12.0, 12.0, 12.0);
    camera.look_at = Vec3::new(0.0, 0.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.init();
    camera.render(&world);
}

fn main() {
    let x = 1;
    match x {
        1 => balls_scene(),
        2 => earth_scene(),
        _ => (),
    }
}
