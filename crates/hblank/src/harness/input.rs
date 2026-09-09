//! Native text editing shared by search and property controls.

use std::{borrow::Cow, ops::Range, rc::Rc, time::Duration};

use crate::gpui::{
    self, App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, DispatchPhase, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, GlobalElementId, InspectorElementId, IntoElement, KeyBinding, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Render, ScrollWheelEvent,
    SharedString, Style, Subscription, Task, TextAlign, TextRun, TextStyle, UTF16Selection,
    UnderlineStyle, Window, WrappedLine, div, fill, point, prelude::*, px, relative, rems, rgb,
    size,
};
use unicode_segmentation::UnicodeSegmentation;

use super::components::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Changed,
    Submit,
    Escape,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Command {
    #[default]
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Start,
    Finish,
    WordLeft,
    WordRight,
    Backspace,
    Delete,
    DeleteWordLeft,
    DeleteWordRight,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    Enter,
    Escape,
}

#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(namespace = hblank_input, no_json)]
struct Edit {
    command: Command,
    select: bool,
}

/// Register editor-scoped key bindings once when starting the application.
pub fn init(cx: &mut App) {
    let bindings = [
        ("left", Command::Left, false),
        ("right", Command::Right, false),
        ("up", Command::Up, false),
        ("down", Command::Down, false),
        ("shift-left", Command::Left, true),
        ("shift-right", Command::Right, true),
        ("shift-up", Command::Up, true),
        ("shift-down", Command::Down, true),
        ("home", Command::Home, false),
        ("end", Command::End, false),
        ("shift-home", Command::Home, true),
        ("shift-end", Command::End, true),
        ("ctrl-home", Command::Start, false),
        ("ctrl-end", Command::Finish, false),
        ("ctrl-shift-home", Command::Start, true),
        ("ctrl-shift-end", Command::Finish, true),
        ("cmd-left", Command::Home, false),
        ("cmd-right", Command::End, false),
        ("cmd-shift-left", Command::Home, true),
        ("cmd-shift-right", Command::End, true),
        ("cmd-up", Command::Start, false),
        ("cmd-down", Command::Finish, false),
        ("cmd-shift-up", Command::Start, true),
        ("cmd-shift-down", Command::Finish, true),
        ("ctrl-left", Command::WordLeft, false),
        ("ctrl-right", Command::WordRight, false),
        ("ctrl-shift-left", Command::WordLeft, true),
        ("ctrl-shift-right", Command::WordRight, true),
        ("alt-left", Command::WordLeft, false),
        ("alt-right", Command::WordRight, false),
        ("alt-shift-left", Command::WordLeft, true),
        ("alt-shift-right", Command::WordRight, true),
        ("backspace", Command::Backspace, false),
        ("delete", Command::Delete, false),
        ("ctrl-backspace", Command::DeleteWordLeft, false),
        ("ctrl-delete", Command::DeleteWordRight, false),
        ("alt-backspace", Command::DeleteWordLeft, false),
        ("alt-delete", Command::DeleteWordRight, false),
        ("ctrl-a", Command::SelectAll, false),
        ("cmd-a", Command::SelectAll, false),
        ("ctrl-c", Command::Copy, false),
        ("cmd-c", Command::Copy, false),
        ("ctrl-x", Command::Cut, false),
        ("cmd-x", Command::Cut, false),
        ("ctrl-v", Command::Paste, false),
        ("cmd-v", Command::Paste, false),
        ("ctrl-z", Command::Undo, false),
        ("cmd-z", Command::Undo, false),
        ("ctrl-shift-z", Command::Redo, false),
        ("cmd-shift-z", Command::Redo, false),
        ("ctrl-y", Command::Redo, false),
        ("enter", Command::Enter, false),
        ("shift-enter", Command::Enter, false),
        ("escape", Command::Escape, false),
    ];
    cx.bind_keys(bindings.into_iter().map(|(key, command, select)| {
        KeyBinding::new(key, Edit { command, select }, Some("HblankTextInput"))
    }));
}

#[derive(Clone)]
struct Snapshot {
    text: SharedString,
    anchor: usize,
    cursor: usize,
}

