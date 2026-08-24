use hblank::{CalloutTone, DocBlock, DocPage};
use hblank::gpui::{App, IntoElement, Window, div, prelude::*, px, rgb, size};
use hblank_project::{FixtureCardProps, fixture_card};

#[derive(Clone, Debug, PartialEq, Eq)]
struct FixtureCardHandle {
    label: String,
}

#[hblank::component(
    title = "Fixture card",
    group = "Dogfood",
    docs = fixture_card_docs,
    handle = FixtureCardHandle
)]
/// A state-free GPUI card with generated boolean, text, numeric, and enum controls. Change any property to verify that the isolated preview rerenders immediately.
fn fixture_card_fixture(
    props: &FixtureCardProps,
    window: &mut Window,
    cx: &mut App,
) -> hblank::Rendered<FixtureCardHandle, impl IntoElement> {
    hblank::Rendered::new(
        fixture_card(props, window, cx),
        FixtureCardHandle {
            label: props.label.clone(),
        },
    )
}

#[hblank::fixture(component = fixture_card_fixture, title = "Default")]
fn fixture_card_default() -> FixtureCardProps {
    FixtureCardProps::default()
}


fn fixture_card_docs() -> DocPage {
    DocPage::new([
        DocBlock::heading(1, "Fixture card"),
        DocBlock::prose("A state-free GPUI card that exercises Hblank's generated controls."),
        DocBlock::fixture(hblank::fixture_ref!(fixture_card_default)),
        DocBlock::props(),
        DocBlock::controls(),
        DocBlock::callout(
            CalloutTone::Success,
            "Dogfooded",
            "This component verifies the same catalog, controls, and theme path users consume.",
        ),
        hblank::custom_doc!(adoption_note, "Ready for project-specific documentation"),
        DocBlock::source(),
    ])
}


#[hblank::doc_block]
fn adoption_note(context: &hblank::DocContext<'_>, payload: &str) -> hblank::gpui::AnyElement {
    let accent = match context.resolved_theme {
        hblank::ResolvedTheme::Light => 0x3e2b86,
        hblank::ResolvedTheme::Dark => 0xddd5ff,
    };
    div()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(rgb(accent))
        .text_color(rgb(accent))
        .child(format!("{}: {payload}", context.component_title))
        .into_any_element()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn component_render_exposes_typed_handle(cx: &mut hblank::testing::TestAppContext) {
        let props = fixture_card_default();
        let cx = cx.add_empty_window();
        let handle = hblank::testing::draw_with_handle(
            cx,
            size(px(480.0), px(320.0)),
            |window, app| {
                hblank::render_handle!(fixture_card_fixture, &props, window, app)
            },
        );

        assert_eq!(handle.label, "Hot reload verified");
    }

    #[test]
    fn fixture_card_default_and_docs_are_explicit() {
        let props = fixture_card_default();
        assert_eq!(props.count, 3);
        assert!(props.active);
        assert_eq!(fixture_card_docs().blocks().len(), 8);
    }
}



