use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::gpui::{TestAppContext, VisualTestContext};
use crate::{
    ComponentMetadata, ControlDefinition, ControlError, FixtureRegistrationData,
    FixtureRegistrationMetadata, HblankProps,
};
use hblank_core::ControlField;

#[derive(Clone)]
struct Props {
    label: String,
    count: u32,
}

impl HblankProps for Props {
    fn definitions(&self) -> &'static [ControlDefinition] {
        &[
            ControlDefinition {
                id: "label",
                label: "Label",
                docs: "",
                kind: ControlKind::Text {
                    mode: TextMode::Multiline,
                },
            },
            ControlDefinition {
                id: "count",
                label: "Count",
                docs: "",
                kind: <u32 as ControlField>::KIND,
            },
        ]
    }
    fn control_value(&self, id: &str) -> Option<ControlValue> {
        match id {
            "label" => Some(self.label.to_control_value()),
            "count" => Some(self.count.to_control_value()),
            _ => None,
        }
    }
    fn set_control(&mut self, id: &str, value: ControlValue) -> Result<(), ControlError> {
        match id {
            "label" => self.label.set_control_value("label", value),
            "count" => self.count.set_control_value("count", value),
            _ => Err(ControlError::UnknownControl(id.to_owned())),
        }
    }
    fn clone_box(&self) -> Box<dyn HblankProps> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct HostEditor(Entity<TextInput>);
impl gpui::Global for HostEditor {}

fn embedded_editor(_: &dyn HblankProps, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
    let input = window.use_keyed_state("embedded-editor", cx, |_, cx| {
        TextInput::new("preview", "", false, cx)
    });
    cx.set_global(HostEditor(input.clone()));
    input.into_any_element()
}

struct StateFile(PathBuf);
impl Drop for StateFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn harness(cx: &mut TestAppContext) -> (Entity<HarnessApp>, &mut VisualTestContext, StateFile) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let state = StateFile(env::temp_dir().join(format!(
        "hblank-input-test-{}-{}.toml",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    cx.update(input::init);
    let (app, cx) = cx.add_window_view(|window, cx| {
        let mut app = HarnessApp::new(window, cx);
        app.state_path.clone_from(&state.0);
        let component = ComponentDefinition::new::<Props>(
            ComponentMetadata {
                id: "component".to_owned(),
                title: "Editable",
                group: "Inputs",
                docs: "",
                declaration: "",
                source: "input_tests.rs",
                line: 1,
            },
            embedded_editor as crate::RenderComponent,
        )
        .with_docs(crate::DocPage::new([
            DocBlock::controls(),
            DocBlock::controls(),
        ]));
        let registrations = ["first", "second"]
            .into_iter()
            .map(|id| {
                FixtureRegistrationData::new(
                    FixtureRegistrationMetadata {
                        id: id.to_owned(),
                        title: id,
                        docs: "",
                        declaration: "",
                        source: "input_tests.rs",
                        line: 1,
                    },
                    "component".to_owned(),
                    Box::new(Props {
                        label: id.to_owned(),
                        count: 3,
                    }),
                )
            })
            .collect();
        (app.components, app.fixtures) =
            hblank_core::assemble_catalog(vec![component], registrations)
                .unwrap()
                .into_parts();
        app.navigation = HarnessApp::navigation_components(&app.components, &app.fixtures);
        app.selected = Some(0);
        app.filter.clear();
        app.search_input
            .update(cx, |input, cx| input.set_text("", cx));
        app.collapsed_groups = BTreeSet::from(["Inputs".to_owned()]);
        app
    });
    (app, cx, state)
}

fn focus_input(
    app: &Entity<HarnessApp>,
    panel: usize,
    id: &str,
    cx: &mut VisualTestContext,
) -> Entity<TextInput> {
    let input = app.read_with(cx, |app, _| {
        app.control_inputs[&(app.selected.unwrap(), panel)][id].clone()
    });
    cx.update(|window, cx| input.read(cx).focus_handle().focus(window));
    input
}

#[gpui::test]
fn numeric_drafts_reset_and_fixture_switches_are_isolated(cx: &mut TestAppContext) {
    let (app, cx, _state) = harness(cx);
    let number = focus_input(&app, 0, "count", cx);
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input("-");
    assert_eq!(
        number.read_with(cx, |input, _| input.text().to_owned()),
        "-"
    );
    assert_eq!(
        app.read_with(cx, |app, _| app.fixtures[0].props().control_value("count")),
        Some(ControlValue::Number(3.0))
    );
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input("7");
    assert_eq!(
        app.read_with(cx, |app, _| app.fixtures[0].props().control_value("count")),
        Some(ControlValue::Number(7.0))
    );
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.on_control(
                0,
                &ControlAction::Set {
                    id: "count",
                    value: ControlValue::Number(8.0),
                },
                window,
                cx,
            );
        });
    });
    assert_eq!(
        number.read_with(cx, |input, _| input.text().to_owned()),
        "8"
    );
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.on_control(0, &ControlAction::Reset, window, cx);
        });
    });
    assert_eq!(
        number.read_with(cx, |input, _| input.text().to_owned()),
        "3"
    );
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.on_navigation(
                &NavigationAction::Select {
                    id: "second".into(),
                },
                window,
                cx,
            );
        });
    });
    cx.run_until_parked();
    let second = focus_input(&app, 0, "label", cx);
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input("changed");
    assert_eq!(
        second.read_with(cx, |input, _| input.text().to_owned()),
        "changed"
    );
    assert_eq!(
        app.read_with(cx, |app, _| app.fixtures[0].props().control_value("label")),
        Some(ControlValue::Text("first".to_owned()))
    );
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.on_toolbar(&ToolbarAction::ShowDocs, window, cx);
        });
    });
    cx.run_until_parked();
    let docs = focus_input(&app, 1, "label", cx);
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input("docs");
    let other = app.read_with(cx, |app, _| app.control_inputs[&(1, 2)]["label"].clone());
    assert_ne!(docs.entity_id(), other.entity_id());
    assert_eq!(
        other.read_with(cx, |input, _| input.text().to_owned()),
        "docs"
    );
    assert_eq!(
        second.read_with(cx, |input, _| input.text().to_owned()),
        "docs"
    );
}

#[gpui::test]
fn search_and_preview_input_have_independent_focus(cx: &mut TestAppContext) {
    let (app, cx, _state) = harness(cx);
    cx.simulate_input("second");
    cx.simulate_keystrokes("down");
    assert_eq!(
        app.read_with(cx, |app, _| app.selected_id().map(str::to_owned)),
        Some("second".to_owned())
    );
    cx.simulate_keystrokes("escape");
    assert_eq!(app.read_with(cx, |app, _| app.filter.clone()), "");
    let preview = cx.update(|_, cx| cx.global::<HostEditor>().0.clone());
    cx.update(|window, cx| preview.read(cx).focus_handle().focus(window));
    cx.simulate_keystrokes("end");
    cx.simulate_input(" text");
    assert_eq!(
        preview.read_with(cx, |input, _| input.text().to_owned()),
        "preview text"
    );
    assert_eq!(app.read_with(cx, |app, _| app.filter.clone()), "");
}
