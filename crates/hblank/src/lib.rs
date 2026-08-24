//! Isolated GPUI component development.

mod control;
mod fixture;
pub mod harness;

pub use control::{
    ControlDefinition, ControlError, ControlKind, ControlValue, HblankEnum, HblankProps,
};
pub use fixture::{
    FixtureDefinition, FixtureMetadata, FixtureRegistration, RegistryError, RenderFixture,
    registered_fixtures,
};
pub use gpui;
pub use harness::run_harness;
pub use hblank_macros::{HblankEnum, HblankProps, fixture};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use crate::control::ControlField;
    pub use inventory;
}
