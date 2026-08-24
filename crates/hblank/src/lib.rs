//! Isolated GPUI component development.

#[cfg(all(feature = "crates-io-gpui", feature = "zed-gpui"))]
compile_error!("enable exactly one GPUI backend feature");
#[cfg(not(any(feature = "crates-io-gpui", feature = "zed-gpui")))]
compile_error!("enable exactly one GPUI backend feature");

mod docs;
mod fixture;
pub mod harness;
mod rendered;
#[cfg(feature = "test-support")]
pub mod testing;
mod theme;

pub use docs::{CustomDocBlockRegistration, CustomDocRenderer, DocContext, registered_doc_block};
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
    CalloutTone, ComponentMetadata, ControlDefinition, ControlError, ControlKind, ControlValue,
    DocBlock, DocPage, FixtureMetadata, FixtureRegistrationData, FixtureRegistrationMetadata,
    HblankControlAdapter, HblankEnum, HblankProps, NumberConstraints, RegistryError, ResolvedTheme,
    TextMode, ThemeMode, canonical_source_id,
};
pub use hblank_macros::{
    HblankEnum, HblankProps, component, custom_doc, doc_block, fixture, fixture_ref, render_handle,
    theme_hook,
};
pub use rendered::Rendered;
pub use theme::{ThemeHook, ThemeHookRegistration, registered_theme_hook};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use hblank_core::ControlField;
    pub use inventory;
}
