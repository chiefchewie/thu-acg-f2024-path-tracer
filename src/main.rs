use clap::Parser;
use std::{env, sync::Arc};

use path_tracer::{
    bsdf::{diffuse::DiffuseBRDF, glass::GlassBSDF, metal::MetalBRDF, principled::PrincipledBSDF},
    camera::{Camera, EnvironmentType},
    hittable::{Cuboid, Instance, Quad, Sphere, TriangleMesh, World},
    material::DiffuseLight,
    texture::{CheckerTexture, ImageTexture, SolidTexture},
    vec3::{random_vector, random_vector_range, Vec3},
};
use rand::{thread_rng, Rng};

fn balls_scene(width: usize, spp: usize) {
    let mut world = World::new();

    let tex1 = SolidTexture::new(Vec3::new(0.2, 0.3, 0.1));
    let tex2 = SolidTexture::new(Vec3::new(0.9, 0.9, 0.9));
    let checker_tex = CheckerTexture::new(0.32, Arc::new(tex1), Arc::new(tex2));

    let mat_ground = Arc::new(DiffuseBRDF::new(Arc::new(checker_tex)));

    world.add_object(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        mat_ground,
    ));

    let mat1 = Arc::new(GlassBSDF::basic(1.5));
    world.add_object(Sphere::new_still(1.0, Vec3::new(0.0, 1.0, 0.0), mat1));

    let mat2 = Arc::new(DiffuseBRDF::from_rgb(Vec3::new(0.4, 0.2, 0.1)));
    world.add_object(Sphere::new_still(1.0, Vec3::new(-4.0, 1.0, 0.0), mat2));

    let mat3 = Arc::new(MetalBRDF::from_rgb(Vec3::new(0.7, 0.6, 0.5), 0.0));
    world.add_object(Sphere::new_still(1.0, Vec3::new(4.0, 1.0, 0.0), mat3));

    let mut rng = rand::thread_rng();
    for a in (-11..11).map(|x| x as f64) {
        for b in (-11..11).map(|x| x as f64) {
            let choose_mat = rng.gen::<f64>();
            let center = Vec3::new(a + 0.9 * rng.gen::<f64>(), 0.2, b + 0.9 * rng.gen::<f64>());
            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = random_vector() * random_vector();
                    let sphere_mat = Arc::new(DiffuseBRDF::from_rgb(albedo));
                    let pos2 = center + Vec3::new(0.0, thread_rng().gen_range(0.0..0.5), 0.0);
                    world.add_object(Sphere::new_moving(0.2, center, pos2, sphere_mat));
                } else if choose_mat < 0.95 {
                    let albedo = random_vector_range(0.5, 1.0);
                    let sphere_mat = Arc::new(MetalBRDF::from_rgb(albedo, 0.0));
                    world.add_object(Sphere::new_still(0.2, center, sphere_mat));
                } else {
                    let sphere_mat = Arc::new(GlassBSDF::basic(1.5));
                    world.add_object(Sphere::new_still(0.2, center, sphere_mat));
                };
            }
        }
    }

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = width;
    camera.samples_per_pixel = spp;
    camera.max_depth = 50;

    camera.vfov = 20.0;
    camera.look_from = Vec3::new(13.0, 2.0, 3.0);
    camera.look_at = Vec3::ZERO;
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.6;

    camera.environment = EnvironmentType::Color(Vec3::new(0.7, 0.8, 1.0));

    camera.init();
    camera.render(&world, "demo/balls.png");
}

fn earth_scene(width: usize, spp: usize) {
    let mut world = World::new();

    let earth_texture = ImageTexture::new("assets/earthmap.jpg");
    let earth_surface = Arc::new(DiffuseBRDF::new(Arc::new(earth_texture)));
    world.add_object(Sphere::new_still(
        1.0,
        Vec3::new(4.9, 1.0, 3.0),
        earth_surface,
    ));

    let mat2 = Arc::new(DiffuseBRDF::from_rgb(Vec3::new(0.4, 0.2, 0.1)));
    world.add_object(Sphere::new_still(1.0, Vec3::new(0.0, 1.0, 0.0), mat2));

    let mat3 = Arc::new(MetalBRDF::from_rgb(Vec3::new(0.7, 0.6, 0.5), 0.1));
    world.add_object(Sphere::new_still(1.0, Vec3::new(4.0, 1.0, 0.0), mat3));

    let tex1 = SolidTexture::new(Vec3::new(0.9, 0.0, 0.1));
    let tex2 = SolidTexture::new(Vec3::new(0.9, 0.9, 0.9));
    let checker_tex = CheckerTexture::new(0.62, Arc::new(tex1), Arc::new(tex2));
    let mat_ground = Arc::new(DiffuseBRDF::new(Arc::new(checker_tex)));
    world.add_object(Sphere::new_still(
        1000.0,
        Vec3::new(0.0, -1000.0, 0.0),
        mat_ground,
    ));

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = width;
    camera.samples_per_pixel = spp;
    camera.max_depth = 50;

    camera.vfov = 28.0;
    camera.look_from = Vec3::new(8.8, 2.0, 3.0);
    camera.look_at = Vec3::ZERO;
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 2.869817807;
    camera.defocus_angle = 2.5;

    camera.environment = EnvironmentType::Color(Vec3::new(0.85, 0.85, 1.0));

    camera.init();
    camera.render(&world, "demo/earth.png");
}

