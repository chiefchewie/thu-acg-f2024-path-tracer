// src/hittable/mesh.rs
use std::sync::Arc;
use tobj::LoadError;

use crate::bsdf::{BxDFMaterial, MatPtr};
use crate::hittable::{HitInfo, Hittable, AABB};
use crate::{interval::Interval, ray::Ray, vec3::Vec3};

use super::HittableList;

// i'm pretty sure this approach is bad for cache locality but i cant be bothered to implement
// a flat array like what TOBJ is doing (and make it work with my BVH)
pub struct Triangle {
    vertices: [Vec3; 3],
    normals: Option<[Vec3; 3]>,
    uvs: Option<[(f64, f64); 3]>,
    material: MatPtr,
    bbox: AABB,
}

impl Triangle {
    fn new(
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        normals: Option<[Vec3; 3]>,
        uvs: Option<[(f64, f64); 3]>,
        material: MatPtr,
    ) -> Self {
        let min_v = v0.min(v1).min(v2);
        let max_v = v0.max(v1).max(v2);
        let bbox = AABB::new(min_v, max_v);
        Self {
            vertices: [v0, v1, v2],
            normals,
            uvs,
            material,
            bbox,
        }
    }

    pub fn area(&self) -> f64 {
        let edge1 = self.vertices[1] - self.vertices[0];
        let edge2 = self.vertices[2] - self.vertices[0];
        0.5 * edge1.cross(edge2).length()
    }
}

impl Hittable for Triangle {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let v0 = self.vertices[0];
        let v1 = self.vertices[1];
        let v2 = self.vertices[2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = ray.direction().cross(edge2);
        let a = edge1.dot(h);

        if a.abs() < 1e-8 {
            return None; // Ray parallel to triangle
        }

        let f = 1.0 / a;
        let s = ray.origin() - v0;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray.direction().dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if !ray_t.contains(t) {
            return None;
        }

        let w = 1.0 - u - v;
        let normal = if let Some(normals) = self.normals {
            (normals[0] * w + normals[1] * u + normals[2] * v).normalize()
        } else {
            edge1.cross(edge2).normalize()
        };

        let (u, v) = if let Some(uvs) = self.uvs {
            let uv0 = uvs[0];
            let uv1 = uvs[1];
            let uv2 = uvs[2];
            (
                uv0.0 * w + uv1.0 * u + uv2.0 * v,
                uv0.1 * w + uv1.1 * u + uv2.1 * v,
            )
        } else {
            (u, v)
        };

        Some(HitInfo::new(
            ray,
            ray.at(t),
            normal,
            t,
            self.material.clone(),
            u,
            v,
        ))
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }

    fn material(&self) -> Option<&dyn BxDFMaterial> {
        Some(self.material.as_ref())
    }

    fn sample(&self,origin: Vec3, _time: f64) -> Option<Vec3> {
        let u: f64 = rand::random();
        let v: f64 = rand::random();
        let w = 1.0 - u - v;
        let point = self.vertices[0] * w + self.vertices[1] * u + self.vertices[2] * v;
        let dir = (point - origin).normalize();
        Some(dir)
    }

    fn pdf(&self, origin: Vec3, direction: Vec3, time: f64) -> f64 {
        let ray = Ray::new(origin, direction, time);
        if let Some(hit) = self.intersects(&ray, Interval::new(0.0, f64::INFINITY)) {
            let area = self.area();
            let dist = hit.dist;
            let cos_theta = direction.dot(hit.geometric_normal).abs();
            dist * dist / (cos_theta * area)
        } else {
            0.0
        }
    }
}

pub struct TriangleMesh {
    triangles: HittableList,
}

impl TriangleMesh {
    pub fn from_obj(path: &str, material: Arc<dyn BxDFMaterial>) -> Result<Self, LoadError> {
        let obj = tobj::load_obj(path, &tobj::OFFLINE_RENDERING_LOAD_OPTIONS)?;
        let (models, _) = obj;
        let mesh = &models[0].mesh;

        // get vertices
        let vertices: Vec<Vec3> = mesh
            .positions
            .chunks(3)
            .map(|v| Vec3::new(v[0] as f64, v[1] as f64, v[2] as f64) * 10.0)
            .collect();

        // get normals
        let normals: Vec<Vec3> = mesh
            .normals
            .chunks(3)
            .map(|n| Vec3::new(n[0] as f64, n[1] as f64, n[2] as f64))
            .collect();

        // get UVs
        let uvs: Vec<(f64, f64)> = mesh
            .texcoords
            .chunks(2)
            .map(|uv| (uv[0] as f64, uv[1] as f64))
            .collect();

        // let mut triangles: Vec<Triangle> = Vec::new();
        let mut triangles = HittableList::new();
        for chunk in mesh.indices.chunks(3) {
            let [i0, i1, i2] = [chunk[0] as usize, chunk[1] as usize, chunk[2] as usize];
            let normals = if normals.is_empty() {
                None
            } else {
                Some([normals[i0], normals[i1], normals[i2]])
            };
            let uvs = if uvs.is_empty() {
                None
            } else {
                Some([uvs[i0], uvs[i1], uvs[i2]])
            };
            triangles.add(Triangle::new(
                vertices[i0],
                vertices[i1],
                vertices[i2],
                normals,
                uvs,
                material.clone(),
            ));
        }

        triangles.build_bvh();
        Ok(Self { triangles })
    }
}

impl Hittable for TriangleMesh {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        self.triangles.intersects(ray, ray_t)
    }

    fn bounding_box(&self) -> AABB {
        self.triangles.bounding_box()
    }

    fn material(&self) -> Option<&dyn BxDFMaterial> {
        None
    }

    fn sample(&self, origin: Vec3, time: f64) -> Option<Vec3> {
        self.triangles.sample(origin,time)
    }

    fn pdf(&self, origin: Vec3, direction: Vec3, time: f64) -> f64 {
        self.triangles.pdf(origin, direction, time)
    }
}
