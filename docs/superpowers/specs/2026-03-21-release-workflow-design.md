# Electrotest Release Workflow Design

## Goal

Add a GitHub Actions release workflow for the Electrotest CLI that publishes a single binary per target platform to GitHub Releases for:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- Windows x64 MSVC (`x86_64-pc-windows-msvc`)
- Linux x64 musl (`x86_64-unknown-linux-musl`)

The workflow must validate the release version, run tests before publishing, generate checksums, and support both tag-triggered releases and manual dispatches.

## Product Intent

Electrotest is distributed as a CLI with one executable per platform. The release workflow should reflect that directly by publishing raw binaries rather than wrapping them in `zip` or `tar.gz` archives.

This keeps the release simple for users:

- download one file
- add execute permission on Unix
- move it into `PATH`

## Scope

### In Scope

- One GitHub Actions workflow under `.github/workflows/`
- Trigger on Git tags matching `v*`
- Trigger through `workflow_dispatch`
- Version verification between Git tag/input and `Cargo.toml`
- Test gate before publication
- Build matrix for the three requested targets
- GitHub Release creation or update
- Release notes via GitHub automatic generation
- Artifact checksum generation

### Out of Scope

- Installers such as `.pkg`, `.msi`, or `.deb`
- Homebrew, Scoop, or package-manager publishing
- Code signing or notarization
- Extra targets beyond the three requested ones
- Archive packaging (`zip`, `tar.gz`)
- Docker-based release distribution

## Release Triggers

The workflow should support two entry paths:

### Tag-Based Release

Primary release path:

- workflow triggers on `push` to tags matching `v*`
- example: `v0.1.0`

This is the canonical release path.

### Manual Release

Secondary release path:

- workflow triggers through `workflow_dispatch`
- manual dispatch must accept exactly one release tag input in the format `vX.Y.Z`
- the workflow must check out and build from that exact matching tag/ref
- the workflow verifies that the requested tag matches `Cargo.toml`

This path exists for operational flexibility such as re-running a release after transient CI issues.

## Version Source of Truth

The workflow must enforce version consistency.

### Rule

- `Cargo.toml` contains the crate version
- release tag must be `v<crate-version>`
- manual dispatch input must also be `v<crate-version>`

### Examples

- `Cargo.toml` version `0.1.0` is valid only with tag `v0.1.0`
- `Cargo.toml` version `0.1.0` with tag `v0.1.1` must fail

The workflow must stop before building if the versions do not match.

## Workflow Architecture

Use one workflow with distinct jobs for verification, testing, build, checksums, and release publication.

### Job 1: Version Verification

Responsibilities:

- checkout repository
- extract crate version from `Cargo.toml`
- determine requested release version from tag or manual input
- in manual dispatch mode, resolve and checkout the exact requested tag before validation/build
- compare both values
- expose normalized version data to downstream jobs

Failure conditions:

- tag missing required `v` format
- dispatch input missing or malformed
- `Cargo.toml` version mismatch

### Job 2: Test Gate

Responsibilities:

- run `cargo test`
- block release if tests fail

Implementation choice:

- run this job once on Linux for the release gate

This keeps the release workflow simpler while still enforcing a real test gate before publication.

### Job 3: Build Matrix

Responsibilities:

- build the CLI in release mode for each target
- emit one raw binary per platform
- name outputs consistently for release upload

Build matrix targets:

- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-musl`

Expected release asset names:

- `electrotest-vX.Y.Z-aarch64-apple-darwin`
- `electrotest-vX.Y.Z-x86_64-pc-windows-msvc.exe`
- `electrotest-vX.Y.Z-x86_64-unknown-linux-musl`

### Job 4: Checksums

Responsibilities:

- download built binaries from matrix jobs
- compute SHA-256 checksums for every release asset
- publish one checksum file named `SHA256SUMS.txt`

### Job 5: Release Publication

Responsibilities:

- create or update the GitHub Release
- attach the three binaries
- attach `SHA256SUMS.txt`
- enable GitHub automatic release notes

The resulting release remains editable afterward through GitHub’s normal UI.

If a rerun targets an existing release, publication should be idempotent:

- update the existing GitHub Release for that tag
- replace assets with the same names rather than producing duplicates

## Platform Strategy

### macOS

- target: `aarch64-apple-darwin`
- build on macOS runner
- publish raw binary only

### Windows

- target: `x86_64-pc-windows-msvc`
- build on Windows runner
- publish raw `electrotest.exe`

### Linux

- target: `x86_64-unknown-linux-musl`
- build on Ubuntu runner
- configure Rust target and musl toolchain as needed
- publish raw binary only

## Artifact Philosophy

Do not wrap release outputs in archives.

Reasoning:

- the CLI is a single executable per platform
- the user explicitly wants direct binary distribution
- release UX stays minimal and predictable

The release assets should therefore be exactly:

- one binary for macOS
- one binary for Windows
- one binary for Linux
- one checksum file

## Data Flow

1. workflow starts from tag push or manual dispatch
2. version verification extracts and validates release version
3. tests run and must pass
4. matrix build creates one binary per target
5. checksum job generates `SHA256SUMS.txt`
6. release job creates or updates the GitHub Release and uploads assets

## Failure Handling

The workflow must fail early and clearly for:

- version mismatch
- unsupported trigger input
- failing test suite
- build failure on any target
- checksum generation failure
- GitHub Release upload failure

Because all release assets are expected for a valid release, any target build failure should fail the entire workflow rather than producing a partial release.

## Security and Permissions

The workflow should use the minimal required GitHub permissions.

Expected permission profile:

- read access to repository contents for build/test jobs
- write access to contents for the release publication job only

No extra permissions should be added unless implementation proves they are required.

## Testing Strategy for the Workflow

Before relying on the workflow for real releases, implementation should support verification through:

- workflow lint/validation where practical
- dry run on `workflow_dispatch`
- test tag such as `v0.1.0-test` only if the final implementation intentionally supports such rehearsal, otherwise use a real semver tag on a non-production test repo or branch strategy

For the actual implementation plan, the workflow should be tested with:

- one manual dispatch using a tag that matches `Cargo.toml`
- one intentional mismatch case to verify early failure
- confirmation that all expected release asset names are produced

## Design Principles

- one workflow, not multiple loosely coupled release pipelines
- fail before build when version metadata is wrong
- fail before publish when tests fail
- publish direct binaries, not archives
- keep the release job deterministic and boring
- keep platform matrix limited to the explicitly requested targets

## Expected Outcome

After implementation, maintainers should be able to:

- tag a release like `v0.1.0`
- let GitHub Actions verify version and tests
- receive a GitHub Release containing:
  - macOS ARM binary
  - Windows x64 MSVC binary
  - Linux x64 musl binary
  - `SHA256SUMS.txt`
- optionally rerun the same release flow via `workflow_dispatch`
