# CLI reference

[Documentation home](README.md) | [Project README](../README.md)

The `hblank` executable creates the private preview crate, finds fixtures, runs the GPUI catalog, and executes fixture tests.

## Command summary

```text
hblank init [--project PATH] [--runtime-path PATH]
hblank dev [--project PATH] [--fixture PATH | --fixture-id ID]
hblank list [--project PATH]
hblank test [--project PATH] [--filter FILTER]
```

Run `hblank COMMAND --help` for the help text installed with your version.

## Project paths

Every command accepts `--project PATH`. The default is the current directory.

The project path must point at the Rust package that owns `.hblank/config.toml`, not the workspace root unless the workspace root is also that package.

Fixture paths passed to `--fixture` behave differently from shell paths:

- Relative paths start at `--project`.
- Absolute paths are accepted.
- The file must exist.
- The file must match one of the fixture globs in `.hblank/config.toml`.

Canonical IDs passed to `--fixture-id` are always project-relative `path#function` values printed by `hblank list`.

## `hblank init`

```text
hblank init [--project PATH] [--runtime-path PATH]
```

`init` creates a private preview package under `.hblank/`:

```text
.hblank/
├── .gitignore
├── config.toml
├── Cargo.toml
├── src/main.rs
└── generated/fixtures.rs
```

It does not edit the host manifest or host source.

### `--runtime-path PATH`

Use a local `hblank` runtime instead of the published crate:

```bash
hblank init --runtime-path /path/to/hblank/crates/hblank
```

The path must contain the runtime crate's `Cargo.toml`. Hblank normalizes relative runtime paths from the project root.

### Existing files

`init` is all or nothing. If any generated path already exists, the command reports those paths and writes nothing. Inspect a partial `.hblank/` directory instead of deleting it blindly.

## `hblank dev`

```text
hblank dev [--project PATH] [--fixture PATH | --fixture-id ID]
```

`dev` performs this sequence:

1. load and validate `.hblank/config.toml`
2. discover matched fixture files
3. regenerate `.hblank/generated/fixtures.rs` when discovery changed
4. build `.hblank/Cargo.toml`
5. start the preview process
6. watch Rust, Cargo, and Hblank configuration files

The command keeps running until the preview exits or you stop it.

### Open a source file

```bash
hblank dev --fixture src/button.hblank.rs
```

This opens the first fixture from that source file in catalog order.

When one file has several variants, use `hblank list` to see which variant sorts first.

### Open an exact fixture

```bash
hblank dev --fixture-id 'src/button.hblank.rs#button_disabled'
```

Hblank builds the preview, checks the runtime registrations, and rejects an unknown ID before opening the window.

`--fixture` and `--fixture-id` cannot be combined.

### Reload behavior

Hblank watches:

- project Rust files
- Cargo manifests and lockfiles
- `.hblank/config.toml`

It ignores build targets, Git state, generated fixture imports, and Hblank runtime state.

On a relevant change, Hblank fingerprints the source to drop duplicate filesystem events, reloads configuration when needed, rediscovers fixtures, regenerates imports, and builds the private preview.

A successful build starts the replacement preview before stopping the old one. A failed build prints the compiler error and keeps the previous preview alive.

Hblank uses supervised process replacement. It does not load Rust dynamic libraries.

### Development session state

One `hblank dev` command owns one session ID. Successful rebuilds restore:

- selected fixture
- search text
- selected System, Light, or Dark mode
- non-default control values

A new `hblank dev` command keeps persisted selection and filter where possible, but starts controls from the fixture functions and resets the theme to System.

The `--fixture` or `--fixture-id` request applies to the first preview process only. Later rebuilds preserve the current selection.

### Window controls

- `↑` and `↓` move through filtered fixtures.
- `Esc` clears the active search filter.
- `Cmd` plus `=` or `+` zooms in.
- `Cmd` plus `-` zooms out.
- Linux uses `Super` where macOS uses `Cmd`. Windows uses `Win`.

## `hblank list`

```text
hblank list [--project PATH]
```

`list` builds the private preview, runs it in catalog mode without opening a window, and prints tab-separated records.

A component record has four columns:

```text
component	COMPONENT_ID	TITLE	GROUP
```

A fixture record has four columns:

```text
fixture	FIXTURE_ID	COMPONENT_ID	TITLE
```

Example:

```text
component	src/button.hblank.rs#button_component	Button	Components
fixture	src/button.hblank.rs#button_default	src/button.hblank.rs#button_component	Default
fixture	src/button.hblank.rs#button_disabled	src/button.hblank.rs#button_component	Disabled
```

The output order is stable. Hblank sorts by group, component title, variant title, and canonical ID.

Use the fixture record's second column with `--fixture-id`. A component ID does not identify a renderable variant.

## `hblank test`

```text
hblank test [--project PATH] [--filter FILTER]
```

`test` rediscovers fixtures, regenerates preview imports, and runs `cargo test` against `.hblank/Cargo.toml` and its target directory.

Hblank runs only explicit tests written inside discovered fixture files. It does not generate a test for every fixture.

### `--filter FILTER`

Pass one Cargo test-name filter:

```bash
hblank test --filter button_disabled_uses_muted_style
```

Cargo still owns test discovery, output, and exit status.

Read [Testing components](testing.md) for `#[gpui::test]`, typed render handles, and drawing helpers.

## Configuration errors

The CLI rejects:

- an empty `fixtures` list
- empty fixture patterns
- an empty `theme_hook` value
- an empty window title
- zero window width or height
- unknown configuration keys

See [Troubleshooting](troubleshooting.md) for discovery and build failures.
