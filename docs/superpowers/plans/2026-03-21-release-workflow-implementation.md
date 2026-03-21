# Release Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a two-stage GitHub Actions release process that first creates a release PR to bump the Electrotest version, then publishes raw platform binaries only when a matching tag is pushed after that PR is merged.

**Architecture:** Use two focused workflow files: one manual workflow creates a release branch and pull request from a requested `X.Y.Z` version, and a second tag-driven workflow verifies the merged crate version, runs tests, builds binaries, generates checksums, and publishes a GitHub Release. Keep repository changes limited to workflow YAML plus concise README updates for the new release process.

**Tech Stack:** GitHub Actions YAML, Rust/Cargo, shell scripting, Python for parsing `Cargo.toml`, GitHub CLI (`gh`) for PR and release operations, native runner toolchains for macOS/Windows/Linux musl.

---

## Proposed File Structure

### Workflow files

- Create: `/.github/workflows/create-release-pr.yml` - manual workflow that validates a requested `X.Y.Z` version, updates release files, pushes a `release/X.Y.Z` branch, and opens a PR to `main`
- Create: `/.github/workflows/release.yml` - tag-triggered workflow that validates `vX.Y.Z` against `Cargo.toml`, runs tests, builds binaries, generates `SHA256SUMS.txt`, and publishes a GitHub Release

### Documentation files

- Modify: `README.md` - add a short, version-agnostic release section that explains the new `release PR -> merge -> tag -> release` flow and the downloadable binaries

## Implementation Notes

- Keep workflows separate to match the approved design and avoid mixed trigger logic.
- `create-release-pr.yml` input must be exactly `X.Y.Z`, not `vX.Y.Z`.
- `release.yml` trigger must be exactly `push` on tags matching `v*`.
- Force the release PR workflow to operate from `main`, even if `workflow_dispatch` is launched from another ref.
- Publish raw binaries only; do not add `zip` or `tar.gz` packaging.
- Use `gh` for both PR creation and idempotent release publication so branch/asset behavior is explicit.
- Keep documentation changes narrow: update release process documentation only, not general marketing copy.
- Make the README release section version-agnostic so future release PRs do not need to rewrite docs on every version bump unless new version-specific release instructions are added later.

### Task 1: Add the release PR workflow skeleton

**Files:**
- Create: `.github/workflows/create-release-pr.yml`
- Test: `.github/workflows/create-release-pr.yml`

- [ ] **Step 1: Write the failing workflow skeleton**

Create `.github/workflows/create-release-pr.yml` with only a workflow name and an incomplete `workflow_dispatch` input block.

```yaml
name: create-release-pr

on:
  workflow_dispatch:
```

- [ ] **Step 2: Verify the workflow is still incomplete**

Run: `grep -n "version:" .github/workflows/create-release-pr.yml`
Expected: no match because the required version input is not defined yet.

- [ ] **Step 3: Implement the trigger, input, and job skeleton**

Add:

```yaml
name: create-release-pr

on:
  workflow_dispatch:
    inputs:
      version:
        description: "Release version in the format X.Y.Z"
        required: true
        type: string

permissions:
  contents: write
  pull-requests: write

jobs:
  prepare-release-pr:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: main
```

- [ ] **Step 4: Verify the trigger and permissions are present**

Run: `grep -n "workflow_dispatch" .github/workflows/create-release-pr.yml && grep -n "pull-requests: write" .github/workflows/create-release-pr.yml`
Expected: both matches found.

- [ ] **Step 5: Commit the workflow skeleton**

```bash
git add .github/workflows/create-release-pr.yml
git commit -m "ci: add release pr workflow skeleton"
```

### Task 2: Implement release version validation and file updates

**Files:**
- Modify: `.github/workflows/create-release-pr.yml`
- Test: `.github/workflows/create-release-pr.yml`

- [ ] **Step 1: Add a failing placeholder for version validation**

Insert a shell step comment showing the missing behaviors:

