// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

fn create_right_click(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    // Chassis useItemOn handles eligible sticky faces before the glue item
    // starts selecting a volume.
    if crate::game::create_port_chassis::right_click(game, pos, face) {
        game.swing_arm();
        *canceled = true;
        return;
    }
    if crate::game::create_port_super_glue::right_click(game, pos) {
        *canceled = true;
        return;
    }
    crate::game::create_port::right_click(game, pos, face, canceled);
}
