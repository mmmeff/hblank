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

The runtime joins components and variants, rejects duplicate canonical ids, unknown components, and mismatched props types, then sorts by group/component/variant/id. The native catalog sidebar renders that hierarchy directly; keyboard and text filtering still operate on the flattened selectable variant order. Docs combine component Rustdoc with optional variant notes and point source context at the component renderer.

## Controls

`HblankProps` exposes static `ControlDefinition` metadata, current typed values, validated mutation, cloning, and downcasting. The derive reads named struct fields in declaration order; `#[hblank(skip)]` excludes a field without requiring `ControlField`.

`String` controls are single-line by default; `#[hblank(multiline)]` selects the multiline editor. Numeric values pass through `f64`; `min`, `max`, and positive `step` literals become enforced metadata and drive direct input plus steppers. Integer updates still reject non-finite, fractional, and out-of-range values rather than truncating.

`HblankEnum` supports unit variants. Use `#[hblank(label = "High contrast")]` when identifier humanization is insufficient. Small enums render as chips and larger enums as a compact list.

`HblankControlAdapter<T>` maps a project domain type onto a built-in carrier type. `#[hblank(adapter = AdapterType)]` makes the derive call that adapter while preserving Hblank-owned metadata validation and editor behavior; adapters do not provide arbitrary control rendering.

## Documentation pages

`ComponentDefinition` owns a framework-neutral `DocPage`. The optional `docs = path` component attribute calls a typed Rust builder; `DocBlock` supports heading, prose, fixture, props, controls, callout, and source blocks in declared order. `fixture_ref!` rewrites a fixture function path to its generated canonical-id helper at compile time.

The GPUI adapter renders fixture blocks as live canvases, props as generated metadata, and controls through the same mutation handlers as the inspector. Components without an authored page receive generated Rustdoc, props, controls, and source blocks. `#[hblank::doc_block]` registers a GPUI renderer under its Rust module path; `custom_doc!` resolves that path at compile time and stores a string payload. Renderers receive only read-only `DocContext` metadata and cannot access harness navigation, controls, or persistence.

## Themes

The harness owns a three-state `ThemeMode`: System, Light, and Dark. System resolves from GPUI `WindowAppearance` and observes changes; an override lasts only for the current supervised dev session. Dynamic chrome colors come from Hblank's light/dark palettes.

Projects opt into preview theme switching with one `#[hblank::theme_hook]` function whose fully qualified Rust path is configured as `theme_hook`. The macro registers `fn(ThemeMode, ResolvedTheme, &mut App)`; the CLI passes the configured path to every preview process. Missing hooks produce harness status instead of silently claiming the component switched.

## Harness state

The GPUI harness owns selection, filtering, active inspector tab, editing target, and mutable fixture props. Presentational harness functions live separately under `hblank::harness`.

Selection and filter persist in `.hblank/state.toml`. The CLI assigns one `HBLANK_SESSION_ID` to every preview process it supervises. Non-default control values are serialized by canonical fixture/control id and reapplied only when that session id matches, so successful rebuilds retain live work while a new `hblank dev` starts from source defaults. Stale or invalid values are ignored. `hblank dev --fixture PATH` still gives the requested source priority only for the first preview process.

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




