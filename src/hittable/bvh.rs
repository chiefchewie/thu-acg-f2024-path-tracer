use crate::{bsdf::BxDFMaterial, hittable::HitInfo, interval::Interval, ray::Ray};
use std::{cmp::Ordering, sync::Arc};

use super::{Hittable, AABB};

pub enum BVHNode {
    Leaf {
        bbox: AABB,
        hittables: Vec<Arc<dyn Hittable>>,
    },
    Internal {
        bbox: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
    },
}

pub struct BVH;

type HitList = Vec<Arc<dyn Hittable>>;
impl BVH {
    const MAX_HITTABLES_PER_LEAF: usize = 4;

    pub fn build(hittables: Vec<Arc<dyn Hittable>>) -> BVHNode {
        Self::build_recursive(hittables)
    }

    fn build_recursive(hittables: Vec<Arc<dyn Hittable>>) -> BVHNode {
        if hittables.len() <= Self::MAX_HITTABLES_PER_LEAF {
            let bbox = hittables
                .iter()
                .fold(AABB::default(), |acc, e| acc.union(e.bounding_box()));
            return BVHNode::Leaf { bbox, hittables };
        }

        let (left_list, right_list) = Self::find_best_split(&hittables);
        if left_list.is_empty() || right_list.is_empty() {
            let bbox = hittables
                .iter()
                .fold(AABB::default(), |acc, e| acc.union(e.bounding_box()));
            return BVHNode::Leaf { bbox, hittables };
        }

        let left_node = Self::build_recursive(left_list);
        let right_node = Self::build_recursive(right_list);
        let bbox = AABB::union(left_node.bounding_box(), right_node.bounding_box());
        BVHNode::Internal {
            bbox,
            left: Box::new(left_node),
            right: Box::new(right_node),
        }
    }

    fn find_best_split(hittables: &[Arc<dyn Hittable>]) -> (HitList, HitList) {
        let parent_bbox = hittables
            .iter()
            .fold(AABB::default(), |acc, obj| acc.union(obj.bounding_box()));
        let mut best_cost = f64::INFINITY;
        let mut best_axis = 0;
        let mut best_split_pos = 0.0;

        for axis in 0..3 {
            let mut positions: Vec<f64> = hittables
                .iter()
                .map(|obj| obj.bounding_box().centroid()[axis])
                .collect();
            positions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            for split_pos in positions {
                let cost = Self::evaluate_sah(axis, split_pos, parent_bbox, hittables);
                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_split_pos = split_pos;
                }
            }
        }

        let (left, right): (Vec<_>, Vec<_>) = hittables
            .iter()
            .cloned()
            .partition(|obj| obj.bounding_box().centroid()[best_axis] < best_split_pos);

        (left, right)
    }

    fn evaluate_sah(
        axis: usize,
        split_pos: f64,
        parent_bbox: AABB,
        hittables: &[Arc<dyn Hittable>],
    ) -> f64 {
        let mut left_bbox = AABB::default();
        let mut left_count = 0;

        let mut right_bbox = AABB::default();
        let mut right_count = 0;

        for obj in hittables {
            if obj.bounding_box().centroid()[axis] < split_pos {
                left_bbox = left_bbox.union(obj.bounding_box());
                left_count += 1;
            } else {
                right_bbox = right_bbox.union(obj.bounding_box());
                right_count += 1;
            }
        }

        if left_count == 0 || right_count == 0 {
            return f64::INFINITY;
        }

        let cost = left_bbox.surface_area() * left_count as f64
            + right_bbox.surface_area() * right_count as f64;
        let parent_cost = parent_bbox.surface_area() * hittables.len() as f64;
        if cost > 0.0 && cost < parent_cost {
            cost
        } else {
            f64::INFINITY
        }
    }
}

impl Hittable for BVHNode {
    fn intersects(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        self.bounding_box().intersects(ray, ray_t)?;
        match self {
            BVHNode::Leaf { hittables, .. } => {
                let mut hit_info: Option<HitInfo> = None;
                let mut closest_hit = ray_t.max;
                for p in hittables {
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

    fn sample_surface(
        &self,
        _hit_info: &HitInfo,
        _time: f64,
    ) -> Option<(crate::vec3::Vec3, crate::vec3::Vec3, f64)> {
        None
    }
}
