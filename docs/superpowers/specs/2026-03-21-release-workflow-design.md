# Electrotest Release Workflow Design

## Goal

Add a GitHub Actions-based release process for the Electrotest CLI with two explicit stages:

1. create a release pull request that bumps the project version
2. publish the actual release only after that pull request is merged and a matching Git tag is pushed

The published release must provide a single raw binary per target platform for:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- Windows x64 MSVC (`x86_64-pc-windows-msvc`)
- Linux x64 musl (`x86_64-unknown-linux-musl`)

## Product Intent

Version changes and release publication are separate actions.

The release PR is the reviewable step where the version bump is proposed and merged. The Git tag is the explicit publication signal that turns an already-reviewed version into a public release.

This keeps release intent clear:

- the version bump is reviewed like any other code change
- the actual release remains an explicit maintainer action
- no release is published merely because a version file changed on `main`

## Scope

### In Scope

- One workflow to create a release PR from a manually supplied version
- One workflow to publish a release from a pushed tag
- Version updates for:
  - `Cargo.toml`
  - `Cargo.lock`
  - release-related documentation where needed
- Release PR creation against `main`
- Strict validation between merged crate version and pushed release tag
- Test gate before publication
- Build matrix for the three requested targets
- Raw binary publication to GitHub Releases
- Release checksum generation
- GitHub automatic release notes

### Out of Scope

- Auto-publishing on merge without a tag
- Automatic tagging after the release PR is merged
- Installers such as `.pkg`, `.msi`, or `.deb`
- Homebrew, Scoop, or other package-manager publication
- Code signing or notarization
- Extra targets beyond the three requested ones
- Archive packaging (`zip`, `tar.gz`)
- Docker-based release distribution

## Release Model

The release process has two distinct workflows.

### Workflow 1: Create Release PR

Purpose:

- prepare the next release version in a normal reviewable pull request

Trigger:

- `workflow_dispatch`

Input:

- `version` in the format `X.Y.Z`

Output:

- branch like `release/0.1.0`
- pull request like `release: 0.1.0`

Responsibilities:

- validate input version format
- refuse invalid or unchanged version values
- update `Cargo.toml`
- update `Cargo.lock`
- update release-related documentation if it contains versioned release instructions or examples
- commit the changes on a release branch
- push the branch
- open a pull request targeting `main`

### Workflow 2: Publish Release

Purpose:

- publish release binaries only after the release PR has been merged

Trigger:

- `push` on tags matching `v*`

Example:

- `v0.1.0`

Responsibilities:

- checkout the tagged revision
- extract `0.1.0` from `v0.1.0`
- verify `Cargo.toml` contains `0.1.0`
- run `cargo test`
- build platform binaries
- generate `SHA256SUMS.txt`
- create or update the GitHub Release
- attach binaries and checksums

## Source of Truth for Versioning

### Release PR Input

- input format is exactly `X.Y.Z`
- example: `0.1.0`

This is the version written into Rust project files.

### Release Tag Format

- publication tag format is exactly `vX.Y.Z`
- example: `v0.1.0`

This is the external release identifier.

### Consistency Rule

- if the merged code contains `version = "0.1.0"` in `Cargo.toml`
- then the only valid release tag is `v0.1.0`

Any mismatch must fail before build or publication.

## Workflow Architecture

Use two workflow files, each with a single clear responsibility.

### Workflow File 1

- path: `.github/workflows/create-release-pr.yml`
- purpose: prepare and open the release PR

### Workflow File 2

- path: `.github/workflows/release.yml`
- purpose: build and publish the release when a matching tag is pushed

## Create Release PR Workflow Design

### Trigger and Input

- trigger: `workflow_dispatch`
- input: `version`
- accepted format: semver-like `X.Y.Z`

The workflow should reject:

- missing version
- malformed version
- same version as the current crate version

### Repository Changes

The workflow updates:

- `Cargo.toml`
- `Cargo.lock`
- documentation only if it contains version-sensitive release instructions that should stay aligned with the process

The workflow should not make unrelated release-time edits.

### Branch and PR Shape

Recommended branch name:

- `release/0.1.0`

Recommended PR title:

