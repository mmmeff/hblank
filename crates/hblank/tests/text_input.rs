#![cfg(feature = "test-support")]

use hblank::gpui::{self, ClipboardItem, EntityInputHandler, TestAppContext};
use hblank::harness::input::{self, TextInput};

#[gpui::test]
fn keyboard_selection_replaces_graphemes_and_supports_clipboard(cx: &mut TestAppContext) {
    cx.update(input::init);
    let (editor, cx) = cx.add_window_view(|window, cx| {
        let input = TextInput::new("Ae\u{301}👩‍💻Z", "", false, cx);
        window.focus(input.focus_handle());
        input
    });

    cx.simulate_keystrokes("end left shift-left shift-left");
    cx.simulate_input("x");
    assert_eq!(
        editor.read_with(cx, |input, _| input.text().to_owned()),
        "AxZ"
    );

    cx.simulate_keystrokes("ctrl-a ctrl-c");
    assert_eq!(
        cx.read_from_clipboard().and_then(|item| item.text()),
        Some("AxZ".to_owned())
    );
    cx.simulate_keystrokes("ctrl-x");
    assert_eq!(editor.read_with(cx, |input, _| input.text().to_owned()), "");
    cx.simulate_keystrokes("ctrl-v");
    assert_eq!(
        editor.read_with(cx, |input, _| input.text().to_owned()),
        "AxZ"
    );
    cx.simulate_keystrokes("home delete");
    assert_eq!(
        editor.read_with(cx, |input, _| input.text().to_owned()),
        "xZ"
    );
}

#[gpui::test]
fn ime_selection_is_relative_to_inserted_composition(cx: &mut TestAppContext) {
    cx.update(input::init);
    let (editor, cx) = cx.add_window_view(|window, cx| {
        let input = TextInput::new("A😀Z", "", false, cx);
        window.focus(input.focus_handle());
        input
    });

    cx.update(|window, cx| {
        editor.update(cx, |input, cx| {
            // Replace the two UTF-16 units of the emoji at a nonzero document offset.
            input.replace_and_mark_text_in_range(Some(1..3), "日本", Some(1..2), window, cx);
            assert_eq!(input.text(), "A日本Z");
            assert_eq!(input.marked_text_range(window, cx), Some(1..3));
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                2..3
            );
            // A composition commit replaces the marked text, not just the selection.
            input.replace_text_in_range(None, "語", window, cx);
            assert_eq!(input.text(), "A語Z");
            assert_eq!(input.marked_text_range(window, cx), None);
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                2..2
            );
        });
    });
}

#[gpui::test]
fn multiline_paste_and_undo_preserve_line_breaks(cx: &mut TestAppContext) {
    cx.update(input::init);
    cx.write_to_clipboard(ClipboardItem::new_string("first\nsecond".to_owned()));
    let (editor, cx) = cx.add_window_view(|window, cx| {
        let input = TextInput::new("", "", true, cx);
        window.focus(input.focus_handle());
        input
    });

    cx.simulate_keystrokes("ctrl-v");
    assert_eq!(
        editor.read_with(cx, |input, _| input.text().to_owned()),
        "first\nsecond"
    );
    cx.simulate_keystrokes("ctrl-z");
    assert_eq!(editor.read_with(cx, |input, _| input.text().to_owned()), "");
    cx.simulate_keystrokes("ctrl-shift-z");
    assert_eq!(
        editor.read_with(cx, |input, _| input.text().to_owned()),
        "first\nsecond"
    );
}

#[gpui::test]
fn overflowing_input_keeps_caret_and_mouse_mapping_in_view(cx: &mut TestAppContext) {
    use gpui::{Bounds, Modifiers, point, px, size};

    cx.update(input::init);
    let text = "long editable value ".repeat(30);
    let end = text.len();
    let (editor, cx) = cx.add_window_view(|window, cx| {
        let input = TextInput::new(text, "", false, cx);
        window.focus(input.focus_handle());
        input
    });
    cx.simulate_resize(size(px(180.0), px(100.0)));
    cx.simulate_keystrokes("end");
    let caret = cx.update(|window, cx| {
        editor.update(cx, |input, cx| {
            input
                .bounds_for_range(
                    end..end,
                    Bounds::new(point(px(0.0), px(0.0)), size(px(180.0), px(100.0))),
                    window,
                    cx,
                )
                .expect("focused input exposes caret bounds")
        })
    });
    assert!(
        caret.left() >= px(0.0) && caret.right() <= px(180.0),
        "{caret:?}"
    );
    cx.simulate_click(caret.center(), Modifiers::none());
    cx.simulate_input("!");
    assert!(editor.read_with(cx, |input, _| input.text().ends_with('!')));
}

#[gpui::test]
fn multiline_wrapping_scrolls_vertically_and_preserves_click_mapping(cx: &mut TestAppContext) {
    use gpui::{Bounds, Modifiers, point, px, size};

    cx.update(input::init);
    let text = "wrapped text on many visual lines ".repeat(20);
    let end = text.len();
    let (editor, cx) = cx.add_window_view(|window, cx| {
        let input = TextInput::new(text, "", true, cx);
        window.focus(input.focus_handle());
        input
    });
    cx.simulate_resize(size(px(180.0), px(120.0)));
    cx.simulate_keystrokes("ctrl-end");
    let caret = cx.update(|window, cx| {
        editor.update(cx, |input, cx| {
            input
                .bounds_for_range(
                    end..end,
                    Bounds::new(point(px(0.0), px(0.0)), size(px(180.0), px(120.0))),
                    window,
                    cx,
                )
                .unwrap()
        })
    });
    assert!(
        caret.left() >= px(0.0) && caret.right() <= px(180.0),
        "{caret:?}"
    );
    assert!(
        caret.top() >= px(0.0) && caret.bottom() <= px(120.0),
        "{caret:?}"
    );
    cx.simulate_click(caret.center(), Modifiers::none());
    cx.simulate_input("!");
    assert!(editor.read_with(cx, |input, _| input.text().ends_with('!')));
    cx.simulate_keystrokes("ctrl-home");
    cx.simulate_input("start ");
    assert!(editor.read_with(cx, |input, _| input.text().starts_with("start ")));
}
