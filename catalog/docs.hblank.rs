use crate::catalog_controls::{ControlProps, ControlsPreview};
use hblank::gpui::{App, IntoElement, Window, div, prelude::*, px};
use hblank::harness::{
    DocsPanelProps, doc_callout, doc_fixture, doc_heading, doc_props, doc_prose, doc_source,
    docs_panel,
};
use hblank::{CalloutTone, HblankEnum, HblankProps};

#[derive(Clone, Debug, HblankProps)]
struct DocProps {
    /// Heading displayed by the documentation panel.
    title: String,
    /// Introductory copy explaining the documentation surface.
    #[hblank(multiline)]
    intro: String,
    /// Source location or declaration shown at the end of the panel.
    #[hblank(multiline)]
    source: String,
}

impl Default for DocProps {
    fn default() -> Self {
        Self {
            title: "Documentation panel".to_owned(),
            intro: "Compose headings, prose, callouts, properties, controls, and source into one readable page.".to_owned(),
            source: "catalog/docs.hblank.rs · docs_panel_fixture".to_owned(),
        }
    }
}

#[hblank::component(title = "Docs panel", group = "Documentation")]
/// A complete documentation page composed from Hblank's native documentation blocks.
fn docs_panel_fixture(props: &DocProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let defaults = ControlProps::default();
    docs_panel(DocsPanelProps {
        title: props.title.clone().into(),
        blocks: vec![
            doc_heading(2, "Documentation blocks"),
            doc_prose(props.intro.clone()),
            doc_callout(
                CalloutTone::Note,
                "Compose, then customize",
                "Each block is a native GPUI renderer and can be authored alongside a fixture.",
            ),
            doc_props(defaults.definitions()),
            doc_source(props.source.clone()),
        ],
    })
    .h(px(480.0))
}

#[hblank::fixture(component = docs_panel_fixture, title = "Default")]
fn docs_panel_default() -> DocProps {
    DocProps::default()
}

#[derive(Clone, Debug, HblankProps)]
struct HeadingProps {
    /// Heading level from one through three.
    #[hblank(min = 1, max = 3, step = 1)]
    level: u8,
    /// Text rendered by the heading.
    text: String,
}

impl Default for HeadingProps {
    fn default() -> Self {
        Self {
            level: 2,
            text: "A heading with semantic scale".to_owned(),
        }
    }
}

#[hblank::component(title = "Heading", group = "Documentation")]
/// A documentation heading with an editable level and label.
fn heading_fixture(props: &HeadingProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div()
        .w(px(360.0))
        .child(doc_heading(props.level, props.text.clone()))
}

#[hblank::fixture(component = heading_fixture, title = "Default")]
fn heading_default() -> HeadingProps {
    HeadingProps::default()
}

#[derive(Clone, Debug, HblankProps)]
struct ProseProps {
    /// Paragraph text rendered in the documentation measure.
    #[hblank(multiline)]
    text: String,
}

impl Default for ProseProps {
    fn default() -> Self {
        Self {
            text: "Hblank keeps documentation close to the component it explains, while native blocks preserve a calm reading rhythm.".to_owned(),
        }
    }
}

#[hblank::component(title = "Prose", group = "Documentation")]
/// A readable paragraph block for concise component guidance.
fn prose_fixture(props: &ProseProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div().w(px(360.0)).child(doc_prose(props.text.clone()))
}

#[hblank::fixture(component = prose_fixture, title = "Default")]
fn prose_default() -> ProseProps {
    ProseProps::default()
}

#[derive(Clone, Copy, Debug, Default, HblankEnum)]
enum CalloutVariant {
    #[default]
    Note,
    Success,
    Warning,
}

#[derive(Clone, Debug, HblankProps)]
struct CalloutProps {
    /// Semantic tone applied to the callout.
    tone: CalloutVariant,
    /// Short callout heading.
    title: String,
    /// Supporting callout copy.
    #[hblank(multiline)]
    body: String,
}

impl Default for CalloutProps {
    fn default() -> Self {
        Self {
            tone: CalloutVariant::Note,
            title: "Note".to_owned(),
            body: "Use a callout to give a component detail a little more emphasis.".to_owned(),
        }
    }
}

