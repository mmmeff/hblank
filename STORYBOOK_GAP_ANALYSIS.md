# Storybook feature audit and Hblank gap analysis

## Scope and classification

This document compares the current Storybook product surface with Hblank's implemented Rust/GPUI component-workbench surface, identifies the gaps, and ranks the lowest-friction improvements. Repository claims are based on the code and tests listed in the Hblank evidence catalog; Storybook claims use first-party sources only.

- **Snapshot:** 2026-08-24. The latest stable release was **Storybook 10.5.10**, released 2026-08-20; `latest` is Storybook's stable release channel. [S1, S2]
- **Documentation context:** unversioned `storybook.js.org/docs/...` links below were read with the current **10.5** documentation selected on 2026-08-24. They are intentionally identified as rolling current-docs URLs. The fixed release link [S1] anchors the snapshot.
- **Core / Essentials:** functionality shipped by Storybook itself, including default Essentials. Some Essentials are implemented as packages internally, but are baseline, zero-configuration workbench functionality rather than optional hosted services. [S16]
- **Official addon:** a separately enabled package documented and maintained as a Storybook addon.
- **Chromatic:** the separately installed `@chromatic-com/storybook` integration and/or Chromatic's hosted service. It is not Storybook core, even where Storybook's official docs recommend it.

## Current Storybook baseline

