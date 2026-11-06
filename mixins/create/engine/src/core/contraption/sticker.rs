// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use crate::placement::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub facing: Direction,
    pub powered: bool,
    pub extended: bool,
}

impl State {
    /// `StickerBlock.neighborChanged` latches on each rising edge.
    pub fn with_signal(self, powered: bool) -> Self {
        Self {
            powered,
            extended: self.extended ^ (powered && !self.powered),
            ..self
        }
    }

    pub fn supported_face(self, side: Direction) -> bool {
        self.extended || side != self.facing
    }

    /// `Contraption.moveBlock` only follows the outgoing front of the latch.
    pub fn attachment(self) -> Option<Direction> {
        self.extended.then_some(self.facing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_power_is_not_an_edge_and_only_rising_edges_toggle() {
        let mut state = State {
            facing: Direction::Down,
            powered: true,
            extended: false,
        };
        for (signal, extended) in [
            (true, false),
            (false, false),
            (true, true),
            (true, true),
            (false, true),
            (true, false),
        ] {
            state = state.with_signal(signal);
            assert_eq!((state.powered, state.extended), (signal, extended));
        }
    }

    #[test]
    fn only_the_extended_head_is_adhesive_and_the_closed_face_is_unsupported() {
        for facing in Direction::ALL {
            for extended in [false, true] {
                let state = State {
                    facing,
                    powered: false,
                    extended,
                };
                assert_eq!(state.attachment(), extended.then_some(facing));
                for side in Direction::ALL {
                    assert_eq!(state.supported_face(side), side != facing || extended);
                }
            }
        }
    }
}