- validate input matches `X.Y.Z`
- update `Cargo.toml`
- regenerate `Cargo.lock`

- [ ] **Step 2: Verify the validation logic is still missing**

Run: `grep -n "invalid version format" .github/workflows/create-release-pr.yml && grep -n "unchanged version" .github/workflows/create-release-pr.yml`
Expected: no match because the final validation messages are not implemented yet.

- [ ] **Step 3: Implement strict version validation and file updates**

Add a shell step that:

```yaml
      - name: Validate and update versioned files
        shell: bash
        run: |
          set -euo pipefail
          version="${{ inputs.version }}"
          [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
            echo "invalid version format: $version" >&2
            exit 1
          }

          current_version=$(python - <<'PY'
from pathlib import Path
import tomllib
data = tomllib.loads(Path('Cargo.toml').read_text())
print(data['package']['version'])
PY
)

          [[ "$version" != "$current_version" ]] || {
            echo "unchanged version: $version" >&2
            exit 1
          }

          python - <<'PY' "$version"
from pathlib import Path
import re
import sys
path = Path('Cargo.toml')
text = path.read_text()
path.write_text(re.sub(r'^version = ".*"$', f'version = "{sys.argv[1]}"', text, count=1, flags=re.MULTILINE))
PY

          cargo check --locked || cargo check
```

Before that shell step, install Rust explicitly in this workflow, for example with `dtolnay/rust-toolchain@stable`, so the lockfile refresh command can run on GitHub-hosted runners. Implementation note for this task: do not hardcode `0.1.0` in the final version-update logic; use a value read from `Cargo.toml` and replace only the package version line. Refresh `Cargo.lock` in a way that captures the version/package metadata change without broadly upgrading dependencies. If release-related docs remain version-agnostic, this task should leave them untouched; only add doc edits here if the repo later grows version-sensitive release instructions that truly need bumping.

- [ ] **Step 4: Verify version validation and update commands are encoded**

Run: `python - <<'PY'
from pathlib import Path
text = Path('.github/workflows/create-release-pr.yml').read_text()
required = ['invalid version format', 'unchanged version']
missing = [item for item in required if item not in text]
if missing:
    raise SystemExit(f'missing required strings: {missing}')
if 'cargo update --workspace' in text:
    raise SystemExit('unexpected cargo update --workspace found')
print('release PR validation logic looks correct')
PY`
Expected: success with `release PR validation logic looks correct`.

- [ ] **Step 5: Commit the version update logic**

```bash
git add .github/workflows/create-release-pr.yml
git commit -m "ci: validate release pr versions"
```

### Task 3: Push the release branch and open the PR

**Files:**
- Modify: `.github/workflows/create-release-pr.yml`
- Test: `.github/workflows/create-release-pr.yml`

- [ ] **Step 1: Add a failing placeholder for branch and PR creation**

Add a shell step comment showing the missing branch naming and PR creation behavior.

- [ ] **Step 2: Verify PR creation is still missing**

Run: `grep -n "gh pr create" .github/workflows/create-release-pr.yml`
Expected: no match.

- [ ] **Step 3: Implement branch push and PR creation with `gh`**

Add steps that:

- configure the release branch name as `release/${{ inputs.version }}`
- verify the workflow is operating from `main` before creating the branch
- configure bot git identity before committing
- commit the changed files
- push the branch to `origin`
- create a PR to `main`

Use a PR body similar to:

```bash
gh pr create \
  --base main \
  --head "release/${version}" \
  --title "release: ${version}" \
  --body "## Summary
- bump Cargo.toml to ${version}
- refresh Cargo.lock
- update release-related documentation if needed

## Next Step
- merge this PR, then push tag v${version} to publish the release"
```

Also make the workflow fail clearly if the branch or PR already exists and would conflict with the requested version.

Authentication/setup requirements for this task:

- use `GITHUB_TOKEN` for `gh` authentication
- export `GH_TOKEN="$GITHUB_TOKEN"` in the workflow step environment before running `gh` commands
- set git identity explicitly, for example:

