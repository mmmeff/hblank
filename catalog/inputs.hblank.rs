use hblank::HblankProps;
use hblank::gpui::{App, Entity, IntoElement, Window, div, prelude::*, rems};
use hblank::harness::input::TextInput;
use hblank::harness::{SearchProps, search};

use crate::catalog_controls::{ControlProps, ControlsPreview};

#[derive(Clone, Debug, Default, HblankProps)]
struct SearchFixtureProps {
    /// Text displayed in the fixture-filter editor.
    query: String,
}

#[hblank::component(title = "Search", group = "Inputs")]
/// The editable fixture filter displayed above Hblank's navigation tree.
fn search_fixture(
    props: &SearchFixtureProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let state = window.use_keyed_state("catalog-search", cx, |_, cx| SearchPreview {
        external_query: props.query.clone(),
        input: cx.new(|cx| TextInput::new(props.query.clone(), "Filter fixtures…", false, cx)),
    });
    let input = state.update(cx, |state, cx| {
        if state.external_query != props.query {
            state.external_query.clone_from(&props.query);
            state
                .input
                .update(cx, |input, cx| input.set_text(props.query.clone(), cx));
        }
        state.input.clone()
    });
    div().w(rems(22.0)).child(search(SearchProps { input }))
}

#[hblank::fixture(component = search_fixture, title = "Default")]
fn search_default() -> SearchFixtureProps {
    SearchFixtureProps::default()
}

struct SearchPreview {
    external_query: String,
    input: Entity<TextInput>,
}

#[derive(Clone, Debug, HblankProps)]
struct TextInputFixtureProps {
    /// Editable text. Local typing survives unrelated property changes.
    #[hblank(multiline)]
    text: String,
    /// Enable line breaks and wrapping in a bounded four-line editor.
    multiline: bool,
}

impl Default for TextInputFixtureProps {
    fn default() -> Self {
        Self {
            text: "Edit this value".to_owned(),
            multiline: false,
        }
    }
}

#[hblank::component(title = "Text input", group = "Inputs")]
/// Hblank's native editor with selection, clipboard, undo, and single-line or multiline editing.
fn text_input_fixture(
    props: &TextInputFixtureProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let state = window.use_keyed_state("catalog-text-input", cx, |_, cx| TextInputPreview {
        external: props.clone(),
        input: cx.new(|cx| TextInput::new(props.text.clone(), "Enter text…", props.multiline, cx)),
    });
    let input = state.update(cx, |state, cx| {
        if state.external.multiline != props.multiline {
            let text = if state.external.text == props.text {
                state.input.read(cx).text().to_owned()
            } else {
                props.text.clone()
            };
            state.input = cx.new(|cx| TextInput::new(text, "Enter text…", props.multiline, cx));
        } else if state.external.text != props.text {
            state
                .input
                .update(cx, |input, cx| input.set_text(props.text.clone(), cx));
        }
        state.external.clone_from(props);
        state.input.clone()
    });
    div().w(rems(27.0)).child(input)
}

#[hblank::fixture(component = text_input_fixture, title = "Single line")]
fn text_input_single_line() -> TextInputFixtureProps {
    TextInputFixtureProps::default()
}

#[hblank::fixture(component = text_input_fixture, title = "Multiline")]
fn text_input_multiline() -> TextInputFixtureProps {
    TextInputFixtureProps {
        text: "Write a component description.\nAdd another line, select text, or paste a longer passage to explore wrapping and scrolling.".to_owned(),
        multiline: true,
    }
}

struct TextInputPreview {
    external: TextInputFixtureProps,
    input: Entity<TextInput>,
}

#[hblank::component(title = "Controls panel", group = "Inputs")]
/// Generated boolean, text, number, and enum controls with editable values and reset.
fn controls_fixture(props: &ControlProps, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = window.use_keyed_state("catalog-controls-panel", cx, |_, cx| {
        ControlsPreview::new(props.clone(), false, cx)
    });
    state.update(cx, |state, cx| state.sync(props, cx));
    state
}

#[hblank::fixture(component = controls_fixture, title = "Default")]
fn controls_default() -> ControlProps {
    ControlProps::default()
}