fn cornell_box_scene(width: usize, spp: usize) {
    let mut world = World::new();

    let red = Arc::new(DiffuseBRDF::from_rgb(Vec3::new(0.65, 0.05, 0.05)));
    let white = Arc::new(DiffuseBRDF::from_rgb(Vec3::new(0.73, 0.73, 0.73)));
    let green = Arc::new(DiffuseBRDF::from_rgb(Vec3::new(0.12, 0.45, 0.15)));
    world.add_object(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    ));
    world.add_object(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    ));
    world.add_object(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    ));
    world.add_object(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    ));
    world.add_object(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    ));

    let diffuse_light = Arc::new(DiffuseLight::from_rgb(Vec3::new(25.0, 25.0, 25.0)));
    world.add_light(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        diffuse_light,
    ));

    let color_tex = Arc::new(SolidTexture::new(Vec3::ONE));
    let mat = Arc::new(PrincipledBSDF::new(
        color_tex, // base_color,
        0.01,      // metallic,
        0.01,      // roughness,
        0.01,      // subsurface,
        0.91,      // specular,
        0.91,      // specular_tint,
        1.5,       // ior,
        0.91,      // spec_trans,
        0.91,      // sheen,
        0.91,      // sheen_tint,
        0.91,      // clearcoat,
        0.01,      // clearcoat_gloss,
    ));
    world.add_object(Sphere::new_still(
        135.0,
        Vec3::new(113.0, 170.0, 372.0),
        mat,
    ));

    let box1 = Arc::new(Cuboid::new(
        Vec3::ZERO,
        Vec3::new(165.0, 330.0, 165.0),
        Arc::new(MetalBRDF::from_rgb(Vec3::ONE, 0.1)),
    ));
    let box1 = Instance::new(box1, Vec3::Y, 0.261799, Vec3::new(265.0, 0.0, 295.0));
    world.add_object(box1);

    let box2 = Arc::new(Cuboid::new(
        Vec3::ZERO,
        Vec3::new(165.0, 165.0, 165.0),
        white.clone(),
    ));
    let box2 = Instance::new(box2, Vec3::Y, -0.29, Vec3::new(130.0, 0.0, 65.0));
    world.add_object(box2);

    world.build_bvh();
    let mut camera = Camera::new();
    camera.aspect_ratio = 1.0;
    camera.image_width = width;
    camera.samples_per_pixel = spp;
    camera.max_depth = 50;

    camera.vfov = 40.0;
    camera.look_from = Vec3::new(278.0, 278.0, -800.0);
    camera.look_at = Vec3::new(278.0, 278.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.environment = EnvironmentType::Color(Vec3::ZERO);

    camera.init();
    camera.render(&world, "demo/cornell.png");
}

