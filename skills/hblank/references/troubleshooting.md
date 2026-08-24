# Troubleshooting Hblank workflows

## Fixture does not appear

Check in this order:

1. the path is under the project passed to `--project`;
2. the path matches a configured fixture file glob;
3. no `ignore` glob excludes it;
4. the file is valid UTF-8 and is a regular file, not a followed symlink;
5. generated imports contain its path after `hblank dev` starts;
6. the discovered registry includes a `#[hblank::component]` renderer and at least one `#[hblank::fixture(component = renderer, ...)]` factory that references it.

Never patch `.hblank/generated/fixtures.rs`; fix config or source discovery.

## `--fixture` rejects the path

- Relative paths resolve from `--project`, not the shell's current directory.
- The path must already exist so Hblank can canonicalize it.
- Existing files that do not match discovery fail with “not matched by the configured fixture file patterns.”
- Use `hblank dev --help` to confirm current syntax.

## `--fixture-id` rejects the id

Run `hblank list --project PATH` and copy a `fixture` record's canonical id exactly. Fixture ids are case-sensitive project-relative `path#function` values; component ids are not launchable variants. Do not add aliases or hand-derive ids. `--fixture` and `--fixture-id` are mutually exclusive.

## Fixture compiles in the host but not the preview

Fixture files compile as modules of the private preview crate, not as modules of the host crate.

- Import production items through `hblank_project::...`.
- Import GPUI through `hblank::gpui::...` or an explicit preview dependency.
- Do not use `crate::...` expecting the host crate.
- Ensure public host types/functions are actually exported.

## GPUI types or backend features conflict

- `enable exactly one GPUI backend feature` means both `crates-io-gpui` and `zed-gpui`, or neither, reached the `hblank` crate.
- Import fixture GPUI types from `hblank::gpui`; do not mix them with a different GPUI package identity.
- Compare the host, `hblank`, and `.hblank/Cargo.toml` GPUI source and revision. Crates.io GPUI and Zed Git GPUI types are not interchangeable even when their APIs look alike.
- Zed-backed previews require `default-features = false` plus `features = ["zed-gpui", "test-support"]` on `hblank`, and a matching direct `gpui` dependency.

Fix the dependency graph. Do not add conversion wrappers around mismatched `App`, `Window`, or element types.

## Derive fails

| Error shape | Cause | Fix |
|---|---|---|
| `HblankProps requires named fields` | tuple/unit struct | use named fixture props |
| missing `ControlField` | unsupported field type | add a fixture adapter with supported fields |
| clone bound failure | fixture state cannot be copied for controls/reset | implement `Clone` |
| enum variant contains data | `HblankEnum` only supports unit variants | map to a fixture-only unit enum |
| duplicate component or fixture id | two registrations use the same source path and function symbol | rename or remove the duplicate registration |

Do not suppress derives or silently omit fields.

## Control rejects an update

- Integer controls reject fractional and out-of-range numbers.
- All numeric controls reject non-finite values.
- Enum values must exactly match generated option labels.
- Value kinds are strict: text cannot be sent to a boolean or numeric control.

Fix the control or adapter value; do not coerce invalid data silently.

## Docs are empty

Component Rustdoc belongs on the `#[hblank::component]` function; fixture Rustdoc belongs on each `#[hblank::fixture]` factory for variant-specific notes; props field Rustdoc becomes control help.

Without `docs = path`, Hblank generates the component page from Rustdoc, props, controls, and captured source. With `docs = path`, that `DocPage` is authoritative: add every desired `DocBlock` explicitly. Source blocks show project-relative locations and normalized declaration tokens, not original formatting or ordinary comments.

## Theme changes only Hblank chrome

This is expected without a project hook. To switch the previewed component, register one function with `#[hblank::theme_hook]`, configure its fully qualified Rust path as `theme_hook`, and keep the required `fn(ThemeMode, ResolvedTheme, &mut App)` signature. The status `Configured theme hook '…' is not registered` means the configured path and macro-generated registration id do not match.

## Reload does not update

1. confirm `hblank dev` is still running;
2. read the compiler error in its terminal;
3. confirm the edited file is a watched Rust/config/manifest input;
4. wait for a successful build and “Reloaded … fixture files” output;
5. inspect the actual GPUI component, not only process output.

The previous preview intentionally stays open on build failure.

## `hblank test` runs no tests or fails before execution

- Hblank runs only explicit inline `#[test]` and `#[gpui::test]` functions in discovered fixture files; it does not synthesize fixture smoke tests.
- Confirm `.hblank/Cargo.toml` enables `test-support` on both `gpui` and `hblank` and keeps a direct `gpui` dependency.
- Confirm `.hblank/src/main.rs` exposes `pub use hblank::gpui;` for `#[gpui::test]` expansion.
- Use `hblank test --project PATH --filter NAME` to isolate one test while preserving ordinary Cargo diagnostics.

## Reload repeats

Hblank fingerprints relevant source content and ignores generated/build state. Repeated builds indicate a relevant input is actually changing. Inspect project generators or tools rewriting Rust, Cargo, or Hblank config files; do not increase debounce as a first response.
