# Contributing to osm-pathfinder

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/<your-username>/osm-pathfinder`
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run checks: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
6. Commit and push: `git push origin feature/my-feature`
7. Open a Pull Request

## Code Quality

Before submitting, ensure:

- [ ] `cargo fmt` — code is formatted
- [ ] `cargo clippy -- -D warnings` — no lint warnings
- [ ] `cargo test` — all tests pass
- [ ] New public APIs have `///` doc comments
- [ ] Non-trivial logic has unit tests

## Architecture Decision Records

For significant design changes, create an ADR in `docs/decisions/` following the existing format. See [docs/decisions/README.md](docs/decisions/README.md).

## Reporting Issues

Open a GitHub issue with:
- Steps to reproduce
- Expected vs. actual behavior
- Relevant logs or error messages
