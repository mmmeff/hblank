//! Isolated GPUI component development.

mod control;
mod example;
pub mod harness;

pub use control::{
    ControlDefinition, ControlError, ControlKind, ControlValue, HblankEnum, HblankProps,
};
pub use example::{
    ExampleDefinition, ExampleMetadata, ExampleRegistration, RegistryError, RenderExample,
    registered_examples,
};
pub use gpui;
pub use harness::run_harness;
pub use hblank_macros::{HblankEnum, HblankProps, example};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use crate::control::ControlField;
    pub use inventory;
}
