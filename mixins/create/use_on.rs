fn create_use_on(
    game: &crate::game::Game<'_>,
    stack: &crate::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
) {
    crate::game::create_port::use_on(game, stack, pos, face);
}
