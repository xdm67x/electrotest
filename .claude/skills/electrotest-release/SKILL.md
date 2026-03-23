---
name: electrotest-release
description: Automate the complete release process for the electrotest project including version bumping, changelog generation, git tagging, GitHub Actions CI workflow, and Homebrew tap updates. Use this skill when the user wants to release a new version, create a tag, generate a changelog, set up CI/CD, or update the Homebrew formula.
---

# Electrotest Release Skill

This skill automates the complete release workflow for the electrotest project (a Rust CLI tool for testing Electron applications using Gherkin syntax).

## Targets

The project builds for 3 targets as defined in `rust-toolchain.toml`:
- `aarch64-apple-darwin` - macOS ARM (Apple Silicon)
- `x86_64-pc-windows-msvc` - Windows x86_64
- `x86_64-unknown-linux-musl` - Linux x86_64 (musl for static linking)

## Release Process

### Phase 1: Pre-release Validation

Before starting, verify:
- You're on the main branch (`git branch --show-current`)
- Working directory is clean (`git status`)
- Tests pass (`cargo test`)

Ask the user for the new version (semver format: `0.1.0`, `1.2.3`) or suggest automatic version bump (patch/minor/major).

### Phase 2: Version Update

**Update `Cargo.toml`:**
```toml
[package]
name = "electrotest"
version = "NEW_VERSION"
```

**Update `Cargo.lock`:**
```bash
cargo update --workspace
```

### Phase 3: Changelog Generation

Generate changelog from git commits since the last tag:

```bash
# Get the last tag
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

# If no previous tag, get all commits
if [ -z "$LAST_TAG" ]; then
  COMMITS=$(git log --pretty=format:"- %s" --no-merges)
else
  COMMITS=$(git log ${LAST_TAG}..HEAD --pretty=format:"- %s" --no-merges)
fi
```

Create or update `CHANGELOG.md` with this structure:
```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [VERSION] - YYYY-MM-DD

### Changes
$COMMITS

**Full Changelog**: https://github.com/xdm67x/electrotest/compare/PREV_TAG...vVERSION
```

If this is the first release, use:
```markdown
## [VERSION] - YYYY-MM-DD

### Initial Release
- First release of electrotest
```

### Phase 4: Commit Version Changes

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to VERSION"
```

### Phase 5: Create and Push Tag

```bash
# Create annotated tag
git tag -a "vVERSION" -m "Release vVERSION"

# Push everything
git push origin main
git push origin "vVERSION"
```

### Phase 6: Monitor CI Run

After pushing the tag, the CI workflow (`.github/workflows/release.yml`) triggers automatically. Monitor the workflow run to know when to proceed with the Homebrew tap update:

```bash
# Check latest workflow run status (requires gh CLI)
gh run list --workflow=release.yml --limit=1

# Watch the run in real-time
gh run watch

# Or poll until completion
while true; do
  STATUS=$(gh run list --workflow=release.yml --limit=1 --json conclusion --jq '.[0].conclusion')
  if [ "$STATUS" = "success" ]; then
    echo "CI succeeded!"
    break
  elif [ "$STATUS" = "failure" ] || [ "$STATUS" = "cancelled" ]; then
    echo "CI failed with status: $STATUS"
    exit 1
  fi
  echo "CI still running... checking again in 30 seconds"
  sleep 30
done
```

**Verify GitHub Release created:**

Once CI succeeds, verify the GitHub release exists:

```bash
gh release view "vVERSION"
```

Only proceed to Phase 7 (Homebrew tap update) after the CI succeeds and the GitHub Release is created with all artifacts.

### Phase 7: Homebrew Tap Update

After the CI completes successfully and the GitHub Release is created, update the Homebrew tap.

The Homebrew tap is at `https://github.com/xdm67x/homebrew-tap`.

**Step 7.1: Get the SHA256 checksums from the release**

```bash
# Download and calculate SHA256 for macOS ARM
curl -sL -o /tmp/electrotest-macos-arm64.tar.gz \
  "https://github.com/xdm67x/electrotest/releases/download/vVERSION/electrotest-macos-arm64.tar.gz"
SHA256_MAC=$(shasum -a 256 /tmp/electrotest-macos-arm64.tar.gz | cut -d' ' -f1)

# Download and calculate SHA256 for Linux x64
curl -sL -o /tmp/electrotest-linux-x64.tar.gz \
  "https://github.com/xdm67x/electrotest/releases/download/vVERSION/electrotest-linux-x64.tar.gz"
SHA256_LINUX=$(shasum -a 256 /tmp/electrotest-linux-x64.tar.gz | cut -d' ' -f1)
```

**Step 7.2: Update the Homebrew formula**

Clone or update the homebrew-tap repo, then create/update `electrotest.rb`:

```ruby
class Electrotest < Formula
  desc "CLI tool for testing Electron applications using Gherkin syntax"
  homepage "https://github.com/xdm67x/electrotest"
  version "VERSION"

  on_macos do
    on_arm do
      url "https://github.com/xdm67x/electrotest/releases/download/vVERSION/electrotest-macos-arm64.tar.gz"
      sha256 "SHA256_MAC"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/xdm67x/electrotest/releases/download/vVERSION/electrotest-linux-x64.tar.gz"
      sha256 "SHA256_LINUX"
    end
  end

  def install
    bin.install "electrotest"
  end

  test do
    system "#{bin}/electrotest", "--version"
  end
end
```

**Step 7.3: Commit and push to homebrew-tap**

```bash
cd /path/to/homebrew-tap
git add electrotest.rb
git commit -m "electrotest: update to VERSION"
git push origin main
```

## Complete Release Sequence

1. **Validate** - Check branch, clean working directory, run tests
2. **Bump version** - Update `Cargo.toml` and `Cargo.lock`
3. **Changelog** - Generate from git commits
4. **Commit** - "chore: bump version to X.Y.Z"
5. **Tag** - Create annotated tag `vX.Y.Z`
6. **Push & Monitor** - Push main branch and tag, monitor CI until success
7. **Verify Release** - Confirm GitHub Release was created with all artifacts
8. **Homebrew** - Update tap with new URLs and SHA256 checksums

## Common Commands

```bash
# Full release workflow
claude "release version 0.2.0"

# Setup CI workflow file (one-time setup)
claude "setup release CI workflow"

# Update homebrew tap after release
claude "update homebrew tap to version 0.2.0"

# Create changelog only
claude "generate changelog for version 0.2.0"

# Monitor latest CI run
claude "monitor release CI status"
```

## Important Notes

- Always use annotated tags (`git tag -a`) to include metadata
- CI triggers automatically when a tag is pushed - monitor it to know when to proceed
- SHA256 checksums are required for Homebrew - they change with each build
- Windows uses `.exe` extension and `.zip` archives
- The homebrew-tap repository is separate and requires additional push permissions
- Wait for CI to succeed before updating Homebrew (artifacts must exist)
- Only update the GitHub Release changelog after CI succeeds

## Files Modified/Created

- `Cargo.toml` - version updated
- `Cargo.lock` - updated via cargo update
- `CHANGELOG.md` - new entry added
- `.github/workflows/release.yml` - CI workflow (created once in project root)
- `electrotest.rb` (in homebrew-tap repo) - formula updated
