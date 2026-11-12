//! Compatibility facade for native simulation types. Engine hooks remain in
//! `mixins/create`; compiling this crate does not validate a Horizon client.
#[cfg(feature = "contraptions")]
pub use create_contraptions::contraption;
pub use create_core::*;
#[cfg(feature = "mechanics")]
pub use create_mechanics::*;
