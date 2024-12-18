use crate::{bsdf::BxDFMaterial, hittable::HitInfo, interval::Interval, ray::Ray};
use std::{cmp::Ordering, sync::Arc};

use super::{Hittable, AABB};

pub enum BVHNode {
    Leaf {
        bbox: AABB,
        primitives: Vec<Arc<dyn Hittable>>,
    },
    Internal {
        bbox: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
    },
}

pub struct BVH;

impl BVH {
    const MAX_PRIMITIVES_PER_LEAF: usize = 4;

    pub fn build(primitives: Vec<Arc<dyn Hittable>>) -> BVHNode {
        Self::build_recursive(primitives)
    }

    // TODO surface area heuristic instead of splitting along longest axis
    fn build_recursive(mut primitives: Vec<Arc<dyn Hittable>>) -> BVHNode {
        if primitives.len() <= Self::MAX_PRIMITIVES_PER_LEAF {
            let bbox = primitives
                .iter()
                .fold(AABB::default(), |acc, e| AABB::union(acc, e.bounding_box()));
            return BVHNode::Leaf { bbox, primitives };
        }

        let total_bbox = primitives
            .iter()
            .fold(AABB::default(), |acc, e| AABB::union(acc, e.bounding_box()));

        let extent = total_bbox.extent();
        let axis = if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        };

        primitives.sort_by(|a, b| {
            let ca = a.bounding_box().centroid()[axis];
            let cb = b.bounding_box().centroid()[axis];
            ca.partial_cmp(&cb).unwrap_or(Ordering::Equal)
        });

        // Split the primitives into two groups
        let mid = primitives.len() / 2;
        let left_primitives = primitives[..mid].to_vec();
        let right_primitives = primitives[mid..].to_vec();

        // Recursively build the left and right child nodes
        let left = Self::build_recursive(left_primitives);
        let right = Self::build_recursive(right_primitives);

        let bbox = AABB::union(left.bounding_box(), right.bounding_box());

        BVHNode::Internal {
            bbox,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl Hittable for BVHNode {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        self.bounding_box().intersects(ray, ray_t)?;
        match self {
            BVHNode::Leaf { primitives, .. } => {
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
            BVHNode::Internal { left, right, .. } => {
                let left_hit = left.bounding_box().intersects(ray, ray_t);
                let right_hit = right.bounding_box().intersects(ray, ray_t);
                match (left_hit, right_hit) {
                    (None, None) => None,
                    (None, Some(_)) => right.intersects(ray, ray_t),
                    (Some(_), None) => left.intersects(ray, ray_t),
                    (Some(_), Some(_)) => {
                        let left_hit = left.intersects(ray, ray_t);
                        let right_hit = right.intersects(ray, ray_t);
                        match (left_hit, right_hit) {
                            (None, None) => None,
                            (None, Some(right_hit)) => Some(right_hit),
                            (Some(left_hit), None) => Some(left_hit),
                            (Some(left_hit), Some(right_hit)) => {
                                if left_hit.dist < right_hit.dist {
                                    Some(left_hit)
                                } else {
                                    Some(right_hit)
                                }
                            }
                        }
                    }
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

    fn material(&self) -> Option<&dyn BxDFMaterial> {
        None
    }
}
