#!/bin/bash
set -euo pipefail

# =============================================================================
# Electrotest Pilot Helper Script
# Auto-discovers the Electron PID and runs electrotest.
# =============================================================================

FEATURES=""
OUTPUT_DIR="./output"
PID=""
ELECTROTEST_BIN=""

usage() {
  cat <<EOF
Usage: $(basename "$0") --features <path> [options]

Required:
  --features <path>     Path to the .feature file

Optional:
  --output-dir <dir>    Output directory for screenshots (default: ./output)
  --pid <pid>           Electron process PID (auto-discovered if omitted)
  --bin <path>          Path to electrotest binary (auto-detected if omitted)
  -h, --help            Show this help message
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --features)
      FEATURES="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --pid)
      PID="$2"
      shift 2
      ;;
    --bin)
      ELECTROTEST_BIN="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

# Validate required arguments
if [[ -z "$FEATURES" ]]; then
  echo "Error: --features is required" >&2
  usage >&2
  exit 1
fi

if [[ ! -f "$FEATURES" ]]; then
  echo "Error: Feature file not found: $FEATURES" >&2
  exit 1
fi

# =============================================================================
# 1. Discover electrotest binary
# =============================================================================
if [[ -z "$ELECTROTEST_BIN" ]]; then
  if command -v electrotest &>/dev/null; then
    ELECTROTEST_BIN="electrotest"
  elif [[ -f "./target/release/electrotest" ]]; then
    ELECTROTEST_BIN="./target/release/electrotest"
  elif [[ -f "./target/debug/electrotest" ]]; then
    ELECTROTEST_BIN="./target/debug/electrotest"
  elif [[ -f "Cargo.toml" ]] && grep -q "^name = \"electrotest\"" Cargo.toml 2>/dev/null; then
    echo "Info: electrotest binary not found, falling back to 'cargo run --'" >&2
    ELECTROTEST_BIN="cargo run --"
  else
    echo "Error: Could not find electrotest binary. Please install it or run from the electrotest repo." >&2
    exit 1
  fi
fi

# =============================================================================
# 2. Discover Electron PID (if not provided)
# =============================================================================
if [[ -z "$PID" ]]; then
  echo "Info: Auto-discovering Electron PID..." >&2

  # Method 1: pgrep
  PID=$(pgrep -f "electron.*remote-debugging-port" 2>/dev/null | head -n 1 || true)

  # Method 2: lsof by common ports
  if [[ -z "$PID" ]]; then
    for port in 9222 9223 9224 9225; do
      PID=$(lsof -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | head -n 1 || true)
      if [[ -n "$PID" ]]; then
        echo "Info: Found Electron on port $port (PID: $PID)" >&2
        break
      fi
    done
  fi

  # Method 3: ps aux fallback
  if [[ -z "$PID" ]]; then
    PID=$(ps aux | grep "[e]lectron.*remote-debugging-port" | awk '{print $2}' | head -n 1 || true)
  fi

  if [[ -z "$PID" ]]; then
    echo "Error: Could not auto-discover Electron PID." >&2
    echo "       Please ensure the Electron app is running with --remote-debugging-port." >&2
    echo "       Or provide the PID explicitly with --pid <PID>." >&2
    exit 1
  fi

  echo "Info: Using Electron PID: $PID" >&2
fi

# Verify the PID exists
if ! kill -0 "$PID" 2>/dev/null; then
  echo "Error: Process $PID does not exist or is not accessible" >&2
  exit 1
fi

# =============================================================================
# 3. Run electrotest
# =============================================================================
mkdir -p "$OUTPUT_DIR"

echo "Running: $ELECTROTEST_BIN --pid $PID --features $FEATURES --output-dir $OUTPUT_DIR" >&2

if [[ "$ELECTROTEST_BIN" == "cargo run --" ]]; then
  cargo run -- --pid "$PID" --features "$FEATURES" --output-dir "$OUTPUT_DIR"
else
  "$ELECTROTEST_BIN" --pid "$PID" --features "$FEATURES" --output-dir "$OUTPUT_DIR"
fi
