// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::Rotation;
use crate::kinetics::{Axis, Pos};

const CONTACT_EPSILON: f64 = 1.0e-7;

/// Contact from a grounded body on a translating full-block top. Rising tops
/// supply lift, while the normal gravity path follows a receding surface.
pub fn linear_top_motion(
    previous_origin: [f64; 3],
    current_origin: [f64; 3],
    full_blocks: impl IntoIterator<Item = Pos>,
    feet: [f64; 3],
    [half_x, half_z]: [f64; 2],
) -> Option<[f64; 3]> {
    let [x, y, z] = std::array::from_fn(|i| feet[i] - previous_origin[i]);
    let supported = full_blocks.into_iter().any(|block| {
        let [block_x, block_y, block_z] = block.map(f64::from);
        (y - block_y - 1.0).abs() <= CONTACT_EPSILON
            && x + half_x > block_x
            && x - half_x < block_x + 1.0
            && z + half_z > block_z
            && z - half_z < block_z + 1.0
    });
    supported.then(|| {
        let [dx, dy, dz] = std::array::from_fn(|i| current_origin[i] - previous_origin[i]);
        [dx, dy.max(0.0), dz]
    })
}

/// Separation of an initially external body crossed by the leading face of
/// a translating full block. The host clips the resulting displacement.
pub fn linear_face_separation(
    previous: [f64; 3],
    current: [f64; 3],
    full_blocks: impl IntoIterator<Item = Pos>,
    body_min: [f64; 3],
    body_max: [f64; 3],
) -> Option<[f64; 3]> {
    let delta: [f64; 3] = std::array::from_fn(|i| current[i] - previous[i]);
    let axis = delta.iter().position(|&step| step != 0.0)?;
    if !delta[axis].is_finite()
        || delta
            .iter()
            .enumerate()
            .any(|(i, &step)| i != axis && step != 0.0)
    {
        return None;
    }
    let positive = delta[axis] > 0.0;
    let mut separation = 0.0f64;
    for block in full_blocks {
        let low: [f64; 3] = std::array::from_fn(|i| previous[i] + f64::from(block[i]));
        if (0..3)
            .filter(|&i| i != axis)
            .any(|i| low[i] + 1.0 <= body_min[i] || low[i] >= body_max[i])
        {
            continue;
        }
        let (gap, overlap) = if positive {
            let face = low[axis] + 1.0;
            (body_min[axis] - face, face + delta[axis] - body_min[axis])
        } else {
            let face = low[axis];
            (face - body_max[axis], body_max[axis] - (face + delta[axis]))
        };
        if gap >= -CONTACT_EPSILON && overlap > 0.0 {
            separation = separation.max(overlap);
        }
    }
    if separation == 0.0 {
        return None;
    }
    let mut response = [0.0; 3];
    response[axis] = (separation + CONTACT_EPSILON).copysign(delta[axis]);
    Some(response)
}

