// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

fn contraption_collision(game: &crate::game::Game<'_>, mut result: Vec<Aabb>) -> Vec<Aabb> {
    horizon_client_render::create_contraption_scene::append_collisions(
        &game.world,
        game.collision,
        &mut result,
        None,
        None,
    );
    result
}
