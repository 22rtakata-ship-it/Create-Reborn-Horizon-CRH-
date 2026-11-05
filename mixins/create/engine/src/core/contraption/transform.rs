// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use crate::{
    kinetics::{Axis, Pos},
    placement::Direction,
};

/// The centre of the local origin block is the pivot. Points are expressed
/// relative to its lower corner, as in ControlledContraptionEntity.
#[derive(Clone, Copy, Debug)]
pub struct Rotation {
    anchor: Pos,
    turn: QuarterTurn,
    sin: f64,
    cos: f64,
}

impl Rotation {
    pub fn new(anchor: Pos, axis: Axis, degrees: f32) -> Option<Self> {
        if !degrees.is_finite() {
            return None;
        }
        let (sin, cos) = f64::from(degrees).to_radians().sin_cos();
        let steps = java_round(degrees / 90.0).rem_euclid(4.0) as u8;
        Some(Self {
            anchor,
            turn: QuarterTurn { axis, steps },
            sin,
            cos,
        })
    }

    pub fn to_world(self, local: [f64; 3]) -> [f64; 3] {
        let rotated = self.rotate_vector(local.map(|coordinate| coordinate - 0.5));
        std::array::from_fn(|i| rotated[i] + 0.5 + f64::from(self.anchor[i]))
    }

    pub fn to_local(self, world: [f64; 3]) -> [f64; 3] {
        let relative = std::array::from_fn(|i| world[i] - f64::from(self.anchor[i]) - 0.5);
        self.rotate(relative, -self.sin)
            .map(|coordinate| coordinate + 0.5)
    }

    pub fn rotate_vector(self, vector: [f64; 3]) -> [f64; 3] {
        self.rotate(vector, self.sin)
    }

    fn rotate(self, [x, y, z]: [f64; 3], sin: f64) -> [f64; 3] {
        let cos = self.cos;
        match self.turn.axis {
            Axis::X => [x, y * cos - z * sin, y * sin + z * cos],
            Axis::Y => [x * cos + z * sin, y, -x * sin + z * cos],
            Axis::Z => [x * cos - y * sin, x * sin + y * cos, z],
        }
    }

    pub fn snapped(self) -> QuarterTurn {
        self.turn
    }
}

/// StructureTransform uses Java's rounding towards positive infinity at ties.
pub(super) fn java_round(value: f32) -> f64 {
    (f64::from(value) + 0.5).floor()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuarterTurn {
    axis: Axis,
    steps: u8,
}

impl QuarterTurn {
    fn rotate(self, mut position: [i64; 3]) -> [i64; 3] {
        for _ in 0..self.steps {
            let [x, y, z] = position;
            position = match self.axis {
                Axis::X => [x, -z, y],
                Axis::Y => [z, y, -x],
                Axis::Z => [-y, x, z],
            };
        }
        position
    }

    pub fn place(self, anchor: Pos, local: Pos) -> Option<Pos> {
        let rotated = self.rotate(local.map(i64::from));
        Some([
            i32::try_from(i64::from(anchor[0]) + rotated[0]).ok()?,
            i32::try_from(i64::from(anchor[1]) + rotated[1]).ok()?,
            i32::try_from(i64::from(anchor[2]) + rotated[2]).ok()?,
        ])
    }

    pub fn facing(self, direction: Direction) -> Direction {
        let vector = self.rotate(direction.vector().map(i64::from));
        Direction::from_vector(vector.map(|coordinate| coordinate as i32))
            .expect("quarter turns preserve cardinal unit vectors")
    }

    pub fn axis(self, axis: Axis) -> Axis {
        self.facing(match axis {
            Axis::X => Direction::East,
            Axis::Y => Direction::Up,
            Axis::Z => Direction::South,
        })
        .axis()
    }
}
