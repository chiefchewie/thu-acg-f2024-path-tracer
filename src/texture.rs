use std::rc::Rc;

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
        let x = (point.x() * self.inv_scale).floor() as i32;
        let y = (point.y() * self.inv_scale).floor() as i32;
        let z = (point.z() * self.inv_scale).floor() as i32;
        if (x + y + z) % 2 == 0 {
            self.tex1.value(u, v, point)
        } else {
            self.tex2.value(u, v, point)
        }
    }
}
