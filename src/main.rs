use core::f64;

use path_tracer::whatever::{Point3, Ray, Vec3};

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

#[derive(Debug, Clone, Copy)]
struct Material {
    color: Vec3,
}
#[derive(Debug, Clone, Copy)]
struct Sphere {
    radius: f64,
    position: Point3,
    material: Material,
}

#[derive(Debug, Clone, Copy)]
struct HitInfo {
    did_hit: bool,
    dist: f64,
    point: Vec3,
    normal: Vec3,
    material: Material,
}

impl HitInfo {
    pub fn new(did_hit: bool, dist: f64, point: Vec3, normal: Vec3, material: Material) -> HitInfo {
        HitInfo {
            did_hit,
            dist,
            point,
            normal,
            material,
        }
    }
}

impl Sphere {
    pub fn intersects(&self, ray: &Ray) -> HitInfo {
        let oc = self.position - ray.origin;
        let a = ray.direction.length_squared();
        let b = ray.direction.dot(&oc) * -2.0;
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant >= 0.0 {
            let dist = (-b - discriminant.sqrt()) / (2.0 * a);
            if dist >= 0.0 {
                let point = ray.at(dist);
                return HitInfo {
                    did_hit: true,
                    dist,
                    point,
                    normal: (point - self.position),
                    material: self.material,
                };
            }
        }
        return HitInfo {
            did_hit: false,
            dist: 0.0,
            point: Vec3::zeroes(),
            normal: Vec3::zeroes(),
            material: self.material,
        };
    }
}

fn intersect_world(ray: &Ray, spheres: &Vec<Sphere>) -> HitInfo {
    let mut closest_hit = HitInfo::new(
        false,
        f64::INFINITY,
        Vec3::zeroes(),
        Vec3::zeroes(),
        Material {
            color: Vec3::zeroes(),
        },
    );
    for sp in spheres.iter() {
        let info = sp.intersects(ray);
        if info.did_hit && info.dist < closest_hit.dist {
            closest_hit = info;
        }
    }
    return closest_hit;
}

fn trace(ray: &Ray, spheres: &Vec<Sphere>) -> Vec3 {
    let closest_hit = intersect_world(ray, spheres);
    return closest_hit.material.color;
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
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 800;
    let image_height = (image_width as f64 / aspect_ratio) as usize;

    let mut pixels = vec![Vec3::new(0.0, 0.0, 0.0); image_width * image_height];

    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let focal_length = 1.0;
    let cam_position = Vec3::new(0.0, 0.0, 0.0);
    let cam_forward = Vec3::new(0.0, 0.0, 1.0);
    let cam_up = Vec3::new(0.0, 1.0, 0.0);
    let cam_right = Vec3::new(1.0, 0.0, 0.0);

    let viewport_center = cam_position + cam_forward * focal_length;

    let viewport_u = cam_right * viewport_width;
    let viewport_v = -cam_up * viewport_height;

    let pixel_du = viewport_u / image_width as f64;
    let pixel_dv = viewport_v / image_height as f64;

    let upperleft = viewport_center - (viewport_u / 2.0) - (viewport_v / 2.0);
    let pixel00 = upperleft + pixel_du * 0.5 + pixel_dv * 0.5;

    let spheres = vec![
        Sphere {
            radius: 5.0,
            position: Point3::new(0.0, 0.0, 20.0),
            material: Material {
                color: Vec3::new(1.0, 0.0, 0.0),
            },
        },
        Sphere {
            radius: 2.0,
            position: Point3::new(3.0, 4.0, 15.0),
            material: Material {
                color: Vec3::new(0.5, 0.5, 0.0),
            },
        },
    ];

    for r in 0..image_height {
        for c in 0..image_width {
            let pixel_position = pixel00 + (pixel_du * c as f64) + (pixel_dv * r as f64);
            let ray_dir = (pixel_position - cam_position).normalized();
            let ray = Ray::new(cam_position, ray_dir);
            let color = trace(&ray, &spheres);
            pixels[r * image_width + c] = color;
        }
    }
    write_to_ppm(&pixels, image_width, image_height);
}
