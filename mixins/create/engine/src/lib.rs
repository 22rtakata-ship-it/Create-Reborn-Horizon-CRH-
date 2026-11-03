//! Game-independent parts, also compiled as the native bundle's engine crate.
#[path = "core/kinetics.rs"]
pub mod kinetics;
#[path = "core/placement.rs"]
pub mod placement;
#[path = "core/pressing.rs"]
pub mod pressing;

#[path = "core/depot.rs"]
pub mod depot;

#[path = "core/controls.rs"]
pub mod controls;
