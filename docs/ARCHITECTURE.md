# Architecture

## Goal

OmniPilot turns a typed mobile task into a sequence of native actions, then
verifies the outcome against fresh evidence. The engine, not the adapter, owns
retry policy, budgets, assertions, and final task status.

## Crates

| Crate | Responsibility |
| --- | --- |
| `omnipilot-core` | Shared models: actions, selectors, assertions, tasks, results, evidence, errors, IDs. |
| `omnipilot-protocol` | JSON-RPC 2.0 contract, NDJSON server/client, adapter trait, schema bundle. |
| `omnipilot-engine` | Policy gate, executor, verification, retries, cancellation, traces, evidence stores. |
| `omnipilot-mock` | Deterministic in-process adapter used by tests, the demo, and protocol conformance. |
| `omnipilot-ios` | Launches the XCUITest adapter process, connects over loopback, and implements `DeviceAdapter`. |
| `omnipilot-cli` | `omnipilot` binary: doctor, devices, ios-devices, ios-demo, run, demo, protocol schema, mcp. |
| `omnipilot-mcp` | MCP stdio server exposing device and task tools; mock by default, optional iOS adapter. |

## Execution flow

1. Validate the `TaskSpec`: goal, budgets, steps, and assertions.
2. Capture an initial observation.
3. For each step:
   - Ask the policy whether the action is allowed.
   - Ask the adapter to perform the action.
   - Capture an action observation when the adapter returns one.
   - Retry retryable failures within the step and task budgets.
4. Capture a fresh final observation after the last step.
5. Evaluate every success criterion against that final observation.
6. Persist evidence and events.
7. Return `Completed`, `Failed`, `Blocked`, `Cancelled`, or `TimedOut`.

A task is `verified` only when at least one success criterion exists, every
criterion passes, and the final observation is fresh. A task with no criteria is
reported as completed but unverified, with an explicit limitation.

## Adapter contract

Adapters implement `AdapterService` for the wire protocol and `DeviceAdapter`
for in-process use. The protocol supports:

- `initialize` with a version and per-session bearer token
- `observe` with optional screenshot and hierarchy
- `act` with a typed action
- `logs`
- `cancel`
- `shutdown`

The server enforces handshake-before-use and rejects a second initialization.
Adapters must report unsupported capabilities honestly instead of silently
degrading behavior.

## Apple path

The iOS target contains a SwiftUI demo app, a fast XCUITest suite, and a
long-lived protocol adapter. XCUITest is the only supported way to drive another
app on a non-jailbroken iOS device, so the adapter runs inside an XCTest process.

Implemented adapter flow:

1. The macOS host (`omnipilot-ios`) launches `xcodebuild test` with the
   `OmniPilotAdapter` scheme.
2. The test runner starts an `NWListener` on loopback and prints
   `OMNIPILOT_ADAPTER_READY port=<port> token=<session-token>`.
3. The host parses the line, redacts the token from logs, connects over TCP, and
   performs the handshake.
4. The runner maps `observe` to an XCUITest hierarchy snapshot and `act` to
   XCUITest gestures and app lifecycle calls.
5. The host performs verification and stores screenshots, hierarchy, step
   results, and `events.jsonl` under `traces/<task_id>/`.
6. The host sends `shutdown`; the test exits and `xcodebuild` terminates.

SwiftUI `Toggle` elements are tapped at normalized offset `(0.9, 0.5)` because a
center tap can hit the row label instead of the control.

The MCP server uses the same runtime. It starts with the mock adapter and, when
`--ios` is set, launches the XCUITest adapter before serving stdio.

Simulator discovery and boot/shutdown lifecycle use `simctl` through
`omnipilot ios-devices`, `ios-boot`, and `ios-shutdown`. Physical-device support
still requires a provisioned runner and explicit signing.

## Android path

Planned adapter flow:

1. Build a Kotlin instrumentation APK using `UiAutomation`/UI Automator.
2. Launch it on the selected emulator or device through `adb`.
3. Forward a loopback port from the host to the instrumentation process.
4. Speak the same versioned NDJSON protocol.

Android tooling is not installed on the current host. A UTM Ubuntu VM is an
available fallback, but the adapter must not assume a VM.

## Evidence

Evidence is stored under `traces/<task_id>/` by default, including screenshots,
hierarchy snapshots, and `events.jsonl`. Set `OMNIPILOT_TRACES` to change the
root. Evidence is referenced by ID from task results rather than embedded in
every MCP response.
