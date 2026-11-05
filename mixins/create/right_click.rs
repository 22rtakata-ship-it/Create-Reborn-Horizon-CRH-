// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

fn create_right_click(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    if crate::game::create_port_super_glue::right_click(game, pos) {
        *canceled = true;
        return;
    }
    crate::game::create_port::right_click(game, pos, face, canceled);
}
