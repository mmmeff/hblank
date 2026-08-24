//! Isolated GPUI component development.

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
    RegisteredCatalog, RenderComponent, registered_catalog, registered_catalog_listing,
    render_fixture,
};
pub use gpui;
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
