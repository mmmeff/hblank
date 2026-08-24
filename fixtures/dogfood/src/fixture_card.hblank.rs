use hblank::gpui::{App, IntoElement, Window};
use hblank_project::{FixtureCardProps, fixture_card};

#[hblank::fixture(id = "dogfood.fixture-card", title = "Fixture card", group = "Dogfood")]
/// A state-free GPUI card with generated boolean, text, numeric, and enum controls. Change any property to verify that the isolated preview rerenders immediately.
fn fixture_card_fixture(
    props: &FixtureCardProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    fixture_card(props, window, cx)
}
