use hblank::HblankProps;
use hblank::gpui::{App, IntoElement, Window, div, prelude::*, px, rgb};

#[derive(Clone, Debug, HblankProps)]
struct LiveDiscoveryProps {
    /// Message displayed by the newly discovered example.
    message: String,
}

impl Default for LiveDiscoveryProps {
    fn default() -> Self {
        Self {
            message: "Discovered while hblank dev was running".to_owned(),
        }
    }
}

#[hblank::example(
    id = "dogfood.live-discovery",
    title = "Live discovery",
    group = "Dogfood"
)]
/// A fixture added while the development harness is running to verify glob discovery and automatic preview replacement.
fn live_discovery_example(
    props: &LiveDiscoveryProps,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    div()
        .w(px(360.0))
        .p_6()
        .rounded_lg()
        .border_1()
        .border_color(rgb(0x7357d8))
        .bg(rgb(0xffffff))
        .text_color(rgb(0x303034))
        .child(props.message.clone())
}
