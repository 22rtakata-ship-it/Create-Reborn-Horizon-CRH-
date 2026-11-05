// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

//! Read-only structure capture, local coordinates and driver rules.
//! The host owns block writes, saved data, entities, collision and rendering.
mod assembly;
pub mod bearing;
mod transform;

pub use assembly::*;
pub use transform::*;

#[cfg(test)]
mod tests;
