# Getting started

[Documentation home](README.md) | [Project README](../README.md)

This guide adds Hblank to an existing GPUI package and opens one component in the desktop catalog.

## Requirements

- Rust 1.85 or newer
- A Rust package with a library target
- GPUI 0.2.2
- Public component types and render functions that the private preview crate can import

## Install the CLI and runtime

Install the CLI from crates.io:

```bash
cargo install hblank-cli
```

Add the runtime to the GPUI package that owns your components:

```bash
cd /path/to/your-project
cargo add hblank
```


## Create the private preview crate

Run `hblank init` from the package root:

```bash
hblank init
```

For a package elsewhere in a workspace:

```bash
hblank init --project crates/ui
```
Hblank creates these files and refuses to replace any of them if they already exist:

```text
.hblank/
├── .gitignore
├── config.toml
├── Cargo.toml
├── src/main.rs
└── generated/fixtures.rs
```

The host manifest and source files stay untouched. `.hblank/Cargo.toml` depends on the host package as `hblank_project`, which is the import name used inside fixture files.

The preview uses GPUI 0.2.2 from crates.io.

## Write a component

Suppose the host package exports this component from `src/lib.rs`:

```rust
use gpui::{App, Div, Window, div, prelude::*, rgb};
use hblank::HblankProps;

#[derive(Clone, HblankProps)]
pub struct NoticeProps {
    /// Message shown to the user.
    pub message: String,
    /// Applies the emphasized treatment.
    pub emphasized: bool,
}

impl Default for NoticeProps {
    fn default() -> Self {
        Self {
            message: "Saved".to_owned(),
            emphasized: false,
        }
    }
}

pub fn notice(props: &NoticeProps, _window: &mut Window, _cx: &mut App) -> Div {
    div()
        .px_3()
        .py_2()
        .rounded_md()
        .when(props.emphasized, |notice| notice.bg(rgb(0x7357d8)))
        .child(props.message.clone())
}
```

`HblankProps` reads named fields in declaration order. `String` becomes a text editor and `bool` becomes a toggle.

## Add the fixture file

The default discovery pattern is `src/**/*.hblank.rs`. Create `src/notice.hblank.rs`:

```rust
use hblank::gpui::{App, IntoElement, Window};
use hblank_project::{NoticeProps, notice};

#[hblank::component(title = "Notice", group = "Components")]
/// A short message that confirms an operation or explains what happened.
fn notice_component(
    props: &NoticeProps,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    notice(props, window, cx)
}

#[hblank::fixture(component = notice_component, title = "Default")]
fn notice_default() -> NoticeProps {
    NoticeProps::default()
}

#[hblank::fixture(component = notice_component, title = "Emphasized")]
fn notice_emphasized() -> NoticeProps {
    NoticeProps {
        emphasized: true,
        ..NoticeProps::default()
    }
}
```

A component registration owns the renderer, props type, title, group, and docs. Each fixture function returns one named starting state for that component.

## Check discovery before opening the window

Run:

```bash
hblank list
```

The output contains component and fixture records separated by tabs. For the example above, the fixture IDs end with:

```text
src/notice.hblank.rs#notice_default
src/notice.hblank.rs#notice_emphasized
```

IDs come from the project-relative file path and Rust function name. Do not assign IDs by hand.

## Open the component

Launch the first fixture from the file:

```bash
hblank dev --fixture src/notice.hblank.rs
```

Or launch one exact state:

```bash
hblank dev --fixture-id 'src/notice.hblank.rs#notice_emphasized'
```

Relative fixture paths start at `--project`. Absolute paths work too. The file must match a configured fixture glob.

## Prove the edit loop

With `hblank dev` still running:

1. Change the message control and toggle emphasized mode.
2. Open Docs and confirm the component Rustdoc appears.
3. Edit a visible label or style in the host component.
4. Save the Rust file.
5. Wait for the preview to rebuild and confirm the window changed.

A compile error leaves the previous preview open. Fix the error and save again. You do not need to restart `hblank dev`.

## Next steps

- [Components and fixtures](authoring.md)
- [CLI reference](cli.md)
- [Testing components](testing.md)
- [Troubleshooting](troubleshooting.md)
