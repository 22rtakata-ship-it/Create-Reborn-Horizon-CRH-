// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Adapted from PressingBehaviour's cycle and getRunningTickSpeed; world/belt/basin
// integration is deliberately outside this game-independent state machine.
pub const CYCLE: i32 = 240;

pub fn tick_speed(rpm: f32) -> i32 {
    if rpm == 0.0 || !rpm.is_finite() {
        return 0;
    }
    (1.0 + (rpm.abs() / 512.0).clamp(0.0, 1.0) * 59.0) as i32
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PressCycle {
    pub running: bool,
    pub ticks: i32,
    pub applied: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Started,
    Press,
    Finished,
}

impl PressCycle {
    pub fn tick(&mut self, rpm: f32, input_ready: bool) -> Action {
        let step = tick_speed(rpm);
        if step == 0 {
            return Action::None;
        }
        if !self.running {
            if !input_ready {
                return Action::None;
            }
            *self = Self {
                running: true,
                ticks: 0,
                applied: false,
            };
            return Action::Started;
        }
        if self.ticks > CYCLE {
            self.running = false;
            return Action::Finished;
        }
        let action = if self.ticks == CYCLE / 2 && !self.applied {
            self.applied = true;
            Action::Press
        } else {
            Action::None
        };
        let before = self.ticks;
        self.ticks += step;
        if before < CYCLE / 2 && self.ticks >= CYCLE / 2 {
            self.ticks = CYCLE / 2;
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_speeds_follow_create_lerp_and_integer_truncation() {
        for (rpm, expected) in [
            (0.0, 0),
            (16.0, 2),
            (32.0, 4),
            (-64.0, 8),
            (128.0, 15),
            (256.0, 30),
            (512.0, 60),
            (1024.0, 60),
        ] {
            assert_eq!(tick_speed(rpm), expected);
        }
    }

    #[test]
    fn processing_occurs_once_at_the_bottom_even_when_a_tick_would_skip_it() {
        let mut cycle = PressCycle::default();
        assert_eq!(cycle.tick(128.0, true), Action::Started);
        let mut presses = 0;
        for _ in 0..40 {
            match cycle.tick(128.0, false) {
                Action::Press => presses += 1,
                Action::Finished => break,
                _ => {}
            }
        }
        assert_eq!(presses, 1);
        assert!(!cycle.running);
    }

    #[test]
    fn losing_power_pauses_the_cycle_without_consuming_another_input() {
        let mut cycle = PressCycle {
            running: true,
            ticks: 120,
            applied: false,
        };
        assert_eq!(cycle.tick(0.0, true), Action::None);
        assert_eq!(cycle.ticks, 120);
        assert_eq!(cycle.tick(16.0, true), Action::Press);
        assert_eq!(cycle.tick(16.0, true), Action::None);
    }
}
