# Contributing to OmniPilot

Thanks for contributing. This is an alpha project built in public; review times
vary and there is no merge SLA.

## Before you start

- Read [`README.md`](README.md), [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md),
  and [`docs/ROADMAP.md`](docs/ROADMAP.md) for the current capability boundary.
- For a new adapter or action type, open an issue first; the typed plan and
  evidence model are part of the contract.
- Security issues go through [`SECURITY.md`](SECURITY.md), not public issues.

## Development setup

Requires Rust 1.90+ (and Xcode 26+ for the iOS adapter on macOS).

```bash
cargo build --workspace
cargo test --workspace
```

## Checks before a pull request

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
```

CI runs the same checks. New behavior needs a test; changed limitations need a
README or docs update.

## Pull requests

- Keep changes focused and explain the verification you actually ran.
- Never commit device dumps, screenshots, credentials, or customer data.
- Do not claim an action is verified unless a fresh observation and assertion
  back it.

## License

Contributions are accepted under the Apache License 2.0.