| Experience area | Current capability | Delivery boundary | Sources |
|---|---|---|---|
| **Story authoring** | Component Story Format (CSF) uses a default metadata export plus named story exports. Stories are declarative objects that can define args, parameters, decorators, loaders, render behavior, and `play` functions; TypeScript types can tie metadata and stories to the component. | **Core** | [S3, S4] |
| **Discovery and navigation** | Configured story globs feed Storybook's story index and sidebar. Titles/file paths form a folder/component/story hierarchy; projects can control sorting, roots, generated IDs and stable story URLs. The sidebar provides search, and tags can include or exclude stories from the sidebar and filter the visible tree. | **Core** | [S5, S6, S7, S8, S28] |
| **Args and Controls** | Args describe a story's inputs at project, component, or story scope and can be overridden by the URL. ArgTypes describe input metadata and mappings. The Controls panel derives editable controls from args/ArgTypes, supports explicit control types and conditional controls, and updates the rendered story interactively. | **Core / Essentials** | [S9, S10, S11] |
| **Actions and layout inspection** | The Actions panel records callback invocations and arguments. Measure reports element dimensions and box-model spacing, while Outline exposes rendered element boundaries; all three are enabled core features in Storybook 10. | **Core / Essentials** | [S40, S41, S42] |
| **Documentation** | Autodocs generates component documentation from stories, metadata, args and ArgTypes when enabled by the `autodocs` tag. MDX supports authored documentation pages that combine Markdown/JSX with stories, while Doc Blocks supply reusable rendered API, source, controls and story sections. | **Core / Essentials (`addon-docs`)** | [S12, S13, S14, S15] |
| **Interaction testing** | A story's `play` function expresses user flows with Testing Library-style queries and assertions. The Test/Vitest addon turns stories into browser-mode component tests, runs them from the Storybook UI (including filtered/grouped runs and debugging), and supports CLI/CI execution. | **Core authoring model + official `@storybook/addon-vitest`** | [S17, S18] |
| **Test coverage** | With the Vitest addon and coverage provider configured, Storybook can collect code coverage for executed story tests, display results, emit reports, and enforce coverage through Vitest/CI. This coverage is scoped to tests run through the Storybook Vitest project, not unrelated test projects. | **Official `@storybook/addon-vitest`** | [S19] |
| **Accessibility** | The accessibility addon runs axe-based checks against the active story, reports violations in an Accessibility panel, supports per-story rule configuration and manual/automatic/test enforcement modes, and can participate in Vitest-addon runs. | **Official `@storybook/addon-a11y`** | [S20] |
| **Visual testing** | The Visual Tests addon captures stories in cloud browsers, compares screenshots against accepted baselines, displays diffs and approval controls inside Storybook, and can hand off to CI. Cross-browser capture, persisted baselines and build results are provided by Chromatic rather than the local Storybook core. | **Chromatic addon + hosted service** | [S21, C1, C2] |
| **Responsive / viewport** | The Viewport toolbar renders a story at predefined or custom dimensions, supports orientation changes, and lets projects/components/stories select or lock the active viewport through parameters/globals. | **Core / Essentials** | [S22] |
| **Backgrounds and themes** | Backgrounds supplies toolbar-selectable project-defined colors plus grid display and per-story/global selection. Theme switching is separately supplied by `@storybook/addon-themes`, with decorators for provider-, CSS-class-, or data-attribute-based themes. | **Backgrounds: Core / Essentials. Themes: official addon.** | [S23, S24] |
| **Global decorators and toolbars** | Decorators wrap rendering at global, component, or story scope and receive story context. Custom global types create toolbar controls for cross-story concerns such as locale or density; selected globals are available to decorators/rendering and can be initialized or overridden per story. | **Core** | [S25, S26] |
| **Component/story status and organization** | Custom tags can encode lifecycle labels such as experimental, stable or deprecated and drive sidebar/test filtering. Storybook 10.5 also has an enabled-by-default **preview** change-detection feature that marks new/modified stories from Git and dependency information and provides a review filter; this is change status, not a hosted approval workflow. | **Core; change detection is explicitly preview** | [S7, S8, S27] |
| **Sharing and static publishing** | `storybook build` produces a static web application that can be deployed to ordinary static hosting. Storybook's publishing guide recommends Chromatic, which builds/uploads Storybook and adds hosted URLs, build history and branch/version-aware access. | **Static build: Core. Managed publishing/history: Chromatic hosted.** | [S29, C3] |
| **Embedding and composition** | A publicly reachable story can be embedded by its iframe URL; Chromatic-hosted Storybooks additionally expose oEmbed. Storybook composition can bring published/local Storybooks into one sidebar, and package composition can resolve a library version to its matching published Storybook. | **Iframe/composition: Core. oEmbed/resolution hosting depends on publisher; Chromatic supplies hosted support.** | [S30, S31, S32] |
| **Review workflow** | Chromatic UI Review groups visual changes into changesets, supports assigned reviewers, discussion and accept/request-change decisions, and reports review status to pull requests. These collaboration and approval capabilities are hosted product functionality, not local Storybook. | **Chromatic hosted** | [C4] |
| **AI and agent integration** | Preview component/docs manifests expose structured story and prop metadata. The preview MCP addon can expose story instructions, documentation, previews, changed-story discovery and story-test execution to agents. Current docs label this surface preview and primarily React-oriented. | **Preview feature + official `@storybook/addon-mcp`** | [S43, S44] |
| **Build and development UX** | The initializer detects supported project/framework setup and installs/configures Storybook. `storybook dev` runs the interactive development server, while `storybook build` emits the deployable static site; CLI options cover configuration directory, port, host, browser opening, diagnostics and build output. Framework integrations and Vite/Webpack builders connect Storybook to the application's compilation environment. | **Core** | [S33, S34, S35] |
| **Extensibility and addons** | Addons can contribute manager-side panels, toolbar tools and tabs, preview decorators/loaders/parameters, events and state through manager/preview APIs. Preset addons can bundle configuration and alter builder/framework configuration; addons are installed into the `addons` configuration and distributed through the official catalog/npm. | **Core extension APIs + separately installed official/community addons** | [S36, S37, S38, S39] |

### Comparison boundaries

