use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    env,
    path::Path,
};

use thiserror::Error;

use crate::{ControlError, ControlValue, DocPage, HblankProps};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentMetadata {
    pub id: String,
    pub title: &'static str,
    pub group: &'static str,
    pub docs: &'static str,
    pub declaration: &'static str,
    pub source: &'static str,
    pub line: u32,
}

pub struct ComponentDefinition<Renderer> {
    metadata: ComponentMetadata,
    props_type: TypeId,
    renderer: Renderer,
    docs: DocPage,
}

impl<Renderer> ComponentDefinition<Renderer> {
    #[must_use]
    pub fn new<Props: HblankProps>(metadata: ComponentMetadata, renderer: Renderer) -> Self {
        Self {
            metadata,
            props_type: TypeId::of::<Props>(),
            renderer,
            docs: DocPage::default(),
        }
    }

    #[must_use]
    pub const fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[must_use]
    pub const fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    #[must_use]
    pub fn with_docs(mut self, docs: DocPage) -> Self {
        self.docs = docs;
        self
    }

    #[must_use]
    pub const fn docs(&self) -> &DocPage {
        &self.docs
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
    pub declaration: &'static str,
    pub source: &'static str,
    pub line: u32,
}

pub struct FixtureRegistrationMetadata {
    pub id: String,
    pub title: &'static str,
    pub docs: &'static str,
    pub declaration: &'static str,
    pub source: &'static str,
    pub line: u32,
}

pub struct FixtureDefinition<Renderer> {
    metadata: FixtureMetadata,
    defaults: Box<dyn HblankProps>,
    props: Box<dyn HblankProps>,
    renderer: Renderer,
}

impl<Renderer> FixtureDefinition<Renderer> {
    fn new(metadata: FixtureMetadata, props: Box<dyn HblankProps>, renderer: Renderer) -> Self {
        Self {
            metadata,
            defaults: props.clone(),
            props,
            renderer,
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
    #[must_use]
    pub const fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    #[must_use]
    pub fn default_control_value(&self, id: &str) -> Option<ControlValue> {
        self.defaults.control_value(id)
    }

    /// Applies valid control values and ignores stale or invalid entries.
    ///
    /// Returns the number of values accepted by this fixture.
    pub fn apply_control_values<'a>(
        &mut self,
        values: impl IntoIterator<Item = (&'a str, &'a ControlValue)>,
    ) -> usize {
        values
            .into_iter()
            .filter(|(id, value)| self.set_control(id, (*value).clone()).is_ok())
            .count()
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
}

pub struct RegisteredCatalog<Renderer> {
    components: Vec<ComponentDefinition<Renderer>>,
    fixtures: Vec<FixtureDefinition<Renderer>>,
}

impl<Renderer> RegisteredCatalog<Renderer> {
    #[must_use]
    pub fn components(&self) -> &[ComponentDefinition<Renderer>] {
        &self.components
    }

    #[must_use]
    pub fn fixtures(&self) -> &[FixtureDefinition<Renderer>] {
        &self.fixtures
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Vec<ComponentDefinition<Renderer>>,
        Vec<FixtureDefinition<Renderer>>,
    ) {
        (self.components, self.fixtures)
    }
}

pub struct FixtureRegistrationData {
    metadata: FixtureRegistrationMetadata,
    component_id: String,
    defaults: Box<dyn HblankProps>,
}

impl FixtureRegistrationData {
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

/// Joins framework adapter registrations into a deterministic component catalog.
///
/// # Errors
/// Returns an error for duplicate identifiers, unknown components, or mismatched props types.
pub fn assemble_catalog<Renderer: Copy>(
    mut components: Vec<ComponentDefinition<Renderer>>,
    registrations: Vec<FixtureRegistrationData>,
) -> Result<RegisteredCatalog<Renderer>, RegistryError> {
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
                declaration: registration.metadata.declaration,
                source: registration.metadata.source,
                line: registration.metadata.line,
            },
            registration.defaults,
            component.renderer,
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

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;
    use crate::{ControlDefinition, ControlError, ControlKind, ControlValue};

    #[derive(Clone)]
    struct Props {
        active: bool,
    }

    impl HblankProps for Props {
        fn definitions(&self) -> &'static [ControlDefinition] {
            &[ControlDefinition {
                id: "active",
                label: "Active",
                docs: "",
                kind: ControlKind::Boolean,
            }]
        }

        fn control_value(&self, id: &str) -> Option<ControlValue> {
            (id == "active").then_some(ControlValue::Boolean(self.active))
        }

        fn set_control(&mut self, id: &str, value: ControlValue) -> Result<(), ControlError> {
            if id != "active" {
                return Err(ControlError::UnknownControl(id.to_owned()));
            }
            let ControlValue::Boolean(value) = value else {
                return Err(ControlError::TypeMismatch {
                    control: "active",
                    expected: "boolean",
                    actual: value.kind_name(),
                });
            };
            self.active = value;
            Ok(())
        }

        fn clone_box(&self) -> Box<dyn HblankProps> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn component() -> ComponentDefinition<&'static str> {
        ComponentDefinition::new::<Props>(
            ComponentMetadata {
                id: "src/card.rs#card".to_owned(),
                title: "Card",
                group: "Components",
                docs: "A card.",
                declaration: "fn card(...) { ... }",
                source: "src/card.rs",
                line: 10,
            },
            "framework renderer",
        )
    }

    fn fixture(component_id: &str) -> FixtureRegistrationData {
        FixtureRegistrationData::new(
            FixtureRegistrationMetadata {
                id: "src/card.rs#default".to_owned(),
                title: "Default",
                docs: "",
                declaration: "fn default() -> Props { ... }",
                source: "src/card.rs",
                line: 20,
            },
            component_id.to_owned(),
            Box::new(Props { active: false }),
        )
    }

    #[test]
    fn assembles_catalog_without_framework_types() {
        let catalog = assemble_catalog(vec![component()], vec![fixture("src/card.rs#card")])
            .expect("framework-neutral registrations should assemble");

        assert_eq!(catalog.components().len(), 1);
        assert_eq!(catalog.fixtures().len(), 1);
        assert_eq!(catalog.fixtures()[0].renderer(), &"framework renderer");
        assert_eq!(catalog.fixtures()[0].metadata().component_title, "Card");
    }

    #[test]
    fn rejects_unknown_framework_component_references() {
        let Err(error) =
            assemble_catalog::<&str>(Vec::new(), vec![fixture("src/missing.rs#missing")])
        else {
            panic!("unknown component should fail");
        };

        assert_eq!(
            error,
            RegistryError::UnknownComponent {
                fixture: "src/card.rs#default".to_owned(),
                component: "src/missing.rs#missing".to_owned(),
            }
        );
    }

    #[test]
    fn reapplies_only_valid_session_control_values() {
        let catalog = assemble_catalog(vec![component()], vec![fixture("src/card.rs#card")])
            .expect("catalog should assemble");
        let fixture = &mut catalog.into_parts().1.remove(0);
        let values = [
            ("active", ControlValue::Boolean(true)),
            ("stale", ControlValue::Text("ignored".to_owned())),
        ];

        assert_eq!(
            fixture.apply_control_values(values.iter().map(|(id, value)| (*id, value))),
            1
        );
        assert_eq!(
            fixture.props().control_value("active"),
            Some(ControlValue::Boolean(true))
        );
    }

    #[test]
    fn builds_portable_source_symbol_ids() {
        assert_eq!(
            canonical_source_id("src/card.hblank.rs", "default"),
            "src/card.hblank.rs#default"
        );
    }
}