- `release: 0.1.0`

Recommended PR body should mention:

- target version
- files updated
- next action after merge: push tag `v0.1.0`

## Publish Release Workflow Design

### Verification Stage

The release workflow must:

- run only on tag push
- extract the version from the tag
- read the crate version from `Cargo.toml`
- fail if they differ

### Test Gate

The workflow must run:

- `cargo test`

No release artifacts should be published if tests fail.

### Build Matrix

Targets:

- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-musl`

The workflow should emit one raw binary per platform.

Expected asset names:

- `electrotest-vX.Y.Z-aarch64-apple-darwin`
- `electrotest-vX.Y.Z-x86_64-pc-windows-msvc.exe`
- `electrotest-vX.Y.Z-x86_64-unknown-linux-musl`

### Checksums

The workflow must generate:

- `SHA256SUMS.txt`

This file contains checksums for all published binaries.

### Release Publication

The workflow must:

- create or update the GitHub Release for the tag
- attach the three binaries
- attach `SHA256SUMS.txt`
- enable GitHub automatic release notes

The GitHub Release remains editable afterward through the GitHub UI.

If a rerun targets an existing release, publication should be idempotent:

- update the existing release
- replace assets with the same names instead of duplicating them

## Artifact Philosophy

Do not create release archives.

The CLI is distributed as one executable per platform, so the release should publish direct binaries only.

Published assets must therefore be exactly:

- one macOS binary
- one Windows binary
- one Linux binary
- one checksum file

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
- configure Rust target and musl tooling as needed
- publish raw binary only

## Data Flow

### Release PR Path

1. maintainer runs `create-release-pr`
2. workflow validates `X.Y.Z`
3. workflow updates versioned files
4. workflow creates `release/X.Y.Z`
5. workflow opens PR to `main`
6. humans review and merge the PR

### Release Publication Path

1. maintainer pushes tag `vX.Y.Z`
2. release workflow starts from the tagged commit
3. workflow validates tag ↔ `Cargo.toml`
4. tests run and must pass
5. matrix build creates the three binaries
6. checksum job generates `SHA256SUMS.txt`
7. release job creates or updates the GitHub Release and uploads assets

## Failure Handling

The workflows must fail early and clearly for:

### Create Release PR

- invalid input version format
- unchanged version value
- failure to update versioned files
- failure to push branch
- failure to open pull request

### Publish Release

- invalid tag format
- version mismatch with `Cargo.toml`
- failing tests
- build failure on any target
- checksum generation failure
- GitHub Release upload failure

Partial releases are not acceptable. Any target build failure should fail the whole release workflow.

## Security and Permissions

Use the minimum required GitHub permissions.

Expected permission profile:

- read access for verification and build jobs
- write access only for jobs that push the release branch / create PRs or publish GitHub Releases

No extra permissions should be granted unless implementation proves they are required.

## Testing Strategy for the Workflows

The implementation should be verified through both workflow paths.

### Create Release PR Workflow

Must be tested with:

- one successful manual run using a version like `0.1.0` or the next appropriate version for the repo state
- one invalid input case to verify early failure

### Publish Release Workflow

Must be tested with:

- one successful tag-triggered release where tag and `Cargo.toml` match
- one intentional mismatch case where the tag does not match `Cargo.toml`, expecting early failure
- confirmation that the uploaded asset names match the spec exactly

## Design Principles

- separate version-preparation from publication
- make the release PR reviewable like normal code changes
- keep the tag as the only publication signal
- fail before build when version metadata is wrong
- fail before publish when tests fail
- publish direct binaries, not archives
- keep each workflow focused on one responsibility
- limit targets to the explicitly requested platforms

## Expected Outcome

After implementation, maintainers should be able to:

- run a workflow with input `0.1.0`
- receive a PR `release: 0.1.0`
- merge that PR into `main`
- push `v0.1.0`
- receive a GitHub Release containing:
  - `electrotest-v0.1.0-aarch64-apple-darwin`
  - `electrotest-v0.1.0-x86_64-pc-windows-msvc.exe`
  - `electrotest-v0.1.0-x86_64-unknown-linux-musl`
  - `SHA256SUMS.txt`