1. A comparable **local workbench** baseline should not silently count Chromatic cloud rendering, storage, publishing, review or CI services as Storybook-core behavior. [S21, S29, C1, C2, C3, C4]
2. Conversely, Controls, Docs, Viewport and Backgrounds are part of the default Essentials experience in this snapshot even though Storybook's architecture is addon-oriented. [S10, S12, S16, S22, S23]
3. Theme switching, accessibility checks and Vitest execution/coverage are official but separately enabled addons. [S18, S19, S20, S24]
4. Story lifecycle labels are project-defined tags; the preview change-detection UI reports source changes; Chromatic supplies the separate human approval workflow. [S7, S8, S27, C4]
5. Component manifests and MCP are current but preview/React-oriented. They are useful differentiation signals, not baseline parity requirements for a Rust/GPUI workbench. [S43, S44]

## Hblank capability audit

### What Hblank provides now

- **First-class authoring:** `#[hblank::component]` owns one typed renderer, props schema, catalog metadata and optional `DocPage`; zero-argument `#[hblank::fixture]` factories register named variants. Canonical ids derive from project-relative `path#function`. [H2, H4]
- **Framework adapter seam:** `hblank-core` contains the UI-framework-neutral control, catalog, documentation and theme models. `hblank` is the concrete GPUI adapter. [H2, H3]
- **Rich controls:** Boolean, single/multiline text, constrained direct numeric input, adaptive enums, skipped fields and project domain adapters are generated from Rust types. Non-default values survive successful rebuilds within one supervised dev session. [H3, H4, H5, H6]
- **Component catalog:** navigation groups components and nests variants; search and keyboard traversal operate over exact variants. The header reports component and fixture counts. [H5, H6]
- **Documentation:** generated Rustdoc/props/controls/source pages and typed `DocPage` composition support headings, prose, live fixtures, interactive controls, callouts, normalized declaration source and stable custom native blocks. [H2, H4, H5, H6]
- **Themes:** harness chrome follows GPUI's OS appearance, supports session Light/Dark overrides and can call one configured project `#[hblank::theme_hook]`. [H4, H5, H6, H7]
- **Testing:** `hblank test` runs explicit inline Rust/GPUI tests from the generated preview target. `Rendered<Handle, _>`, `render_handle!` and `hblank::testing` preserve typed project state over GPUI test contexts without adding a selector/assertion DSL. [H4, H10]
- **CLI targeting:** `hblank list` emits runtime-validated component/fixture records; `--fixture-id PATH#FUNCTION` preflights and launches one exact variant, while source-path launch remains available. [H8, H10]
- **Agent guidance:** the repository ships the Hblank skill with current authoring, testing, docs, theme and verification contracts. [H1, H9]

### Implemented parity stories

Each story is an independently verified commit on `main`:

| Commit | Parity story | Result |
|---|---|---|
| `b490df6` | First-class component definitions | Component renderers and named fixture variants replace fixture-owned rendering. |
| `0ba6635` | Framework-neutral core | Catalog/control contracts moved behind a reusable adapter seam. |
| `c12e1a5` | Rich generated controls | Bounds, steps, direct numbers, multiline text, adaptive enums and skipped fields. |
| `270f44c` | Domain control adapters | Project newtypes map onto Hblank-owned built-in editors. |
| `24846bb` | Rebuild continuity | Valid non-default control values survive only the current dev session. |
| `390484e` | System themes | Live OS appearance, Light/Dark override and configured project hook. |
| `70eb835` | Component-first catalog | Group/component/variant hierarchy with nested filtering and navigation. |
| `7072654` | Typed component docs | Generated and authored native blocks, fixture refs and live controls. |
| `51bfc7b` | Custom native doc blocks | Stable read-only context and compile-time renderer references. |
| `def64a3` | Declaration source | Normalized component/fixture declarations captured at compile time. |
| `ec466a4` | Generated fixture tests | Explicit inline tests run through the private Cargo target. |
| `142e1de` | Typed GPUI test handles | Typed render state plus low-level draw/input helpers. |
| `7fc1c2a` | Canonical fixture CLI | Runtime listing, strict exact-id preflight and direct launch. |

### Gap matrix after implementation