```bash
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
gh auth status
```

- [ ] **Step 4: Verify branch and PR commands are encoded**

Run: `grep -n "release/\${version}" .github/workflows/create-release-pr.yml && grep -n "gh pr create" .github/workflows/create-release-pr.yml`
Expected: branch naming, git identity setup, and PR creation commands all present.

- [ ] **Step 5: Commit release PR creation support**

```bash
git add .github/workflows/create-release-pr.yml
git commit -m "ci: create release pull requests"
```

### Task 4: Add the release workflow skeleton for tag publication

**Files:**
- Create: `.github/workflows/release.yml`
- Test: `.github/workflows/release.yml`

- [ ] **Step 1: Write the failing release workflow skeleton**

Create `.github/workflows/release.yml` with only the workflow name and an incomplete tag trigger.

```yaml
name: release

on:
  push:
```

- [ ] **Step 2: Verify the tag trigger is incomplete**

Run: `grep -n "tags:" .github/workflows/release.yml`
Expected: no match.

- [ ] **Step 3: Implement the tag trigger and top-level job skeleton**

Add:

```yaml
name: release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: read

jobs:
  verify-version:
    runs-on: ubuntu-latest
    outputs:
      release_tag: ${{ steps.version.outputs.release_tag }}
      release_version: ${{ steps.version.outputs.release_version }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
```

- [ ] **Step 4: Verify the tag trigger exists**

Run: `grep -n "v\*" .github/workflows/release.yml`
Expected: match found.

- [ ] **Step 5: Commit the release workflow skeleton**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add tag release workflow skeleton"
```

### Task 5: Validate tag version and add the test gate

**Files:**
- Modify: `.github/workflows/release.yml`
- Test: `.github/workflows/release.yml`

- [ ] **Step 1: Add a failing placeholder for tag verification and tests**

Insert placeholder comments for:

- extracting `X.Y.Z` from `vX.Y.Z`
- comparing against `Cargo.toml`
- running `cargo test`

- [ ] **Step 2: Verify the validation and tests are still missing**

Run: `grep -n "cargo test" .github/workflows/release.yml && grep -n "does not match Cargo.toml version" .github/workflows/release.yml`
Expected: missing or incomplete.

- [ ] **Step 3: Implement version verification and test gate**

Add a `verify-version` step that:

- reads `github.ref_name`
- requires `^v[0-9]+\.[0-9]+\.[0-9]+$`
- extracts `release_version` by removing the `v`
- reads `Cargo.toml` version
- fails if they differ
- verifies the tagged commit is already reachable from `origin/main`

Then add a `test` job that:

- needs `verify-version`
- checks out `${{ needs.verify-version.outputs.release_tag }}`
- installs stable Rust
- runs `cargo test`

The merge guard should be explicit, for example:

```bash
git fetch origin main --depth=0
git merge-base --is-ancestor "$GITHUB_SHA" origin/main || {
  echo "tag commit is not merged into main" >&2
  exit 1
}
```

Use full history for this ancestry check: either keep `actions/checkout@v4` with `fetch-depth: 0` in `verify-version`, or perform an equivalent unshallow/full fetch before `git merge-base --is-ancestor`.

- [ ] **Step 4: Verify the release gate is encoded**

Run: `grep -n "cargo test" .github/workflows/release.yml && grep -n "release tag" .github/workflows/release.yml`
Expected: version-check logic, main-merge guard, and test-gate logic present.

- [ ] **Step 5: Commit tag verification and test gate**

```bash
git add .github/workflows/release.yml
git commit -m "ci: validate release tags before publish"
```

### Task 6: Implement the cross-platform build matrix and checksums

**Files:**
- Modify: `.github/workflows/release.yml`
- Test: `.github/workflows/release.yml`

- [ ] **Step 1: Add a failing placeholder for builds and checksums**

Define empty `build` and `checksums` jobs without real commands yet.

- [ ] **Step 2: Verify artifact publication is still missing**

Run: `grep -n "SHA256SUMS.txt" .github/workflows/release.yml && grep -n "upload-artifact" .github/workflows/release.yml`
Expected: missing or incomplete.

- [ ] **Step 3: Implement the build matrix and checksum job**

Add a `build` matrix for:

- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-musl`

