# Electrotest

A CLI automation tool for testing Electron applications using Gherkin syntax and the Chrome DevTools Protocol (CDP).

## Overview

Electrotest enables you to write end-to-end tests for Electron applications using human-readable Gherkin feature files. It connects to a running Electron process via the Chrome DevTools Protocol to execute navigation, interaction, and assertion steps.

## Features

- **Gherkin-based testing**: Write tests in natural language using `.feature` files
- **CDP integration**: Connects directly to Electron's remote debugging protocol
- **Screenshot capture**: Automatically capture screenshots during test execution
- **Navigation & interaction**: Navigate to URLs, interact with elements, and verify page content
- **Async runtime**: Built on Tokio for efficient async operations

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/electrotest.git
cd electrotest

# Build the project
cargo build --release

# The binary will be available at target/release/electrotest
```

## Usage

Electrotest supports two modes of operation:

### Mode 1: Connect to a running Electron process

#### 1. Start your Electron app with remote debugging enabled

```bash
electron --remote-debugging-port=9223 .
```

#### 2. Run the tests

```bash
electrotest --pid <ELECTRON_PID> --features ./test.feature --output-dir ./output
```

The application will **remain running** after tests complete.

### Mode 2: Launch and test (automatic)

Electrotest can automatically launch your Electron app, run tests, and close it.

**With auto-detection of Electron executable:**
```bash
electrotest \
  --app-path /path/to/your/app \
  --features ./test.feature \
  --output-dir ./output
```

Electrotest will automatically detect the Electron executable from `node_modules/.bin/electron` in your project directory (or parent directories for monorepos).

**With explicit Electron path:**
```bash
electrotest \
  --electron-path /path/to/electron \
  --app-path /path/to/your/app \
  --features ./test.feature \
  --output-dir ./output \
  --port 9222
```

The application will be **automatically terminated** after tests complete.

### Create a feature file

Create a `.feature` file describing your test scenarios:

```gherkin
Feature: Application smoke test

  Scenario: Verify homepage loads correctly
    Given the window size is 1920x1080

    When I navigate to "https://example.com"
    And I take a screenshot "homepage.png"
    And I wait 1 seconds

    Then the page should contain "Example Domain"
```

### Command-line options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--pid` | `-p` | PID of an already running Electron process to connect to | - |
| `--electron-path` | - | Path to the Electron executable (launch mode). Optional - auto-detected from `--app-path` if not provided | - |
| `--app-path` | - | Path to your Electron app directory or main file (launch mode) | - |
| `--port` | - | Port for remote debugging (auto-incremented if in use) | `9222` |
| `--app-args` | - | Additional arguments to pass to the Electron app | `""` |
| `--features` | `-f` | Path to the `.feature` file | Required |
| `--output-dir` | `-o` | Output directory for screenshots | `./output` |

**Note:** Either `--pid` or `--app-path` (with optional `--electron-path`) must be provided.

## Supported Gherkin Steps

### Navigation
- `Given the window size is <WIDTH>x<HEIGHT>` - Set the browser window size
- `When I navigate to "<URL>"` - Navigate to a URL

### Interaction
- `And I wait <N> seconds` - Pause execution for N seconds
- `And I take a screenshot "<FILENAME>"` - Capture a screenshot

### Assertions
- `Then the page should contain "<TEXT>"` - Verify page contains text

## Project Structure

```
electrotest/
├── src/
│   ├── main.rs           # Application entry point
│   ├── cli/              # CLI argument parsing and runner
│   │   ├── args.rs       # CLI argument definitions
│   │   ├── context.rs    # Test execution context
│   │   ├── feature.rs    # Feature file structures
│   │   ├── parser.rs     # Gherkin feature file parser
│   │   ├── runner.rs     # Test execution engine
│   │   └── steps/        # Step implementations
│   │       ├── navigation.rs
│   │       ├── interaction.rs
│   │       └── assertion.rs
│   └── cdp/              # Chrome DevTools Protocol client
│       ├── client.rs     # WebSocket CDP client
│       └── messages.rs   # CDP message types
├── examples/
│   ├── features/         # Example Gherkin feature files
│   └── electron-app/     # Sample Electron application
└── Cargo.toml
```

## Architecture

Electrotest uses a layered architecture:

1. **CLI Layer**: Parses command-line arguments and orchestrates test execution
2. **Parser Layer**: Reads and parses Gherkin `.feature` files into structured data
3. **Runner Layer**: Executes scenarios step by step, managing state and context
4. **CDP Layer**: Communicates with Electron via WebSocket using the Chrome DevTools Protocol

The CDP client sends commands to control the browser (navigate, resize, execute JavaScript) and receives events (page load, console messages) to enable assertions and synchronization.

## Dependencies

- **tokio** - Async runtime
- **tokio-tungstenite** - WebSocket client for CDP communication
- **clap** - CLI argument parsing
- **serde** - JSON serialization for CDP messages
- **sysinfo** - Process information for finding CDP port
- **reqwest** - HTTP client for CDP discovery
- **anyhow** - Error handling

## Development

```bash
# Run tests
cargo test

# Run with example
cargo run -- --pid 12345 --features ./examples/features/test.feature
```

## License

MIT
