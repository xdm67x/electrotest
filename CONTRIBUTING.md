# Contributing to electrotest

Thanks for contributing! Before opening a pull request, please follow the guidelines below.

## Commit Messages

Every commit **must follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) pattern**, and **the scope must always be declared**:

```
<type>(<scope>): <short summary in lowercase, imperative mood>
```

Both `type` and `scope` are required — commits without a scope will be asked to be amended.

### Type

Use one of the standard Conventional Commits types:

| Type       | Purpose                                            |
| ---------- | -------------------------------------------------- |
| `feat`     | New feature                                        |
| `fix`      | Bug fix                                            |
| `docs`     | Documentation only                                 |
| `style`    | Formatting (code that does not change meaning)     |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf`     | Performance improvement                            |
| `test`     | Adding or fixing tests                             |
| `build`    | Build system or external dependencies              |
| `ci`       | CI configuration and workflows                    |
| `chore`    | Maintenance tasks that don't fit the above         |
| `revert`   | Reverting a previous commit                        |

For breaking changes, append `!` before the colon: `feat(parser)!: ...` and explain the change in the commit body.

### Scope

The scope identifies the part of the codebase the commit affects. Use the lowercase name of the module, component, or area — for example:

- `cli`, `launcher`, `parser`, `runner`, `context`, `steps` — modules under `src/cli/`
- `cdp` — the CDP client under `src/cdp/`
- `deps` — dependency updates in `Cargo.toml`/`Cargo.lock`
- `release` — version bumps, tags, release tooling
- `ci` — GitHub Actions workflows and mise tasks
- `docs` — top-level documentation (`README.md`, `AGENTS.md`, this file)

If a commit genuinely spans the whole project, use the broadest accurate scope rather than omitting it.

### Examples

```
feat(steps): add scroll step handlers
fix(launcher): kill child renderer processes on exit
docs(readme): document the --app-path launch mode
chore(deps): update sysinfo to 0.39.6
ci(release): use mise-action for toolchain installs
```

## Building and Testing

See [AGENTS.md](AGENTS.md) for build commands, mise tasks, and architecture notes. Run `cargo fmt` before committing, and make sure `cargo test` and `cargo clippy` pass.
