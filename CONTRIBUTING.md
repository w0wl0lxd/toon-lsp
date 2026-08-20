# Contributing to toon-lsp

## DCO
This project requires the [Developer Certificate of Origin](DCO.txt) (DCO).
All contributions fall under the dual license (AGPL-3.0-only + Commercial).
The DCO certifies you have the right to submit. We need it to distribute your code under both licenses.
### Sign off
Add `Signed-off-by` to every commit:
```
feat(parser): Add support for nested arrays

Signed-off-by: Your Name <your.email@example.com>
```
Add it with `git commit -s`:
```bash
git commit -s -m "feat(parser): Add support for nested arrays"
```

## What we accept
- **Bug fixes** - fix existing behavior
- **Docs** - improve docs, examples, comments
- **Tests** - add coverage
- **Performance** - optimize with benchmarks

## Discuss first
Open an issue or discussion before you start:
- **New features** - align on design
- **Breaking changes** - coordinate with release plan
- **Large refactors** - agree on approach

## Code style
- Run `cargo fmt` before you commit
- Run `cargo clippy -- -D warnings` and fix all warnings
- Add tests for new behavior
- Update docs when behavior changes

## Git hooks (optional)
`.githooks/` has `pre-commit` (runs `cargo fmt` + `cargo clippy`) and `commit-msg` (checks DCO).
Enable once per clone: `git config core.hooksPath .githooks`

## Commit messages
Follow [Conventional Commits](https://www.conventionalcommits.org/):
```
<type>(<scope>): <description>

[body]

Signed-off-by: Your Name <your.email@example.com>
```
Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore` / Scopes: `scanner`, `parser`, `ast`, `lsp`, `docs`

## Pull requests
1. Fork the repo and branch from `main`
2. Add DCO sign-off to every commit
3. Run `cargo test` and `cargo clippy -- -D warnings`
4. Open a pull request

## Questions
- [GitHub Discussions](https://github.com/w0wl0lxd/toon-lsp/discussions)
- Email: w0wl0lxd@tuta.com

## License
You agree to license your contributions under the dual license.
See [LICENSING.md](LICENSING.md) for details.
