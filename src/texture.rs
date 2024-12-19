use std::sync::Arc;

use image::{ImageBuffer, ImageReader, Pixel, Rgb};

use crate::vec3::Vec3;

pub trait Texture<T: Clone + Send + Sync>: Send + Sync {
    fn value(&self, u: f64, v: f64, point: &Vec3) -> T;
}

pub struct SolidTexture<T> {
    value: T,
}

impl<T> SolidTexture<T> {
    pub fn new(value: T) -> Self {
        SolidTexture { value }
    }
}

impl<T: Clone + Send + Sync> Texture<T> for SolidTexture<T> {
    fn value(&self, _u: f64, _v: f64, _point: &Vec3) -> T {
        self.value.clone()
    }
}

pub struct CheckerTexture<T> {
    inv_scale: f64,
    tex1: Arc<dyn Texture<T>>,
    tex2: Arc<dyn Texture<T>>,
}

impl<T> CheckerTexture<T> {
    pub fn new(scale: f64, tex1: Arc<dyn Texture<T>>, tex2: Arc<dyn Texture<T>>) -> Self {
        CheckerTexture {
            inv_scale: scale.recip(),
            tex1,
            tex2,
        }
    }
}

impl<T: Clone + Send + Sync> Texture<T> for CheckerTexture<T> {
    fn value(&self, u: f64, v: f64, point: &Vec3) -> T {
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

#[derive(Debug)]
pub struct ImageTexture {
    pub img: ImageBuffer<Rgb<u8>, Vec<u8>>,
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

impl Texture<Vec3> for ImageTexture {
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
