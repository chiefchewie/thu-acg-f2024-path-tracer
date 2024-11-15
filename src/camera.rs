use core::ffi;

use crate::{ray::Ray, vec3::Vec3, Hittable, World};
use rand::Rng;

// TODO camera props: look-at: vec3, right: vec3, up: vec3
#[derive(Debug, Clone, Copy, Default)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: usize,
    pub samples_per_pixel: usize,
    pub max_depth: usize,

    pub vfov: f64,
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,

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

    pub fn width(&self) -> usize {
        self.image_width
    }

    pub fn height(&self) -> usize {
        self.image_height
    }

    pub fn init(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        self.pixel_sample_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.look_from;

        let focal_length = (self.look_from - self.look_at).length();
        let theta = self.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        let v = (self.look_from - self.look_at).normalized(); // forward
        let r = self.up.cross(&v).normalized(); // right
        let u = v.cross(&r); // up

        let viewport_u = r * viewport_width;
        let viewport_v = u * -viewport_height;

        self.pixel_du = viewport_u / self.image_width as f64;
        self.pixel_dv = viewport_v / self.image_height as f64;

        let upperleft = self.center - (v * focal_length) - (viewport_u / 2.0) - (viewport_v / 2.0);
        self.pixel00 = upperleft + (self.pixel_du + self.pixel_dv) * 0.5;
    }

    pub fn render(&self, world: &World, pixels: &mut Vec<Vec3>) {
        for r in 0..self.image_height {
            for c in 0..self.image_width {
                let mut color = Vec3::zeroes();
                // TODO instead of multiple random rays per pixel, could try other Anti-Alias methods
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(r, c);
                    color = color + self.trace(&ray, self.max_depth, &world);
                }
                pixels[r * self.image_width + c] = color * self.pixel_sample_scale;
            }
        }
    }

    // offsets from the pixel center but are still in its 'square'
    fn sample_square() -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3::new(rng.gen::<f64>() - 0.5, rng.gen::<f64>() - 0.5, 0.0)
    }

    fn ambient_light(ray: &Ray) -> Vec3 {
        let a = 0.5 * (ray.direction().y() + 1.0);
        Vec3::new(1.0, 1.0, 1.0) * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
    }

    fn get_ray(&self, r: usize, c: usize) -> Ray {
        let offset = Self::sample_square();
        let sample_location = self.pixel00
            + (self.pixel_dv * (r as f64 + offset.y()))
            + (self.pixel_du * (c as f64 + offset.x()));
        let ray_dir = sample_location - self.center;
        Ray::new(self.center, ray_dir)
    }

    fn trace(&self, ray: &Ray, depth: usize, world: &World) -> Vec3 {
        if depth <= 0 {
            return Self::ambient_light(ray);
        }

        let info = world.intersects(ray, 0.0001, f64::INFINITY);
        if info.did_hit {
            let (_, attenuation, scatter_ray) = info.mat.scatter(ray, &info);
            self.trace(&scatter_ray, depth - 1, world) * attenuation
        } else {
            Self::ambient_light(ray)
        }
    }
}