#[derive(Clone)]
struct VisualRow {
    line: usize,
    range: Range<usize>,
    x: Pixels,
    width: Pixels,
}

struct TextGeometry {
    lines: Vec<WrappedLine>,
    starts: Vec<usize>,
    rows: Vec<VisualRow>,
    line_height: Pixels,
    width: Pixels,
    text: SharedString,
    style: TextStyle,
    rem_size: Pixels,
    wrap_width: Option<Pixels>,
    marked: Option<Range<usize>>,
}

fn visual_rows(lines: &[WrappedLine]) -> (Vec<usize>, Vec<VisualRow>, Pixels) {
    let mut starts = Vec::with_capacity(lines.len());
    let mut rows = Vec::new();
    let mut start = 0;
    let mut width = px(0.);
    for (line_index, line) in lines.iter().enumerate() {
        starts.push(start);
        let mut row_start = 0;
        let mut x = px(0.);
        for (end, end_x) in line
            .wrap_boundaries
            .iter()
            .map(|boundary| {
                let glyph = &line.runs()[boundary.run_ix].glyphs[boundary.glyph_ix];
                (glyph.index, glyph.position.x)
            })
            .chain(std::iter::once((line.len(), line.unwrapped_layout.width)))
        {
            let row_width = end_x - x;
            width = width.max(row_width);
            rows.push(VisualRow {
                line: line_index,
                range: start + row_start..start + end,
                x,
                width: row_width,
            });
            row_start = end;
            x = end_x;
        }
        start += line.len() + 1;
    }
    (starts, rows, width)
}

impl TextGeometry {
    fn row_for_index(&self, index: usize) -> usize {
        self.rows
            .partition_point(|row| row.range.start <= index)
            .saturating_sub(1)
    }

    fn caret_row(&self, index: usize, upstream: bool) -> usize {
        let row = self.row_for_index(index);
        if upstream && row > 0 && self.rows[row - 1].range.end == index {
            row - 1
        } else {
            row
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Nonnegative pixel distance deliberately floors to a row index, then clamps to the layout."
    )]
    fn row_at_y(&self, y: Pixels) -> usize {
        ((y.max(px(0.)) / self.line_height) as usize).min(self.rows.len() - 1)
    }

    fn position(&self, index: usize, upstream: bool) -> Point<Pixels> {
        let row_index = self.caret_row(index, upstream);
        let row = &self.rows[row_index];
        let line = &self.lines[row.line].unwrapped_layout;
        point(
            line.x_for_index(index.min(row.range.end) - self.starts[row.line]) - row.x,
            self.line_height * row_index,
        )
    }

    fn index_at(&self, position: Point<Pixels>, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        let row_index = self.row_at_y(position.y);
        let row = &self.rows[row_index];
        let line = &self.lines[row.line].unwrapped_layout;
        // GPUI's closest_index_for_x skips the trailing glyph midpoint for multi-byte
        // glyphs. Comparing grapheme positions also avoids splitting shaped clusters.
        let mut closest = row.range.start;
        let mut distance = Pixels::MAX;
        for index in text[row.range.clone()]
            .grapheme_indices(true)
            .map(|(i, _)| row.range.start + i)
            .chain(std::iter::once(row.range.end))
        {
            let x = line.x_for_index(index - self.starts[row.line]) - row.x;
            let next_distance = (x - position.x).abs();
            if next_distance < distance {
                closest = index;
                distance = next_distance;
            }
        }
        closest
    }
}

/// An editable, focusable single-line or bounded multiline field.
#[expect(
    clippy::struct_excessive_bools,
    reason = "Multiline mode, caret affinity, scroll reveal, visibility and focus are independent editor state."
)]
pub struct TextInput {
    focus_handle: FocusHandle,
    text: SharedString,
    placeholder: SharedString,
    multiline: bool,
    anchor: usize,
    cursor: usize,
    upstream: bool,
    marked: Option<Range<usize>>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    composition: Option<Snapshot>,
    geometry: Option<Rc<TextGeometry>>,
    bounds: Option<Bounds<Pixels>>,
    scroll: Point<Pixels>,
    reveal_cursor: bool,
    preferred_x: Option<Pixels>,
    drag: Option<Range<usize>>,
    drag_clicks: usize,
    caret_visible: bool,
    blink: Option<Task<()>>,
    focused: bool,
    focus_subscriptions: Option<[Subscription; 2]>,
}

