// src/hittable/mesh.rs
use std::sync::Arc;
use tobj::LoadError;

use crate::bsdf::{BxDFMaterial, MatPtr};
use crate::hittable::{HitInfo, Hittable, AABB};
use crate::{interval::Interval, ray::Ray, vec3::Vec3};

pub struct TriangleMesh {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<[f64; 2]>,
    indices: Vec<[usize; 3]>, // Triangle indices
    material: MatPtr,
    bbox: AABB,
}

impl TriangleMesh {
    pub fn from_obj(path: &str, material: Arc<dyn BxDFMaterial>) -> Result<Self, LoadError> {
        let obj = tobj::load_obj(path, &tobj::OFFLINE_RENDERING_LOAD_OPTIONS)?;
        let (models, _) = obj;
        let mesh = &models[0].mesh;

        // Convert vertices
        let vertices: Vec<Vec3> = mesh
            .positions
            .chunks(3)
            .map(|v| Vec3::new(v[0] as f64, v[1] as f64, v[2] as f64) * 10.0)
            .collect();

        let min_vertex = vertices.iter().fold(Vec3::INFINITY, |a, b| a.min(*b));
        let max_vertex = vertices.iter().fold(Vec3::NEG_INFINITY, |a, b| a.max(*b));

        // Convert normals
        let normals = mesh
            .normals
            .chunks(3)
            .map(|n| Vec3::new(n[0] as f64, n[1] as f64, n[2] as f64))
            .collect();

        // Convert UVs
        let uvs = mesh
            .texcoords
            .chunks(2)
            .map(|uv| [uv[0] as f64, uv[1] as f64])
            .collect();

        // Convert indices to triangles
        let indices = mesh
            .indices
            .chunks(3)
            .map(|idx| [idx[0] as usize, idx[1] as usize, idx[2] as usize])
            .collect();

        // Calculate bounding box
        let bbox = AABB::new(min_vertex, max_vertex);

        Ok(Self {
            vertices,
            normals,
            uvs,
            indices,
            material,
            bbox,
        })
    }
}

impl Hittable for TriangleMesh {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let mut closest_hit = ray_t.max;
        let mut hit_info = None;

        // Test each triangle
        for triangle in &self.indices {
            let v0 = self.vertices[triangle[0]];
            let v1 = self.vertices[triangle[1]];
            let v2 = self.vertices[triangle[2]];

            // Möller–Trumbore intersection algorithm
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let h = ray.direction().cross(edge2);
            let a = edge1.dot(h);

            if a.abs() < 1e-6 {
                continue; // Ray parallel to triangle
            }

            let f = 1.0 / a;
            let s = ray.origin() - v0;
            let u = f * s.dot(h);

            if !(0.0..=1.0).contains(&u) {
                continue;
            }

            let q = s.cross(edge1);
            let v = f * ray.direction().dot(q);

            if v < 0.0 || u + v > 1.0 {
                continue;
            }

            let t = f * edge2.dot(q);
            if t < ray_t.min || t > closest_hit {
                continue;
            }

            // We have a hit - compute normal and UV
            let w = 1.0 - u - v;
            let normal = if !self.normals.is_empty() {
                (self.normals[triangle[0]] * w
                    + self.normals[triangle[1]] * u
                    + self.normals[triangle[2]] * v)
                    .normalize()
            } else {
                edge1.cross(edge2).normalize()
            };

            let uv = if !self.uvs.is_empty() {
                let uv0 = self.uvs[triangle[0]];
                let uv1 = self.uvs[triangle[1]];
                let uv2 = self.uvs[triangle[2]];
                [
                    uv0[0] * w + uv1[0] * u + uv2[0] * v,
                    uv0[1] * w + uv1[1] * u + uv2[1] * v,
                ]
            } else {
                [u, v]
            };

            closest_hit = t;
            hit_info = Some(HitInfo::new(
                ray,
                ray.at(t),
                normal,
                t,
                self.material.clone(),
                uv[0],
                uv[1],
            ));
            // hit_info = Some(HitInfo::new(
            //     ray.at(t),
            //     normal,
            //     t,
            //     uv[0],
            //     uv[1],
            //     ray.direction().dot(normal) < 0.0,
            //     self.material.as_ref(),
            // ));
        }

        hit_info
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }

    fn material(&self) -> Option<&dyn BxDFMaterial> {
        Some(self.material.as_ref())
    }
}
