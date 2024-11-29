use rayon::prelude::*;
use std::{f64::consts::PI, time::Instant};

use crate::{
    interval::Interval,
    material::{Material, MaterialType},
    ray::Ray,
    vec3::{Luminance, Vec2, Vec3},
    Hittable, World,
};
use image::{ImageBuffer, Rgb};
use rand::{thread_rng, Rng};

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
    pub ambient_light: Vec3,

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

        self.forward = (self.look_from - self.look_at).normalize(); // forward
        self.right = self.vup.cross(self.forward).normalize(); // right
        self.up = self.forward.cross(self.right); // up

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

    pub fn render(&self, world: &World, filename: &str) {
        let start = Instant::now();
        let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(self.image_width as u32, self.image_height as u32);

        if cfg!(debug_assertions) {
            println!("rendering debug");
            imgbuf.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
                let (r, c) = (y as usize, x as usize);
                let mut color = Vec3::ZERO;
                // TODO instead of multiple random rays per pixel, could try other Anti-Alias methods
                for _ in 0..self.samples_per_pixel {
                    color += self.trace(r, c, world);
                }
                color *= self.pixel_sample_scale;

                let rbyte = (Self::gamma_correct(color.x).clamp(0.0, 0.999) * 256.0) as u8;
                let gbyte = (Self::gamma_correct(color.y).clamp(0.0, 0.999) * 256.0) as u8;
                let bbyte = (Self::gamma_correct(color.z).clamp(0.0, 0.999) * 256.0) as u8;
                *pixel = image::Rgb([rbyte, gbyte, bbyte]);
            });
        } else {
            println!("rendering production");
            imgbuf.par_enumerate_pixels_mut().for_each(|(x, y, pixel)| {
                let (r, c) = (y as usize, x as usize);
                let mut color = Vec3::ZERO;
                // TODO instead of multiple random rays per pixel, could try other Anti-Alias methods
                for _ in 0..self.samples_per_pixel {
                    color += self.trace(r, c, world);
                }
                color *= self.pixel_sample_scale;

                let rbyte = (Self::gamma_correct(color.x).clamp(0.0, 0.999) * 256.0) as u8;
                let gbyte = (Self::gamma_correct(color.y).clamp(0.0, 0.999) * 256.0) as u8;
                let bbyte = (Self::gamma_correct(color.z).clamp(0.0, 0.999) * 256.0) as u8;
                *pixel = image::Rgb([rbyte, gbyte, bbyte]);
            });
        }

        match imgbuf.save(filename) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Failed to save image {err}");
            }
        }

        dbg!(start.elapsed().as_secs_f64());
    }

    fn gamma_correct(x: f64) -> f64 {
        x.max(0.0).sqrt()
    }

    // random point on the unit circle for offsets in blur anti-aliasing and depth-of-field
    fn random_offsets() -> Vec2 {
        let mut rng = rand::thread_rng();
        let radius = rng.gen::<f64>().sqrt();
        let angle = rng.gen::<f64>() * 2.0 * PI;
        Vec2::new(radius * angle.cos(), radius * angle.sin())
    }

    fn ambient_light(&self, ray: &Ray) -> Vec3 {
        let _ = ray;
        self.ambient_light
    }

    fn generate_ray(&self, r: usize, c: usize) -> Ray {
        let blur_offset = Self::random_offsets() * self.blur_strength;
        let sample_location = self.pixel00
            + (self.pixel_dv * (r as f64 + blur_offset.x))
            + (self.pixel_du * (c as f64 + blur_offset.y));

        let radius = (self.defocus_angle / 2.0).to_radians().tan() * self.focal_length;
        let dof_offset_right = self.right * radius;
        let dof_offset_up = self.up * radius;
        let p = Self::random_offsets();

        let ray_origin = self.center + (dof_offset_right * p.x) + (dof_offset_up * p.y);
        let ray_direction = sample_location - ray_origin;
        let ray_time = thread_rng().gen::<f64>();
        Ray::new(ray_origin, ray_direction, ray_time)
    }

    fn trace(&self, r: usize, c: usize, world: &World) -> Vec3 {
        let eps = 1e-3;
        let min_bounces = 5; // TODO make min_bounces a parameter

        let mut ray = self.generate_ray(r, c);

        let mut radiance = Vec3::ZERO;
        let mut throughput = Vec3::ONE;
        for bounces in 0..self.max_depth {
            let Some(hit_info) = world.intersects(&ray, Interval::new(eps, f64::INFINITY)) else {
                radiance += throughput * self.ambient_light(&ray);
                break;
            };

            // explictily sampling point lights for diffuse materials
            // TODO explicitly sample ALL lights, for ALL materials
            if let MaterialType::DIFFUSE(_) = hit_info.mat {
                for light in &world.lights {
                    let result = world.shadow_ray(hit_info.point, light.position, ray.time());
                    if result {
                        let light_dir = (light.position - hit_info.point).normalize();
                        let len = (light.position - hit_info.point).length();
                        let color =
                            (light.power / (len * len)) * hit_info.normal.dot(light_dir).max(0.0);
                        radiance += throughput * color;
                    }
                }
            }

            let emission = match hit_info.mat {
                MaterialType::DIFFUSE(ref diffuse) => {
                    diffuse.emitted(hit_info.u, hit_info.v, hit_info.point)
                }
                MaterialType::SPECULAR(ref specular) => {
                    specular.emitted(hit_info.u, hit_info.v, hit_info.point)
                }
                MaterialType::REFRACTIVE(ref refractive) => {
                    refractive.emitted(hit_info.u, hit_info.v, hit_info.point)
                }
                MaterialType::LIGHT(ref diffuse_light) => {
                    diffuse_light.emitted(hit_info.u, hit_info.v, hit_info.point)
                }
                MaterialType::MIX(ref mix_material) => {
                    mix_material.emitted(hit_info.u, hit_info.v, hit_info.point)
                }
            };
            radiance += emission * throughput;

            if bounces > min_bounces {
                // russian roulette
                // let p = throughput.max_element();
                let p = throughput.luminance();
                if thread_rng().gen::<f64>() > p {
                    break;
                }
                throughput /= p;
            }

            // attenuation = brdf / pdf in the lingo
            let (attenuation, next_ray) = match hit_info.mat {
                MaterialType::DIFFUSE(ref diffuse) => diffuse.scatter(&ray, &hit_info),
                MaterialType::SPECULAR(ref specular) => specular.scatter(&ray, &hit_info),
                MaterialType::REFRACTIVE(ref refractive) => refractive.scatter(&ray, &hit_info),
                MaterialType::LIGHT(ref diffuse_light) => diffuse_light.scatter(&ray, &hit_info),
                MaterialType::MIX(ref mix_material) => mix_material.scatter(&ray, &hit_info),
            };
            if let Some(scatter_ray) = next_ray {
                throughput *= attenuation;
                ray = scatter_ray;
            } else {
                break;
            }
        }
        radiance
    }
}
