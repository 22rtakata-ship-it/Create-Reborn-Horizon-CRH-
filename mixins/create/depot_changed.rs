// A successful automation transfer ending the exposed stack destroys its
// transported identity immediately, even if another hopper refills this tick.
fn create_depot_changed(game: &mut crate::game::Game<'_>, pos: (i32, i32, i32)) {
    if let Some(crate::containers::BlockEntity::Mod(entity)) =
        game.world.block_entities.get_mut(&pos)
        && entity.id == "create:depot"
        && entity.items.first().is_none_or(Option::is_none)
    {
        entity.nbt.remove(&"HeldItem".into());
    }
}
