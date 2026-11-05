// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use glam::{DQuat, DVec3};
use horizon_core::geometry::{Aabb, Vec3};
use horizon_world::{
    attachments::{AttachmentCodec, AttachmentError, AttachmentType},
    block_collision::BlockCollisionRegistry,
    chunk::BlockState,
    nbt::{Compound, Tag},
};
use std::borrow::Cow;

pub type Pos = [i32; 3];
pub const MAX_BLOCKS: usize = 64;
pub const MAX_RECORDS: usize = 4;
pub const KEY: &str = "moving_structures";

#[derive(Clone, Debug)]
pub struct Record {
    pub controller: Pos,
    pub anchor: Pos,
    pub axis: usize,
    pub angle: f32,
    pub previous_angle: f32,
    pub rpm: f32,
    pub return_home: bool,
    pub stroke: Option<Stroke>,
    pub clock_hand: Option<ClockHand>,
    pub blocks: Vec<(Pos, BlockState)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClockHand {
    Hour,
    Minute,
}

#[derive(Clone, Copy, Debug)]
pub struct Stroke {
    pub kind: StrokeKind,
    pub initial: u32,
    pub limit: u32,
    pub offset: f32,
    pub previous: f32,
    pub waiting: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeKind {
    Piston,
    RopePulley,
    Gantry,
}

impl Record {
    pub fn stroke_kind(&self) -> Option<StrokeKind> {
        self.stroke.map(|stroke| stroke.kind)
    }

    pub fn origin_at(&self, partial: f32) -> [f64; 3] {
        let Some(stroke) = self.stroke else {
            return self.anchor.map(f64::from);
        };
        let distance = stroke.previous + (stroke.offset - stroke.previous) * partial;
        let mut origin = self.anchor.map(f64::from);
        let sign = match stroke.kind {
            StrokeKind::Gantry => 1,
            _ => self.anchor[self.axis] - self.controller[self.axis],
        };
        origin[self.axis] += f64::from(sign) * f64::from(distance);
        origin
    }

    pub fn orientation(&self, angle: f32) -> DQuat {
        let axis = [DVec3::X, DVec3::Y, DVec3::Z][self.axis];
        DQuat::from_axis_angle(axis, f64::from(angle).to_radians())
    }

