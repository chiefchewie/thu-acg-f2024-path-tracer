use std::env;
use std::sync::Arc;

use path_tracer::{
    bsdf::principled::PrincipledBSDF,
    camera::Camera,
    hittable::{Quad, Sphere, World},
    light::PointLight,
    material::{Diffuse, DiffuseLight, MaterialType, MixMaterial, Refractive, Specular},
    texture::{CheckerTexture, ImageTexture},
    vec3::{random_vector, random_vector_range, Vec3},
};
use rand::{thread_rng, Rng};

fn balls_scene() {
    let mut world = World::new();

    let checker_tex =
        CheckerTexture::from_colors(0.32, Vec3::new(0.2, 0.3, 0.1), Vec3::new(0.9, 0.9, 0.9));

    let mat_ground = MaterialType::DIFFUSE(Diffuse::new(Arc::new(checker_tex)));

    world.add(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        mat_ground,
    ));

    let mat1 = MaterialType::REFRACTIVE(Refractive::new(1.5));
    world.add(Sphere::new_still(1.0, Vec3::new(0.0, 1.0, 0.0), mat1));

    let mat2 = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.4, 0.2, 0.1)));
    world.add(Sphere::new_still(1.0, Vec3::new(-4.0, 1.0, 0.0), mat2));

    let mat3 = MaterialType::SPECULAR(Specular::from_rgb(Vec3::new(0.7, 0.6, 0.5), 0.0));
    world.add(Sphere::new_still(1.0, Vec3::new(4.0, 1.0, 0.0), mat3));

    let mut rng = rand::thread_rng();
    for a in (-11..11).map(|x| x as f64) {
        for b in (-11..11).map(|x| x as f64) {
            let choose_mat = rng.gen::<f64>();
            let center = Vec3::new(a + 0.9 * rng.gen::<f64>(), 0.2, b + 0.9 * rng.gen::<f64>());
            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_material = if choose_mat < 0.8 {
                    let albedo = random_vector() * random_vector();
                    MaterialType::DIFFUSE(Diffuse::from_rgb(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = random_vector_range(0.5, 1.0);
                    MaterialType::SPECULAR(Specular::from_rgb(albedo, 0.0))
                } else {
                    MaterialType::REFRACTIVE(Refractive::new(1.5))
                };
                if let MaterialType::DIFFUSE(_) = sphere_material {
                    let pos2 = center + Vec3::new(0.0, thread_rng().gen_range(0.0..0.5), 0.0);
                    world.add(Sphere::new_moving(0.2, center, pos2, sphere_material));
                } else {
                    world.add(Sphere::new_still(0.2, center, sphere_material));
                }
            }
        }
    }

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 720;
    camera.samples_per_pixel = 100;
    camera.max_depth = 10;

    camera.vfov = 20.0;
    camera.look_from = Vec3::new(13.0, 2.0, 3.0);
    camera.look_at = Vec3::ZERO;
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.6;

    camera.ambient_light = Vec3::new(0.7, 0.8, 1.0);

    camera.init();
    camera.render(&world, "demo/balls.png");
}

fn earth_scene() {
    let mut world = World::new();

    let earth_texture = ImageTexture::new("earthmap.jpg");
    let earth_surface = MaterialType::DIFFUSE(Diffuse::new(Arc::new(earth_texture)));
    world.add(Sphere::new_still(
        1.0,
        Vec3::new(4.9, 1.0, 3.0),
        earth_surface,
    ));

    let mat2 = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.4, 0.2, 0.1)));
    world.add(Sphere::new_still(1.0, Vec3::new(0.0, 1.0, 0.0), mat2));

    let mat3 = MaterialType::SPECULAR(Specular::from_rgb(Vec3::new(0.7, 0.6, 0.5), 0.0));
    world.add(Sphere::new_still(1.0, Vec3::new(4.0, 1.0, 0.0), mat3));

    let checker_tex =
        CheckerTexture::from_colors(0.62, Vec3::new(0.9, 0.0, 0.1), Vec3::new(0.9, 0.9, 0.9));
    let mat_ground = MaterialType::DIFFUSE(Diffuse::new(Arc::new(checker_tex)));
    world.add(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        mat_ground,
    ));

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 1024;
    camera.samples_per_pixel = 200;
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
    camera.render(&world, "demo/earth.png");
}