| Experience area | Hblank now | Assessment | Remaining gap |
|---|---|---|---|
| Story authoring | First-class component definitions, typed props, named variants, docs pages and render handles. | **Strong partial** | No loaders, arbitrary renderer wrappers, lifecycle tags or project/component inheritance model. |
| Discovery and navigation | Globs/ignores, component hierarchy, search, deterministic order, runtime listing and exact canonical launch. | **Strong partial** | No custom roots/sort, tags, Git change detection or remotely shareable location. |
| Args and Controls | Rich primitive editors, constraints, skip, domain adapters, validation, reset and session continuity. | **Strong partial** | No recursive object/collection editor, conditional visibility, scope inheritance or URL overrides. |
| Actions and layout inspection | No callback action log, element measurement or outline mode. | **Missing** | Requires GPUI-specific instrumentation; DOM implementations do not transfer. |
| Documentation | Generated catalog plus typed/custom native pages, live fixtures/controls and captured declarations. | **Strong partial** | No Markdown/MDX language, documentation-only hierarchy or static docs export. |
| Interaction testing | Explicit generated-target tests, GPUI contexts/input helpers and typed handles. | **Partial** | No public selector, user-event, wait/assertion DSL, results panel or authored play lifecycle. |
| Test coverage | Cargo tests run, but Hblank does not aggregate coverage. | **Missing** | Needs a proven interaction corpus and Rust coverage adapter. |
| Accessibility | No fixture-level audit or semantics panel. | **Missing** | Pinned GPUI has no accessibility semantics tree to consume. |
| Visual testing | No deterministic screenshot capture, baselines or image diff. | **Missing** | Pinned GPUI has no offscreen/readback screenshot contract. |
| Responsive / viewport | Project window dimensions and harness zoom only. | **Missing** | No constrained canvas presets, orientation or per-fixture viewport. |
| Backgrounds and themes | System/Light/Dark chrome and project hook are implemented. | **Partial** | No project-defined canvas backgrounds or grid. |
| Global decorators and toolbars | A narrow theme hook and fixed workbench toolbar. | **Partial** | No general globals, wrappers or arbitrary toolbar registration. |
| Component status and organization | Stable hierarchy and filtering. | **Partial** | No lifecycle status, custom tags or Git-aware change state. |
| Sharing and static publishing | Native local executable only. | **Missing** | No static/exported catalog, permalink, remote hosting or build history. |
| Embedding and composition | No remote/embed/composition model. | **Missing** | Requires a different distribution/runtime architecture. |
| Review workflow | No hosted review service. | **External** | Chromatic collaboration is not local Storybook core. |
| AI and agent integration | Hblank skill, canonical listing and deterministic test command. | **Partial / preview gap** | No stable machine manifest, changed-fixture query or MCP server. |
| Build and development UX | Init, watch/build/relaunch, failed-build resilience, session continuity, exact launch and explicit tests. | **Strong partial** | Diagnostics remain terminal-only; no static build or remote server. |
| Extensibility and addons | Stable domain-control, custom-doc and theme-hook interfaces. | **Partial** | No generic package discovery, panels, event bus, decorators or addon lifecycle. |

### Clean-cutover migration

Pre-component authoring:

```rust
#[hblank::fixture(id = "components.badge", title = "Badge", group = "Components")]
fn badge_fixture(props: &BadgeProps, window: &mut Window, cx: &mut App) -> impl IntoElement {
    badge(props, window, cx)
}
```

Current authoring:

```rust
#[hblank::component(title = "Badge", group = "Components")]
fn badge_component(props: &BadgeProps, window: &mut Window, cx: &mut App) -> impl IntoElement {
    badge(props, window, cx)
}

#[hblank::fixture(component = badge_component, title = "Default")]
fn badge_default() -> BadgeProps {
    BadgeProps::default()
}
```

Migration rules:

1. Move rendering and shared Rustdoc to one `#[hblank::component]` function.
2. Replace each old fixture renderer with a zero-argument props factory referencing that component.
3. Delete explicit ids; canonical ids are project-relative `path#function` and have no aliases.
4. Use `hblank list` before updating scripts to strict `--fixture-id` values.
5. Re-run `hblank init` only for new projects. Existing preview manifests need direct GPUI plus Hblank `test-support` features and the root GPUI re-export shown in the README.

