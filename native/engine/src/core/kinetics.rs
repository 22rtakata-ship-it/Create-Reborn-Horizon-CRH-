// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Rust implementation of the first supported RotationPropagator connection rules
// and KineticNetwork's capacity/stress accounting. See THIRD_PARTY_NOTICES.md.
use std::collections::{BTreeMap, VecDeque};

pub type Pos = [i32; 3];
pub const MAX_NODES: usize = 256;
pub const MAX_RPM: f32 = 256.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Motor {
        facing: Pos,
        rpm: f32,
        capacity: f32,
    },
    Shaft,
    Cog {
        large: bool,
    },
    Press,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub pos: Pos,
    pub axis: Axis,
    pub kind: Kind,
}

impl Node {
    fn shaft_port(self, delta: Pos) -> bool {
        match self.kind {
            Kind::Motor { facing, .. } => delta == facing,
            _ => delta[self.axis.index()].abs() == 1,
        }
    }

    fn impact(self) -> f32 {
        // Default Create stress impact for a mechanical press; shafts/cogs are free.
        if matches!(self.kind, Kind::Press) {
            8.0
        } else {
            0.0
        }
    }
}

/// Target RPM / source RPM, for supported neighboring connections.
/// Perpendicular large-cog connections, belts, gearboxes and encased drives are pending.
pub fn ratio(from: Node, to: Node) -> Option<f32> {
    let delta = std::array::from_fn(|i| to.pos[i] - from.pos[i]);
    let distance: i32 = delta.iter().map(|n| n.abs()).sum();
    if from.axis != to.axis {
        return None;
    }
    let axis = from.axis.index();
    if distance == 1 && delta[axis] != 0 {
        let opposite = delta.map(|n| -n);
        return (from.shaft_port(delta) && to.shaft_port(opposite)).then_some(1.0);
    }
    if delta[axis] != 0 {
        return None;
    }
    match (from.kind, to.kind) {
        (Kind::Cog { large: false }, Kind::Cog { large: false }) if distance == 1 => Some(-1.0),
        (Kind::Cog { large: a }, Kind::Cog { large: b })
            if a != b && distance == 2 && delta.iter().filter(|n| n.abs() == 1).count() == 2 =>
        {
            Some(if a { -2.0 } else { -0.5 })
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,
    Overstressed,
    ConflictingSources,
    InconsistentCycle,
    TooFast,
    TooLarge,
}

#[derive(Clone, Debug)]
pub struct Network {
    pub status: Status,
    pub capacity: f32,
    pub stress: f32,
    pub theoretical: BTreeMap<Pos, f32>,
}

impl Network {
    pub fn speed(&self, pos: Pos) -> f32 {
        if self.status == Status::Running {
            self.theoretical.get(&pos).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.0001 * a.abs().max(b.abs()).max(1.0)
}

/// Discovers only loaded nodes supplied by `read`; never loads world chunks.
/// Ratios are relative to the starting node. Stopping the whole component on a
/// conflict is a conservative prototype policy, not Create's block-breaking rule.
pub fn solve(start: Node, mut read: impl FnMut(Pos) -> Option<Node>) -> Network {
    let mut nodes = BTreeMap::from([(start.pos, start)]);
    let mut factors = BTreeMap::from([(start.pos, 1.0_f32)]);
    let mut queue = VecDeque::from([start]);
    let mut failure = None;
    while let Some(from) = queue.pop_front() {
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let offset: Pos = [x, y, z];
                    let distance: i32 = offset.iter().map(|n| n.abs()).sum();
                    if !(1..=2).contains(&distance) {
                        continue;
                    }
                    let pos = std::array::from_fn(|i| from.pos[i] + offset[i]);
                    let Some(to) = read(pos) else { continue };
                    let Some(ratio) = ratio(from, to) else {
                        continue;
                    };
                    let factor = factors[&from.pos] * ratio;
                    if let Some(old) = factors.get(&pos) {
                        if !close(*old, factor) {
                            failure = Some(Status::InconsistentCycle);
                        }
                    } else {
                        if nodes.len() >= MAX_NODES {
                            failure = Some(Status::TooLarge);
                            queue.clear();
                            break;
                        }
                        nodes.insert(pos, to);
                        factors.insert(pos, factor);
                        queue.push_back(to);
                    }
                }
            }
        }
    }
    let mut base = None;
    let mut capacity = 0.0;
    for node in nodes.values() {
        if let Kind::Motor {
            rpm,
            capacity: per_rpm,
            ..
        } = node.kind
        {
            if !rpm.is_finite() || !per_rpm.is_finite() || per_rpm < 0.0 {
                failure = Some(Status::ConflictingSources);
                continue;
            }
            // A stopped source supplies no rotation and does not oppose another motor.
            if rpm == 0.0 {
                continue;
            }
            let proposed = rpm / factors[&node.pos];
            if base.is_some_and(|old| !close(old, proposed)) {
                failure = Some(Status::ConflictingSources);
            }
            base = Some(proposed);
            capacity += per_rpm * rpm.abs();
        }
    }
    let base = base.unwrap_or(0.0);
    let theoretical: BTreeMap<_, _> = factors
        .iter()
        .map(|(p, factor)| (*p, base * factor))
        .collect();
    let stress = nodes
        .values()
        .map(|n| n.impact() * theoretical[&n.pos].abs())
        .sum();
    let status = failure.unwrap_or_else(|| {
        if theoretical
            .values()
            .any(|rpm| !rpm.is_finite() || rpm.abs() > MAX_RPM)
        {
            Status::TooFast
        } else if base == 0.0 {
            Status::Idle
        } else if stress > capacity {
            Status::Overstressed
        } else {
            Status::Running
        }
    });
    Network {
        status,
        capacity,
        stress,
        theoretical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(pos: Pos, kind: Kind) -> Node {
        Node {
            pos,
            axis: Axis::X,
            kind,
        }
    }
    fn motor(pos: Pos, rpm: f32, capacity: f32) -> Node {
        node(
            pos,
            Kind::Motor {
                facing: [1, 0, 0],
                rpm,
                capacity,
            },
        )
    }
    fn network(nodes: &[Node]) -> Network {
        solve(nodes[0], |p| nodes.iter().copied().find(|n| n.pos == p))
    }

    #[test]
    fn shaft_and_press_share_speed_and_scale_stress_by_absolute_rpm() {
        let n = network(&[
            motor([0, 0, 0], -32.0, 16.0),
            node([1, 0, 0], Kind::Shaft),
            node([2, 0, 0], Kind::Press),
        ]);
        assert_eq!(n.status, Status::Running);
        assert_eq!(n.speed([2, 0, 0]), -32.0);
        assert_eq!((n.capacity, n.stress), (512.0, 256.0));
    }

    #[test]
    fn overstress_stops_the_component_but_keeps_theoretical_rpm() {
        let n = network(&[motor([0, 0, 0], 16.0, 4.0), node([1, 0, 0], Kind::Press)]);
        assert_eq!(n.status, Status::Overstressed);
        assert_eq!(n.speed([1, 0, 0]), 0.0);
        assert_eq!(n.theoretical[&[1, 0, 0]], 16.0);
    }

    #[test]
    fn meshing_small_cogs_reverse_rotation() {
        let cog = Kind::Cog { large: false };
        let n = network(&[
            motor([0, 0, 0], 16.0, 16.0),
            node([1, 0, 0], cog),
            node([1, 1, 0], cog),
            node([2, 1, 0], Kind::Press),
        ]);
        assert_eq!(n.speed([2, 1, 0]), -16.0);
    }

    #[test]
    fn large_to_small_doubles_rpm_and_reverses_it() {
        let n = network(&[
            motor([0, 0, 0], 16.0, 16.0),
            node([1, 0, 0], Kind::Cog { large: true }),
            node([1, 1, 1], Kind::Cog { large: false }),
            node([2, 1, 1], Kind::Press),
        ]);
        assert_eq!(n.speed([2, 1, 1]), -32.0);
        assert_eq!(n.stress, 256.0);
    }

    #[test]
    fn removing_a_shaft_disconnects_the_press() {
        let press = node([2, 0, 0], Kind::Press);
        let n = solve(press, |p| (p == [0, 0, 0]).then(|| motor(p, 16.0, 16.0)));
        assert_eq!(n.status, Status::Idle);
        assert_eq!(n.speed(press.pos), 0.0);
    }

    #[test]
    fn motor_has_only_its_facing_shaft_port() {
        let n = network(&[motor([0, 0, 0], 16.0, 16.0), node([-1, 0, 0], Kind::Shaft)]);
        assert!(!n.theoretical.contains_key(&[-1, 0, 0]));
    }

    #[test]
    fn incompatible_drives_stop_the_connected_component() {
        let mut second = motor([2, 0, 0], 32.0, 16.0);
        second.kind = Kind::Motor {
            facing: [-1, 0, 0],
            rpm: 32.0,
            capacity: 16.0,
        };
        let n = network(&[
            motor([0, 0, 0], 16.0, 16.0),
            node([1, 0, 0], Kind::Shaft),
            second,
        ]);
        assert_eq!(n.status, Status::ConflictingSources);
        assert_eq!(n.speed([1, 0, 0]), 0.0);
    }

    #[test]
    fn traversal_is_bounded_for_long_loaded_networks() {
        let n = solve(motor([0, 0, 0], 16.0, 16.0), |p| {
            (p[0] >= 1 && p[1] == 0 && p[2] == 0).then(|| node(p, Kind::Shaft))
        });
        assert_eq!(n.status, Status::TooLarge);
        assert_eq!(n.theoretical.len(), MAX_NODES);
    }
}
