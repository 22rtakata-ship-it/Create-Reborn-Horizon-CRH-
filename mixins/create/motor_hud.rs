fn motor_hud(game: &mut crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    crate::game::create_port_motor_control::hud(game, batch);
}