### Product conclusion and remaining roadmap

Hblank now covers the high-leverage local-native loop: typed component authoring, exact variants, rich controls, component documentation, system themes, supervised rebuild continuity and explicit native tests. The remaining Storybook gaps are mostly framework instrumentation, quality infrastructure or hosted distribution—not missing fixture fundamentals.

Adoption work should now prove the adapter seam with a **second real UI-framework adapter**, not add hypothetical generic interfaces. Public testing should remain typed-handle/GPUI based until repeated project tests justify extracting selectors or interactions. Viewports/backgrounds, coverage, accessibility and visual capture should enter implementation only with a concrete adopter and viable framework substrate. Hosted publishing/review remains a separate product decision.

## Hblank evidence catalog

| ID | Repository source | Evidence used |
|---|---|---|
| H1 | [README.md](README.md) | Current setup, authoring, controls, docs, themes, testing and CLI workflows. |
| H2 | [core catalog](crates/hblank-core/src/catalog.rs) and [GPUI registry](crates/hblank/src/fixture.rs) | Framework-neutral component/fixture model, canonical metadata and adapter assembly. |
| H3 | [core controls](crates/hblank-core/src/control.rs) | Built-in editors, constraints, serialization and domain adapter interface. |
| H4 | [macros](crates/hblank-macros/src/lib.rs) | Component/fixture/docs/theme/test-handle compile-time contracts. |
| H5 | [harness app](crates/hblank/src/harness/app.rs) | Catalog state, themes, docs composition, control continuity and exact selection. |
| H6 | [harness components](crates/hblank/src/harness/components.rs) | Nested catalog, rich controls, dynamic palettes and docs block renderers. |
| H7 | [CLI config](crates/hblank-cli/src/config.rs), [discovery](crates/hblank-cli/src/discovery.rs), [generation](crates/hblank-cli/src/generate.rs), [initialization](crates/hblank-cli/src/init.rs) | Config, deterministic discovery and generated preview target. |
| H8 | [dev supervisor](crates/hblank-cli/src/dev.rs) and [catalog CLI](crates/hblank-cli/src/catalog.rs) | Build/reload lifecycle, runtime listing and strict exact-id preflight. |
| H9 | [Hblank skill](skills/hblank/SKILL.md) and [references](skills/hblank/references/) | Agent-facing contracts and verification guidance. |
| H10 | [test command](crates/hblank-cli/src/test.rs), [testing helpers](crates/hblank/src/testing.rs) and [dogfood fixture tests](fixtures/dogfood/src/fixture_card.hblank.rs) | Explicit generated-target tests and typed GPUI handles. |

## Source catalog

All sources are first-party. **SB current 10.5** means the rolling Storybook documentation with version 10.5 selected, accessed 2026-08-24. **Fixed release** is immutable release evidence. **Chromatic current** means first-party unversioned Chromatic documentation accessed 2026-08-24.

### Version and baseline packaging

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S1 | https://github.com/storybookjs/storybook/releases/tag/v10.5.10 | **Fixed release**, published 2026-08-20; accessed 2026-08-24 | Stable snapshot version and date. |
| S2 | https://storybook.js.org/docs/releases | **SB current 10.5**, accessed 2026-08-24 | Stable `latest` and prerelease `next` channel semantics. |
| S16 | https://storybook.js.org/docs/essentials | **SB current 10.5**, accessed 2026-08-24 | Default Essentials feature bundle and configuration boundary. |