impl TextInput {
    pub fn new(
        text: impl Into<SharedString>,
        placeholder: impl Into<SharedString>,
        multiline: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let text = text.into();
        let text = match normalize_text(&text, multiline) {
            Cow::Borrowed(_) => text,
            Cow::Owned(normalized) => normalized.into(),
        };
        Self {
            focus_handle: cx.focus_handle(),
            text,
            placeholder: placeholder.into(),
            multiline,
            anchor: 0,
            cursor: 0,
            marked: None,
            undo: Vec::new(),
            redo: Vec::new(),
            composition: None,
            geometry: None,
            bounds: None,
            scroll: point(px(0.), px(0.)),
            reveal_cursor: true,
            focus_subscriptions: None,
            upstream: false,
            preferred_x: None,
            drag: None,
            drag_clicks: 1,
            caret_visible: true,
            blink: None,
            focused: false,
        }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Synchronize an external value without emitting `Changed` or disturbing an unchanged selection.
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        let text = text.into();
        let normalized = normalize_text(&text, self.multiline);
        if self.text.as_ref() == normalized.as_ref() {
            return;
        }
        self.text = match normalized {
            Cow::Borrowed(_) => text,
            Cow::Owned(normalized) => normalized.into(),
        };
        self.anchor = floor_grapheme(&self.text, self.anchor.min(self.text.len()));
        self.cursor = floor_grapheme(&self.text, self.cursor.min(self.text.len()));
        self.marked = None;
        self.composition = None;
        self.undo.clear();
        self.redo.clear();
        self.geometry = None;
        self.changed_selection(cx);
    }

