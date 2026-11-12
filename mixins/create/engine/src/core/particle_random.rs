// SPDX-License-Identifier: MIT
// Copyright (c) The Create Team / The Creators of Create
//! Shared particle random interface, independent of contraptions.
/// Supply the particle's own random, or the level random where explicitly noted.
/// Keeping these separate preserves the source's draw ordering.
pub trait ParticleRandom {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

pub fn offset_randomly(random: &mut impl ParticleRandom, range: f32) -> [f64; 3] {
    std::array::from_fn(|_| f64::from((random.next_float() - 0.5) * 2.0 * range))
}
