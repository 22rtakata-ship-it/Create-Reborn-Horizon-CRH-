fn create_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::create_port_placement::destroy(game, pos, canceled);
}
