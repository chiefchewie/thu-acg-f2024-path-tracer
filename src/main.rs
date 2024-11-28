use std::{rc::Rc, sync::Arc};

use path_tracer::{
    brdf::{BRDFData, BRDFMaterialProps},
    camera::Camera,
    light::PointLight,
    material::{Diffuse, DiffuseLight, MaterialType, Refractive, Specular},
    quad::Quad,
    sphere::Sphere,
    texture::{CheckerTexture, ImageTexture},
    vec3::{random_vector, random_vector_range, Vec3},
    World,
};
use rand::{thread_rng, Rng};

// fn balls_scene() {
//     let mut world = World::new();

//     let checker_tex =
//         CheckerTexture::from_colors(0.32, Vec3::new(0.2, 0.3, 0.1), Vec3::new(0.9, 0.9, 0.9));

//     let mat_ground = MaterialType::DIFFUSE(Diffuse::new(Rc::new(checker_tex)));

//     world.add(Sphere::new_still(
//         1000.0,
//         Vec3::new(0.0, -1000.0, 0.0),
//         mat_ground,
//     ));

//     let mat1 = MaterialType::REFRACTIVE(Refractive::new(1.5));
//     world.add(Sphere::new_still(1.0, Vec3::new(0.0, 1.0, 0.0), mat1));

//     let mat2 = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.4, 0.2, 0.1)));
//     world.add(Sphere::new_still(1.0, Vec3::new(-4.0, 1.0, 0.0), mat2));

//     let mat3 = MaterialType::SPECULAR(Specular::new(0.7, 0.6, 0.5));
//     world.add(Sphere::new_still(1.0, Vec3::new(4.0, 1.0, 0.0), mat3));

//     let mut rng = rand::thread_rng();
//     for a in (-11..11).map(|x| x as f64) {
//         for b in (-11..11).map(|x| x as f64) {
//             let choose_mat = rng.gen::<f64>();
//             let center = Vec3::new(a + 0.9 * rng.gen::<f64>(), 0.2, b + 0.9 * rng.gen::<f64>());
//             if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
//                 let sphere_material = if choose_mat < 0.8 {
//                     let albedo = random_vector() * random_vector();
//                     MaterialType::DIFFUSE(Diffuse::from_rgb(albedo))
//                 } else if choose_mat < 0.95 {
//                     let albedo = random_vector_range(0.5, 1.0);
//                     MaterialType::SPECULAR(Specular::from_rgb(albedo))
//                 } else {
//                     MaterialType::REFRACTIVE(Refractive::new(1.5))
//                 };
//                 if let MaterialType::DIFFUSE(_) = sphere_material {
//                     let pos2 = center + Vec3::new(0.0, thread_rng().gen_range(0.0..0.5), 0.0);
//                     world.add(Sphere::new_moving(0.2, center, pos2, sphere_material));
//                 } else {
//                     world.add(Sphere::new_still(0.2, center, sphere_material));
//                 }
//             }
//         }
//     }

//     world.build_bvh();

//     let mut camera = Camera::new();
//     camera.aspect_ratio = 16.0 / 9.0;
//     camera.image_width = 720;
//     camera.samples_per_pixel = 100;
//     camera.max_depth = 10;

//     camera.vfov = 20.0;
//     camera.look_from = Vec3::new(13.0, 2.0, 3.0);
//     camera.look_at = Vec3::ZERO;
//     camera.vup = Vec3::new(0.0, 1.0, 0.0);

//     camera.blur_strength = 0.5;
//     camera.focal_length = 10.0;
//     camera.defocus_angle = 0.6;

//     camera.ambient_light = Vec3::new(0.7, 0.8, 1.0);

//     camera.init();
//     camera.render(&world);
// }

// fn earth_scene() {
//     let mut world = World::new();

//     let earth_texture = ImageTexture::new("earthmap.jpg");
//     let earth_surface = MaterialType::DIFFUSE(Diffuse::new(Rc::new(earth_texture)));
//     world.add(Sphere::new_still(
//         1.0,
//         Vec3::new(4.9, 1.0, 3.0),
//         earth_surface,
//     ));

//     let mat2 = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.4, 0.2, 0.1)));
//     world.add(Sphere::new_still(1.0, Vec3::new(0.0, 1.0, 0.0), mat2));

//     let mat3 = MaterialType::SPECULAR(Specular::new(0.7, 0.6, 0.5));
//     world.add(Sphere::new_still(1.0, Vec3::new(4.0, 1.0, 0.0), mat3));

//     let checker_tex =
//         CheckerTexture::from_colors(0.62, Vec3::new(0.9, 0.0, 0.1), Vec3::new(0.9, 0.9, 0.9));
//     let mat_ground = MaterialType::DIFFUSE(Diffuse::new(Rc::new(checker_tex)));
//     world.add(Sphere::new_still(
//         1000.0,
//         Vec3::new(0.0, -1000.0, 0.0),
//         mat_ground,
//     ));

