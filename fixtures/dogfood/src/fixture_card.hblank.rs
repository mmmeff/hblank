use hblank::{CalloutTone, DocBlock, DocPage};
use hblank::gpui::{App, IntoElement, Window};
use hblank_project::{FixtureCardProps, fixture_card};

#[hblank::component(
    title = "Fixture card",
    group = "Dogfood",
    docs = fixture_card_docs
)]
/// A state-free GPUI card with generated boolean, text, numeric, and enum controls. Change any property to verify that the isolated preview rerenders immediately.
fn fixture_card_fixture(
    props: &FixtureCardProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    fixture_card(props, window, cx)
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
        DocBlock::source(),
    ])
}

