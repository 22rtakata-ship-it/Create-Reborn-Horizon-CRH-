//! Game-independent create-contraptions simulation, extracted from the preserved native bundle.
pub use create_core::{gantry_transmission, kinetics, placement, sail, source_trig};
#[path = "../../../mixins/create/engine/src/core/contraption/mod.rs"]
pub mod contraption;
