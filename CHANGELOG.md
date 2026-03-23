# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-03-23

### Initial Release

First release of electrotest - a CLI tool for testing Electron applications using Gherkin syntax.

### Features
- CLI tool with Gherkin test support
- CDP (Chrome DevTools Protocol) support for Electron automation
- Automatic refresh for Electron processes every 2 seconds
- Interactive Electron process picker
- Cross-platform builds (macOS ARM, Linux x86_64, Windows x86_64)
- GitHub Actions CI/CD workflow for automated releases

### Commits
- docs: add CLAUDE.md with build commands and architecture overview
- docs: add comprehensive README with usage examples and architecture overview
- feat: add electrotest-release skill and GitHub Actions workflow
- chore: add rust-toolchain.toml for cross-platform builds
- chore: update Rust dependencies to latest versions
- fix: resolve all compiler warnings
- refactor: replace TUI with CLI and add Gherkin test support
- feat: add CDP (Chrome DevTools Protocol) support for Electron automation
- feat: add automatic refresh for Electron processes every 2 seconds
- feat: remove unnecessary
- Refactor TUI into focused prompt modules
- Replace clap CLI with ratatui Electron picker
- Add interactive Electron prompt with crossterm
- feat: zed config for oxfmt
- feat: basic electron example app
- feat: basic cli