//     world.build_bvh();

//     let mut camera = Camera::new();
//     camera.aspect_ratio = 16.0 / 9.0;
//     camera.image_width = 1024;
//     camera.samples_per_pixel = 1000;
//     camera.max_depth = 10;

//     camera.vfov = 28.0;
//     camera.look_from = Vec3::new(8.8, 2.0, 3.0);
//     camera.look_at = Vec3::ZERO;
//     camera.vup = Vec3::new(0.0, 1.0, 0.0);

//     camera.blur_strength = 0.5;
//     camera.focal_length = 2.869817807;
//     camera.defocus_angle = 2.5;

//     camera.ambient_light = Vec3::new(0.7, 0.8, 1.0);

//     camera.init();
//     camera.render(&world);
// }

// fn quads_scene() {
//     let red = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(1.0, 0.2, 0.2)));
//     let green = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.2, 1.0, 0.2)));
//     let blue = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.2, 0.2, 1.0)));
//     let orange = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(1.0, 0.5, 0.0)));
//     let teal = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.2, 0.8, 0.8)));

//     let mut world = World::new();
//     world.add(Quad::new(
//         Vec3::new(-3.0, -2.0, 5.0),
//         Vec3::new(0.0, 0.0, -4.0),
//         Vec3::new(0.0, 4.0, 0.0),
//         red,
//     ));
//     world.add(Quad::new(
//         Vec3::new(-2.0, -2.0, 0.0),
//         Vec3::new(4.0, 0.0, 0.0),
//         Vec3::new(0.0, 4.0, 0.0),
//         green,
//     ));
//     world.add(Quad::new(
//         Vec3::new(3.0, -2.0, 1.0),
//         Vec3::new(0.0, 0.0, 4.0),
//         Vec3::new(0.0, 4.0, 0.0),
//         blue,
//     ));
//     world.add(Quad::new(
//         Vec3::new(-2.0, 3.0, 1.0),
//         Vec3::new(4.0, 0.0, 0.0),
//         Vec3::new(0.0, 0.0, 4.0),
//         orange,
//     ));
//     world.add(Quad::new(
//         Vec3::new(-2.0, -3.0, 5.0),
//         Vec3::new(4.0, 0.0, 0.0),
//         Vec3::new(0.0, 0.0, -4.0),
//         teal,
//     ));

//     world.build_bvh();

//     let mut camera = Camera::new();
//     camera.aspect_ratio = 1.0;
//     camera.image_width = 400;
//     camera.samples_per_pixel = 10;
//     camera.max_depth = 10;

//     camera.vfov = 80.0;
//     camera.look_from = Vec3::new(0.0, 0.0, 9.0);
//     camera.look_at = Vec3::ZERO;
//     camera.vup = Vec3::new(0.0, 1.0, 0.0);

//     camera.blur_strength = 0.5;
//     camera.focal_length = 10.0;
//     camera.defocus_angle = 0.0;

//     camera.ambient_light = Vec3::new(0.7, 0.8, 1.0);

//     camera.init();
//     camera.render(&world);
// }

fn basic_light_scene() {
    let mut world = World::new();

    let red = MaterialType::BRDF(BRDFMaterialProps::basic_diffuse(Vec3::new(
        0.65, 0.05, 0.05,
    )));
    world.add(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        red.clone(),
    ));
    world.add(Sphere::new_still(
        2.0,
        Vec3::new(0.0, 2.0, 0.0),
        red.clone(),
    ));

    let diffuse_light = MaterialType::BRDF(BRDFMaterialProps::light(Vec3::new(14.0, 14.0, 14.0)));
    world.add(Quad::new(
        Vec3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        diffuse_light,
    ));

    world.add_light(PointLight {
        position: Vec3::new(4.0, 5.0, 0.0),
        power: Vec3::new(1.0, 9.0, 1.0),
    });

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;

    camera.vfov = 20.0;
    camera.look_from = Vec3::new(26.0, 3.0, 6.0);
    camera.look_at = Vec3::new(0.0, 2.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.ambient_light = Vec3::new(0.0, 0.0, 0.0);

    camera.init();
    camera.render(&world, "lights.png");
}

// fn cornell_box_scene() {
//     let mut world = World::new();

//     let mat1 = MaterialType::REFRACTIVE(Refractive::new(1.5));
//     world.add(Sphere::new_still(
//         105.0,
//         Vec3::new(413.0, 170.0, 372.0),
//         mat1,
//     ));

//     let mat2 = MaterialType::SPECULAR(Specular::new(0.7, 0.6, 0.5));
//     world.add(Sphere::new_still(
//         135.0,
//         Vec3::new(113.0, 170.0, 372.0),
//         mat2,
//     ));