### Authoring, discovery, args and docs

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S3 | https://storybook.js.org/docs/writing-stories | **SB current 10.5**, accessed 2026-08-24 | CSF story and metadata authoring model. |
| S4 | https://storybook.js.org/docs/writing-stories/typescript | **SB current 10.5**, accessed 2026-08-24 | Typed `Meta`/`StoryObj` authoring. |
| S5 | https://storybook.js.org/docs/api/main-config/main-config-stories | **SB current 10.5**, accessed 2026-08-24 | Story discovery globs and indexing inputs. |
| S6 | https://storybook.js.org/docs/writing-stories/naming-components-and-hierarchy | **SB current 10.5**, accessed 2026-08-24 | Implicit/explicit hierarchy and story sorting. |
| S7 | https://storybook.js.org/docs/writing-stories/tags | **SB current 10.5**, accessed 2026-08-24 | Built-in/custom tags, lifecycle-label examples and filtering behavior. |
| S8 | https://storybook.js.org/docs/api/main-config/main-config-tags | **SB current 10.5**, accessed 2026-08-24 | Tag filter defaults in project configuration. |
| S9 | https://storybook.js.org/docs/writing-stories/args | **SB current 10.5**, accessed 2026-08-24 | Args scopes, reuse, URL overrides and mappings. |
| S10 | https://storybook.js.org/docs/essentials/controls | **SB current 10.5**, accessed 2026-08-24 | Controls panel behavior and configuration. |
| S11 | https://storybook.js.org/docs/api/arg-types | **SB current 10.5**, accessed 2026-08-24 | ArgTypes metadata, mappings and control selection. |
| S12 | https://storybook.js.org/docs/writing-docs | **SB current 10.5**, accessed 2026-08-24 | Documentation system overview. |
| S13 | https://storybook.js.org/docs/writing-docs/autodocs | **SB current 10.5**, accessed 2026-08-24 | Generated documentation and `autodocs` tag. |
| S14 | https://storybook.js.org/docs/writing-docs/mdx | **SB current 10.5**, accessed 2026-08-24 | Authored MDX documentation pages. |
| S15 | https://storybook.js.org/docs/writing-docs/doc-blocks | **SB current 10.5**, accessed 2026-08-24 | Reusable documentation blocks. |

### Testing and quality

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S17 | https://storybook.js.org/docs/writing-tests/interaction-testing | **SB current 10.5**, accessed 2026-08-24 | `play`-function interactions, assertions and debugging. |
| S18 | https://storybook.js.org/docs/writing-tests/integrations/vitest-addon | **SB current 10.5**, accessed 2026-08-24 | Official Vitest addon, real-browser story tests and UI/CLI execution. |
| S19 | https://storybook.js.org/docs/writing-tests/test-coverage | **SB current 10.5**, accessed 2026-08-24 | Story test coverage setup, reports and scope. |
| S20 | https://storybook.js.org/docs/writing-tests/accessibility-testing | **SB current 10.5**, accessed 2026-08-24 | Official axe-based accessibility addon and test integration. |
| S21 | https://storybook.js.org/docs/writing-tests/visual-testing | **SB current 10.5**, accessed 2026-08-24 | Visual Tests addon workflow and Chromatic dependency. |
| C1 | https://www.chromatic.com/docs/visual-tests-addon/ | **Chromatic current**, accessed 2026-08-24 | In-Storybook visual testing integration and cloud synchronization. |
| C2 | https://www.chromatic.com/docs/visual/ | **Chromatic current**, accessed 2026-08-24 | Hosted capture, baselines, diffs and visual test workflow. |

### Preview environment and globals

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S22 | https://storybook.js.org/docs/essentials/viewport | **SB current 10.5**, accessed 2026-08-24 | Viewport dimensions, orientation, custom options and globals. |
| S23 | https://storybook.js.org/docs/essentials/backgrounds | **SB current 10.5**, accessed 2026-08-24 | Background options, grid and per-story/global selection. |
| S24 | https://storybook.js.org/docs/essentials/themes | **SB current 10.5**, accessed 2026-08-24 | `@storybook/addon-themes` and its theme decorators. |
| S25 | https://storybook.js.org/docs/writing-stories/decorators | **SB current 10.5**, accessed 2026-08-24 | Global/component/story decorators and context. |
| S26 | https://storybook.js.org/docs/essentials/toolbars-and-globals | **SB current 10.5**, accessed 2026-08-24 | Custom toolbars, global types, initial globals and consumption. |

