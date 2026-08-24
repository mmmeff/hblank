//! Isolated GPUI component development.

#[cfg(all(feature = "crates-io-gpui", feature = "zed-gpui"))]
compile_error!("enable exactly one GPUI backend feature");
#[cfg(not(any(feature = "crates-io-gpui", feature = "zed-gpui")))]
compile_error!("enable exactly one GPUI backend feature");

mod fixture;
pub mod harness;

pub use fixture::{
    ComponentDefinition, ComponentRegistration, FixtureDefinition, FixtureRegistration,
    RegisteredCatalog, RenderComponent, registered_catalog, render_fixture,
};
#[cfg(feature = "crates-io-gpui")]
pub use gpui_crates_io as gpui;
#[cfg(feature = "zed-gpui")]
pub use gpui_zed as gpui;
pub use harness::run_harness;
pub use hblank_core::{
    ComponentMetadata, ControlDefinition, ControlError, ControlKind, ControlValue, FixtureMetadata,
    FixtureRegistrationData, FixtureRegistrationMetadata, HblankEnum, HblankProps, RegistryError,
    canonical_source_id,
};
pub use hblank_macros::{HblankEnum, HblankProps, component, fixture};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use hblank_core::ControlField;
    pub use inventory;
}