    fn selection(&self) -> Range<usize> {
        self.anchor.min(self.cursor)..self.anchor.max(self.cursor)
    }
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            text: self.text.clone(),
            anchor: self.anchor,
            cursor: self.cursor,
        }
    }

    fn reset_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_visible = true;
        self.blink = None;
        if !self.focused {
            return;
        }
        self.blink = Some(cx.spawn(async move |input, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(530))
                    .await;
                if input
                    .update(cx, |input, cx| {
                        input.caret_visible = !input.caret_visible;
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }

    fn changed_selection(&mut self, cx: &mut Context<Self>) {
        self.reveal_cursor = true;
        self.preferred_x = None;
        self.upstream = false;
        self.reset_blink(cx);
        cx.notify();
    }

    fn move_to(&mut self, index: usize, select: bool, cx: &mut Context<Self>) {
        self.finish_composition();
        self.cursor = floor_grapheme(&self.text, index.min(self.text.len()));
        if !select {
            self.anchor = self.cursor;
        }
        self.changed_selection(cx);
    }

    fn finish_composition(&mut self) {
        self.marked = None;
        if let Some(before) = self.composition.take() {
            if before.text != self.text {
                self.undo.push(before);
            }
        }
    }

    fn replace(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        composing: bool,
        cx: &mut Context<Self>,
    ) {
        if composing {
            if self.composition.is_none() {
                self.composition = Some(self.snapshot());
            }
        } else if self.composition.is_none() && self.text[range.clone()] != *new_text {
            self.undo.push(self.snapshot());
        }
        let changed = self.text[range.clone()] != *new_text;
        if changed {
            let mut text = String::with_capacity(self.text.len() - range.len() + new_text.len());
            text.push_str(&self.text[..range.start]);
            text.push_str(new_text);
            text.push_str(&self.text[range.end..]);
            self.text = text.into();
            self.redo.clear();
            self.geometry = None;
        }
        self.cursor = range.start + new_text.len();
        self.anchor = self.cursor;
        if !composing {
            self.finish_composition();
        }
        self.changed_selection(cx);
        if changed {
            cx.emit(InputEvent::Changed);
        }
    }

    fn restore(&mut self, redo: bool, cx: &mut Context<Self>) {
        self.finish_composition();
        let target = if redo {
            self.redo.pop()
        } else {
            self.undo.pop()
        };
        if let Some(target) = target {
            let current = self.snapshot();
            if redo {
                self.undo.push(current);
            } else {
                self.redo.push(current);
            }
            self.text = target.text;
            self.anchor = target.anchor;
            self.cursor = target.cursor;
            self.geometry = None;
            self.changed_selection(cx);
            cx.emit(InputEvent::Changed);
        }
    }

    fn word_boundary(&self, forward: bool) -> usize {
        if forward {
            self.text
                .unicode_word_indices()
                .map(|(i, word)| i + word.len())
                .find(|end| *end > self.cursor)
                .unwrap_or(self.text.len())
        } else {
            self.text
                .unicode_word_indices()
                .map(|(i, _)| i)
                .take_while(|start| *start < self.cursor)
                .last()
                .unwrap_or(0)
        }
    }

    fn move_vertical(&mut self, down: bool, select: bool, cx: &mut Context<Self>) {
        if !self.multiline {
            cx.emit(if down {
                InputEvent::Down
            } else {
                InputEvent::Up
            });
            return;
        }
        if let Some(geometry) = &self.geometry {
            let position = geometry.position(self.cursor, self.upstream);
            let x = self.preferred_x.unwrap_or(position.x);
            let y = if down {
                position.y + geometry.line_height
            } else {
                position.y - geometry.line_height
            };
            let target = geometry.index_at(point(x, y), &self.text);
            let upstream = target == geometry.rows[geometry.row_at_y(y)].range.end;
            self.move_to(target, select, cx);
            self.preferred_x = Some(x);
            self.upstream = upstream;
        }
    }

    fn line_boundary(&self, end: bool) -> usize {
        if !self.multiline {
            return if end { self.text.len() } else { 0 };
        }
        self.geometry.as_ref().map_or_else(
            || {
                let range = logical_line(&self.text, self.cursor);
                if end {
                    range.end - usize::from(self.text[..range.end].ends_with('\n'))
                } else {
                    range.start
                }
            },
            |geometry| {
                let row = &geometry.rows[geometry.caret_row(self.cursor, self.upstream)];
                if end { row.range.end } else { row.range.start }
            },
        )
    }

    fn edit(&mut self, action: &Edit, window: &mut Window, cx: &mut Context<Self>) {
        use Command::{
            Backspace, Copy, Cut, Delete, DeleteWordLeft, DeleteWordRight, Down, End, Enter,
            Escape, Finish, Home, Left, Paste, Redo, Right, SelectAll, Start, Undo, Up, WordLeft,
            WordRight,
        };
        let selection = self.selection();
        let target = match action.command {
            Left if !action.select && !selection.is_empty() => selection.start,
            Right if !action.select && !selection.is_empty() => selection.end,
            Left => previous_grapheme(&self.text, self.cursor),
            Right => next_grapheme(&self.text, self.cursor),
            WordLeft => self.word_boundary(false),
            WordRight => self.word_boundary(true),
            Start => 0,
            Finish => self.text.len(),
            Home | End => self.line_boundary(action.command == End),
            Up | Down => {
                self.move_vertical(action.command == Down, action.select, cx);
                return;
            }
            Backspace | Delete | DeleteWordLeft | DeleteWordRight => {
                self.finish_composition();
                let range = if selection.is_empty() {
                    match action.command {
                        Backspace => previous_grapheme(&self.text, self.cursor)..self.cursor,
                        Delete => self.cursor..next_grapheme(&self.text, self.cursor),
                        DeleteWordLeft => self.word_boundary(false)..self.cursor,
                        _ => self.cursor..self.word_boundary(true),
                    }
                } else {
                    selection
                };
                self.replace(range, "", false, cx);
                return;
            }
            SelectAll => {
                self.anchor = 0;
                self.cursor = self.text.len();
                self.finish_composition();
                self.changed_selection(cx);
                return;
            }
            Copy | Cut => {
                if !selection.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        self.text[selection.clone()].to_owned(),
                    ));
                    if action.command == Cut {
                        self.finish_composition();
                        self.replace(selection, "", false, cx);
                    }
                }
                return;
            }
            Paste => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.replace_text_in_range(None, &text, window, cx);
                }
                return;
            }
            Undo | Redo => {
                self.restore(action.command == Redo, cx);
                return;
            }
            Enter => {
                if self.multiline {
                    self.replace_text_in_range(None, "\n", window, cx);
                } else {
                    self.finish_composition();
                    cx.emit(InputEvent::Submit);
                }
                return;
            }
            Escape => {
                self.finish_composition();
                cx.emit(InputEvent::Escape);
                cx.notify();
                return;
            }
        };
        self.move_to(target, action.select, cx);
        self.upstream = action.command == End;
    }

    fn index_at(&self, position: Point<Pixels>) -> usize {
        let (Some(bounds), Some(geometry)) = (self.bounds, &self.geometry) else {
            return self.cursor;
        };
        if self.text.is_empty() {
            return 0;
        }
        geometry.index_at(position - bounds.origin + self.scroll, &self.text)
    }

    fn mouse_affinity(&mut self, position: Point<Pixels>) {
        if let (Some(bounds), Some(geometry)) = (self.bounds, &self.geometry) {
            let row = geometry.row_at_y(position.y - bounds.top() + self.scroll.y);
            self.upstream = self.cursor == geometry.rows[row].range.end;
        }
    }

    fn mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window);
        self.focused = true;
        self.finish_composition();
        let index = self.index_at(event.position);
        self.drag_clicks = event.click_count;
        let range = match event.click_count {
            2 => word_at(&self.text, index),
            3.. => logical_line(&self.text, index),
            _ => index..index,
        };
        if event.modifiers.shift {
            self.cursor = index;
        } else {
            self.anchor = range.start;
            self.cursor = range.end;
        }
        self.drag = Some(if event.modifiers.shift {
            self.anchor..self.anchor
        } else {
            range
        });
        self.changed_selection(cx);
        self.mouse_affinity(event.position);
        cx.stop_propagation();
    }

    fn mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(origin) = self.drag.clone() else {
            return;
        };
        let index = self.index_at(event.position);
        let range = match self.drag_clicks {
            2 => word_at(&self.text, index),
            3.. => logical_line(&self.text, index),
            _ => index..index,
        };
        if index < origin.start {
            self.anchor = origin.end;
            self.cursor = range.start;
        } else {
            self.anchor = origin.start;
            self.cursor = range.end;
        }
        self.changed_selection(cx);
        self.mouse_affinity(event.position);
        cx.stop_propagation();
    }

    fn mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.drag = None;
    }

    fn scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let (Some(bounds), Some(geometry)) = (self.bounds, &self.geometry) else {
            return;
        };
        let delta = event.delta.pixel_delta(geometry.line_height);
        let next = point(
            (self.scroll.x - delta.x)
                .clamp(px(0.), (geometry.width - bounds.size.width).max(px(0.))),
            if self.multiline {
                (self.scroll.y - delta.y).clamp(
                    px(0.),
                    (geometry.line_height * geometry.rows.len() - bounds.size.height).max(px(0.)),
                )
            } else {
                px(0.)
            },
        );
        if next != self.scroll {
            self.scroll = next;
            self.reveal_cursor = false;
            cx.notify();
            cx.stop_propagation();
        }
    }
}

