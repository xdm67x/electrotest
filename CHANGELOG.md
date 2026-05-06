# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2026-05-06

### Changes
- style: apply cargo fmt
- docs: add cargo fmt requirement before commit
- chore: remove .vscode directory
- fix(ci): add rustfmt component to toolchain
- fix(ci): specify Rust toolchain 1.95.0 in workflows
- refactor: simplify code for junior devs and fix all clippy warnings
- ci: add clippy component and update Rust to 1.95.0
- fix: properly terminate Electron app and all child processes after tests
- fix(ci): add npm install for electron-app dependencies

**Full Changelog**: https://github.com/xdm67x/electrotest/compare/v0.3.0...v0.4.0

## [0.3.0] - 2026-03-24

### Changes
- ci: add GitHub Actions workflow for build and test on push
- feat: auto-detect Electron path from node_modules
- feat: add launch mode to automatically start and stop Electron app
- test: add unit tests for all step handlers and feature validation files
- feat: add type text step for typing into input fields
- feat: add electrotest-pilot skill for Gherkin automation
- refactor: move release skill to generic .agents/skills/ directory
- docs: add AGENTS.md and symlink CLAUDE.md to it
- chore: move from zed to vscode
- docs: update skill to use Formula/ subdirectory for homebrew

**Full Changelog**: https://github.com/xdm67x/electrotest/compare/v0.2.0...v0.3.0

## [0.2.0] - 2026-03-24

### Changes
- fix: click on the button

**Full Changelog**: https://github.com/xdm67x/electrotest/compare/v0.1.0...v0.2.0

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
