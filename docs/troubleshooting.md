# Troubleshooting

[Documentation home](README.md) | [Project README](../README.md)

Start with the first section that matches what you can observe. Fix the source, configuration, or dependency graph. Do not edit `.hblank/generated/fixtures.rs`.

## A fixture does not appear

Check these in order:

1. `--project` points at the package that owns the fixture.
2. The fixture path matches a `fixtures` glob in `.hblank/config.toml`.
3. No `ignore` glob removes it.
4. The file is valid UTF-8 and is a regular file.
5. The file registers a component and at least one fixture variant.
6. `hblank list --project PATH` includes the expected records.

The default configuration is:

```toml
fixtures = ["src/**/*.hblank.rs"]
ignore = ["target/**", ".hblank/**"]
```

Hblank does not follow symlinks during discovery.

## `--fixture` rejects a path

Relative fixture paths start at `--project`, not the shell's current directory.

The file must already exist and must match a configured fixture glob. Hblank reports this case as:

```text
not matched by the configured fixture file patterns
```

Use an absolute path if the relative root is unclear, then fix the project path once the file opens.

## `--fixture-id` rejects an ID

Run:

```bash
hblank list --project PATH
```

Copy the second column from a `fixture` record. IDs are case-sensitive `path#function` values. A component record has its own ID, but that ID does not identify a renderable variant.

Do not add an alias or an explicit ID. Rename the fixture function only if you want its canonical ID to change.

`--fixture` and `--fixture-id` cannot be used together.

## The host compiles but the preview does not

Fixture files compile as modules of the private preview crate.

- Import production items through `hblank_project::...`.
- Import GPUI through `hblank::gpui::...`.
- Do not use `crate::...` when you mean the host package.
- Make host types and render functions public.
- Add fixture-only dependencies to `.hblank/Cargo.toml`.

Run the build through `hblank dev` or `hblank list` so the error points at the generated preview crate.

## GPUI types do not match

If `App`, `Window`, or element types look identical but Rust rejects them, inspect the dependency sources.

The host, `hblank`, and `.hblank/Cargo.toml` must resolve GPUI to the same package identity. Fix the dependency graph instead of adding conversions between `App`, `Window`, or elements.

The private preview needs `test-support` on both `gpui` and `hblank`.

Read [Crates and GPUI backends](crates.md#gpui-backend) for manifest examples.

## `HblankProps` fails to derive

| Error | Cause | Fix |
|---|---|---|
| `HblankProps requires named fields` | Tuple or unit struct | Use a named-field props struct |
| Missing `ControlField` | Unsupported field type | Use `#[hblank(skip)]`, a domain adapter, or fixture-only props |
| Clone bound failure | Props cannot be cloned for editing and reset | Implement `Clone` or use a smaller fixture props type |
| Enum variant contains data | `HblankEnum` only accepts unit variants | Map the data to a unit enum used by the fixture |

Do not remove the derive just to make the build pass. The fix should state how the field participates in the fixture.

## A control rejects a value

Hblank validates controls before changing props.

- Integer fields reject fractional values.
- Numeric fields reject non-finite values.
- Rust numeric bounds still apply.
- Configured `min`, `max`, and `step` values apply.
- Enum values must match a generated option.
- Text, boolean, number, and enum values cannot be assigned across kinds.

If a domain adapter is involved, check both conversion methods with the same value that failed in the window.

## Docs are empty or missing content

Put component Rustdoc on the `#[hblank::component]` function. Put variant notes on the `#[hblank::fixture]` function. Put control help on props fields.

Without `docs = function`, Hblank generates Rustdoc, props, controls, and source blocks.

With `docs = function`, the returned `DocPage` is the whole page. Add every block you expect to see. Hblank does not merge the generated page into it.

Source blocks show normalized declarations captured by the macros. They do not preserve original whitespace or ordinary comments.

## A custom docs block does not render

Check that:

1. the renderer has `#[hblank::doc_block]`
2. `custom_doc!` points at that function path
3. the renderer returns `hblank::gpui::AnyElement`
4. the payload is a string
5. the fixture file containing the registration is discovered

A custom renderer only receives read-only `DocContext` and its payload. It cannot reach Hblank's app entity or mutate controls.

## Theme changes only Hblank's UI

This is expected until the project registers a theme hook.

The hook must use `#[hblank::theme_hook]` and this signature:

```rust
fn(
    hblank::ThemeMode,
    hblank::ResolvedTheme,
    &mut hblank::gpui::App,
)
```

Set its fully qualified path in `.hblank/config.toml`:

```toml
theme_hook = "my_app::apply_hblank_theme"
```

If the status says the configured hook is not registered, the path in TOML does not match the macro-generated Rust path.

## Reload does not update the window

1. Confirm `hblank dev` is still running.
2. Read the compiler error in its terminal.
3. Confirm the edited file is Rust source, Cargo input, or Hblank config under the project.
4. Wait for `Reloaded N Hblank fixture files`.
5. Inspect the GPUI window, not only terminal output.

The previous window stays open after a failed build. That does not mean the new code loaded.

## Reload keeps repeating

Hblank fingerprints watched source content and ignores targets, Git state, generated imports, and runtime state.

Repeated builds mean another process is changing a watched file. Look for formatters, code generators, or tools that rewrite Rust, Cargo manifests, lockfiles, or `.hblank/config.toml`.

Increasing the debounce hides the symptom and usually makes the loop slower. Find the writer instead.

## `hblank test` runs zero tests

Hblank only runs explicit `#[test]` and `#[gpui::test]` functions inside discovered fixture files.

Check that the test is:

- inside a matched fixture file
- under `#[cfg(test)]` if it lives in a test module
- named closely enough to match `--filter`, if provided

Hblank does not create one test per fixture.

## `#[gpui::test]` fails before the test runs

A current private preview needs:

- `test-support` on the direct `gpui` dependency
- `test-support` on `hblank`
- a direct dependency named `gpui`
- `pub use hblank::gpui;` in `.hblank/src/main.rs`

Older previews may be missing one of these lines. Compare them with the templates described in [Migrating to 0.3](migration-0.3.md).

## Still stuck

Collect three pieces of evidence before changing code again:

```bash
hblank --version
hblank list --project PATH
hblank dev --project PATH --fixture PATH
```

Keep the complete compiler or runtime error. The first useful line is often above the final Cargo summary.