impl EventEmitter<InputEvent> for TextInput {}
impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = from_utf16_range(&self.text, range);
        *actual = Some(to_utf16_range(&self.text, range.clone()));
        Some(self.text[range].to_owned())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: to_utf16_range(&self.text, self.selection()),
            reversed: self.cursor < self.anchor,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked
            .clone()
            .map(|range| to_utf16_range(&self.text, range))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.finish_composition();
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .map(|range| from_utf16_range(&self.text, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection());
        let text = normalize_text(text, self.multiline);
        self.replace(range, &text, false, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .map(|range| from_utf16_range(&self.text, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection());
        let start = range.start;
        let text = normalize_text(text, self.multiline);
        self.replace(range, &text, true, cx);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        if let Some(selected) = selected {
            // IME selection offsets are relative to the replacement, not the document.
            let selected = from_utf16_range(&text, selected);
            self.anchor = start + selected.start;
            self.cursor = start + selected.end;
        }
        if self.marked.is_none() {
            self.finish_composition();
        }
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let geometry = self.geometry.as_ref()?;
        let bounds = self.bounds?;
        let range = from_utf16_range(&self.text, range);
        let upstream = range.is_empty() && range.start == self.cursor && self.upstream;
        let start = geometry.position(range.start, upstream);
        let row = &geometry.rows[geometry.caret_row(range.start, upstream)];
        let end_x = geometry.lines[row.line]
            .unwrapped_layout
            .x_for_index(range.end.min(row.range.end) - geometry.starts[row.line])
            - row.x;
        Some(Bounds::new(
            bounds.origin + start - self.scroll,
            size((end_x - start.x).max(px(1.)), geometry.line_height),
        ))
    }
    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        self.bounds?;
        self.geometry.as_ref()?;
        Some(self.text[..self.index_at(point)].encode_utf16().count())
    }
}

