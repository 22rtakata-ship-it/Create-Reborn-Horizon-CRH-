// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

//! Item cutting in an upward-facing stationary Mechanical Saw.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CuttingCycle {
    pub remaining: f32,
    pub duration: f32,
    pub applied: bool,
    pub recipe_index: usize,
}

impl Default for CuttingCycle {
    fn default() -> Self {
        Self {
            remaining: -1.0,
            duration: 0.0,
            applied: false,
            recipe_index: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Idle,
    Start,
    Advance,
    Apply,
    Export,
}

impl CuttingCycle {
    /// Duration for each eligible recipe in the world recipe order. Vanilla
    /// stonecutting uses 50, while Create cutting uses its declared duration.
    pub fn start(&mut self, input_count: u32, recipe_durations: &[u32]) {
        let duration = if recipe_durations.is_empty() {
            10
        } else {
            self.recipe_index = if self.recipe_index >= recipe_durations.len() - 1 {
                0
            } else {
                self.recipe_index + 1
            };
            recipe_durations[self.recipe_index].saturating_mul((input_count / 5).max(1))
        };
        self.duration = duration as f32;
        self.remaining = self.duration;
        self.applied = false;
    }

    /// The recipe list can change between insertion and the application tick.
    pub fn select_current(&mut self, candidates: usize) -> Option<usize> {
        if candidates == 0 {
            return None;
        }
        if self.recipe_index >= candidates {
            self.recipe_index = 0;
        }
        Some(self.recipe_index)
    }

    pub fn tick(&mut self, rpm: f32, has_input: bool) -> Action {
        if rpm == 0.0 || !rpm.is_finite() {
            return Action::Idle;
        }
        if self.remaining == -1.0 {
            return if has_input && !self.applied {
                Action::Start
            } else {
                Action::Idle
            };
        }
        self.remaining -= (rpm.abs() / 24.0).clamp(1.0, 128.0);
        if self.remaining < 5.0 && !self.applied {
            return Action::Apply;
        }
        if self.remaining > 0.0 {
            return Action::Advance;
        }
        self.remaining = 0.0;
        Action::Export
    }

    /// Call after the host commits recipe results. An unmatched stack is
    /// carried through the same output delay and then ejected unchanged.
    pub fn finish_application(&mut self) {
        self.applied = true;
        self.duration = 20.0;
        self.remaining = self.duration;
    }

    pub fn finish_export(&mut self) {
        *self = Self {
            recipe_index: self.recipe_index,
            ..Self::default()
        };
    }
}

/// Unit vector toward the output edge from SawBlockEntity.getItemMovementVec.
pub fn item_motion(axis_along_first: bool, rpm: f32) -> [i32; 3] {
    let sign = if rpm < 0.0 { -1 } else { 1 };
    if axis_along_first {
        [0, 0, -sign]
    } else {
        [sign, 0, 0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_oak_log_and_unmatched_item_boundaries_at_128_rpm() {
        for (durations, apply_tick, export_tick) in [(&[50][..], 9, 13), (&[][..], 1, 5)] {
            let mut cycle = CuttingCycle::default();
            assert_eq!(cycle.tick(128.0, true), Action::Start);
            cycle.start(1, durations);
            for tick in 1..=export_tick {
                let action = cycle.tick(128.0, true);
                match tick {
                    tick if tick == apply_tick => {
                        assert_eq!(action, Action::Apply);
                        cycle.finish_application();
                        assert_eq!(cycle.remaining, 20.0);
                    }
                    tick if tick == export_tick => {
                        assert_eq!(action, Action::Export);
                        cycle.finish_export();
                        assert_eq!(cycle.remaining, -1.0);
                    }
                    _ => assert_eq!(action, Action::Advance),
                }
            }
        }
    }

    #[test]
    fn cycles_candidates_rechecks_changed_recipes_and_scales_bulk() {
        let mut cycle = CuttingCycle::default();
        cycle.start(16, &[200, 50, 10]);
        assert_eq!(cycle.select_current(3), Some(1));
        assert_eq!(cycle.duration, 150.0);
        cycle.start(1, &[200, 50, 10]);
        assert_eq!(cycle.select_current(1), Some(0));
        assert_eq!(cycle.duration, 10.0);
        let before = cycle;
        assert_eq!(cycle.tick(0.0, true), Action::Idle);
        assert_eq!(cycle, before);
    }

    #[test]
    fn output_direction_reverses_on_both_axes() {
        assert_eq!(item_motion(false, 128.0), [1, 0, 0]);
        assert_eq!(item_motion(false, -128.0), [-1, 0, 0]);
        assert_eq!(item_motion(true, 128.0), [0, 0, -1]);
        assert_eq!(item_motion(true, -128.0), [0, 0, 1]);
    }
}
