// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

fn transmission_neighbor(
    level: &crate::game::GameLevel<'_>,
    dimension: horizon_world::level::Dimension,
    pos: horizon_core::position::BlockPos,
    state: horizon_world::registry::BlockStateId,
) {
    use crate::game::create_port_kinetic_runtime::{transmission_block, transmission_changed};
    if level
        .native_state_block(state)
        .is_some_and(transmission_block)
    {
        use horizon_world::level::Dimension;
        let id = match dimension {
            Dimension::Overworld => "minecraft:overworld",
            Dimension::Nether => "minecraft:the_nether",
            Dimension::End => "minecraft:the_end",
        };
        transmission_changed(id, [pos.x, pos.y, pos.z]);
    }
}
