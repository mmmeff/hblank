//! Framework-neutral catalog, fixture, and control contracts for Hblank adapters.

mod catalog;
mod control;
mod docs;
mod theme;
pub use catalog::{
    ComponentDefinition, ComponentMetadata, FixtureDefinition, FixtureMetadata,
    FixtureRegistrationData, FixtureRegistrationMetadata, RegisteredCatalog, RegistryError,
    assemble_catalog, canonical_source_id,
};
pub use control::{
    ControlDefinition, ControlError, ControlKind, ControlValue, HblankControlAdapter, HblankEnum,
    HblankProps, NumberConstraints, TextMode,
};
pub use docs::{CalloutTone, DocBlock, DocPage};
pub use theme::{ResolvedTheme, ThemeMode};

#[doc(hidden)]
pub use control::ControlField;
