fn create_gauge_has_analog(
    game: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    output: &mut bool,
) {
    if matches!(
        game.facts.get(state).name.as_ref(),
        "create:speedometer" | "create:stressometer"
    ) {
        *output = true;
    }
}
