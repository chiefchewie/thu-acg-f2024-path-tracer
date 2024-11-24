use std::{cmp::Ordering, rc::Rc};

use crate::{aabb::AABB, hit_info::HitInfo, interval::Interval, ray::Ray, Hittable};

pub enum BVHNode {
    Leaf {
        bbox: AABB,
        primitives: Vec<Rc<dyn Hittable>>,
    },
    Internal {
        bbox: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
    },
}

pub struct BVH;

impl BVH {
    pub fn build(primitives: Vec<Rc<dyn Hittable>>) -> BVHNode {
        Self::build_recursive(primitives)
    }

    fn build_recursive(mut primitives: Vec<Rc<dyn Hittable>>) -> BVHNode {
        if primitives.len() == 1 {
            let bbox = primitives[0].bounding_box();
            return BVHNode::Leaf { bbox, primitives };
        }

        let total_bbox = primitives
            .iter()
            .fold(primitives[0].bounding_box(), |acc, e| {
                AABB::union(acc, e.bounding_box())
            });

        let extent = total_bbox.extent();
        let axis = if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        };

        primitives.sort_by(|a, b| {
            let centroid_a = a.bounding_box().centroid();
            let centroid_b = b.bounding_box().centroid();
            let ca = match axis {
                0 => centroid_a.x,
                1 => centroid_a.y,
                2 => centroid_a.z,
                _ => unreachable!(),
            };
            let cb = match axis {
                0 => centroid_b.x,
                1 => centroid_b.y,
                2 => centroid_b.z,
                _ => unreachable!(),
            };
            ca.partial_cmp(&cb).unwrap_or(Ordering::Equal)
        });

        // Split the primitives into two groups
        let mid = primitives.len() / 2;
        let left_primitives = primitives[..mid].to_vec();
        let right_primitives = primitives[mid..].to_vec();

        // Recursively build the left and right child nodes
        let left = Self::build_recursive(left_primitives);
        let right = Self::build_recursive(right_primitives);

        // Return an internal node
        let bbox = match (&left, &right) {
            (
                BVHNode::Leaf {
                    bbox: left_aabb, ..
                },
                BVHNode::Leaf {
                    bbox: right_aabb, ..
                },
            ) => AABB::union(*left_aabb, *right_aabb),
            _ => total_bbox,
        };

        BVHNode::Internal {
            bbox,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl Hittable for BVHNode {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        if !self.bounding_box().intersects(ray, ray_t) {
            return None;
        }

        match self {
            BVHNode::Leaf { bbox: _, primitives } => {
                let mut hit_info: Option<HitInfo> = None;
                let mut closest_hit = ray_t.max;
                for p in primitives {
                    if let Some(info) = p.intersects(ray, Interval::new(ray_t.min, closest_hit)) {
                        closest_hit = info.dist;
                        hit_info = Some(info);
                    }
                }
                hit_info
            }
            BVHNode::Internal { bbox: _, left, right } => {
                let left_hit = left.intersects(ray, ray_t);
                let right_hit = if let Some(ref hit) = left_hit {
                    right.intersects(ray, Interval::new(ray_t.min, hit.dist))
                } else {
                    right.intersects(ray, ray_t)
                };
                match (left_hit, right_hit) {
                    (None, None) => None,
                    (_, Some(info)) => Some(info),
                    (Some(info), None) => Some(info),
                }
            }
        }
    }

    fn bounding_box(&self) -> AABB {
        match self {
            BVHNode::Leaf { bbox, .. } => *bbox,
            BVHNode::Internal { bbox, .. } => *bbox,
        }
    }
}
