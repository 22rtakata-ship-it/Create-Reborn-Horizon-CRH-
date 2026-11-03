fn motor_drag(
    game: &crate::game::Game<'_>,
    screen: &crate::mod_api::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    crate::game::create_port_motor_control::click(game, screen, button, canceled);
}
