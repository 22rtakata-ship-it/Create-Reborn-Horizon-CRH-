fn create_placed(game: &mut crate::game::Game<'_>, stack: &crate::items::ItemStack, pos: [i32; 3]) {
    crate::game::create_port::placed(game, stack, pos);
}
