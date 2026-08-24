use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    env,
    path::Path,
};

use crate::gpui::{AnyElement, App, Window};
use thiserror::Error;

use crate::{ControlError, ControlValue, HblankProps};

pub type RenderComponent = fn(&dyn HblankProps, &mut Window, &mut App) -> AnyElement;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentMetadata {
    pub id: String,
    pub title: &'static str,
    pub group: &'static str,
    pub docs: &'static str,
    pub source: &'static str,
    pub line: u32,
}

pub struct ComponentDefinition {
    metadata: ComponentMetadata,
    props_type: TypeId,
    render: RenderComponent,
}

impl ComponentDefinition {
    #[doc(hidden)]
    #[must_use]
    pub fn new<Props: HblankProps>(metadata: ComponentMetadata, render: RenderComponent) -> Self {
        Self {
            metadata,
            props_type: TypeId::of::<Props>(),
            render,
        }
    }

    #[must_use]
    pub const fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureMetadata {
    pub id: String,
    pub component_id: String,
    pub component_title: &'static str,
    pub group: &'static str,
    pub title: &'static str,
    pub docs: &'static str,
    pub source: &'static str,
    pub line: u32,
}

#[doc(hidden)]
pub struct FixtureRegistrationMetadata {
    pub id: String,
    pub title: &'static str,
    pub docs: &'static str,
    pub source: &'static str,
    pub line: u32,
}

pub struct FixtureDefinition {
    metadata: FixtureMetadata,
    defaults: Box<dyn HblankProps>,
    props: Box<dyn HblankProps>,
    render: RenderComponent,
}

impl FixtureDefinition {
    fn new(
        metadata: FixtureMetadata,
        props: Box<dyn HblankProps>,
        render: RenderComponent,
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

pub struct RegisteredCatalog {
    components: Vec<ComponentDefinition>,
    fixtures: Vec<FixtureDefinition>,
}

impl RegisteredCatalog {
    #[must_use]
    pub fn components(&self) -> &[ComponentDefinition] {
        &self.components
    }

    #[must_use]
    pub fn fixtures(&self) -> &[FixtureDefinition] {
        &self.fixtures
    }

    #[must_use]
    pub fn into_parts(self) -> (Vec<ComponentDefinition>, Vec<FixtureDefinition>) {
        (self.components, self.fixtures)
    }
}

#[doc(hidden)]
pub struct ComponentRegistration {
    pub build: fn() -> ComponentDefinition,
}

inventory::collect!(ComponentRegistration);

#[doc(hidden)]
pub struct FixtureRegistrationData {
    metadata: FixtureRegistrationMetadata,
    component_id: String,
    defaults: Box<dyn HblankProps>,
}

impl FixtureRegistrationData {
    #[doc(hidden)]
    #[must_use]
    pub fn new(
        metadata: FixtureRegistrationMetadata,
        component_id: String,
        defaults: Box<dyn HblankProps>,
    ) -> Self {
        Self {
            metadata,
            component_id,
            defaults,
        }
    }
}

#[doc(hidden)]
pub struct FixtureRegistration {
    pub build: fn() -> FixtureRegistrationData,
}

inventory::collect!(FixtureRegistration);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("multiple components use the id '{0}'")]
    DuplicateComponentId(String),
    #[error("multiple fixtures use the id '{0}'")]
    DuplicateFixtureId(String),
    #[error("fixture '{fixture}' references unknown component '{component}'")]
    UnknownComponent { fixture: String, component: String },
    #[error("fixture '{fixture}' uses props that do not match component '{component}'")]
    PropsTypeMismatch { fixture: String, component: String },
}

/// Builds all linked components and fixture variants in deterministic hierarchy order.
///
/// # Errors
/// Returns an error for duplicate identifiers, unknown components, or mismatched props types.
pub fn registered_catalog() -> Result<RegisteredCatalog, RegistryError> {
    let mut components = inventory::iter::<ComponentRegistration>
        .into_iter()
        .map(|registration| (registration.build)())
        .collect::<Vec<_>>();
    components.sort_by(|left, right| {
        let left = left.metadata();
        let right = right.metadata();
        (left.group, left.title, left.id.as_str()).cmp(&(
            right.group,
            right.title,
            right.id.as_str(),
        ))
    });

    let mut component_ids = HashSet::with_capacity(components.len());
    for component in &components {
        if !component_ids.insert(component.metadata.id.as_str()) {
            return Err(RegistryError::DuplicateComponentId(
                component.metadata.id.clone(),
            ));
        }
    }
    let component_indexes = components
        .iter()
        .enumerate()
        .map(|(index, component)| (component.metadata.id.as_str(), index))
        .collect::<HashMap<_, _>>();

    let registrations = inventory::iter::<FixtureRegistration>
        .into_iter()
        .map(|registration| (registration.build)())
        .collect::<Vec<_>>();
    let mut fixture_ids = HashSet::with_capacity(registrations.len());
    let mut fixtures = Vec::with_capacity(registrations.len());
    for registration in registrations {
        if !fixture_ids.insert(registration.metadata.id.clone()) {
            return Err(RegistryError::DuplicateFixtureId(registration.metadata.id));
        }
        let Some(&component_index) = component_indexes.get(registration.component_id.as_str())
        else {
            return Err(RegistryError::UnknownComponent {
                fixture: registration.metadata.id,
                component: registration.component_id,
            });
        };
        let component = &components[component_index];
        if registration.defaults.as_any().type_id() != component.props_type {
            return Err(RegistryError::PropsTypeMismatch {
                fixture: registration.metadata.id,
                component: registration.component_id,
            });
        }
        fixtures.push(FixtureDefinition::new(
            FixtureMetadata {
                id: registration.metadata.id,
                component_id: component.metadata.id.clone(),
                component_title: component.metadata.title,
                group: component.metadata.group,
                title: registration.metadata.title,
                docs: registration.metadata.docs,
                source: registration.metadata.source,
                line: registration.metadata.line,
            },
            registration.defaults,
            component.render,
        ));
    }
    fixtures.sort_by(|left, right| {
        let left = left.metadata();
        let right = right.metadata();
        (
            left.group,
            left.component_title,
            left.title,
            left.id.as_str(),
        )
            .cmp(&(
                right.group,
                right.component_title,
                right.title,
                right.id.as_str(),
            ))
    });

    Ok(RegisteredCatalog {
        components,
        fixtures,
    })
}

#[doc(hidden)]
#[must_use]
pub fn canonical_source_id(source: &str, symbol: &str) -> String {
    let source = Path::new(source);
    let project_root = env::var_os("HBLANK_PROJECT_ROOT")
        .map(std::path::PathBuf::from)
        .or_else(|| env::current_dir().ok());
    let relative = project_root
        .as_deref()
        .and_then(|root| source.strip_prefix(root).ok())
        .unwrap_or(source);
    let portable = relative.to_string_lossy().replace('\\', "/");
    format!("{portable}#{symbol}")
}
