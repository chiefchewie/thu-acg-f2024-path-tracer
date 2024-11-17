use std::{f64::consts::PI, fs::File, time::Instant};

use crate::{ray::Ray, vec3::Vec3, Hittable, World};
use image::{codecs::png::PngEncoder, ImageEncoder};
use rand::Rng;

#[derive(Debug, Clone, Copy, Default)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: usize,
    pub samples_per_pixel: usize,
    pub max_depth: usize,

    pub vfov: f64,
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub vup: Vec3,

    pub blur_strength: f64,
    pub focal_length: f64,
    pub defocus_angle: f64,

    forward: Vec3,
    right: Vec3,
    up: Vec3,

    image_height: usize,
    pixel_sample_scale: f64,
    center: Vec3,
    pixel00: Vec3,
    pixel_du: Vec3,
    pixel_dv: Vec3,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            ..Default::default()
        }
    }

    pub fn init(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        self.pixel_sample_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.look_from;

        let theta = self.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focal_length;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        self.forward = (self.look_from - self.look_at).normalized(); // forward
        self.right = self.vup.cross(&self.forward).normalized(); // right
        self.up = self.forward.cross(&self.right); // up

        let viewport_u = self.right * viewport_width;
        let viewport_v = self.up * -viewport_height;

        self.pixel_du = viewport_u / self.image_width as f64;
        self.pixel_dv = viewport_v / self.image_height as f64;

        let upperleft = self.center
            - (self.forward * self.focal_length)
            - (viewport_u / 2.0)
            - (viewport_v / 2.0);
        self.pixel00 = upperleft + (self.pixel_du + self.pixel_dv) * 0.5;
    }

    pub fn render(&self, world: &World) {
        let start = Instant::now();
        let mut pixels: Vec<u8> = vec![0; self.image_width * self.image_height * 3];
        for r in 0..self.image_height {
            for c in 0..self.image_width {
                let mut color = Vec3::zeroes();
                // TODO instead of multiple random rays per pixel, could try other Anti-Alias methods
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(r, c);
                    color = color + Self::trace(&ray, self.max_depth, world);
                }
                color = color * self.pixel_sample_scale;

                let rbyte = (Self::gamma_correct(color.x()).clamp(0.0, 0.999) * 256.0) as u8;
                let gbyte = (Self::gamma_correct(color.y()).clamp(0.0, 0.999) * 256.0) as u8;
                let bbyte = (Self::gamma_correct(color.z()).clamp(0.0, 0.999) * 256.0) as u8;
                let idx = r * self.image_width + c;
                pixels[idx * 3] = rbyte;
                pixels[idx * 3 + 1] = gbyte;
                pixels[idx * 3 + 2] = bbyte;
            }
        }
        dbg!(start.elapsed().as_secs_f64());

        let file = File::create("image.png");
        match file {
            Ok(ting) => {
                let encoder = PngEncoder::new(ting);
                match encoder.write_image(
                    &pixels,
                    self.image_width as u32,
                    self.image_height as u32,
                    image::ExtendedColorType::Rgb8,
                ) {
                    Ok(_) => {}
                    Err(_) => panic!("error tryna write file"),
                }
            }
            Err(_) => panic!("error tryna create file"),
        }
    }

    fn gamma_correct(x: f64) -> f64 {
        if x > 0.0 {
            x.sqrt()
        } else {
            0.0
        }
    }

    // random point on the unit circle for offsets in blur anti-aliasing and depth-of-field
    // TODO make this a vec2 for clarity
    fn random_offsets() -> Vec3 {
        let mut rng = rand::thread_rng();
        let radius = rng.gen::<f64>().sqrt();
        let angle = rng.gen::<f64>() * 2.0 * PI;
        Vec3::new(radius * angle.cos(), radius * angle.sin(), 0.0)
    }

    fn ambient_light(ray: &Ray) -> Vec3 {
        let a = 0.5 * (ray.direction().y() + 1.0);
        Vec3::new(1.0, 1.0, 1.0) * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
    }

    fn get_ray(&self, r: usize, c: usize) -> Ray {
        let blur_offset = Self::random_offsets() * self.blur_strength;
        let sample_location = self.pixel00
            + (self.pixel_dv * (r as f64 + blur_offset.x()))
            + (self.pixel_du * (c as f64 + blur_offset.y()));

        let radius = (self.defocus_angle / 2.0).to_radians().tan() * self.focal_length;
        // let r = self.focal_length / self.fstop / 2.0;
        let dof_offset_right = self.right * radius;
        let dof_offset_up = self.up * radius;
        let p = Self::random_offsets();
        let ray_origin = self.center + (dof_offset_right * p.x()) + (dof_offset_up * p.y());

        let ray_dir = sample_location - ray_origin;
        Ray::new(ray_origin, ray_dir)
    }

    fn trace(ray: &Ray, depth: usize, world: &World) -> Vec3 {
        if depth == 0 {
            return Self::ambient_light(ray);
        }

        let eps = 1e-3;
        match world.intersects(ray, eps, f64::INFINITY) {
            Some(info) => {
                let (should_bounce, attenuation, scatter_ray) = match info.mat {
                    crate::material::Material::DIFFUSE(material) => material.scatter(&info),
                    crate::material::Material::SPECULAR(material) => material.scatter(ray, &info),
                    crate::material::Material::REFRACTIVE(material) => material.scatter(ray, &info),
                };
                if should_bounce {
                    Self::trace(&scatter_ray, depth - 1, world) * attenuation
                } else {
                    attenuation
                }
            }
            None => Self::ambient_light(ray),
        }
    }
}