struct TextElement {
    input: Entity<TextInput>,
}
impl IntoElement for TextElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = Rc<TextGeometry>;
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height =
            (window.line_height() * if self.input.read(cx).multiline { 4 } else { 1 }).into();
        style.min_size.width = px(0.).into();
        (window.request_layout(style, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        (): &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Rc<TextGeometry> {
        let input = self.input.read(cx);
        let mut style = window.text_style();
        let text = if input.text.is_empty() {
            style.color = rgb(theme::text_muted()).into();
            input.placeholder.clone()
        } else {
            input.text.clone()
        };
        let wrap_width = input.multiline.then_some(bounds.size.width.max(px(1.)));
        let line_height = window.line_height();
        let geometry = if let Some(geometry) = input.geometry.as_ref().filter(|geometry| {
            geometry.text == text
                && geometry.style == style
                && geometry.rem_size == window.rem_size()
                && geometry.wrap_width == wrap_width
                && geometry.marked == input.marked
        }) {
            geometry.clone()
        } else {
            let run = style.to_run(text.len());
            let mut runs = Vec::with_capacity(3);
            if let Some(marked) = &input.marked {
                if marked.start > 0 {
                    runs.push(TextRun {
                        len: marked.start,
                        ..run.clone()
                    });
                }
                runs.push(TextRun {
                    len: marked.len(),
                    underline: Some(UnderlineStyle {
                        thickness: px(1.),
                        color: Some(style.color),
                        wavy: false,
                    }),
                    ..run.clone()
                });
                if marked.end < text.len() {
                    runs.push(TextRun {
                        len: text.len() - marked.end,
                        ..run
                    });
                }
            } else {
                runs.push(run);
            }
            let lines = window
                .text_system()
                .shape_text(
                    text.clone(),
                    style.font_size.to_pixels(window.rem_size()),
                    &runs,
                    wrap_width,
                    None,
                )
                .expect("valid text runs must shape")
                .into_iter()
                .collect::<Vec<_>>();
            let (starts, rows, width) = visual_rows(&lines);
            Rc::new(TextGeometry {
                lines,
                starts,
                rows,
                line_height,
                width,
                text,
                style,
                rem_size: window.rem_size(),
                wrap_width,
                marked: input.marked.clone(),
            })
        };
        self.input.update(cx, |input, _| {
            input.bounds = Some(bounds);
            let position = if input.text.is_empty() {
                point(px(0.), px(0.))
            } else {
                geometry.position(input.cursor, input.upstream)
            };
            let content_height = line_height * geometry.rows.len();
            input.scroll.x = input
                .scroll
                .x
                .min((geometry.width + px(1.) - bounds.size.width).max(px(0.)));
            input.scroll.y = input
                .scroll
                .y
                .min((content_height - bounds.size.height).max(px(0.)));
            if input.reveal_cursor {
                input.scroll.x = reveal_axis(input.scroll.x, position.x, px(1.), bounds.size.width);
                input.scroll.y =
                    reveal_axis(input.scroll.y, position.y, line_height, bounds.size.height);
                input.reveal_cursor = false;
            }
            input.geometry = Some(geometry.clone());
        });
        geometry
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        (): &mut (),
        geometry: &mut Rc<TextGeometry>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let input = self.input.read(cx);
        let selection = input.selection();
        let cursor = input.cursor;
        let upstream = input.upstream;
        let scroll = input.scroll;
        let focused = input.focus_handle.is_focused(window);
        let caret_visible = input.caret_visible;
        let empty = input.text.is_empty();
        let focus_handle = input.focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        let input = self.input.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture && input.read(cx).drag.is_some() {
                input.update(cx, |input, cx| input.mouse_move(event, window, cx));
            }
        });
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            let origin = bounds.origin - scroll;
            if !selection.is_empty() {
                for (index, row) in geometry.rows.iter().enumerate() {
                    if selection.end <= row.range.start || selection.start > row.range.end {
                        continue;
                    }
                    let line = &geometry.lines[row.line].unwrapped_layout;
                    let start = selection.start.max(row.range.start);
                    let end = selection.end.min(row.range.end);
                    let x = line.x_for_index(start - geometry.starts[row.line]) - row.x;
                    let end_x = if selection.end > row.range.end {
                        row.width + px(4.)
                    } else {
                        line.x_for_index(end - geometry.starts[row.line]) - row.x
                    };
                    window.paint_quad(fill(
                        Bounds::new(
                            origin + point(x, geometry.line_height * index),
                            size((end_x - x).max(px(1.)), geometry.line_height),
                        ),
                        rgb(theme::accent_wash()),
                    ));
                }
            }
            let mut y = px(0.);
            for line in &geometry.lines {
                let height = line.size(geometry.line_height).height;
                if origin.y + y + height >= bounds.top() && origin.y + y <= bounds.bottom() {
                    line.paint(
                        origin + point(px(0.), y),
                        geometry.line_height,
                        TextAlign::Left,
                        None,
                        window,
                        cx,
                    )
                    .expect("shaped text must paint");
                }
                y += height;
            }
            if focused && caret_visible && selection.is_empty() {
                let position = if empty {
                    point(px(0.), px(0.))
                } else {
                    geometry.position(cursor, upstream)
                };
                window.paint_quad(fill(
                    Bounds::new(origin + position, size(px(1.), geometry.line_height)),
                    rgb(theme::accent()),
                ));
            }
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_subscriptions.is_none() {
            self.focus_subscriptions = Some([
                cx.on_focus(&self.focus_handle, window, |input, _, cx| {
                    input.focused = true;
                    input.changed_selection(cx);
                }),
                cx.on_blur(&self.focus_handle, window, |input, _, cx| {
                    input.focused = false;
                    input.drag = None;
                    input.finish_composition();
                    input.reset_blink(cx);
                    cx.notify();
                }),
            ]);
        }
        let focused = self.focus_handle.is_focused(window);
        if focused != self.focused {
            self.focused = focused;
            if !focused {
                self.drag = None;
                self.finish_composition();
            }
            self.reset_blink(cx);
        }
        div()
            .w_full()
            .min_w_0()
            .flex_none()
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .border_color(rgb(if focused {
                theme::accent()
            } else {
                theme::line_strong()
            }))
            .bg(rgb(theme::paper()))
            .text_color(rgb(theme::text()))
            .text_xs()
            .line_height(rems(1.25))
            .key_context("HblankTextInput")
            .track_focus(&self.focus_handle)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::edit))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_scroll_wheel(cx.listener(Self::scroll))
            .child(TextElement { input: cx.entity() })
    }
}

