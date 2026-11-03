// Create's SharedDepotBlockMethods.onUse returns SUCCESS on the client too.
// End the click after its normal UseItemOn packet, before item prediction or
// the other hand can send another exchange. Inventories remain server-owned.
fn create_depot_prediction(
    game: &crate::game::Game<'_>,
    hit: &crate::game::Hit,
) -> std::ops::ControlFlow<(InteractionResult, DryUse)> {
    if crate::game::create_port::depot_reacts(game, hit.pos, hit.face) {
        std::ops::ControlFlow::Break((InteractionResult::Success, DryUse::default()))
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
