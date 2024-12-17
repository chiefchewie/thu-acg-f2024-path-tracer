use crate::bsdf::BxDFMaterial;
use crate::{interval::Interval, ray::Ray};

pub mod aabb;
pub use self::aabb::*;

pub mod cuboid;
pub use self::cuboid::*;

pub mod bvh;
pub use self::bvh::*;

pub mod hit_info;
pub use self::hit_info::*;

pub mod quad;
pub use self::quad::*;

pub mod sphere;
pub use self::sphere::*;

pub mod world;
pub use self::world::*;

pub mod instance;
pub use self::instance::*;

pub mod light;
pub use self::light::*;

pub mod list;
pub use self::list::*;

pub mod mesh;
pub use self::mesh::*;

pub trait Hittable: Send + Sync {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo>;
    fn bounding_box(&self) -> AABB;
    fn material(&self) -> Option<&dyn BxDFMaterial>;
}