fn quads_scene() {
    let red = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(1.0, 0.2, 0.2)));
    let green = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.2, 1.0, 0.2)));
    let blue = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.2, 0.2, 1.0)));
    let orange = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(1.0, 0.5, 0.0)));
    let teal = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.2, 0.8, 0.8)));

    let mut world = World::new();
    world.add(Quad::new(
        Vec3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        red,
    ));
    world.add(Quad::new(
        Vec3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        green,
    ));
    world.add(Quad::new(
        Vec3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        blue,
    ));
    world.add(Quad::new(
        Vec3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        orange,
    ));
    world.add(Quad::new(
        Vec3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        teal,
    ));

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 1.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 10;
    camera.max_depth = 10;

    camera.vfov = 80.0;
    camera.look_from = Vec3::new(0.0, 0.0, 9.0);
    camera.look_at = Vec3::ZERO;
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.ambient_light = Vec3::new(0.7, 0.8, 1.0);

    camera.init();
    camera.render(&world, "demo/quads.png");
}

fn basic_light_scene() {
    let mut world = World::new();

    let red = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.65, 0.05, 0.05)));
    world.add(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        red.clone(),
    ));

    // plasticy cermaicy material???
    let mat1a = Diffuse::from_rgb(Vec3::new(0.7, 0.9, 0.5));
    let mat1b = Specular::from_rgb(Vec3::ONE, 0.1);
    let mat1 = MaterialType::MIX(MixMaterial::new(0.05, Arc::new(mat1a), Arc::new(mat1b)));
    world.add(Sphere::new_still(2.0, Vec3::new(-4.0, 2.0, 0.0), mat1));

    let mat_diffuse = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::ONE));
    world.add(Sphere::new_still(
        2.0,
        Vec3::new(0.0, 2.0, 0.0),
        mat_diffuse,
    ));

    let mat_metal = MaterialType::SPECULAR(Specular::from_rgb(Vec3::new(0.8, 0.6, 0.2), 0.2));
    world.add(Sphere::new_still(2.0, Vec3::new(4.0, 2.0, 0.0), mat_metal));

    let diffuse_light = MaterialType::LIGHT(DiffuseLight::from_rgb(Vec3::new(10.0, 10.0, 10.0)));
    world.add(Quad::new(
        Vec3::new(-2.0, 6.5, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        diffuse_light,
    ));

    world.add_light(PointLight {
        position: Vec3::new(-2.0, 2.0, 8.0),
        power: Vec3::new(0.1, 0.5, 8.1),
    });

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 1000;
    camera.max_depth = 50;

    camera.vfov = 30.0;
    camera.look_from = Vec3::new(0.0, 3.0, 17.0);
    camera.look_at = Vec3::new(0.0, 2.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.ambient_light = Vec3::new(0.2, 0.2, 0.2);

    camera.init();
    camera.render(&world, "demo/lights.png");
}

fn cornell_box_scene() {
    let mut world = World::new();

    let red = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.65, 0.05, 0.05)));
    let white = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.73, 0.73, 0.73)));
    let green = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.12, 0.45, 0.15)));
    let diffuse_light = MaterialType::LIGHT(DiffuseLight::from_rgb(Vec3::new(25.0, 25.0, 25.0)));
    world.add(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ));
    world.add(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        diffuse_light,
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    ));
    world.add(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    ));
    world.add(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    ));

    let mat = MaterialType::TEST(PrincipledBSDF::new(
        Vec3::new(0.65, 0.05, 0.05), // base_color,
        0.01,                     // metallic,
        0.01,                     // roughness,
        0.01,                     // subsurface,
        0.91,                     // specular,
        0.91,                     // specular_tint,
        1.5,                      // ior,
        0.01,                     // spec_trans,
        0.91,                     // sheen,
        0.91,                     // sheen_tint,
        0.91,                     // clearcoat,
        0.01,                     // clearcoat_gloss,
    ));
    world.add(Sphere::new_still(
        105.0,
        Vec3::new(413.0, 170.0, 372.0),
        mat,
    ));
    world.add(Sphere::new_still(
        135.0,
        Vec3::new(113.0, 170.0, 372.0),
        MaterialType::SPECULAR(Specular::from_rgb(Vec3::ONE, 0.0)),
    ));

    // world.add(Quad::new(
    //     Vec3::new(343.0, 354.0, 332.0),
    //     Vec3::new(-130.0, 0.0, 0.0),
    //     Vec3::new(0.0, 0.0, -105.0),
    //     MaterialType::TEST(GlassBSDF::new(0.01, 1.5)),
    // ));

    // let box1 = Arc::new(Cuboid::new(
    //     Vec3::ZERO,
    //     Vec3::new(165.0, 330.0, 165.0),
    //     specular_brdf,
    // ));
    // let box1 = Instance::new(box1, Vec3::Y, 0.261799, Vec3::new(265.0, 0.0, 295.0));
    // world.add(box1);

    // let box2 = Arc::new(Cuboid::new(
    //     Vec3::ZERO,
    //     Vec3::new(165.0, 165.0, 165.0),
    //     white.clone(),
    // ));
    // let box2 = Instance::new(box2, Vec3::Y, -0.29, Vec3::new(130.0, 0.0, 65.0));
    // world.add(box2);

    world.build_bvh();
    let mut camera = Camera::new();
    camera.aspect_ratio = 1.0;
    camera.image_width = 900;
    camera.samples_per_pixel = 100;
    camera.max_depth = 20;

    camera.vfov = 40.0;
    camera.look_from = Vec3::new(278.0, 278.0, -800.0);
    camera.look_at = Vec3::new(278.0, 278.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.ambient_light = Vec3::ZERO;

    camera.init();
    camera.render(&world, "demo/cornell.png");
}

