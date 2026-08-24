use std::fmt::Write as _;

use crate::gpui::{AnyElement, App, Window};

use crate::HblankProps;

pub type RenderComponent = fn(&dyn HblankProps, &mut Window, &mut App) -> AnyElement;
pub type ComponentDefinition = hblank_core::ComponentDefinition<RenderComponent>;
pub type FixtureDefinition = hblank_core::FixtureDefinition<RenderComponent>;
pub type RegisteredCatalog = hblank_core::RegisteredCatalog<RenderComponent>;

#[doc(hidden)]
pub struct ComponentRegistration {
    pub build: fn() -> ComponentDefinition,
}

inventory::collect!(ComponentRegistration);

#[doc(hidden)]
pub struct FixtureRegistration {
    pub build: fn() -> hblank_core::FixtureRegistrationData,
}

inventory::collect!(FixtureRegistration);

/// Builds the GPUI adapter's linked components and variants in deterministic hierarchy order.
///
/// # Errors
/// Returns an error for duplicate identifiers, unknown components, or mismatched props types.
pub fn registered_catalog() -> Result<RegisteredCatalog, hblank_core::RegistryError> {
    let components = inventory::iter::<ComponentRegistration>
        .into_iter()
        .map(|registration| (registration.build)())
        .collect();
    let fixtures = inventory::iter::<FixtureRegistration>
        .into_iter()
        .map(|registration| (registration.build)())
        .collect();
    hblank_core::assemble_catalog(components, fixtures)
}

#[must_use]
pub fn render_fixture(
    fixture: &FixtureDefinition,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    (fixture.renderer())(fixture.props(), window, cx)
}

/// Returns the registered catalog as stable tab-separated records for CLI discovery.
///
/// # Errors
/// Returns registry validation errors before producing any records.
pub fn registered_catalog_listing() -> Result<String, hblank_core::RegistryError> {
    let catalog = registered_catalog()?;
    let mut output = String::new();
    for component in catalog.components() {
        let metadata = component.metadata();
        writeln!(
            output,
            "component\t{}\t{}\t{}",
            metadata.id, metadata.title, metadata.group,
        )
        .expect("writing to String cannot fail");
    }
    for fixture in catalog.fixtures() {
        let metadata = fixture.metadata();
        writeln!(
            output,
            "fixture\t{}\t{}\t{}",
            metadata.id, metadata.component_id, metadata.title,
        )
        .expect("writing to String cannot fail");
    }
    Ok(output)
}
