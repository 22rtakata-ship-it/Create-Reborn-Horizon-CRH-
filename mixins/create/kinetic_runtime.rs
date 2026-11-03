// SPDX-License-Identifier: MIT
// Per-server-tick connected-component cache. Reading only loaded blocks never
// tickets chunks; the two halves of an open clutch get separate cache entries.
use super::Game;
use create_engine::kinetics::{self, Network, Node, Pos};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
#[derive(Default)]
struct Cache {
    world: Option<(usize, i64, String)>,
    components: BTreeMap<(Pos, u8), Rc<Network>>,
}
thread_local! { static CACHE: RefCell<Cache> = RefCell::new(Cache::default()); }
pub(super) fn invalidate() {
    CACHE.with(|c| c.borrow_mut().components.clear());
}
pub(super) fn network(game: &Game<'_>, start: Node, half: u8) -> Rc<Network> {
    CACHE.with(|cell| {
        let mut cache = cell.borrow_mut();
        let world = (
            game as *const Game<'_> as usize,
            game.world.game_time,
            game.dim.id().to_owned(),
        );
        if cache.world.as_ref() != Some(&world) {
            cache.world = Some(world);
            cache.components.clear();
        }
        if let Some(network) = cache.components.get(&(start.pos, half)) {
            return Rc::clone(network);
        }
        let network = Rc::new(kinetics::solve_half(start, half, |pos| {
            if pos == start.pos {
                Some(start)
            } else {
                crate::game::create_port::node(game, pos)
            }
        }));
        for key in &network.vertices {
            cache.components.insert(*key, Rc::clone(&network));
        }
        network
    })
}
