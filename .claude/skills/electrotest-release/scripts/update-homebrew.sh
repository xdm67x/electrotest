#!/bin/bash
# Update Homebrew tap for electrotest
# Usage: ./update-homebrew.sh VERSION

set -e

VERSION="$1"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 VERSION"
    echo "Example: $0 0.2.0"
    exit 1
fi

# Remove 'v' prefix if provided
VERSION="${VERSION#v}"

echo "Updating homebrew tap for electrotest v${VERSION}..."

# Create temp directory
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# Download release artifacts
echo "Downloading release artifacts..."

MAC_URL="https://github.com/xdm67x/electrotest/releases/download/v${VERSION}/electrotest-macos-arm64.tar.gz"
LINUX_URL="https://github.com/xdm67x/electrotest/releases/download/v${VERSION}/electrotest-linux-x64.tar.gz"

curl -sL -o "$TMPDIR/electrotest-macos-arm64.tar.gz" "$MAC_URL"
curl -sL -o "$TMPDIR/electrotest-linux-x64.tar.gz" "$LINUX_URL"

# Calculate SHA256 checksums
echo "Calculating SHA256 checksums..."

SHA256_MAC=$(shasum -a 256 "$TMPDIR/electrotest-macos-arm64.tar.gz" | cut -d' ' -f1)
SHA256_LINUX=$(shasum -a 256 "$TMPDIR/electrotest-linux-x64.tar.gz" | cut -d' ' -f1)

echo "macOS ARM SHA256: $SHA256_MAC"
echo "Linux x64 SHA256: $SHA256_LINUX"

# Output the formula
cat << RUBY
class Electrotest < Formula
  desc "CLI tool for testing Electron applications using Gherkin syntax"
  homepage "https://github.com/xdm67x/electrotest"
  version "${VERSION}"

  on_macos do
    on_arm do
      url "${MAC_URL}"
      sha256 "${SHA256_MAC}"
    end
  end

  on_linux do
    on_intel do
      url "${LINUX_URL}"
      sha256 "${SHA256_LINUX}"
    end
  end

  def install
    bin.install "electrotest"
  end

  test do
    system "#{bin}/electrotest", "--version"
  end
end
RUBY

echo ""
echo "Formula generated. Copy this to your homebrew-tap repo as electrotest.rb"
