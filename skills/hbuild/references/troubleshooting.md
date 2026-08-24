# Troubleshooting Hbuild workflows

## Fixture does not appear

Check in this order:

1. the path is under the project passed to `--project`;
2. the path matches an `examples` glob;
3. no `ignore` glob excludes it;
4. the file is valid UTF-8 and is a regular file, not a followed symlink;
5. generated imports contain its path after `hblank dev` starts;
6. the file contains at least one `#[hblank::example]` function.

Never patch `.hblank/generated/examples.rs`; fix config or source discovery.

## `--fixture` rejects the path

- Relative paths resolve from `--project`, not the shell's current directory.
- The path must already exist so Hblank can canonicalize it.
- Existing files that do not match discovery fail with “not matched by the configured example patterns.”
- Use `hblank dev --help` to confirm current syntax.

## Fixture compiles in the host but not the preview

Fixture files compile as modules of the private preview crate, not as modules of the host crate.

- Import production items through `hblank_project::...`.
- Import GPUI through `hblank::gpui::...` or an explicit preview dependency.
- Do not use `crate::...` expecting the host crate.
- Ensure public host types/functions are actually exported.

## Derive fails

| Error shape | Cause | Fix |
|---|---|---|
| `HblankProps requires named fields` | tuple/unit struct | use named fixture props |
| missing `ControlField` | unsupported field type | add a fixture adapter with supported fields |
| clone/default bound failure | example builder cannot create/reset props | implement `Clone` and `Default` |
| enum variant contains data | `HblankEnum` only supports unit variants | map to a fixture-only unit enum |
| duplicate example id | inventory registry collision | assign a stable unique id |

Do not suppress derives or silently omit fields.

## Control rejects an update

- Integer controls reject fractional and out-of-range numbers.
- All numeric controls reject non-finite values.
- Enum values must exactly match generated option labels.
- Value kinds are strict: text cannot be sent to a boolean or numeric control.

Fix the control or adapter value; do not coerce invalid data silently.

## Docs are empty

Put `///` comments directly above the function carrying `#[hblank::example]`. Put control help directly above named props fields. Comments elsewhere are not the metadata captured by those derives/macros.

## Reload does not update

1. confirm `hblank dev` is still running;
2. read the compiler error in its terminal;
3. confirm the edited file is a watched Rust/config/manifest input;
4. wait for a successful build and “Reloaded … examples” output;
5. inspect the actual GPUI component, not only process output.

The previous preview intentionally stays open on build failure.

## Reload repeats

Hblank fingerprints relevant source content and ignores generated/build state. Repeated builds indicate a relevant input is actually changing. Inspect project generators or tools rewriting Rust, Cargo, or Hblank config files; do not increase debounce as a first response.