#[hblank::component(title = "Callout", group = "Documentation")]
/// A semantic documentation callout with Note, Success, and Warning treatments.
fn callout_fixture(props: &CalloutProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let tone = match props.tone {
        CalloutVariant::Note => CalloutTone::Note,
        CalloutVariant::Success => CalloutTone::Success,
        CalloutVariant::Warning => CalloutTone::Warning,
    };
    div()
        .w(px(360.0))
        .child(doc_callout(tone, props.title.clone(), props.body.clone()))
}

#[hblank::fixture(component = callout_fixture, title = "Note")]
fn callout_note() -> CalloutProps {
    CalloutProps::default()
}

#[hblank::fixture(component = callout_fixture, title = "Success")]
fn callout_success() -> CalloutProps {
    CalloutProps {
        tone: CalloutVariant::Success,
        title: "Ready to compose".to_owned(),
        body: "This block is ready to sit beside a live fixture and its controls.".to_owned(),
    }
}

#[hblank::fixture(component = callout_fixture, title = "Warning")]
fn callout_warning() -> CalloutProps {
    CalloutProps {
        tone: CalloutVariant::Warning,
        title: "Keep the context nearby".to_owned(),
        body:
            "Long explanations are easier to scan when they remain close to the rendered component."
                .to_owned(),
    }
}

#[derive(Clone, Debug, HblankProps)]
struct FixturePreviewProps {
    /// Label displayed above the embedded preview.
    label: String,
    /// Heading rendered inside the bounded preview.
    title: String,
    /// Supporting text rendered inside the bounded preview.
    body: String,
}

impl Default for FixturePreviewProps {
    fn default() -> Self {
        Self {
            label: "Heading preview".to_owned(),
            title: "Rendered in context".to_owned(),
            body: "A fixture preview keeps the live surface close to its docs.".to_owned(),
        }
    }
}

#[hblank::component(title = "Fixture preview", group = "Documentation")]
/// A bounded live preview container for a documented component surface.
fn fixture_preview_fixture(
    props: &FixturePreviewProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let preview = div()
        .w(px(240.0))
        .h(px(88.0))
        .flex()
        .flex_col()
        .justify_center()
        .gap_2()
        .child(doc_heading(3, props.title.clone()))
        .child(doc_prose(props.body.clone()))
        .into_any_element();
    div()
        .w(px(360.0))
        .child(doc_fixture(props.label.clone(), preview))
}

#[hblank::fixture(component = fixture_preview_fixture, title = "Default")]
fn fixture_preview_default() -> FixturePreviewProps {
    FixturePreviewProps::default()
}

#[derive(Clone, Debug, Default, HblankProps)]
struct PropertiesTableProps {}

#[hblank::component(title = "Properties table", group = "Documentation")]
/// A generated property table sourced from Hblank control definitions.
fn properties_table_fixture(
    _props: &PropertiesTableProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let defaults = ControlProps::default();
    div().w(px(360.0)).child(doc_props(defaults.definitions()))
}

#[hblank::fixture(component = properties_table_fixture, title = "Default")]
fn properties_table_default() -> PropertiesTableProps {
    PropertiesTableProps::default()
}

#[hblank::component(title = "Controls", group = "Documentation")]
/// Interactive documentation controls backed by the shared Hblank control view.
fn doc_controls_fixture(
    props: &ControlProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let state = window.use_keyed_state("catalog-doc-controls", cx, |_, cx| {
        ControlsPreview::new(props.clone(), true, cx)
    });
    state.update(cx, |preview, cx| preview.sync(props, cx));
    div()
        .id("catalog-doc-controls-scroll")
        .w(px(360.0))
        .max_h(px(480.0))
        .overflow_y_scroll()
        .child(state)
}

#[hblank::fixture(component = doc_controls_fixture, title = "Default")]
fn doc_controls_default() -> ControlProps {
    ControlProps::default()
}

#[derive(Clone, Debug, HblankProps)]
struct SourceProps {
    /// Source text or location shown in the documentation panel.
    #[hblank(multiline)]
    source: String,
}

impl Default for SourceProps {
    fn default() -> Self {
        Self {
            source: "doc_heading(2, \"Component documentation\")".to_owned(),
        }
    }
}

#[hblank::component(title = "Source", group = "Documentation")]
/// A source block that keeps declaration context visible and editable.
fn source_fixture(props: &SourceProps, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div().w(px(360.0)).child(doc_source(props.source.clone()))
}

#[hblank::fixture(component = source_fixture, title = "Default")]
fn source_default() -> SourceProps {
    SourceProps::default()
}
