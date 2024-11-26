use std::rc::Rc;

use image::{ImageBuffer, ImageReader, Pixel, Rgb};

use crate::vec3::Vec3;

pub trait Texture {
    fn value(&self, u: f64, v: f64, point: &Vec3) -> Vec3;
}

pub struct SolidColorTexture {
    albedo: Vec3,
}

impl SolidColorTexture {
    pub fn new(r: f64, g: f64, b: f64) -> SolidColorTexture {
        SolidColorTexture {
            albedo: Vec3::new(r, g, b),
        }
    }

    pub fn from_vec(albedo: Vec3) -> SolidColorTexture {
        SolidColorTexture { albedo }
    }
}

impl Texture for SolidColorTexture {
    fn value(&self, _u: f64, _v: f64, _point: &Vec3) -> Vec3 {
        self.albedo
    }
}

pub struct CheckerTexture {
    inv_scale: f64,
    tex1: Rc<dyn Texture>,
    tex2: Rc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, tex1: Rc<dyn Texture>, tex2: Rc<dyn Texture>) -> CheckerTexture {
        CheckerTexture {
            inv_scale: scale.recip(),
            tex1,
            tex2,
        }
    }

    pub fn from_colors(scale: f64, color1: Vec3, color2: Vec3) -> CheckerTexture {
        CheckerTexture {
            inv_scale: scale.recip(),
            tex1: Rc::new(SolidColorTexture::from_vec(color1)),
            tex2: Rc::new(SolidColorTexture::from_vec(color2)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, point: &Vec3) -> Vec3 {
        let x = (point.x * self.inv_scale).floor() as i32;
        let y = (point.y * self.inv_scale).floor() as i32;
        let z = (point.z * self.inv_scale).floor() as i32;
        if (x + y + z) % 2 == 0 {
            self.tex1.value(u, v, point)
        } else {
            self.tex2.value(u, v, point)
        }
    }
}

pub struct ImageTexture {
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl ImageTexture {
    pub fn new(filename: &str) -> ImageTexture {
        let img = ImageReader::open(filename)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgb8();
        ImageTexture { img }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _point: &Vec3) -> Vec3 {
        if self.img.height() == 0 {
            return Vec3::new(0.0, 1.0, 1.0);
        }

        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let i = (u * self.img.width() as f64) as u32;
        let j = (v * self.img.height() as f64) as u32;
        let pixel = self.img.get_pixel(i, j);
        let color_scale = 1.0 / 255.0;

        Vec3::new(
            color_scale * pixel.channels()[0] as f64,
            color_scale * pixel.channels()[1] as f64,
            color_scale * pixel.channels()[2] as f64,
        )
    }
}
