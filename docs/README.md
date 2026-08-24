# Hblank documentation

[Back to the project README](../README.md)

Hblank runs GPUI components in a private desktop catalog so you can work on one state without starting the full application.

## Start here

New to Hblank? Follow these in order:

1. [Getting started](getting-started.md) installs Hblank, creates the private preview crate, and opens the first component.
2. [Components and fixtures](authoring.md) explains props, generated controls, named variants, docs, and themes.

## Find what you need

| Task | Guide |
|---|---|
| Learn every command and path rule | [CLI reference](cli.md) |
| Test fixture data or rendered GPUI components | [Testing components](testing.md) |
| Understand the four crates or select a GPUI backend | [Crates and GPUI backends](crates.md) |
| Fix discovery, build, control, docs, theme, or test failures | [Troubleshooting](troubleshooting.md) |
| Move an older project to components and variants | [Migrating to 0.3](migration-0.3.md) |
| Publish a release | [Releasing Hblank](releasing.md) |

## The short version

A normal Hblank workflow looks like this:

```bash
hblank init
hblank list
hblank dev --fixture src/button.hblank.rs
hblank test
```

Components use ordinary Rust props. `HblankProps` turns supported fields into controls, `#[hblank::component]` registers the renderer, and `#[hblank::fixture]` names each state you want in the catalog.

The private preview compiles the same GPUI code as the host project. Hblank does not translate components into HTML or mirror props in JSON.

## Current release status

`main` is at 0.3.0 and targets GPUI 0.2.2. Crates.io does not yet have the complete four-crate release, so the getting started guide installs Hblank from a checkout.
