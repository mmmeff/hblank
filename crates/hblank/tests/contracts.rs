use hblank::gpui::{App, IntoElement, ParentElement, Window, div};
use hblank::{
    ControlError, ControlKind, ControlValue, DocBlock, DocContext, DocPage, HblankControlAdapter,
    HblankEnum, HblankProps, NumberConstraints, TextMode, component, fixture, registered_catalog,
};

#[derive(Clone, Default, HblankEnum)]
enum Tone {
    #[default]
    Neutral,
    #[hblank(label = "High contrast")]
    HighContrast,
}

#[derive(Clone, Default)]
struct Percentage(u8);

struct PercentageControl;

impl HblankControlAdapter<Percentage> for PercentageControl {
    type Value = u8;

    fn to_control(value: &Percentage) -> Self::Value {
        value.0
    }

    fn apply_control(value: &mut Percentage, control: Self::Value) {
        value.0 = control;
    }
}

#[derive(Clone, Default, HblankProps)]
struct DemoProps {
    /// Whether the component is emphasized.
    emphasized: bool,
    /// Text displayed in the component.
    #[hblank(multiline)]
    label: String,
    /// Number of visible markers.
    #[hblank(min = 0, max = 10, step = 2)]
    count: u32,
    /// Color treatment used by the component.
    tone: Tone,
    /// Completion percentage mapped from a domain newtype.
    #[hblank(adapter = PercentageControl, min = 0, max = 100, step = 5)]
    progress: Percentage,
    #[hblank(skip)]
    internal_path: std::path::PathBuf,
}

#[component(title = "Demo", group = "Tests", docs = demo_docs)]
/// A presentational component used to verify generated documentation.
fn demo(props: &DemoProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let _ = &props.internal_path;
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

#[hblank::doc_block]
fn demo_custom_block(context: &DocContext<'_>, payload: &str) -> hblank::gpui::AnyElement {
    div()
        .child(format!("{}: {payload}", context.component_title))
        .into_any_element()
}

fn demo_docs() -> DocPage {
    DocPage::new([
        DocBlock::heading(1, "Demo"),
        DocBlock::fixture(hblank::fixture_ref!(demo_default)),
        DocBlock::props(),
        DocBlock::controls(),
        hblank::custom_doc!(demo_custom_block, "custom payload"),
        DocBlock::source(),
    ])
}

#[test]
fn derives_control_metadata_and_values() {
    let props = DemoProps::default();
    let definitions = props.definitions();

    assert_eq!(definitions.len(), 5);
    assert_eq!(definitions[0].id, "emphasized");
    assert_eq!(definitions[0].label, "Emphasized");
    assert_eq!(definitions[0].docs, "Whether the component is emphasized.");
    assert_eq!(definitions[0].kind, ControlKind::Boolean);
    assert_eq!(
        definitions[1].kind,
        ControlKind::Text {
            mode: TextMode::Multiline
        }
    );
    assert_eq!(
        definitions[2].kind,
        ControlKind::Number {
            constraints: NumberConstraints {
                min: Some(0.0),
                max: Some(10.0),
                step: 2.0,
            }
        }
    );
    assert_eq!(
        definitions[3].kind,
        ControlKind::Enum {
            options: &["Neutral", "High contrast"]
        }
    );
    assert_eq!(
        definitions[4].kind,
        ControlKind::Number {
            constraints: NumberConstraints {
                min: Some(0.0),
                max: Some(100.0),
                step: 5.0,
            }
        }
    );
    assert_eq!(
        props.control_value("tone"),
        Some(ControlValue::Enum("Neutral".to_owned()))
    );
    assert_eq!(
        props.control_value("progress"),
        Some(ControlValue::Number(0.0))
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
        .set_control("count", ControlValue::Number(4.0))
        .expect("constrained numeric control should update");
    fixture
        .set_control("tone", ControlValue::Enum("High contrast".to_owned()))
        .expect("enum control should update");
    fixture
        .set_control("progress", ControlValue::Number(65.0))
        .expect("adapted domain control should update");

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
        Some(ControlValue::Number(4.0))
    );
    assert_eq!(
        fixture.props().control_value("tone"),
        Some(ControlValue::Enum("High contrast".to_owned()))
    );
    assert_eq!(
        fixture.props().control_value("progress"),
        Some(ControlValue::Number(65.0))
    );
    assert_eq!(
        fixture
            .props()
            .as_any()
            .downcast_ref::<DemoProps>()
            .expect("fixture should retain typed props")
            .progress
            .0,
        65
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
        props.set_control("count", ControlValue::Number(-2.0)),
        Err(ControlError::BelowMinimum {
            control: "count",
            min: 0.0,
            value: -2.0,
        })
    );
    assert_eq!(
        props.set_control("count", ControlValue::Number(12.0)),
        Err(ControlError::AboveMaximum {
            control: "count",
            max: 10.0,
            value: 12.0,
        })
    );
    assert_eq!(
        props.set_control("count", ControlValue::Number(3.0)),
        Err(ControlError::StepMismatch {
            control: "count",
            step: 2.0,
            value: 3.0,
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
        props.set_control("internal_path", ControlValue::Text("hidden".to_owned())),
        Err(ControlError::UnknownControl("internal_path".to_owned()))
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
    assert_eq!(component.docs().blocks().len(), 6);
    assert!(hblank::registered_doc_block("contracts::demo_custom_block").is_some());
    assert_eq!(
        component.docs().blocks()[1],
        DocBlock::Fixture {
            id: fixture.metadata().id.clone()
        }
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
