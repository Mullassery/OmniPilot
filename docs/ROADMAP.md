# Roadmap

## Phase 0: provenance and audit

- [x] Audit the upstream checkout, tests, tooling, and licensing.
- [x] Preserve upstream README files under `docs/upstream/`.
- [x] Document provenance in `NOTICE` and `docs/UPSTREAM.md`.
- [ ] Replace the upstream `origin` with the intended OmniPilot remote before
      any push.

## Phase 1: core contract

- [x] Versioned task, action, assertion, observation, result, and evidence models.
- [x] Normalized error taxonomy.
- [x] Policy gate with UI-mutation and app-allowlist controls.
- [x] Verification-first executor with budgets, retries, cancellation, and traces.
- [x] NDJSON JSON-RPC adapter protocol with handshake-before-use.
- [x] Deterministic mock adapter and demo scenario.
- [x] CLI and MCP stdio server.
- [x] Protocol/engine integration tests.

## Phase 2: Apple-first vertical slice

- [x] Rust workspace builds in release mode on macOS.
- [x] `omnipilot demo` completes and verifies the mock scenario.
- [x] SwiftUI demo app with stable accessibility identifiers.
- [x] XCUITest target builds and passes on the iPhone 17 simulator.
- [x] Generic iOS device build compiles without signing.
- [x] Long-lived XCUITest protocol adapter over loopback.
- [x] Simulator discovery through `simctl` (`omnipilot ios-devices`).
- [x] CLI iOS lifecycle: `ios-demo` and `run --platform ios`.
- [x] End-to-end iOS task: host -> XCUITest adapter -> verification -> evidence.
- [x] Simulator boot/shutdown lifecycle (`ios-boot`, `ios-shutdown`).
- [ ] Physical-device runner.
- [x] MCP wiring for the iOS simulator adapter (`mcp --ios --ios-device <udid>`).
- [x] macOS packaging/install instructions for the CLI and MCP server (`docs/INSTALL_MACOS.md`).

## Phase 3: Android

- [ ] Install or provision JDK, Android SDK, `adb`, and an emulator.
- [ ] Kotlin instrumentation adapter using `UiAutomation`/UI Automator.
- [ ] adb port forwarding and process lifecycle.
- [ ] Capability negotiation and unsupported-action reporting.
- [ ] End-to-end Android task with the same engine and protocol.

## Phase 4: ergonomics and scale

- [x] MCP client examples and tool documentation (`docs/MCP.md`, `examples/mcp/client.py`).
- [ ] Natural-language planner that emits typed plans for approval.
- [x] JUnit XML output for CLI task runs.
- [x] Persisted task records and Markdown/JUnit/JSON report export.
- [ ] XCTest/JUnit runner integrations and CI.
- [ ] Interactive evidence viewer.
- [ ] CI for Rust, Apple, and Android.
- [ ] Packaged releases and signed binaries.

## Non-goals for v1

- Unrestricted shell or OS-level bypasses.
- Device-fleet management.
- A desktop GUI.
- Mandatory cloud or paid model dependencies.
- Claiming success without post-action verification.
