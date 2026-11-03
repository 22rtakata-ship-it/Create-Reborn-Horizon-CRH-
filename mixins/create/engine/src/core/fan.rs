// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
//! Source: IAirCurrentSource, AirCurrent, FanProcessing, EncasedFanRenderer.
use crate::placement::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessingType {
    Blasting,
    Haunting,
    Smoking,
    Splashing,
}
impl ProcessingType {
    pub fn id(self) -> &'static str {
        match self {
            Self::Blasting => "create:blasting",
            Self::Haunting => "create:haunting",
            Self::Smoking => "create:smoking",
            Self::Splashing => "create:splashing",
        }
    }
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "create:blasting" | "BLASTING" => Some(Self::Blasting),
            "create:haunting" | "HAUNTING" => Some(Self::Haunting),
            "create:smoking" | "SMOKING" => Some(Self::Smoking),
            "create:splashing" | "SPLASHING" => Some(Self::Splashing),
            _ => None,
        }
    }
}
pub fn max_distance(rpm: f32) -> f32 {
    3.0 + (rpm.abs() / 256.0).min(1.0) * 17.0
}
pub fn pushing(rpm: f32, facing: Direction) -> bool {
    rpm * if facing.vector().iter().any(|v| *v < 0) {
        -1.0
    } else {
        1.0
    } > 0.0
}
pub fn propeller_speed(rpm: f32) -> f32 {
    let speed = rpm * 5.0;
    if speed > 0.0 {
        speed.clamp(80.0, 1280.0)
    } else if speed < 0.0 {
        speed.clamp(-1280.0, -80.0)
    } else {
        0.0
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Processing {
    pub kind: Option<ProcessingType>,
    pub time: i32,
}
impl Processing {
    fn duration(count: u32) -> i32 {
        150 * ((count.saturating_sub(1) / 16) + 1) as i32 + 1
    }
    pub fn dropped(&mut self, kind: ProcessingType, count: u32, can_process: bool) -> bool {
        if self.kind == Some(kind) {
            if self.time == -1 || self.time < -1 && !can_process {
                return false;
            }
        } else {
            if !can_process {
                return false;
            }
            self.kind = Some(kind);
            self.time = Self::duration(count);
        }
        self.time -= 1;
        self.time == 0
    }
    pub fn transported(&mut self, kind: ProcessingType, count: u32, can_process: bool) -> bool {
        if self.kind != Some(kind) {
            self.kind = Some(kind);
            self.time = if can_process {
                Self::duration(count)
            } else {
                -1
            };
            return false;
        }
        if self.time == -1 {
            return false;
        }
        let before = self.time;
        self.time -= 1;
        before <= 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub end: i32,
    pub kind: Option<ProcessingType>,
}
pub fn segments(
    distance: f32,
    push: bool,
    mut read: impl FnMut(i32) -> Option<ProcessingType>,
) -> Vec<Segment> {
    let limit = distance.ceil() as i32;
    let (start, end, step, offset) = if push {
        (1, limit, 1, -1)
    } else {
        (limit, 1, -1, 0)
    };
    let mut result = Vec::new();
    let mut kind = None;
    let mut current = None;
    let mut i = start;
    while i * step <= end * step {
        if let Some(next) = read(i) {
            kind = Some(next);
        }
        if current.is_some_and(|previous| previous != kind) {
            result.push(Segment {
                end: i + offset,
                kind: current.unwrap(),
            });
        }
        current = Some(kind);
        i += step;
    }
    if let Some(kind) = current {
        result.push(Segment {
            end: end + step + offset,
            kind,
        });
    }
    result
}
pub fn type_at(
    distance: f32,
    push: bool,
    segments: &[Segment],
    offset: f32,
) -> Option<ProcessingType> {
    if !(0.0..=distance).contains(&offset) {
        return None;
    }
    segments
        .iter()
        .find(|s| {
            if push {
                offset <= s.end as f32
            } else {
                offset >= s.end as f32
            }
        })
        .and_then(|s| s.kind)
}
/// VoxelShape.min/max samples in the two transverse axes (AxisCycle order).
pub fn shape_depth(boxes: &[[f64; 6]], facing: Direction) -> f64 {
    let axis = facing.axis().index();
    let a = (axis + 1) % 3;
    let b = (axis + 2) % 3;
    let positive = facing.vector()[axis] > 0;
    let mut maximum = 0.0_f64;
    for [x, y] in [
        [0.25, 0.25],
        [0.25, 0.75],
        [0.5, 0.5],
        [0.75, 0.25],
        [0.75, 0.75],
    ] {
        let depth = boxes
            .iter()
            .filter(|v| x >= v[a] && x < v[a + 3] && y >= v[b] && y < v[b + 3])
            .map(|v| if positive { v[axis] } else { 1.0 - v[axis + 3] })
            .fold(f64::INFINITY, f64::min);
        if !depth.is_finite() {
            return depth;
        }
        maximum = maximum.max(depth);
    }
    maximum
}
pub fn accelerated(
    velocity: [f64; 3],
    position: [f64; 3],
    origin: [f64; 3],
    flow: [i32; 3],
    rpm: f32,
    distance: f32,
    sneaking: bool,
) -> [f64; 3] {
    let separation = position
        .iter()
        .zip(origin)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    let acceleration = (f64::from(rpm.abs())
        / if sneaking { 4096.0 } else { 512.0 }
        / (separation / f64::from(distance))) as f32;
    std::array::from_fn(|i| {
        velocity[i]
            + (f64::from(flow[i]) * f64::from(acceleration) - velocity[i]).clamp(-5.0, 5.0) / 8.0
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn media_order_and_inclusive_boundaries_reverse_with_flow() {
        use ProcessingType::*;
        let read = |i| {
            if i == 2 {
                Some(Splashing)
            } else if i == 4 {
                Some(Smoking)
            } else {
                None
            }
        };
        let s = segments(6.0, true, read);
        assert_eq!(type_at(6.0, true, &s, 1.0), None);
        assert_eq!(type_at(6.0, true, &s, 1.01), Some(Splashing));
        assert_eq!(type_at(6.0, true, &s, 3.0), Some(Splashing));
        assert_eq!(type_at(6.0, true, &s, 6.0), Some(Smoking));
        let s = segments(6.0, false, read);
        assert_eq!(type_at(6.0, false, &s, 4.0), None);
        assert_eq!(type_at(6.0, false, &s, 3.99), Some(Smoking));
        assert_eq!(type_at(6.0, false, &s, 2.0), Some(Smoking));
        assert_eq!(type_at(6.0, false, &s, 1.99), Some(Splashing));
    }
    #[test]
    fn partial_obstacles_and_any_clear_sample_match_shape_depth() {
        assert_eq!(
            shape_depth(&[[0.0, 0.0, 0.0, 1.0, 0.5, 1.0]], Direction::Down),
            0.5
        );
        assert!(shape_depth(&[[0.0, 0.0, 0.0, 0.1, 1.0, 1.0]], Direction::South).is_infinite());
        assert_eq!(shape_depth(&[[0.0; 6]], Direction::Up), f64::INFINITY);
    }
    #[test]
    fn stack_threshold_and_transported_extra_ticks() {
        for count in [1, 16, 17, 32, 33, 48, 49, 64] {
            let mut a = Processing::default();
            let mut b = Processing::default();
            let dropped = (1..1000)
                .find(|_| a.dropped(ProcessingType::Splashing, count, true))
                .unwrap();
            let held = (1..1000)
                .find(|_| b.transported(ProcessingType::Splashing, count, true))
                .unwrap();
            assert_eq!(dropped, 150 * ((count - 1) / 16 + 1) + 1);
            assert_eq!(held, dropped + 2);
        }
    }
}
