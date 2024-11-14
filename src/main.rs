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
struct Sphere {
    radius: f64,
    position: Point3,
    color: Color,
}

impl Sphere {
    pub fn intersects(&self, ray: &Ray) -> bool {
        let oc = self.position - ray.origin;
        let a = ray.direction.length_squared();
        let b = ray.direction.dot(&oc) * -2.0;
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        discriminant >= 0.0
    }
}

fn trace(ray: &Ray, spheres: &Vec<Sphere>) -> Color {
    let mut min_d = f64::INFINITY;
    let mut color = Color::new(0,0,0);
    for sp in spheres.iter() {
        if sp.intersects(ray) {
            let d = (ray.origin - sp.position).length();
            if d < min_d {
                min_d = d;
                color = sp.color;
            }
        }
    }
    return color;
}

fn write_to_ppm(colors: &Vec<Color>, width: usize, height: usize) {
    println!("P3\n{} {}\n255\n", width, height);
    for r in 0..height {
        for c in 0..width {
            let color = colors[r * width + c];
            println!("{} {} {}", color.r, color.g, color.b);
        }
    }
}

fn main() {
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as usize;

    let mut pixels = vec![Color { r: 0, g: 0, b: 0 }; image_width * image_height];

    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let focal_length = 1.0;
    let cam_position = Vec3::new(0.0, 0.0, 0.0);
    let cam_forward = Vec3::new(0.0, 0.0, -1.0);
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
            color: Color::new(255, 0, 0),
        },
        Sphere {
            radius: 2.0,
            position: Point3::new(3.0, 4.0, 15.0),
            color: Color::new(0, 0, 255),
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