    pub fn angle_at(&self, partial: f32) -> f32 {
        let delta = (self.angle - self.previous_angle + 180.0).rem_euclid(360.0) - 180.0;
        self.previous_angle + partial * delta
    }
}

pub const RECORDS: AttachmentType<Vec<Record>> = AttachmentType {
    namespace: "create",
    name: KEY,
    default: Vec::new,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: false,
    sync: true,
};

pub const ACTIVE: AttachmentType<bool> = AttachmentType {
    namespace: "create",
    name: "has_moving_structures",
    default: || false,
    codec: None,
    copy_on_death: false,
    sync: false,
};

fn encode(records: &Vec<Record>) -> Result<Compound, AttachmentError> {
    if records.len() > MAX_RECORDS {
        return Err(AttachmentError::Encode);
    }
    let entries = records
        .iter()
        .map(|record| {
            let blocks = record
                .blocks
                .iter()
                .map(|(local, state)| {
                    Tag::Compound(Compound::from([
                        ("Pos".into(), Tag::IntArray(local.to_vec())),
                        ("State".into(), state.to_nbt()),
                    ]))
                })
                .collect();
            let mut entry = Compound::from([
                (
                    "Controller".into(),
                    Tag::IntArray(record.controller.to_vec()),
                ),
                ("Anchor".into(), Tag::IntArray(record.anchor.to_vec())),
                ("Axis".into(), Tag::Int(record.axis as i32)),
                ("Angle".into(), Tag::Float(record.angle.to_bits())),
                (
                    "PreviousAngle".into(),
                    Tag::Float(record.previous_angle.to_bits()),
                ),
                ("Speed".into(), Tag::Float(record.rpm.to_bits())),
                ("ReturnHome".into(), Tag::Byte(i8::from(record.return_home))),
                ("Blocks".into(), Tag::List(blocks)),
            ]);
            if let Some(stroke) = record.stroke {
                entry.insert(
                    "Stroke".into(),
                    Tag::Compound(Compound::from([
                        (
                            "Kind".into(),
                            Tag::Byte(match stroke.kind {
                                StrokeKind::Piston => 0,
                                StrokeKind::RopePulley => 1,
                                StrokeKind::Gantry => 2,
                            }),
                        ),
                        ("Initial".into(), Tag::Int(stroke.initial as i32)),
                        ("Limit".into(), Tag::Int(stroke.limit as i32)),
                        ("Offset".into(), Tag::Float(stroke.offset.to_bits())),
                        ("Previous".into(), Tag::Float(stroke.previous.to_bits())),
                        ("Waiting".into(), Tag::Byte(i8::from(stroke.waiting))),
                    ])),
                );
            }
            if let Some(hand) = record.clock_hand {
                let value = match hand {
                    ClockHand::Hour => 0,
                    ClockHand::Minute => 1,
                };
                entry.insert("ClockHand".into(), Tag::Byte(value));
            }
            Tag::Compound(entry)
        })
        .collect();
    Ok(Compound::from([
        ("Version".into(), Tag::Int(1)),
        ("Entries".into(), Tag::List(entries)),
    ]))
}

fn decode(encoded: &Compound) -> Result<Vec<Record>, AttachmentError> {
    let invalid = AttachmentError::Decode;
    if encoded.get(&"Version".into()) != Some(&Tag::Int(1)) {
        return Err(invalid);
    }
    let Some(Tag::List(entries)) = encoded.get(&"Entries".into()) else {
        return Err(invalid);
    };
    if entries.len() > MAX_RECORDS {
        return Err(invalid);
    }
    let position = |entry: &Tag, name: &str| match entry.get(name) {
        Some(Tag::IntArray(values)) => <Pos>::try_from(values.as_slice()).map_err(|_| invalid),
        _ => Err(invalid),
    };
    let float = |entry: &Tag, name: &str| match entry.get(name) {
        Some(Tag::Float(bits)) if f32::from_bits(*bits).is_finite() => Ok(f32::from_bits(*bits)),
        _ => Err(invalid),
    };
    let mut owners = std::collections::BTreeSet::new();
    let records = entries
        .iter()
        .map(|entry| {
            let controller = position(entry, "Controller")?;
            let anchor = position(entry, "Anchor")?;
            let Some(Tag::Int(axis @ 0..=2)) = entry.get("Axis") else {
                return Err(invalid);
            };
            let Some(Tag::List(blocks)) = entry.get("Blocks") else {
                return Err(invalid);
            };
            let clock_hand = match entry.get("ClockHand") {
                None => None,
                Some(Tag::Byte(0)) => Some(ClockHand::Hour),
                Some(Tag::Byte(1)) => Some(ClockHand::Minute),
                _ => return Err(invalid),
            };
            let axis = *axis as usize;
            let distance = anchor[axis].abs_diff(controller[axis]);
            if blocks.len() > MAX_BLOCKS
                || !owners.insert((controller, clock_hand))
                || (0..3).any(|i| i != axis && controller[i] != anchor[i])
            {
                return Err(invalid);
            }
            let stroke = if let Some(tag) = entry.get("Stroke") {
                let kind = match tag.get("Kind") {
                    None | Some(Tag::Byte(0)) => StrokeKind::Piston,
                    Some(Tag::Byte(1)) => StrokeKind::RopePulley,
                    Some(Tag::Byte(2)) => StrokeKind::Gantry,
                    _ => return Err(invalid),
                };
                let max = if kind == StrokeKind::RopePulley {
                    384
                } else {
                    64
                };
                let Some(Tag::Int(initial)) = tag.get("Initial") else {
                    return Err(invalid);
                };
                let Some(Tag::Int(limit)) = tag.get("Limit") else {
                    return Err(invalid);
                };
                let offset = float(tag, "Offset")?;
                let previous = float(tag, "Previous")?;
                if !(1..=max).contains(limit)
                    || !(0..=*limit).contains(initial)
                    || !(0.0..=*limit as f32).contains(&offset)
                    || !(0.0..=*limit as f32).contains(&previous)
                {
                    return Err(invalid);
                }
                Some(Stroke {
                    kind,
                    initial: *initial as u32,
                    limit: *limit as u32,
                    offset,
                    previous,
                    waiting: matches!(tag.get("Waiting"), Some(Tag::Byte(1))),
                })
            } else {
                None
            };
            let return_home = matches!(entry.get("ReturnHome"), Some(Tag::Byte(1)));
            let angle = float(entry, "Angle")?;
            let previous_angle = float(entry, "PreviousAngle")?;
            let pulley = stroke.is_some_and(|stroke| stroke.kind == StrokeKind::RopePulley);
            let gantry = stroke.filter(|stroke| stroke.kind == StrokeKind::Gantry);
            let right_distance = match (gantry, clock_hand) {
                (Some(gantry), _) => controller[axis] >= anchor[axis] && distance == gantry.initial,
                (_, Some(ClockHand::Minute)) => (2..=16).contains(&distance),
                _ => distance == 1,
            };
            if clock_hand.is_some() && (stroke.is_some() || !return_home) {
                return Err(invalid);
            }
            if !right_distance
                || (blocks.is_empty() && !pulley)
                || (pulley
                    && (axis != 1
                        || anchor[1] >= controller[1]
                        || angle != 0.0
                        || previous_angle != 0.0
                        || return_home))
                || (gantry.is_some() && (angle != 0.0 || previous_angle != 0.0 || return_home))
            {
                return Err(invalid);
            }
            let mut locals = std::collections::BTreeSet::new();
            let blocks: Vec<(Pos, BlockState)> = blocks
                .iter()
                .map(|block| {
                    let local = position(block, "Pos")?;
                    if local
                        .iter()
                        .any(|v| v.unsigned_abs() > if stroke.is_some() { 81 } else { 15 })
                        || !locals.insert(local)
                    {
                        return Err(invalid);
                    }
                    let state = BlockState::from_nbt(block.get("State").ok_or(invalid)?)
                        .map_err(|_| invalid)?;
                    Ok((local, state))
                })
                .collect::<Result<_, _>>()?;
            if gantry.is_some()
                && !blocks.iter().any(|(local, state)| {
                    *local == [0; 3]
                        && state.name == "create:gantry_carriage"
                        && state
                            .properties
                            .get("facing")
                            .is_some_and(|facing| match axis {
                                0 => matches!(facing.as_str(), "up" | "down" | "north" | "south"),
                                1 => matches!(facing.as_str(), "east" | "west" | "north" | "south"),
                                _ => matches!(facing.as_str(), "up" | "down" | "east" | "west"),
                            })
                })
            {
                return Err(invalid);
            }
            Ok(Record {
                controller,
                anchor,
                axis,
                angle,
                previous_angle,
                rpm: float(entry, "Speed")?,
                return_home,
                stroke,
                clock_hand,
                blocks,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for record in &records {
        let Some(hand) = record.clock_hand else {
            continue;
        };
        if owners.contains(&(record.controller, None)) {
            return Err(invalid);
        }
        if hand == ClockHand::Minute {
            let Some(hour) = records.iter().find(|candidate| {
                candidate.controller == record.controller
                    && candidate.clock_hand == Some(ClockHand::Hour)
            }) else {
                return Err(invalid);
            };
            let axis = record.axis;
            if hour.axis != axis
                || (hour.anchor[axis] > hour.controller[axis])
                    != (record.anchor[axis] > record.controller[axis])
            {
                return Err(invalid);
            }
        }
    }
    Ok(records)
}

pub fn read(chunk: &crate::chunk::Chunk) -> Result<Cow<'_, [Record]>, AttachmentError> {
    use horizon_world::attachments::create_contraption_attachment::{decoded, encoded};
    let Some(saved) = &chunk.saved else {
        return Ok(Cow::Borrowed(&[]));
    };
    if let Some(records) = decoded(&saved.attachments, &RECORDS) {
        return Ok(Cow::Borrowed(records));
    }
    let Some(value) = encoded(&saved.attachments, "create", KEY) else {
        return if saved.attachments.has(&RECORDS) {
            Err(AttachmentError::Decode)
        } else {
            Ok(Cow::Borrowed(&[]))
        };
    };
    decode(value).map(Cow::Owned)
}

fn vector(v: [f64; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}

fn collision_rotation(angle: f32) -> [f64; 2] {
    let angle = f64::from(angle).rem_euclid(360.0);
    match angle {
        0.0 => [1.0, 0.0],
        90.0 => [0.0, 1.0],
        180.0 => [-1.0, 0.0],
        270.0 => [0.0, -1.0],
        _ => {
            let (sin, cos) = angle.to_radians().sin_cos();
            [cos, sin]
        }
    }
}

fn rotate_plane([u, v]: [usize; 2], [cos, sin]: [f64; 2], mut point: [f64; 3]) -> [f64; 3] {
    [point[u], point[v]] = [
        cos * point[u] - sin * point[v],
        sin * point[u] + cos * point[v],
    ];
    point
}

/// Minecraft's movement solver takes axis-aligned shapes. Slice the rotated
/// rectangle in its rotation plane, keeping the exact span along the pivot axis.
/// All full cubes in the same contraption share these center-relative boxes.
fn block_cross_section([u, v]: [usize; 2], [cos, sin]: [f64; 2], bounds: Aabb) -> Vec<Aabb> {
    let min = bounds.min_position().components();
    let max = bounds.max_position().components();
    if cos == 0.0 || sin == 0.0 {
        return vec![Aabb::new(
            vector(rotate_plane([u, v], [cos, sin], min)),
            vector(rotate_plane([u, v], [cos, sin], max)),
        )];
    }
    let points = [
        [min[u], min[v]],
        [max[u], min[v]],
        [max[u], max[v]],
        [min[u], max[v]],
    ]
    .map(|[x, y]| [cos * x - sin * y, sin * x + cos * y]);
    let start = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let end = points
        .iter()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max);
    const STRIPS: i32 = 8;
    let boundary = |index| start + (end - start) * f64::from(index) / f64::from(STRIPS);
    let mut strips = Vec::with_capacity(STRIPS as usize);
    for index in 0..STRIPS {
        let lo = boundary(index);
        let hi = if index + 1 == STRIPS {
            end
        } else {
            boundary(index + 1)
        };
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        let mut include = |y: f64| {
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        };
        for edge in 0..4 {
            let [x, y] = points[edge];
            let [next_x, next_y] = points[(edge + 1) % 4];
            if (lo..=hi).contains(&x) {
                include(y);
            }
            for boundary in [lo, hi] {
                if boundary > x.min(next_x) && boundary < x.max(next_x) {
                    include(y + (next_y - y) * (boundary - x) / (next_x - x));
                }
            }
        }
        let mut strip_min = min;
        let mut strip_max = max;
        strip_min[u] = lo;
        strip_max[u] = hi;
        strip_min[v] = y_min;
        strip_max[v] = y_max;
        strips.push(Aabb::new(vector(strip_min), vector(strip_max)));
    }
    strips
}

/// Vanilla payload states admitted by capture fill a cube except honey.
pub fn full_cube_payload(state: &BlockState) -> bool {
    state.name.starts_with("minecraft:") && state.name != "minecraft:honey_block"
}

pub fn append_collisions(
    world: &crate::world::World,
    binding: &BlockCollisionRegistry<'_>,
    obstacles: &mut Vec<Aabb>,
) {
    if !world.attachments.has(&ACTIVE) {
        return;
    }
    let mut lookup = crate::collisions::LookupCollisions {
        block: &|_, _, _| world.air,
        binding,
    };
    let lower = Vec3::new(-0.5, -0.5, -0.5);
    let cube = Aabb::unit_cube(lower);
    for chunk in world.chunks.values() {
        let Ok(records) = read(chunk) else { continue };
        for record in records.iter() {
            let plane = [(record.axis + 1) % 3, (record.axis + 2) % 3];
            let rotation = collision_rotation(record.angle);
            let full_cube = block_cross_section(plane, rotation, cube);
            let origin = vector(record.origin_at(1.0).map(|component| component + 0.5));
            for (local, state) in &record.blocks {
                let center =
                    origin.add_vector(vector(rotate_plane(plane, rotation, local.map(f64::from))));
                // All vanilla blocks admitted by capture currently fill a cube.
                if full_cube_payload(state) {
                    obstacles.extend(full_cube.iter().map(|aabb| aabb.move_by(center)));
                    continue;
                }
                // The captured Harvester and piston parts have static shapes
                // keyed by block state. Resolve their registered boxes before
                // moving them. World-dependent shapes need a local block view.
                let Some(id) = binding.registry().id(state) else {
                    continue;
                };
                let Ok(shape) = binding.collision_shape(&mut lookup, id, *local) else {
                    continue;
                };
                for aabb in shape.boxes() {
                    obstacles.extend(
                        block_cross_section(plane, rotation, aabb.move_by(lower))
                            .into_iter()
                            .map(|aabb| aabb.move_by(center)),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    #[test]
    fn clockwork_codec_keeps_independent_hands_with_one_controller() {
        let hand = |clock_hand, distance, angle| Record {
            controller: [1, 2, 3],
            anchor: [1 + distance, 2, 3],
            axis: 0,
            angle,
            previous_angle: angle,
            rpm: 32.0,
            return_home: true,
            stroke: None,
            clock_hand: Some(clock_hand),
            blocks: vec![(
                [0; 3],
                BlockState {
                    name: "minecraft:stone".into(),
                    properties: Default::default(),
                },
            )],
        };
        let hour = hand(ClockHand::Hour, 1, 15.0);
        let minute = hand(ClockHand::Minute, 3, 45.0);
        let saved = encode(&vec![hour.clone(), minute.clone()]).unwrap();
        let restored = decode(&saved).unwrap();
        assert_eq!(restored[0].clock_hand, Some(ClockHand::Hour));
        assert_eq!(restored[1].clock_hand, Some(ClockHand::Minute));
        assert_eq!(restored[1].anchor, [4, 2, 3]);
        assert_eq!(restored[1].angle, 45.0);
        assert!(decode(&encode(&vec![minute]).unwrap()).is_err());
        let opposite = hand(ClockHand::Minute, -3, 45.0);
        assert!(decode(&encode(&vec![hour.clone(), opposite]).unwrap()).is_err());
        assert!(decode(&encode(&vec![hour.clone(), hour]).unwrap()).is_err());
    }

    #[test]
    fn quarter_turns_use_an_exact_cube_for_every_axis() {
        let cube = Aabb::unit_cube(Vec3::new(-0.5, -0.5, -0.5));
        for axis in 0..3 {
            let axes = [(axis + 1) % 3, (axis + 2) % 3];
            for angle in [-450.0, -90.0, 0.0, 90.0, 180.0, 270.0, 360.0] {
                assert_eq!(
                    block_cross_section(axes, collision_rotation(angle), cube),
                    [cube]
                );
            }
        }
    }

    #[test]
    fn strips_cover_rotated_cube_and_leave_far_corners_empty() {
        let [cos, sin] = collision_rotation(45.0);
        let cube = Aabb::unit_cube(Vec3::new(-0.5, -0.5, -0.5));
        let strips = block_cross_section([0, 2], [cos, sin], cube);
        for x in [-0.49, -0.25, 0.0, 0.25, 0.49] {
            for z in [-0.49, -0.25, 0.0, 0.25, 0.49] {
                let point = Vec3::new(cos * x - sin * z, 0.0, sin * x + cos * z);
                assert!(strips.iter().any(|strip| strip.contains(point)));
            }
        }
        let corner = Vec3::new(0.6, 0.0, 0.6);
        assert!(!strips.iter().any(|strip| strip.contains(corner)));
    }

    #[test]
    fn off_center_part_rotates_without_inflating_into_a_full_block() {
        let south_harvester = Aabb::new(
            Vec3::new(-0.5, -0.375, -0.5),
            Vec3::new(0.5, 0.375, -0.3125),
        );
        let east_harvester = Aabb::new(
            Vec3::new(-0.5, -0.375, -0.5),
            Vec3::new(-0.3125, 0.375, 0.5),
        );
        assert_eq!(
            block_cross_section([2, 0], collision_rotation(90.0), south_harvester),
            [east_harvester]
        );
        let rotation = collision_rotation(33.0);
        let boxes = block_cross_section([2, 0], rotation, south_harvester);
        let mid = vector(rotate_plane([2, 0], rotation, [0.0, 0.0, -0.4]));
        assert!(boxes.iter().any(|box_| box_.contains(mid)));
        assert!(!boxes.iter().any(|box_| box_.contains(Vec3::ZERO)));
    }

    #[test]
    fn piston_collision_origin_uses_the_full_current_stroke() {
        let mut record = Record {
            controller: [3, 60, 2],
            anchor: [3, 59, 2],
            axis: 1,
            angle: 0.0,
            previous_angle: 0.0,
            rpm: 8.0,
            return_home: false,
            clock_hand: None,
            stroke: Some(Stroke {
                kind: StrokeKind::Piston,
                initial: 0,
                limit: 4,
                offset: 1.75,
                previous: 1.5,
                waiting: false,
            }),
            blocks: Vec::new(),
        };
        assert_eq!(record.origin_at(0.0), [3.0, 57.5, 2.0]);
        assert_eq!(record.origin_at(1.0), [3.0, 57.25, 2.0]);
        record.anchor[1] = 61;
        assert_eq!(record.origin_at(1.0), [3.0, 62.75, 2.0]);
    }

    #[test]
    fn rope_only_pulley_persists_a_long_downward_stroke() {
        let mut record = Record {
            controller: [3, 150, 2],
            anchor: [3, 149, 2],
            axis: 1,
            angle: 0.0,
            previous_angle: 0.0,
            rpm: 16.0,
            return_home: false,
            clock_hand: None,
            blocks: Vec::new(),
            stroke: Some(Stroke {
                kind: StrokeKind::RopePulley,
                initial: 6,
                limit: 210,
                offset: 191.5,
                previous: 191.0,
                waiting: false,
            }),
        };
        let decoded = decode(&encode(&vec![record.clone()]).unwrap()).unwrap();
        assert_eq!(decoded[0].stroke_kind(), Some(StrokeKind::RopePulley));
        assert_eq!(decoded[0].origin_at(1.0), [3.0, -42.5, 2.0]);
        assert!(decoded[0].blocks.is_empty());
        record.anchor[1] = 151;
        assert!(decode(&encode(&vec![record]).unwrap()).is_err());
    }
}