fn normalize_text(text: &str, multiline: bool) -> Cow<'_, str> {
    if multiline {
        if text.contains('\r') {
            Cow::Owned(text.replace("\r\n", "\n").replace('\r', "\n"))
        } else {
            Cow::Borrowed(text)
        }
    } else if text.contains(['\r', '\n']) {
        Cow::Owned(text.replace("\r\n", " ").replace(['\r', '\n'], " "))
    } else {
        Cow::Borrowed(text)
    }
}
fn previous_grapheme(text: &str, index: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .take_while(|i| *i < index)
        .last()
        .unwrap_or(0)
}
fn next_grapheme(text: &str, index: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .find(|i| *i > index)
        .unwrap_or(text.len())
}
fn floor_grapheme(text: &str, index: usize) -> usize {
    if index >= text.len() {
        text.len()
    } else {
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|i| *i <= index)
            .last()
            .unwrap_or(0)
    }
}
fn word_at(text: &str, index: usize) -> Range<usize> {
    let index = if index == text.len() {
        previous_grapheme(text, index)
    } else {
        index
    };
    text.split_word_bound_indices()
        .find(|(i, word)| *i <= index && index < *i + word.len())
        .map_or(index..index, |(i, word)| i..i + word.len())
}
fn logical_line(text: &str, index: usize) -> Range<usize> {
    let start = text[..index].rfind('\n').map_or(0, |i| i + 1);
    let end = text[index..]
        .find('\n')
        .map_or(text.len(), |i| index + i + 1);
    start..end
}
fn from_utf16(text: &str, offset: usize, round_up: bool) -> usize {
    let mut count = 0;
    for (index, ch) in text.char_indices() {
        if count == offset {
            return index;
        }
        count += ch.len_utf16();
        if count > offset {
            return if round_up {
                index + ch.len_utf8()
            } else {
                index
            };
        }
    }
    text.len()
}
fn from_utf16_range(text: &str, range: Range<usize>) -> Range<usize> {
    let start = from_utf16(text, range.start, false);
    let end = if range.is_empty() {
        start
    } else {
        from_utf16(text, range.end, true)
    };
    start.min(end)..start.max(end)
}
fn to_utf16_range(text: &str, range: Range<usize>) -> Range<usize> {
    text[..range.start].encode_utf16().count()..text[..range.end].encode_utf16().count()
}
fn reveal_axis(scroll: Pixels, caret: Pixels, extent: Pixels, viewport: Pixels) -> Pixels {
    if caret < scroll {
        caret.max(px(0.))
    } else if caret + extent > scroll + viewport {
        (caret + extent - viewport).max(px(0.))
    } else {
        scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolling_reveals_both_edges_without_jumping_visible_caret() {
        assert_eq!(reveal_axis(px(90.), px(120.), px(1.), px(80.)), px(90.));
        assert_eq!(reveal_axis(px(90.), px(10.), px(1.), px(80.)), px(10.));
        assert_eq!(reveal_axis(px(0.), px(100.), px(1.), px(80.)), px(21.));
        assert_eq!(reveal_axis(px(0.), px(80.), px(20.), px(80.)), px(20.));
    }

    #[test]
    fn utf16_ranges_never_slice_surrogate_pairs() {
        assert_eq!(from_utf16_range("a😀z", 2..2), 1..1);
        assert_eq!(from_utf16_range("a😀z", 2..3), 1..5);
        assert_eq!(from_utf16_range("a😀z", 100..200), 6..6);
    }

    #[test]
    fn word_and_line_selection_preserve_newline_boundaries() {
        assert_eq!(word_at("one two", 7), 4..7);
        assert_eq!(logical_line("one\ntwo\n", 4), 4..8);
        assert_eq!(logical_line("one\ntwo\n", 8), 8..8);
    }

    #[test]
    fn soft_wrap_affinity_distinguishes_line_end_from_next_line_start() {
        let geometry = TextGeometry {
            lines: Vec::new(),
            starts: Vec::new(),
            rows: [0..4, 4..8, 9..9]
                .into_iter()
                .map(|range| VisualRow {
                    line: 0,
                    range,
                    x: px(0.),
                    width: px(40.),
                })
                .collect(),
            line_height: px(20.),
            width: px(40.),
            text: "abcdefgh\n".into(),
            style: TextStyle::default(),
            rem_size: px(16.),
            wrap_width: Some(px(40.)),
            marked: None,
        };
        assert_eq!(geometry.caret_row(4, true), 0);
        assert_eq!(geometry.caret_row(4, false), 1);
        assert_eq!(geometry.caret_row(9, true), 2);
        assert_eq!(geometry.row_at_y(px(-20.)), 0);
        assert_eq!(geometry.row_at_y(px(60.)), 2);
    }
}
