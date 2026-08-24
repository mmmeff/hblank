# Hblank framework reference

## Naming

The installable skill, framework packages, and command are all named `hblank`:

| Role | Name |
|---|---|
| runtime crate | `hblank` |
| procedural macro crate | `hblank-macros` |
| CLI package | `hblank-cli` |
| executable | `hblank` |
| fixture files | `*.hblank.rs` by default |
| host-crate alias inside preview | `hblank_project` |

Always confirm these with the checked-out repository; use code truth if a later version renames them.

## Project layout

`hblank init` creates a private preview crate without modifying host source:

```text
.hblank/
├── config.toml
├── Cargo.toml
├── src/main.rs
├── generated/fixtures.rs
└── target/
```

The host package remains the source of production components. The preview crate depends on both `hblank` and the host package under the alias `hblank_project`.

## Discovery

Configuration is project-root-relative:

```toml
fixtures = ["src/**/*.hblank.rs"]
ignore = ["target/**", ".hblank/**"]

[window]
title = "my-app · Hblank"
width = 1440
height = 900
```

The CLI walks files without following symlinks, normalizes paths, applies include and ignore glob sets, sorts relative paths, and gives each imported module a path-derived stable name. Duplicate file stems are safe.

`hblank dev` rewrites generated imports only when discovered content changes. Generated source uses absolute `#[path]` imports so fixture files can live anywhere under the project root.

## Registration

`#[hblank::component]` registers one typed renderer and its title, group, Rustdoc, source, and canonical `path#function` id. `#[hblank::fixture]` registers a named default-props variant against that component.

The generated component adapter downcasts dynamic props to the component's declared type and converts its output to `gpui::AnyElement`. The fixture factory takes no arguments and returns the same props type; the macro enforces that relationship at compile time.

The runtime joins components and variants, rejects duplicate canonical ids, unknown components, and mismatched props types, then sorts by group/component/variant/id.

## Controls

`HblankProps` exposes static `ControlDefinition` metadata, current typed values, mutation, cloning, and downcasting. The derive reads named struct fields in declaration order.

`HblankEnum` supports unit variants. Use `#[hblank(label = "High contrast")]` on enum variants or fields when identifier humanization is insufficient.

Numeric values pass through `f64` in the control model. Integer updates reject non-finite, fractional, and out-of-range values rather than truncating silently.

## Harness state

The GPUI harness owns selection, filtering, active inspector tab, editing target, and mutable fixture props. Presentational harness functions live separately under `hblank::harness`.

Selection and filter persist in `.hblank/state.toml`. `hblank dev --fixture PATH` passes `HBLANK_INITIAL_FIXTURE` only to the first preview process. The harness gives that source path priority, clears an excluding filter, persists the resulting id, and then lets later reloads restore normal user state.

## Reload lifecycle

The CLI watches project Rust files, manifests, lockfiles, and Hblank config while excluding targets, Git state, generated imports, and Hblank runtime state.

On relevant content change:

1. compute a source fingerprint to discard duplicate filesystem events;
2. reload config when needed;
3. rediscover and regenerate imports;
4. run `cargo build` for the private preview crate;
5. spawn the new preview only after a successful build;
6. stop the old preview after replacement is ready.

A failed build leaves the old preview alive. Hblank deliberately avoids unstable Rust dynamic-library ABIs.
