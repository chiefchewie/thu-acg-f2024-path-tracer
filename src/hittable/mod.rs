use crate::bsdf::BxDFMaterial;
use crate::vec3::Vec3;
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

    /// sample a point P on the surface of the hittable
    fn sample(&self, origin: Vec3, time: f64) -> Option<Vec3>;

    /// pdf of point P on surface
    fn pdf(&self, origin: Vec3, direction: Vec3, time: f64) -> f64;
}