//     let red = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.65, 0.05, 0.05)));
//     let white = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.73, 0.73, 0.73)));
//     let green = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.12, 0.45, 0.15)));
//     let diffuse_light = MaterialType::LIGHT(DiffuseLight::from_rgb(Vec3::new(25.0, 25.0, 25.0)));
//     world.add(Quad::new(
//         Vec3::new(555.0, 0.0, 0.0),
//         Vec3::new(0.0, 555.0, 0.0),
//         Vec3::new(0.0, 0.0, 555.0),
//         green,
//     ));
//     world.add(Quad::new(
//         Vec3::new(0.0, 0.0, 0.0),
//         Vec3::new(0.0, 555.0, 0.0),
//         Vec3::new(0.0, 0.0, 555.0),
//         red,
//     ));
//     world.add(Quad::new(
//         Vec3::new(343.0, 554.0, 332.0),
//         Vec3::new(-130.0, 0.0, 0.0),
//         Vec3::new(0.0, 0.0, -105.0),
//         diffuse_light,
//     ));
//     world.add(Quad::new(
//         Vec3::new(0.0, 0.0, 0.0),
//         Vec3::new(555.0, 0.0, 0.0),
//         Vec3::new(0.0, 0.0, 555.0),
//         white.clone(),
//     ));
//     world.add(Quad::new(
//         Vec3::new(555.0, 555.0, 555.0),
//         Vec3::new(-555.0, 0.0, 0.0),
//         Vec3::new(0.0, 0.0, -555.0),
//         white.clone(),
//     ));
//     world.add(Quad::new(
//         Vec3::new(0.0, 0.0, 555.0),
//         Vec3::new(555.0, 0.0, 0.0),
//         Vec3::new(0.0, 555.0, 0.0),
//         white.clone(),
//     ));

//     world.build_bvh();

//     let mut camera = Camera::new();
//     camera.aspect_ratio = 1.0;
//     camera.image_width = 900;
//     camera.samples_per_pixel = 1000;
//     camera.max_depth = 20;

//     camera.vfov = 40.0;
//     camera.look_from = Vec3::new(278.0, 278.0, -800.0);
//     camera.look_at = Vec3::new(278.0, 278.0, 0.0);
//     camera.vup = Vec3::new(0.0, 1.0, 0.0);

//     camera.blur_strength = 0.5;
//     camera.focal_length = 10.0;
//     camera.defocus_angle = 0.0;

//     camera.ambient_light = Vec3::ZERO;

//     camera.init();
//     camera.render(&world);
// }

// fn main() {
//     let x = 4;
//     match x {
//         1 => balls_scene(),
//         2 => earth_scene(),
//         3 => quads_scene(),
//         4 => basic_light_scene(),
//         5 => cornell_box_scene(),
//         _ => (),
//     }
// }

fn earth_scene() {
    let mut world = World::new();

    let earth_surface = MaterialType::BRDF(BRDFMaterialProps::texture_diffuse(Arc::new(
        ImageTexture::new("earthmap.jpg"),
    )));
    world.add(Sphere::new_still(
        1.0,
        Vec3::new(4.9, 1.0, 3.0),
        earth_surface,
    ));

    let mat2 = MaterialType::BRDF(BRDFMaterialProps::basic_diffuse(Vec3::new(0.0, 0.9, 0.1)));
    world.add(Sphere::new_still(1.0, Vec3::new(0.0, 1.0, 0.0), mat2));

    let mat3 = MaterialType::BRDF(BRDFMaterialProps::basic_glossy(
        Vec3::new(0.7, 0.6, 0.5),
        0.9,
    ));
    world.add(Sphere::new_still(1.0, Vec3::new(4.0, 1.0, 0.0), mat3));

    let checker_tex =
        CheckerTexture::from_colors(0.62, Vec3::new(0.9, 0.0, 0.1), Vec3::new(0.9, 0.9, 0.9));
    let mat_ground = MaterialType::BRDF(BRDFMaterialProps::texture_diffuse(Arc::new(checker_tex)));
    world.add(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        mat_ground,
    ));

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 1024;
    camera.samples_per_pixel = 100;
    camera.max_depth = 10;

    camera.vfov = 28.0;
    camera.look_from = Vec3::new(8.8, 2.0, 3.0);
    camera.look_at = Vec3::ZERO;
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 2.869817807;
    camera.defocus_angle = 2.5;

    camera.ambient_light = Vec3::new(0.7, 0.8, 1.0);

    camera.init();
    camera.render(&world, "earth.png");
}

fn main() {
    let x = 1;
    match x {
        1 => earth_scene(),
        2 => basic_light_scene(),
        _ => (),
    }
}
