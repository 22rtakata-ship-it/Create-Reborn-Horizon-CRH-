fn motor_level(game: &mut crate::game::Game<'_>) {
    crate::game::create_port_motor_control::level_tick(game);
    crate::game::create_port_instruments::flush(game);
}
