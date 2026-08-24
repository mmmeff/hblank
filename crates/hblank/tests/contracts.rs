use hblank::gpui::{App, IntoElement, ParentElement, Window, div};
use hblank::{
    ControlError, ControlKind, ControlValue, HblankEnum, HblankProps, component, fixture,
    registered_catalog,
};

#[derive(Clone, Default, HblankEnum)]
enum Tone {
    #[default]
    Neutral,
    #[hblank(label = "High contrast")]
    HighContrast,
}

#[derive(Clone, Default, HblankProps)]
struct DemoProps {
    /// Whether the component is emphasized.
    emphasized: bool,
    /// Text displayed in the component.
    label: String,
    /// Number of visible markers.
    count: u32,
    /// Color treatment used by the component.
    tone: Tone,
}

#[component(title = "Demo", group = "Tests")]
/// A presentational component used to verify generated documentation.
fn demo(props: &DemoProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div().child(props.label.clone())
}

#[fixture(component = demo, title = "Default")]
fn demo_default() -> DemoProps {
    DemoProps::default()
}

#[fixture(component = demo, title = "Alternate")]
fn demo_alternate() -> DemoProps {
    DemoProps {
        label: "Alternate".to_owned(),
        count: 2,
        ..DemoProps::default()
    }
}

#[test]
fn derives_control_metadata_and_values() {
    let props = DemoProps::default();
    let definitions = props.definitions();

    assert_eq!(definitions.len(), 4);
    assert_eq!(definitions[0].id, "emphasized");
    assert_eq!(definitions[0].label, "Emphasized");
    assert_eq!(definitions[0].docs, "Whether the component is emphasized.");
    assert_eq!(definitions[0].kind, ControlKind::Boolean);
    assert_eq!(definitions[1].kind, ControlKind::Text);
    assert_eq!(definitions[2].kind, ControlKind::Number);
    assert_eq!(
        definitions[3].kind,
        ControlKind::Enum {
            options: &["Neutral", "High contrast"]
        }
    );
    assert_eq!(
        props.control_value("tone"),
        Some(ControlValue::Enum("Neutral".to_owned()))
    );
}

#[test]
fn mutates_every_supported_control_type_and_resets() {
    let mut fixture = registered_catalog()
        .expect("registry should be valid")
        .into_parts()
        .1
        .into_iter()
        .find(|fixture| fixture.metadata().id.ends_with("contracts.rs#demo_default"))
        .expect("test fixture should be registered");

    fixture
        .set_control("emphasized", ControlValue::Boolean(true))
        .expect("boolean control should update");
    fixture
        .set_control("label", ControlValue::Text("Changed".to_owned()))
        .expect("text control should update");
    fixture
        .set_control("count", ControlValue::Number(3.0))
        .expect("numeric control should update");
    fixture
        .set_control("tone", ControlValue::Enum("High contrast".to_owned()))
        .expect("enum control should update");

    assert_eq!(
        fixture.props().control_value("emphasized"),
        Some(ControlValue::Boolean(true))
    );
    assert_eq!(
        fixture.props().control_value("label"),
        Some(ControlValue::Text("Changed".to_owned()))
    );
    assert_eq!(
        fixture.props().control_value("count"),
        Some(ControlValue::Number(3.0))
    );
    assert_eq!(
        fixture.props().control_value("tone"),
        Some(ControlValue::Enum("High contrast".to_owned()))
    );

    fixture.reset();
    assert_eq!(
        fixture.props().control_value("emphasized"),
        Some(ControlValue::Boolean(false))
    );
    assert_eq!(
        fixture.props().control_value("label"),
        Some(ControlValue::Text(String::new()))
    );
}

#[test]
fn rejects_invalid_control_updates() {
    let mut props = DemoProps::default();

    assert_eq!(
        props.set_control("count", ControlValue::Number(1.5)),
        Err(ControlError::InvalidNumber {
            control: "count",
            value: 1.5,
        })
    );
    assert_eq!(
        props.set_control("tone", ControlValue::Enum("Unknown".to_owned())),
        Err(ControlError::InvalidOption {
            control: "tone",
            value: "Unknown".to_owned(),
        })
    );
    assert_eq!(
        props.set_control("missing", ControlValue::Boolean(true)),
        Err(ControlError::UnknownControl("missing".to_owned()))
    );
}

#[test]
fn captures_fixture_rustdoc_and_source_metadata() {
    let catalog = registered_catalog().expect("registry should be valid");
    let component = catalog
        .components()
        .iter()
        .find(|component| component.metadata().id.ends_with("contracts.rs#demo"))
        .expect("test component should be registered");
    let fixture = catalog
        .fixtures()
        .iter()
        .find(|fixture| fixture.metadata().id.ends_with("contracts.rs#demo_default"))
        .expect("test fixture should be registered");

    assert_eq!(component.metadata().title, "Demo");
    assert_eq!(component.metadata().group, "Tests");
    assert_eq!(
        component.metadata().docs,
        "A presentational component used to verify generated documentation."
    );
    assert_eq!(fixture.metadata().title, "Default");
    assert_eq!(fixture.metadata().component_id, component.metadata().id);
    assert!(fixture.metadata().source.ends_with("tests/contracts.rs"));
    assert!(fixture.metadata().line > 0);
}

#[test]
fn registers_multiple_variants_under_one_component() {
    let catalog = registered_catalog().expect("registry should be valid");
    let component = catalog
        .components()
        .iter()
        .find(|component| component.metadata().id.ends_with("contracts.rs#demo"))
        .expect("test component should be registered");
    let fixtures = catalog
        .fixtures()
        .iter()
        .filter(|fixture| fixture.metadata().component_id == component.metadata().id)
        .collect::<Vec<_>>();

    assert_eq!(fixtures.len(), 2);
    assert_eq!(fixtures[0].metadata().title, "Alternate");
    assert!(
        fixtures[0]
            .metadata()
            .id
            .ends_with("contracts.rs#demo_alternate")
    );
    assert_eq!(fixtures[1].metadata().title, "Default");
    assert!(
        fixtures[1]
            .metadata()
            .id
            .ends_with("contracts.rs#demo_default")
    );
}