fn environment_map_scene(width: usize, spp: usize) {
    let mut world = World::new();

    let my_mat = Arc::new(MetalBRDF::from_rgb(Vec3::ONE, 0.001));
    world.add_object(Sphere::new_still(9.0, Vec3::new(4.0, 2.0, 0.0), my_mat));

    let diffuse_light = Arc::new(DiffuseLight::from_rgb(Vec3::new(10.0, 10.0, 10.0)));
    world.add_object(Quad::new(
        Vec3::new(-2.0, 6.5, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        diffuse_light,
    ));

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = width;
    camera.samples_per_pixel = spp;
    camera.max_depth = 50;

    camera.vfov = 90.0;
    camera.look_from = Vec3::new(0.0, 3.0, 17.0);
    camera.look_at = Vec3::new(0.0, 2.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);

    camera.blur_strength = 0.5;
    camera.focal_length = 17.0;
    camera.defocus_angle = 1.5;

    let env_map = ImageTexture::new("assets/grace_probe_latlong.hdr");
    camera.environment = EnvironmentType::Map(Arc::new(env_map));

    camera.init();
    camera.render(&world, "demo/lights.png");
}

fn bunny_scene(width: usize, spp: usize) {
    let mut world = World::new();

    let material_ground = Arc::new(DiffuseBRDF::from_rgb(Vec3::new(0.8, 0.8, 0.0)));
    let material_center = Arc::new(GlassBSDF::basic(1.5));
    let color_tex = Arc::new(SolidTexture::new(Vec3::new(0.8, 0.2, 0.2)));
    let material_left = Arc::new(PrincipledBSDF::new(
        color_tex, // base_color,
        0.00,      // metallic,
        0.91,      // roughness,
        0.01,      // subsurface,
        0.00,      // specular,
        0.01,      // specular_tint,
        1.5,       // ior,
        0.09,      // spec_trans,
        0.01,      // sheen,
        0.01,      // sheen_tint,
        0.01,      // clearcoat,
        0.01,      // clearcoat_gloss,
    ));
    let material_right = Arc::new(MetalBRDF::from_rgb(Vec3::new(0.8, 0.1, 0.2), 0.3));

    world.add_object(Sphere::new_still(
        100.0,
        Vec3::new(0.0, -100.5, -1.0),
        material_ground,
    ));

    let bunny_obj =
        tobj::load_obj("assets/bunny.obj", &tobj::OFFLINE_RENDERING_LOAD_OPTIONS).unwrap();
    let (models, _) = bunny_obj;
    let bunny_mesh = &models[0].mesh;
    world.add_object(Instance::new(
        Arc::new(TriangleMesh::from_obj(10.0, bunny_mesh, material_center).unwrap()),
        Vec3::Z,
        0.0,
        Vec3::new(0.1, -0.8, -2.0),
    ));
    world.add_object(Sphere::new_still(
        0.5,
        Vec3::new(-1.0, 0.0, -1.0),
        material_left,
    ));
    world.add_object(Sphere::new_still(
        0.5,
        Vec3::new(1.0, 0.0, -1.0),
        material_right,
    ));

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = width;
    camera.samples_per_pixel = spp;
    camera.max_depth = 50;

    camera.vfov = 40.0;
    camera.look_from = Vec3::new(0.0, 0.1, 0.5);
    camera.look_at = Vec3::new(0.0, 0.1, 0.0);
    camera.vup = Vec3::Y;

    camera.blur_strength = 0.5;
    camera.focal_length = 10.0;
    camera.defocus_angle = 0.0;

    camera.environment = EnvironmentType::Color(Vec3::new(0.7, 0.8, 1.0));

    camera.init();
    camera.render(&world, "demo/bunny.png");
}

fn my_scene(width: usize, spp: usize) {
    let mut world = World::new();

    let tex1 = SolidTexture::new(Vec3::new(0.2, 0.3, 0.1));
    let tex2 = SolidTexture::new(Vec3::new(0.9, 0.9, 0.9));
    let checker_tex = CheckerTexture::new(0.92, Arc::new(tex1), Arc::new(tex2));
    let material_ground = Arc::new(DiffuseBRDF::from_textures(Arc::new(checker_tex), None));
    world.add_object(Quad::new(
        Vec3::new(-1000.0, 0.0, -1000.0),
        Vec3::new(0.0, 0.0, 5000.0),
        Vec3::new(5000.0, 0.0, 0.0),
        material_ground,
    ));

    let mat1 = MetalBRDF::from_rgb(Vec3::ONE, 0.001);
    world.add_object(Sphere::new_still(
        2.0,
        Vec3::new(-4.0, 2.0, 9.8),
        Arc::new(mat1),
    ));

    let mat2 = GlassBSDF::basic(1.5);
    world.add_object(Sphere::new_still(
        1.0,
        Vec3::new(4.0, 1.0, 6.0),
        Arc::new(mat2),
    ));

    let box1 = Cuboid::new(
        Vec3::ZERO,
        Vec3::new(1.0, 2.0, 1.0),
        Arc::new(DiffuseBRDF::from_rgb(Vec3::new(0.0, 0.5, 1.0))),
    );
    let box1 = Instance::new(Arc::new(box1), Vec3::Y, 0.5, Vec3::new(1.2, 0.0, 6.0));
    world.add_object(box1);

    let bunny_obj =
        tobj::load_obj("assets/bunny.obj", &tobj::OFFLINE_RENDERING_LOAD_OPTIONS).unwrap();
    let (models, _) = bunny_obj;
    let bunny_mesh = &models[0].mesh;
    let color_tex = Arc::new(SolidTexture::new(Vec3::ONE));
    let bunny_material = Arc::new(PrincipledBSDF::new(
        color_tex, // base_color,
        0.91,      // metallic,
        0.01,      // roughness,
        0.01,      // subsurface,
        0.01,      // specular,
        0.91,      // specular_tint,
        1.5,       // ior,
        0.01,      // spec_trans,
        0.91,      // sheen,
        0.91,      // sheen_tint,
        0.91,      // clearcoat,
        0.01,      // clearcoat_gloss,
    ));
    world.add_object(Instance::new(
        Arc::new(TriangleMesh::from_obj(10.0, bunny_mesh, bunny_material).unwrap()),
        Vec3::Y,
        3.14,
        Vec3::new(0.1, -0.327, 5.0),
    ));

    let obj = tobj::load_obj("assets/spot.obj", &tobj::OFFLINE_RENDERING_LOAD_OPTIONS).unwrap();
    let (models, _) = obj;
    let mesh = &models[0].mesh;
    let color_tex = Arc::new(SolidTexture::new(Vec3::new(0.65, 0.05, 0.05)));
    let obj_mat = Arc::new(PrincipledBSDF::new(
        color_tex, // base_color,
        0.01,      // metallic,
        0.01,      // roughness,
        0.91,      // subsurface,
        0.01,      // specular,
        0.01,      // specular_tint,
        1.5,       // ior,
        0.01,      // spec_trans,
        0.91,      // sheen,
        0.91,      // sheen_tint,
        0.91,      // clearcoat,
        0.01,      // clearcoat_gloss,
    ));
    world.add_object(Instance::new(
        Arc::new(TriangleMesh::from_obj(0.65, mesh, obj_mat).unwrap()),
        Vec3::Y,
        0.87,
        Vec3::new(-1.5, 2.8, 4.3),
    ));

    let obj = tobj::load_obj("assets/cow.obj", &tobj::OFFLINE_RENDERING_LOAD_OPTIONS).unwrap();
    let (models, _) = obj;
    let mesh = &models[0].mesh;
    let color_tex = Arc::new(SolidTexture::new(Vec3::new(0.05, 0.65, 0.05)));
    let obj_mat = Arc::new(PrincipledBSDF::new(
        color_tex, // base_color,
        0.91,      // metallic,
        0.21,      // roughness,
        0.91,      // subsurface,
        0.01,      // specular,
        0.01,      // specular_tint,
        1.5,       // ior,
        0.01,      // spec_trans,
        0.91,      // sheen,
        0.91,      // sheen_tint,
        0.91,      // clearcoat,
        0.01,      // clearcoat_gloss,
    ));
    world.add_object(Instance::new(
        Arc::new(TriangleMesh::from_obj(0.75, mesh, obj_mat).unwrap()),
        Vec3::Y,
        0.93,
        Vec3::new(2.5, 3.8, 12.0),
    ));

    let light_mat = DiffuseLight::from_rgb(Vec3::new(20.0, 20.0, 10.0));
    world.add_object(Sphere::new_still(
        0.1,
        Vec3::new(1.0, 0.1, 3.0),
        Arc::new(light_mat),
    ));

    let mat4 = MetalBRDF::from_rgb(Vec3::new(0.6, 0.05, 0.05), 0.1);
    world.add_object(Sphere::new_still(
        0.2,
        Vec3::new(0.0, 0.2, 3.0),
        Arc::new(mat4),
    ));

    {
        let base_color = Arc::new(SolidTexture::new(Vec3::new(0.7, 0.3, 0.3)));
        let roughness = Arc::new(SolidTexture::new(0.3));
        let mat5 = GlassBSDF::new(base_color, roughness, 0.0, 1.5);
        world.add_object(Sphere::new_still(
            0.3,
            Vec3::new(1.2, 0.3, 3.4),
            Arc::new(mat5),
        ));
    }

    world.build_bvh();

    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = width;
    camera.samples_per_pixel = spp;
    camera.max_depth = 50;

    camera.vfov = 60.0;
    camera.look_from = Vec3::new(0.0, 1.5, 0.0);
    camera.look_at = Vec3::new(0.0, 1.5, 100000.0);
    camera.vup = Vec3::Y;

    camera.blur_strength = 0.5;
    camera.focal_length = 6.0;
    camera.defocus_angle = 1.0;

    camera.environment = EnvironmentType::Map(Arc::new(ImageTexture::new(
        "assets/grace_probe_latlong.hdr",
        // "assets/envmap.jpg",
    )));

    camera.init();
    camera.render(&world, "demo/scene6.png");
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    quality: bool,
    #[arg(short, long, default_value_t = 1)]
    scene: usize,
}

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let args = Args::parse();
    let quality = args.quality;
    let (width, spp) = if quality { (1920, 4000) } else { (1920, 100) };

    match args.scene {
        1 => balls_scene(width, spp),
        2 => earth_scene(width, spp),
        3 => cornell_box_scene(width, spp),
        4 => environment_map_scene(width, spp),
        5 => bunny_scene(width, spp),
        6 => my_scene(width, spp),
        _ => (),
    }
}
