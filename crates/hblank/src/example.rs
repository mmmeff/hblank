use std::collections::HashSet;

use gpui::{AnyElement, App, Window};
use thiserror::Error;

use crate::{ControlError, ControlValue, HblankProps};

pub type RenderExample = fn(&dyn HblankProps, &mut Window, &mut App) -> AnyElement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExampleMetadata {
    pub id: &'static str,
    pub title: &'static str,
    pub group: &'static str,
    pub docs: &'static str,
    pub source: &'static str,
    pub line: u32,
}

pub struct ExampleDefinition {
    metadata: ExampleMetadata,
    defaults: Box<dyn HblankProps>,
    props: Box<dyn HblankProps>,
    render: RenderExample,
}

impl ExampleDefinition {
    #[must_use]
    pub fn new(
        metadata: ExampleMetadata,
        props: Box<dyn HblankProps>,
        render: RenderExample,
    ) -> Self {
        Self {
            metadata,
            defaults: props.clone(),
            props,
            render,
        }
    }

    #[must_use]
    pub const fn metadata(&self) -> &ExampleMetadata {
        &self.metadata
    }

    #[must_use]
    pub fn props(&self) -> &dyn HblankProps {
        self.props.as_ref()
    }

    /// Replaces one generated property control value.
    ///
    /// # Errors
    /// Returns an error when the identifier or value is invalid for this example's props.
    pub fn set_control(&mut self, id: &str, value: ControlValue) -> Result<(), ControlError> {
        self.props.set_control(id, value)
    }

    pub fn reset(&mut self) {
        self.props = self.defaults.clone();
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        (self.render)(self.props.as_ref(), window, cx)
    }
}

pub struct ExampleRegistration {
    pub build: fn() -> ExampleDefinition,
}

inventory::collect!(ExampleRegistration);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("multiple examples use the id '{0}'")]
    DuplicateId(&'static str),
}

/// Builds all linked examples in deterministic group/title/id order.
///
/// # Errors
/// Returns an error when multiple linked examples declare the same stable identifier.
pub fn registered_examples() -> Result<Vec<ExampleDefinition>, RegistryError> {
    let mut examples = inventory::iter::<ExampleRegistration>
        .into_iter()
        .map(|registration| (registration.build)())
        .collect::<Vec<_>>();
    examples.sort_by_key(|example| {
        let metadata = example.metadata();
        (metadata.group, metadata.title, metadata.id)
    });

    let mut ids = HashSet::with_capacity(examples.len());
    for example in &examples {
        if !ids.insert(example.metadata.id) {
            return Err(RegistryError::DuplicateId(example.metadata.id));
        }
    }
    Ok(examples)
}
