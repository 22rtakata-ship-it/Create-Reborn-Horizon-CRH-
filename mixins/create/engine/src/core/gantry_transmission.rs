// SPDX-License-Identifier: MIT
// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
//! Shared gantry transmission geometry; independent of mobile structures.
use crate::{kinetics::Axis, placement::Direction};
pub fn pinion_ratio(shaft: Direction, carriage: Direction) -> Option<f32> {
    pinion_axis(carriage, shaft)?;
    let axis = shaft.axis();
    let sign = shaft.vector()[axis.index()] as f32;
    let invert = matches!(
        (axis, carriage),
        (Axis::X, Direction::Down | Direction::South)
            | (Axis::Y, Direction::North | Direction::East)
            | (Axis::Z, Direction::Up | Direction::West)
    );
    Some(if invert { -sign } else { sign })
}

/// The carriage's output axis is perpendicular to its mounting face and rail.
pub fn pinion_axis(carriage_facing: Direction, shaft_facing: Direction) -> Option<Axis> {
    let facing = carriage_facing.axis();
    let rail = shaft_facing.axis();
    if facing == rail {
        return None;
    }
    [Axis::X, Axis::Y, Axis::Z]
        .into_iter()
        .find(|&axis| axis != facing && axis != rail)
}
