use crate::{FixtureDefinition, gpui, render_fixture};

use gpui::{AnyElement, App, Bounds, Modifiers, Pixels, Size, Window, point, px};
pub use gpui::{TestAppContext, VisualTestContext};

pub fn draw_fixture(cx: &mut VisualTestContext, fixture: &FixtureDefinition, size: Size<Pixels>) {
    cx.draw(point(px(0.0), px(0.0)), size, |window, app| {
        render_fixture(fixture, window, app)
    });
}

/// Draws a component renderer and returns the typed handle produced with its element.
///
/// # Panics
/// Panics only if GPUI's synchronous `draw` contract stops invoking the supplied render closure.
pub fn draw_with_handle<Handle>(
    cx: &mut VisualTestContext,
    size: Size<Pixels>,
    render: impl FnOnce(&mut Window, &mut App) -> (AnyElement, Handle),
) -> Handle {
    let mut handle = None;
    cx.draw(point(px(0.0), px(0.0)), size, |window, app| {
        let (element, rendered_handle) = render(window, app);
        handle = Some(rendered_handle);
        element
    });
    handle.expect("render closure always stores its typed handle")
}

pub fn click_bounds(cx: &mut VisualTestContext, bounds: Bounds<Pixels>) {
    cx.simulate_click(bounds.center(), Modifiers::none());
}
