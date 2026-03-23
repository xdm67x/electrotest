# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build the project (development)
cargo build

# Build release binary
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Run the binary with example feature file
cargo run -- --pid 12345 --features ./examples/features/test.feature --output-dir ./output
```

## Rust Toolchain

The project uses a pinned Rust version (1.94) with specific targets defined in `rust-toolchain.toml`:
- `aarch64-apple-darwin` (macOS ARM)
- `x86_64-pc-windows-msvc` (Windows)
- `x86_64-unknown-linux-musl` (Linux musl)

## High-Level Architecture

Electrotest is a CLI tool that runs Gherkin feature files against Electron applications via the Chrome DevTools Protocol (CDP).

### Layer Structure

```
┌─────────────────────────────────────────────────────────────┐
│  CLI Layer (src/cli/)                                       │
│  - args.rs: clap-based CLI argument parsing                  │
│  - parser.rs: Regex-based Gherkin feature file parser        │
│  - runner.rs: Executes scenarios step-by-step                │
│  - context.rs: Shared state (CDP client, output dir)         │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│  Step Registry (src/cli/steps/)                             │
│  - Trait-based step handlers (StepHandler trait)             │
│  - Handlers implement can_handle() + execute()             │
│  - And/But keywords resolve to previous keyword type       │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│  CDP Layer (src/cdp/)                                         │
│  - WebSocket client using tokio-tungstenite                  │
│  - Request/response pattern with oneshot channels          │
│  - Auto-incrementing message IDs for CDP correlation         │
│  - HTTP discovery via /json/list endpoint                    │
└─────────────────────────────────────────────────────────────┘
```

### Key Patterns

**Step Handler Pattern**: New Gherkin steps are added by implementing `StepHandler` in `src/cli/steps/` and registering in `StepRegistry::new()`. Each handler checks `can_handle()` with regex-like matching on step text.

**CDP Request/Response**: The CdpClient uses a `HashMap<MessageId, oneshot::Sender>` to correlate async CDP responses. A background task reads WebSocket messages and routes them to waiting requesters.

**PID to Port Discovery**: The CLI extracts `--remote-debugging-port` from the Electron process command line using sysinfo, then discovers CDP targets via HTTP before establishing WebSocket connection.

### Adding New Steps

1. Create a struct implementing `StepHandler` in appropriate `src/cli/steps/` file
2. Implement `can_handle()` to match step text pattern
3. Implement `execute()` using `ctx.cdp_client.evaluate()` or other CDP methods
4. Register in `StepRegistry::new()` in `src/cli/steps/mod.rs`

### Testing

The parser has unit tests in `src/cli/parser.rs` using `#[cfg(test)]`. There are no integration tests - testing requires a running Electron process with remote debugging enabled.
