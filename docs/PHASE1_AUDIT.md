# Phase 1 audit

## Host

- macOS 26.5.2, Apple silicon (`aarch64`)
- Xcode 26.6 with iOS 26.5 simulators
- Rust 1.99.0
- Swift 6.3.3
- XcodeGen 2.46.0
- Python 3.12 via `uv`
- Node 26

## Baseline findings

- The upstream Python test suite passed during the initial audit.
- `xcode-select -p` points at `/Library/Developer/CommandLineTools`; Xcode is
  usable by setting `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer`.
- `adb`, Java, and the Android SDK are not installed. `scrcpy` and `ffmpeg` are
  also unavailable.
- UTM is available as a fallback for an Ubuntu Android toolchain VM.
- The Git remote still points at the upstream repository. Do not push until the
  intended OmniPilot remote is configured.
- The working tree contains the upstream Python tree plus the new Rust and Apple
  work. Nothing has been committed.

## Decisions

- Build Apple first: validate macOS host tooling, then iOS Simulator automation.
- Use XCUITest for iOS because non-jailbroken iOS apps cannot be driven by an
  external process alone.
- Use a long-lived UI-test runner and the same NDJSON protocol as Android.
- Keep verification in the engine, not in adapters.
- Require explicit typed steps until a planner is implemented.
- Do not enable unrestricted shell execution or hidden OS bypasses.
- Keep local-first operation and avoid mandatory paid services.

## Current verification status

- `cargo check --workspace --all-targets`: passed.
- `cargo test --workspace`: 21 tests passed (3 policy, 7 engine flow, 5 protocol
  flow, 1 schema, 3 iOS adapter, 1 MCP, 1 CLI).
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo build --release --workspace`: passed.
- `target/release/omnipilot demo`: completed and verified.
- iOS UI tests: 2 passed, 0 failures on iPhone 17.
- Generic iOS device build: passed with code signing disabled.
- iOS adapter host smoke: initialize, observe, act, verify, shutdown passed.
- `target/release/omnipilot ios-demo`: completed and verified on iPhone 17.
- `target/release/omnipilot run --platform ios`: completed and verified with
  fresh hierarchy and screenshot evidence.
- MCP iOS smoke: `list_devices` returned the simulator and mock device; a typed
  `run_task` completed and verified with 4 evidence items.
- Simulator lifecycle smoke: `ios-boot` and `ios-shutdown` were idempotent on
  iPhone 17e.