The job must:

- checkout the verified tag
- install stable Rust and the target
- assign runners explicitly:
  - macOS target on `macos-latest`
  - Windows target on `windows-latest`
  - Linux musl target on `ubuntu-latest`
- install Linux musl tooling where needed
- run `cargo build --release --target <target>`
- rename the resulting executable to:
  - `electrotest-v${{ needs.verify-version.outputs.release_version }}-aarch64-apple-darwin`
  - `electrotest-v${{ needs.verify-version.outputs.release_version }}-x86_64-pc-windows-msvc.exe`
  - `electrotest-v${{ needs.verify-version.outputs.release_version }}-x86_64-unknown-linux-musl`
- upload each raw binary as an artifact

Then add a `checksums` job that downloads all binaries and writes:

```bash
find release-assets -maxdepth 1 -type f -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS.txt
```

Make the artifact layout explicit before checksum generation: either download with `merge-multiple: true` into `release-assets/`, or adjust the checksum command to recurse through the actual artifact subdirectories and write checksums against the final asset filenames only.

- [ ] **Step 4: Verify target and checksum coverage**

Run: `grep -n "aarch64-apple-darwin" .github/workflows/release.yml && grep -n "x86_64-pc-windows-msvc" .github/workflows/release.yml && grep -n "x86_64-unknown-linux-musl" .github/workflows/release.yml && grep -n "SHA256SUMS.txt" .github/workflows/release.yml`
Expected: all targets found, with explicit runner mapping and checksum generation.

- [ ] **Step 5: Commit build and checksum support**

```bash
git add .github/workflows/release.yml
git commit -m "ci: build release binaries and checksums"
```

### Task 7: Publish or update the GitHub Release

**Files:**
- Modify: `.github/workflows/release.yml`
- Test: `.github/workflows/release.yml`

- [ ] **Step 1: Add a failing placeholder for release publication**

Add a `release` job without `gh` commands yet.

- [ ] **Step 2: Verify release publication is still missing**

Run: `grep -n -- "gh release create" .github/workflows/release.yml && grep -n -- "gh release upload" .github/workflows/release.yml`
Expected: missing or incomplete.

- [ ] **Step 3: Implement idempotent release publication with `gh`**

Add a `release` job that:

- needs `verify-version`, `build`, and `checksums`
- uses `permissions: contents: write`
- downloads all binary artifacts and `SHA256SUMS.txt`
- authenticates `gh` using `GITHUB_TOKEN`
- creates the release if it does not exist:

```bash
gh release view "$release_tag" >/dev/null 2>&1 || \
  gh release create "$release_tag" --title "$release_tag" --generate-notes
```

- uploads assets with replacement semantics:

```bash
find release-assets -type f -maxdepth 2 -print0 | \
  xargs -0 gh release upload "$release_tag" --clobber
```

This exact `gh`-based strategy is the required implementation direction for the plan.

- [ ] **Step 4: Verify release publication behavior is encoded**

Run: `grep -n -- "gh release create" .github/workflows/release.yml && grep -n -- "--generate-notes" .github/workflows/release.yml && grep -n -- "gh release upload" .github/workflows/release.yml && grep -n -- "--clobber" .github/workflows/release.yml && grep -n "GITHUB_TOKEN" .github/workflows/release.yml`
Expected: all matches found.

- [ ] **Step 5: Commit GitHub Release publication support**

```bash
git add .github/workflows/release.yml
git commit -m "ci: publish github releases for cli binaries"
```

### Task 8: Document the new release process

**Files:**
- Modify: `README.md`
- Test: `README.md`

- [ ] **Step 1: Write the failing docs expectation**

Document that README must explain:

