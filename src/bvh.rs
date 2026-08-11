use crate::aabb::Aabb;
use crate::ray::Ray;
use crate::shapes::{Shape, ShapeHit};
use crate::vec3::Vec3;

const BINS: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct BvhNode {
    pub aabb: Aabb,
    /// u32::MAX for leaves, else left child node index.
    pub left: u32,
    /// Right child node index (leaves: unused).
    pub right: u32,
    /// For leaves: primitive index range [start, end).
    pub start: u32,
    pub end: u32,
}

pub struct Bvh {
    nodes: Vec<BvhNode>,
    /// Primitive indices into `Scene.shapes`, sorted spatially in leaf ranges.
    prims: Vec<u32>,
}

impl Bvh {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[doc(hidden)]
    pub fn debug_nodes(&self) -> &[BvhNode] {
        &self.nodes
    }

    #[doc(hidden)]
    pub fn debug_prims(&self) -> &[u32] {
        &self.prims
    }

    pub fn build(shapes: &[Shape]) -> Self {
        let mut prims: Vec<u32> = (0..shapes.len() as u32).collect();
        let aabbs: Vec<Aabb> = shapes.iter().map(|s| s.aabb()).collect();
        let mut nodes: Vec<BvhNode> = Vec::with_capacity(shapes.len() * 2);
        Self::build_node(&mut nodes, &mut prims, &aabbs, 0, shapes.len(), 0);
        Bvh { nodes, prims }
    }

    fn build_node(
        nodes: &mut Vec<BvhNode>,
        prims: &mut [u32],
        aabbs: &[Aabb],
        start: usize,
        end: usize,
        depth: usize,
    ) -> u32 {
        let mut bounds = Aabb::empty();
        for &p in &prims[start..end] {
            bounds.grow_aabb(&aabbs[p as usize]);
        }

        let node_index = nodes.len() as u32;
        nodes.push(BvhNode {
            aabb: bounds,
            left: u32::MAX,
            right: u32::MAX,
            start: start as u32,
            end: end as u32,
        });

        let count = end - start;
        let max_leaves = 4;
        if count <= max_leaves || depth >= 40 {
            return node_index;
        }

        // Choose split axis by largest centroid extent.
        let mut cmin = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut cmax = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for &p in &prims[start..end] {
            let c = aabbs[p as usize].centroid();
            cmin = cmin.min(c);
            cmax = cmax.max(c);
        }
        let axis = (cmax - cmin).max_axis();
        let centroid_range = cmax.component(axis) - cmin.component(axis);
        if centroid_range < 1e-8 {
            return node_index;
        }

        // Binned SAH.
        let leaf_cost = (count as f32) * bounds.surface_area();
        let mut bin_count = [0usize; BINS];
        let mut bin_bounds = [Aabb::empty(); BINS];
        let mut bin_costs = [0f32; BINS];
        let inv_range = 1.0 / centroid_range;
        for &p in &prims[start..end] {
            let c = aabbs[p as usize].centroid().component(axis);
            let b =
                (((c - cmin.component(axis)) * inv_range * (BINS as f32)) as usize).min(BINS - 1);
            bin_count[b] += 1;
            bin_bounds[b].grow_aabb(&aabbs[p as usize]);
        }
        let mut best_cost = leaf_cost;
        let mut best_split = 0usize;
        let mut left_count = 0usize;
        let mut left_bounds = Aabb::empty();
        for i in 0..(BINS - 1) {
            left_count += bin_count[i];
            left_bounds.grow_aabb(&bin_bounds[i]);
            if left_count == 0 || left_count == count {
                continue;
            }
            let right_count = count - left_count;
            let mut right_bounds = Aabb::empty();
            for bb in &bin_bounds[(i + 1)..] {
                right_bounds.grow_aabb(bb);
            }
            let cost = left_count as f32 * left_bounds.surface_area()
                + right_count as f32 * right_bounds.surface_area();
            bin_costs[i] = cost;
            if cost < best_cost {
                best_cost = cost;
                best_split = i;
            }
        }

        if best_cost >= leaf_cost {
            return node_index;
        }

        // Partition primitives around the chosen bin boundary.
        let mut left = start;
        let mut right = end;
        let split_pos =
            cmin.component(axis) + ((best_split + 1) as f32 / (BINS as f32)) * centroid_range;
        while left < right {
            while left < right
                && aabbs[prims[left] as usize].centroid().component(axis) <= split_pos
            {
                left += 1;
            }
            while left < right
                && aabbs[prims[right - 1] as usize].centroid().component(axis) > split_pos
            {
                right -= 1;
            }
            if left < right {
                prims.swap(left, right - 1);
            }
        }
        if left == start || left == end {
            return node_index;
        }

        let left_node = Self::build_node(nodes, prims, aabbs, start, left, depth + 1);
        let right_node = Self::build_node(nodes, prims, aabbs, left, end, depth + 1);
        nodes[node_index as usize].left = left_node;
        nodes[node_index as usize].right = right_node;
        nodes[node_index as usize].start = start as u32;
        nodes[node_index as usize].end = end as u32;
        node_index
    }

    /// Find closest hit. Returns hit plus primitive index.
    #[inline]
    pub fn intersect(
        &self,
        shapes: &[Shape],
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(ShapeHit, u32)> {
        let inv_dir = Vec3::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);
        let mut stack = [0u32; 64];
        let mut sp = 0usize;
        stack[sp] = 0;
        let mut best_t = t_max;
        let mut best: Option<(ShapeHit, u32)> = None;

        while sp < 64 {
            let idx = stack[sp];
            sp = sp.wrapping_sub(1);
            let node = &self.nodes[idx as usize];
            if let Some((t0, _t1)) = node.aabb.hit(ray.origin, inv_dir) {
                if t0 > best_t {
                    continue;
                }
            } else {
                continue;
            }

            if node.left == u32::MAX {
                for &p in &self.prims[node.start as usize..node.end as usize] {
                    if let Some(h) = shapes[p as usize].hit(ray, t_min, best_t)
                        && (h.t < best_t || (h.t == best_t && best.is_none_or(|(_, bp)| p < bp)))
                    {
                        best_t = h.t;
                        best = Some((h, p));
                    }
                }
            } else {
                sp += 1;
                stack[sp] = node.left;
                sp += 1;
                stack[sp] = node.right;
            }
        }
        best
    }

    /// Shadow ray test — true if anything blocks the segment (t_min, t_max).
    #[inline]
    pub fn occluded(&self, shapes: &[Shape], ray: &Ray, t_min: f32, t_max: f32) -> bool {
        let inv_dir = Vec3::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);
        let mut stack = [0u32; 64];
        let mut sp = 0usize;
        stack[sp] = 0;

        while sp < 64 {
            let idx = stack[sp];
            sp = sp.wrapping_sub(1);
            let node = &self.nodes[idx as usize];
            if let Some((t0, t1)) = node.aabb.hit(ray.origin, inv_dir) {
                if t0 > t_max || t1 < t_min {
                    continue;
                }
            } else {
                continue;
            }

            if node.left == u32::MAX {
                for &p in &self.prims[node.start as usize..node.end as usize] {
                    if shapes[p as usize].hit(ray, t_min, t_max).is_some() {
                        return true;
                    }
                }
            } else {
                sp += 1;
                stack[sp] = node.left;
                sp += 1;
                stack[sp] = node.right;
            }
        }
        false
    }
}
