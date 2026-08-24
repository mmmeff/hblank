//! Isolated GPUI component development.

#[cfg(all(feature = "crates-io-gpui", feature = "zed-gpui"))]
compile_error!("enable exactly one GPUI backend feature");
#[cfg(not(any(feature = "crates-io-gpui", feature = "zed-gpui")))]
compile_error!("enable exactly one GPUI backend feature");

mod control;
mod fixture;
pub mod harness;

pub use control::{
    ControlDefinition, ControlError, ControlKind, ControlValue, HblankEnum, HblankProps,
};
pub use fixture::{
    ComponentDefinition, ComponentMetadata, ComponentRegistration, FixtureDefinition,
    FixtureMetadata, FixtureRegistration, FixtureRegistrationData, FixtureRegistrationMetadata,
    RegisteredCatalog, RegistryError, RenderComponent, canonical_source_id, registered_catalog,
};
#[cfg(feature = "crates-io-gpui")]
pub use gpui_crates_io as gpui;
#[cfg(feature = "zed-gpui")]
pub use gpui_zed as gpui;
pub use harness::run_harness;
pub use hblank_macros::{HblankEnum, HblankProps, component, fixture};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use crate::control::ControlField;
    pub use inventory;
}
