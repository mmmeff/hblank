# Crates and GPUI backends

[Documentation home](README.md) | [Project README](../README.md)

Hblank is split into four crates so the component model does not depend on one UI framework.

## Crate map

| Crate | What it owns |
|---|---|
| `hblank-core` | Props, controls, component and fixture metadata, catalog assembly, docs, themes, canonical IDs, and registry errors |
| `hblank-macros` | `HblankProps`, `HblankEnum`, component, fixture, docs, theme, and typed-handle macros |
| `hblank` | GPUI adapter, inventory registration, rendering, desktop catalog, session state, theme hooks, and test helpers |
| `hblank-cli` | `hblank` executable, initialization, discovery, generated imports, builds, watching, listing, and tests |

The crates use one lockstep version.

## `hblank-core`

`hblank-core` has no GPUI dependency. It defines the data and behavior another UI adapter would need:

- `HblankProps` and `HblankEnum`
- `ControlDefinition`, `ControlKind`, `ControlValue`, and validation errors
- `HblankControlAdapter<T>`
- component and fixture metadata
- `RegisteredCatalog` and catalog assembly
- canonical `path#function` IDs
- `DocPage`, `DocBlock`, and `CalloutTone`
- `ThemeMode` and `ResolvedTheme`

Application code normally imports these through `hblank`, which re-exports the public core types. An adapter for another Rust UI framework can depend on `hblank-core` directly.

## `hblank-macros`

The macro crate generates metadata and adapters at compile time.

| Macro | Job |
|---|---|
| `#[derive(HblankProps)]` | Generate control metadata, reads, writes, cloning, and downcasting for named props fields |
| `#[derive(HblankEnum)]` | Generate labels and conversion for a unit enum |
| `#[hblank::component]` | Register a typed renderer and component metadata |
| `#[hblank::fixture]` | Register one named props factory against a component |
| `hblank::fixture_ref!` | Resolve a fixture function to its generated canonical ID helper |
| `#[hblank::doc_block]` | Register a native GPUI docs renderer |
| `hblank::custom_doc!` | Store a custom docs renderer path and string payload |
| `#[hblank::theme_hook]` | Register a project theme callback |
| `hblank::render_handle!` | Call the typed render helper generated for a component with `handle = Type` |

Component and fixture macros also capture normalized declaration tokens for the source block in Docs.

## `hblank`

The runtime crate is the complete GPUI adapter. It:

- re-exports GPUI as `hblank::gpui`
- collects macro registrations through `inventory`
- validates and sorts the catalog
- renders component fixtures, controls, docs, and themes
- runs the desktop window
- saves selection, filter, theme, and current-session control values
- exposes `Rendered<Handle, Content>` and test helpers

Fixture files should import GPUI types from `hblank::gpui`. This keeps their `App`, `Window`, and element types tied to the GPUI crate Hblank compiles against.

## `hblank-cli`

The CLI crate builds the `hblank` executable. It does not render components itself. It creates and supervises a private preview binary that depends on the host package and `hblank` runtime.

Read the [CLI reference](cli.md) for command behavior.

## GPUI backend

Hblank uses GPUI 0.2.2 from crates.io and re-exports it as `hblank::gpui`.

For a normal host dependency:

```toml
[dependencies]
gpui = "0.2.2"
hblank = "0.4.0"
```

Fresh `hblank init` output enables `test-support` in the private preview:

```toml
[dependencies]
gpui = { version = "0.2.2", features = ["test-support"] }
hblank = { version = "0.4.0", features = ["test-support"] }
hblank_project = { package = "your-package", path = ".." }
```

The host, runtime, and preview must resolve GPUI to the same package identity. Fix the dependency graph instead of adding conversions between `App`, `Window`, or elements.

The generated preview main re-exports the backend:

```rust
pub use hblank::gpui;
```

Keep that line. The `#[gpui::test]` macro needs a crate named `gpui` in the preview.

## The private preview crate

`.hblank/Cargo.toml` is intentionally separate from the host package. It compiles fixture files as preview modules and imports the host library as `hblank_project`.

This split has a few consequences:

- `crate::...` inside a fixture points at the preview crate, not the host.
- Host items used by fixtures must be public.
- Fixture-only dependencies belong in `.hblank/Cargo.toml`.
- Generated fixture imports belong to Hblank and should not be edited.
- The preview has its own target directory and lockfile behavior.

## Publishing order

The four crates publish in dependency order:

1. `hblank-core`
2. `hblank-macros`
3. `hblank`
4. `hblank-cli`

The release script waits for crates.io to index each crate before publishing the next one. Read [Releasing](releasing.md) for the full process.