fn test_scene() {
    let mut world = World::new();
    let material_ground = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.8, 0.8, 0.0)));
    let material_center = MaterialType::DIFFUSE(Diffuse::from_rgb(Vec3::new(0.1, 0.2, 0.5)));
    // let material_left = MaterialType::TEST(MetalBRDF::new(Vec3::new(0.8, 0.1, 0.2), 0.3));
    // let material_left = MaterialType::TEST(GlassBSDF::new(0.1, 1.5));
    // let material_left = MaterialType::TEST(ClearcoatBRDF::new(0.5));
    let material_left = MaterialType::TEST(PrincipledBSDF::new(
        Vec3::new(0.8, 0.2, 0.2), // base_color,
        0.00,                     // metallic,
        0.01,                     // roughness,
        0.01,                     // subsurface,
        0.00,                     // specular,
        0.01,                     // specular_tint,
        1.5,                      // ior,
        0.09,                     // spec_trans,
        0.01,                     // sheen,
        0.01,                     // sheen_tint,
        0.01,                     // clearcoat,
        0.01,                     // clearcoat_gloss,
    ));
    // let material_left = MaterialType::REFRACTIVE(Refractive::new(1.5));
    let material_right = MaterialType::SPECULAR(Specular::from_rgb(Vec3::new(0.8, 0.1, 0.2), 0.3));

    world.add(Sphere::new_still(
        100.0,
        Vec3::new(0.0, -100.5, -1.0),
        material_ground,
    ));
    world.add(Sphere::new_still(
        0.5,
        Vec3::new(0.0, 0.0, -1.2),
        material_center,
    ));
    world.add(Sphere::new_still(
        0.5,
        Vec3::new(-1.0, 0.0, -1.0),
        material_left,
    ));
    world.add(Sphere::new_still(
        0.5,
        Vec3::new(1.0, 0.0, -1.0),
        material_right,
    ));

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 900;
    camera.samples_per_pixel = 100;
    camera.max_depth = 20;

    camera.vfov = 90.0;
    camera.look_from = Vec3::ZERO;
    camera.look_at = -Vec3::Z;
    camera.vup = Vec3::Y;

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.ambient_light = Vec3::new(0.7, 0.8, 1.0);

    camera.init();
    camera.render(&world, "demo/test.png");
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let x = 5;
    match x {
        1 => balls_scene(),
        2 => earth_scene(),
        3 => quads_scene(),
        4 => basic_light_scene(),
        5 => cornell_box_scene(),
        6 => test_scene(),
        _ => (),
    }
}