/// Horizontal contact displacement on the top of a Y-axis rotary contraption.
/// The host supplies a grounded body's feet center and horizontal half extents,
/// then clips the returned movement through the world collision resolver.
pub fn rotary_top_motion(
    anchor: Pos,
    [previous_angle, current_angle]: [f32; 2],
    full_blocks: impl IntoIterator<Item = Pos>,
    feet: [f64; 3],
    [half_x, half_z]: [f64; 2],
) -> Option<[f64; 3]> {
    let previous = Rotation::new(anchor, Axis::Y, previous_angle)?;
    let current = Rotation::new(anchor, Axis::Y, current_angle)?;
    let local = previous.to_local(feet);
    let [cos, _, neg_sin] = previous.rotate_vector([1.0, 0.0, 0.0]);
    let (cos, sin) = (cos.abs(), neg_sin.abs());
    let player_local_x = cos * half_x + sin * half_z;
    let player_local_z = sin * half_x + cos * half_z;
    let block_world_radius = (cos + sin) * 0.5;
    let supported = full_blocks.into_iter().any(|[x, y, z]| {
        // ContraptionCollider inflates the entity's collision bounds by 1e-7.
        if (local[1] - f64::from(y) - 1.0).abs() > CONTACT_EPSILON {
            return false;
        }
        // Check both pairs of separating axes in the horizontal plane.
        let x = local[0] - f64::from(x) - 0.5;
        let z = local[2] - f64::from(z) - 0.5;
        if x.abs() >= 0.5 + player_local_x || z.abs() >= 0.5 + player_local_z {
            return false;
        }
        let [x, _, z] = previous.rotate_vector([x, 0.0, z]);
        x.abs() < block_world_radius + half_x && z.abs() < block_world_radius + half_z
    });
    if !supported {
        return None;
    }

    // AbstractContraptionEntity.getContactPointMotion maps the previous local
    // point into the current transform. ContraptionCollider applies its X/Z
    // displacement after clipping it against the world's normal collision shapes.
    let next = current.to_world(local);
    Some([next[0] - feet[0], 0.0, next[2] - feet[2]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_motion(actual: Option<[f64; 3]>, expected: [f64; 3]) {
        let actual = actual.expect("body must be supported");
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1.0e-12,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn a_leading_face_separates_bodies_on_both_sides() {
        let start = [0.0; 3];
        let east = linear_face_separation(
            start,
            [0.25, 0.0, 0.0],
            [[0; 3]],
            [1.125, 0.0, 0.2],
            [1.725, 1.675, 0.8],
        )
        .unwrap();
        let west = linear_face_separation(
            start,
            [-0.25, 0.0, 0.0],
            [[0; 3]],
            [-0.725, 0.0, 0.2],
            [-0.125, 1.675, 0.8],
        )
        .unwrap();
        assert_eq!(east, [0.125 + CONTACT_EPSILON, 0.0, 0.0]);
        assert_eq!(west, [-0.125 - CONTACT_EPSILON, 0.0, 0.0]);
        for (min, max) in [
            ([1.26, 0.0, 0.2], [1.86, 1.675, 0.8]),
            ([1.10, 1.0, 0.2], [1.70, 2.675, 0.8]),
            ([-0.5, 0.0, 0.2], [0.1, 1.675, 0.8]),
        ] {
            assert!(linear_face_separation(start, [0.25, 0.0, 0.0], [[0; 3]], min, max).is_none());
        }
    }

    #[test]
    fn rider_moves_around_the_center_of_the_anchor_block() {
        let anchor = [10, 64, -3];
        assert_motion(
            rotary_top_motion(
                anchor,
                [0.0, 90.0],
                [[2, 0, 0]],
                [12.5, 65.0, -2.5],
                [0.3; 2],
            ),
            [-2.0, 0.0, -2.0],
        );
        assert_motion(
            rotary_top_motion(anchor, [0.0, 90.0], [[0; 3]], [10.5, 65.0, -2.5], [0.3; 2]),
            [0.0; 3],
        );
    }

    #[test]
    fn rotation_wrap_keeps_contact_motion_continuous() {
        let (sin, cos) = 10_f64.to_radians().sin_cos();
        assert_motion(
            rotary_top_motion(
                [0; 3],
                [350.0, 10.0],
                [[2, 0, 0]],
                [0.5 + 2.0 * cos, 1.0, 0.5 + 2.0 * sin],
                [0.3; 2],
            ),
            [0.0, 0.0, -4.0 * sin],
        );
    }

    #[test]
    fn only_a_footprint_over_the_actual_top_is_supported() {
        let motion = |feet| rotary_top_motion([0; 3], [45.0, 55.0], [[0; 3]], feet, [0.025; 2]);
        assert!(motion([0.68, 1.0, 0.68]).is_some());
        assert!(motion([1.10, 1.0, 1.10]).is_none());
        assert!(motion([0.68, 0.0, 0.68]).is_none());
        assert!(motion([0.68, 1.25, 0.68]).is_none());
        // A rider can stand with the center of their feet outside the block.
        assert!(
            rotary_top_motion([0; 3], [0.0, 1.0], [[0; 3]], [1.25, 1.0, 0.5], [0.3; 2]).is_some()
        );
    }

    #[test]
    fn rider_follows_horizontal_stroke_and_upward_lift() {
        let previous = [4.25, 16.0, -2.0];
        let feet = [4.45, 17.0, -1.5];
        let motion = |current| linear_top_motion(previous, current, [[0; 3]], feet, [0.3; 2]);
        assert_motion(motion([4.0, 16.0, -2.0]), [-0.25, 0.0, 0.0]);
        assert_motion(motion([4.25, 16.25, -2.0]), [0.0, 0.25, 0.0]);
        assert_motion(motion([4.25, 15.75, -2.0]), [0.0; 3]);
        assert!(
            linear_top_motion(
                previous,
                [4.5, 16.0, -2.0],
                [[0; 3]],
                [3.9, 17.0, -1.5],
                [0.3; 2]
            )
            .is_none()
        );
    }
}
