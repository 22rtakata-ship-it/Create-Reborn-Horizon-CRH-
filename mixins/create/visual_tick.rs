fn create_visual_tick(game: &mut crate::game::Game<'_>) {
    crate::game::create_port_presentation::tick(game);
    crate::game::create_port_millstone::client_tick(game);
}