### Actions, inspection and AI

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S40 | https://storybook.js.org/docs/essentials/actions | **SB current 10.5**, accessed 2026-08-24 | Callback spies, manual actions and the Actions panel. |
| S41 | https://storybook.js.org/docs/essentials/measure-and-outline/ | **SB current 10.5**, accessed 2026-08-24 | Measure and Outline toolbar behavior. |
| S42 | https://storybook.js.org/docs/api/main-config/main-config-features | **SB current 10.5**, accessed 2026-08-24 | Enabled-by-default core feature flags and component-manifest configuration. |
| S43 | https://storybook.js.org/docs/ai/manifests | **SB current 10.5**, accessed 2026-08-24; page labels feature preview | Structured component/docs manifests and current framework boundary. |
| S44 | https://storybook.js.org/docs/ai/mcp/overview | **SB current 10.5**, accessed 2026-08-24; page labels feature preview | Official MCP addon, exposed agent workflows and current React-first boundary. |

### Organization and status

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S27 | https://storybook.js.org/docs/configure/user-interface/change-detection | **SB current 10.5**, accessed 2026-08-24; page labels feature preview | Git/dependency-aware new/modified story indicators and review filter. |
| S28 | https://storybook.js.org/docs/configure/user-interface/sidebar-and-urls | **SB current 10.5**, accessed 2026-08-24 | Sidebar roots, hierarchy, IDs, URLs and story index relationship. |

### Sharing, publishing, embedding and review

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S29 | https://storybook.js.org/docs/sharing/publish-storybook | **SB current 10.5**, accessed 2026-08-24 | Static build/deployment and recommended Chromatic publishing path. |
| S30 | https://storybook.js.org/docs/sharing/embed | **SB current 10.5**, accessed 2026-08-24 | Story iframe embedding and Chromatic oEmbed distinction. |
| S31 | https://storybook.js.org/docs/sharing/storybook-composition | **SB current 10.5**, accessed 2026-08-24 | Composing multiple Storybooks into one UI. |
| S32 | https://storybook.js.org/docs/sharing/package-composition | **SB current 10.5**, accessed 2026-08-24 | Package-version-aware Storybook composition. |
| C3 | https://www.chromatic.com/docs/publish/ | **Chromatic current**, accessed 2026-08-24 | Hosted Storybook publishing, CDN URLs and version/branch access. |
| C4 | https://www.chromatic.com/docs/review/ | **Chromatic current**, accessed 2026-08-24 | Hosted UI Review, reviewers, discussions, decisions and checks. |

### Build, development and extension APIs

| ID | Exact URL | Access/version context | What it establishes |
|---|---|---|---|
| S33 | https://storybook.js.org/docs/get-started/install | **SB current 10.5**, accessed 2026-08-24 | Initializer, framework detection and generated setup. |
| S34 | https://storybook.js.org/docs/api/cli-options | **SB current 10.5**, accessed 2026-08-24 | `dev`/`build` commands and command-line UX. |
| S35 | https://storybook.js.org/docs/builders | **SB current 10.5**, accessed 2026-08-24 | Builder role and supported Vite/Webpack integrations. |
| S36 | https://storybook.js.org/docs/addons | **SB current 10.5**, accessed 2026-08-24 | Addon architecture and manager/preview extension surfaces. |
| S37 | https://storybook.js.org/docs/addons/addons-api | **SB current 10.5**, accessed 2026-08-24 | Registration, panels, tools, tabs, hooks, state and events APIs. |
| S38 | https://storybook.js.org/docs/addons/addon-types | **SB current 10.5**, accessed 2026-08-24 | UI addons versus preset addons and their entry points. |
| S39 | https://storybook.js.org/docs/addons/install-addons | **SB current 10.5**, accessed 2026-08-24 | Addon installation and project registration. |