- release PR workflow input format `X.Y.Z`
- merge then tag process `vX.Y.Z`
- supported release targets
- raw binary distribution
- keep the instructions version-agnostic so later releases do not require README churn

- [ ] **Step 2: Verify the new release process docs are missing**

Run: `grep -n "release PR" README.md && grep -n "vX.Y.Z" README.md`
Expected: missing or incomplete.

- [ ] **Step 3: Update `README.md` with a concise release section**

Add a section similar to:

```markdown
## Releases

Electrotest uses a two-step release flow:

1. run the release PR workflow with a version like `0.1.0`
2. review and merge the generated PR
3. push tag `v0.1.0`

The release workflow publishes raw binaries for:

- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-musl`

Keep this section generic: examples may use `0.1.0`, but the prose should describe the workflow pattern rather than claim a fixed current release version.
```

- [ ] **Step 4: Verify the README release section exists**

Run: `grep -n "## Releases" README.md && grep -n "x86_64-unknown-linux-musl" README.md && grep -n "release PR" README.md`
Expected: all matches found.

- [ ] **Step 5: Commit the documentation update**

```bash
git add README.md
git commit -m "docs: describe release pr and tag workflow"
```

### Task 9: Verify the workflows before handoff

**Files:**
- Modify: `.github/workflows/create-release-pr.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `README.md`

- [ ] **Step 1: Validate both workflow files structurally**

Run: `python - <<'PY'
from pathlib import Path
import sys

checks = {
    '.github/workflows/create-release-pr.yml': [
        'workflow_dispatch',
        'version:',
        'release/',
        'gh pr create',
    ],
    '.github/workflows/release.yml': [
        'tags:',
        'cargo test',
        'aarch64-apple-darwin',
        'x86_64-pc-windows-msvc',
        'x86_64-unknown-linux-musl',
        'SHA256SUMS.txt',
        'gh release create',
        'gh release upload',
    ],
}

for path, required in checks.items():
    text = Path(path).read_text()
    missing = [item for item in required if item not in text]
    if missing:
        print(f'{path} missing: {", ".join(missing)}')
        sys.exit(1)

print('workflow structure looks complete')
PY`
Expected: success with `workflow structure looks complete`.

- [ ] **Step 2: Lint both workflow files with a real workflow linter**

Run: `actionlint .github/workflows/create-release-pr.yml .github/workflows/release.yml`
Expected: PASS.

If `actionlint` is not installed locally, install it first with the platform-appropriate method and rerun the command before marking this step complete.

- [ ] **Step 3: Re-run the Rust test suite to ensure the repo still passes**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 4: Verify no unintended archive/installer publishing was introduced**

Run: `grep -n "tar.gz\|\.msi\|\.pkg" README.md .github/workflows/create-release-pr.yml .github/workflows/release.yml`
Expected: no matches for archive or installer publishing behavior beyond explicit out-of-scope wording if added to docs. Do not grep for plain `zip` because the repo already documents `trace.zip` as a test artifact.

- [ ] **Step 5: Record required GitHub-side verification after merge**

Add an implementation note in `README.md` or workflow comments that release readiness is not proven until maintainers perform:

- one successful `create-release-pr` run with a valid `X.Y.Z`
- one invalid release PR input case
- one successful tagged release with matching `vX.Y.Z`
- one intentional mismatch tag case that fails early
- one check that the actual uploaded GitHub Release assets are named exactly:
  - `electrotest-vX.Y.Z-aarch64-apple-darwin`
  - `electrotest-vX.Y.Z-x86_64-pc-windows-msvc.exe`
  - `electrotest-vX.Y.Z-x86_64-unknown-linux-musl`
  - `SHA256SUMS.txt`

- [ ] **Step 6: Confirm final git state**

Run: `git status --short`
Expected: clean tree or only the expected workflow and README changes if the final commit has not yet been made.

- [ ] **Step 7: Commit final verification adjustments if needed**

```bash
git add .github/workflows/create-release-pr.yml .github/workflows/release.yml README.md
git commit -m "chore: finalize release workflows"
```
