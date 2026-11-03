// Copyright (c) The Create Team / The Creators of Create
// SPDX-License-Identifier: MIT
// DepotItemHandler.insertItem: the ordinary depot does not merge incoming
// automation stacks, nor accept a new input while its output buffer is occupied.
fn create_depot_can_place(
    containers: &crate::game::machines::Containers<'_>,
    view: &crate::game::machines::View,
    slot: usize,
) -> std::ops::ControlFlow<bool> {
    let Some((pos, local)) = view.locate(slot) else {
        return std::ops::ControlFlow::Continue(());
    };
    if let Some(crate::containers::BlockEntity::Mod(entity)) =
        containers.world.block_entities.get(&pos)
    {
        if entity.id == "create:depot" {
            return std::ops::ControlFlow::Break(
                local == 0 && entity.items.iter().all(Option::is_none),
            );
        }
    }
    std::ops::ControlFlow::Continue(())
}
