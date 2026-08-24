# Releasing Hblank

[Documentation home](README.md) | [Project README](../README.md)

This guide is for maintainers. Hblank uses semantic-release on `main` and publishes all four crates at one version.

## Release inputs

Conventional commits determine whether a release exists and which version to choose.

| Commit | Result |
|---|---|
| `fix: ...` | Patch release |
| `feat: ...` | Minor release |
| `type!: ...` | Major release |
| `BREAKING CHANGE:` footer | Major release |
| `docs:`, `test:`, `chore:` | No release |

The pre-1.0 history starts from a `v0.0.0` tag on the repository root commit. The workflow checks that tag before running semantic-release.

## Workflow
`.github/workflows/release.yml` runs on every push to `main` and through manual dispatch.

Before semantic-release, the job resumes a tagged workspace version's missing crate publication. It then creates GitHub Releases for up to 20 tagged versions whose four crates are all indexed. Both recovery steps are idempotent.

The job:

1. checks out full Git history
2. verifies the `v0.0.0` baseline tag
3. installs Node and Rust dependencies
4. runs `cargo fmt --all -- --check`
5. runs `cargo test --workspace --locked`
6. requests a short-lived crates.io token through GitHub OIDC
7. uses the bootstrap secret when it exists
8. resumes an incomplete publication for the current tagged workspace version
9. publishes missing GitHub Releases for fully indexed crate versions
10. runs semantic-release

GitHub must allow workflow write access. In repository Settings, open Actions, then General. Under Workflow permissions, select `Read and write permissions`. The workflow uses the built-in `GITHUB_TOKEN` to create releases.

Semantic-release then:

1. analyzes commits since the last tag
2. chooses the next version
3. updates `CHANGELOG.md`
4. runs `scripts/release-version.mjs set VERSION`
5. runs `cargo check --workspace`
6. commits version and lockfile changes
7. tags `vVERSION`
8. publishes crates
9. creates the GitHub release

Release commits use:

```text
chore(release): VERSION [skip ci]
```

## Versioned files

`scripts/release-version.mjs` keeps the workspace version and internal dependency requirements aligned. The release commit includes:

- `CHANGELOG.md`
- `Cargo.lock`
- root `Cargo.toml`
- `crates/hblank-core/Cargo.toml`
- `crates/hblank/Cargo.toml`

The macro and CLI crates inherit the workspace version without extra manifest edits.

## Crate publishing order

`scripts/publish-crates.sh` publishes:

1. `hblank-core`
2. `hblank-macros`
3. `hblank`
4. `hblank-cli`

The script waits up to five minutes for each version to appear in the crates.io index before moving to the next crate.

A retry with the same version checks crates.io first and skips any crate that already exists. This makes a partial publish recoverable without uploading the same crate twice.

`scripts/publish-github-releases.sh` creates any missing GitHub Release only after all four crates for its tag are indexed. It checks the 20 newest reachable release tags, in version order. Its release body comes from `CHANGELOG.md`.

Check a release without publishing:

```bash
bash scripts/publish-crates.sh 0.3.0 --dry-run
```

The dry run asks Cargo to list each package and prints the versions it would publish.

## First-time crates.io setup

Trusted Publishing cannot claim a crate name before that crate exists. The first release therefore needs a temporary crates.io token.

Run the interactive setup wizard from the repository root:

```bash
scripts/setup-release.sh
```

The wizard has eight stages:

1. create a temporary crates.io token with publish-new and publish-update permissions
2. save it as the `CARGO_REGISTRY_TOKEN` GitHub secret
3. allow GitHub Actions workflows to write repository contents
4. create or verify the `v0.0.0` baseline tag
5. dispatch the first automated release
6. add `release.yml` as a Trusted Publisher for each of the four crates
7. delete the bootstrap secret
8. dispatch a verification run that must use a short-lived token

The wizard requires an authenticated GitHub CLI and checks that it is running in `mmmeff/hblank`.

Each crates.io Trusted Publisher entry uses:

```text
Owner: mmmeff
Repository: hblank
Workflow: release.yml
Environment: empty
```

Keep the bootstrap secret until all four crate names exist and have trusted publishers. The bootstrap token has `publish-new` permission. A trusted token cannot claim a new crate name.

Delete the bootstrap secret after the trusted publishers exist. Then dispatch the verification run. Its log must contain:

```text
Using a short-lived crates.io trusted-publishing token
```

After that check, future releases use OIDC and do not need a long-lived crates.io secret.

## Recovering a partial publish

First check which versions crates.io has:

```bash
cargo search hblank --limit 10
cargo info hblank-core@VERSION
cargo info hblank-macros@VERSION
cargo info hblank@VERSION
cargo info hblank-cli@VERSION
```

Then rerun the publish script with the same version in an environment that has `CARGO_REGISTRY_TOKEN`:

```bash
bash scripts/publish-crates.sh VERSION
```

The script skips indexed crates and resumes at the first missing one.

If semantic-release tagged a version before crate publication failed, the next `release.yml` run publishes its missing crates and GitHub Release. Do not delete the tag. The recovery steps need it.

Do not bump the workspace version to work around a missing dependency crate. Publish the missing crate at the version already referenced by the release manifests.

## Before merging a release commit

Semantic-release creates the version commit. Normal feature and fix pull requests should not edit workspace versions by hand.

Before merging a change that should release:

- use the correct Conventional Commit type
- run the workspace formatting and test commands
- check that internal crate requirements still use the workspace version
- keep the publish order valid
- update this guide if setup or credentials change

A docs-only commit does not publish a release.
