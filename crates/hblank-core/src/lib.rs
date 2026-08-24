//! Framework-neutral catalog, fixture, and control contracts for Hblank adapters.

mod catalog;
mod control;

pub use catalog::{
    ComponentDefinition, ComponentMetadata, FixtureDefinition, FixtureMetadata,
    FixtureRegistrationData, FixtureRegistrationMetadata, RegisteredCatalog, RegistryError,
    assemble_catalog, canonical_source_id,
};
pub use control::{
    ControlDefinition, ControlError, ControlKind, ControlValue, HblankEnum, HblankProps,
};

#[doc(hidden)]
pub use control::ControlField;
