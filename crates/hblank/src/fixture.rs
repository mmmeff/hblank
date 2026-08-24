use std::collections::HashSet;

use crate::gpui::{AnyElement, App, Window};
use thiserror::Error;

use crate::{ControlError, ControlValue, HblankProps};

pub type RenderFixture = fn(&dyn HblankProps, &mut Window, &mut App) -> AnyElement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixtureMetadata {
    pub id: &'static str,
    pub title: &'static str,
    pub group: &'static str,
    pub docs: &'static str,
    pub source: &'static str,
    pub line: u32,
}

pub struct FixtureDefinition {
    metadata: FixtureMetadata,
    defaults: Box<dyn HblankProps>,
    props: Box<dyn HblankProps>,
    render: RenderFixture,
}

impl FixtureDefinition {
    #[must_use]
    pub fn new(
        metadata: FixtureMetadata,
        props: Box<dyn HblankProps>,
        render: RenderFixture,
    ) -> Self {
        Self {
            metadata,
            defaults: props.clone(),
            props,
            render,
        }
    }

    #[must_use]
    pub const fn metadata(&self) -> &FixtureMetadata {
        &self.metadata
    }

    #[must_use]
    pub fn props(&self) -> &dyn HblankProps {
        self.props.as_ref()
    }

    /// Replaces one generated property control value.
    ///
    /// # Errors
    /// Returns an error when the identifier or value is invalid for this fixture's props.
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

pub struct FixtureRegistration {
    pub build: fn() -> FixtureDefinition,
}

inventory::collect!(FixtureRegistration);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("multiple fixtures use the id '{0}'")]
    DuplicateId(&'static str),
}

/// Builds all linked fixtures in deterministic group/title/id order.
///
/// # Errors
/// Returns an error when multiple linked fixtures declare the same stable identifier.
pub fn registered_fixtures() -> Result<Vec<FixtureDefinition>, RegistryError> {
    let mut fixtures = inventory::iter::<FixtureRegistration>
        .into_iter()
        .map(|registration| (registration.build)())
        .collect::<Vec<_>>();
    fixtures.sort_by_key(|fixture| {
        let metadata = fixture.metadata();
        (metadata.group, metadata.title, metadata.id)
    });

    let mut ids = HashSet::with_capacity(fixtures.len());
    for fixture in &fixtures {
        if !ids.insert(fixture.metadata.id) {
            return Err(RegistryError::DuplicateId(fixture.metadata.id));
        }
    }
    Ok(fixtures)
}
