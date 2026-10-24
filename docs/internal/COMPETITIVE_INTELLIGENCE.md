# Competitive intelligence

## OSS gap analysis — 2026-10-24

**What matters most here:** Android and physical-device coverage. OmniPilot's verification-first story is only credible where teams actually test, and Appium, Maestro, and mobile-mcp already cover both platforms on emulators and real hardware. Missing Android is a scope gap, not a polish item.

Scope: OmniPilot 0.1.0 (Rust workspace: core, protocol, engine, mock, iOS, CLI, MCP) against OSS mobile automation frameworks, judged on what teams select for trusted iOS/Android task execution. OmniPilot has no AUDIT_FINDINGS.md or prior COMPETITIVE_INTELLIGENCE.md; `pyproject.toml` is the upstream Artemis SDK, so the shipped surface assessed here is the Cargo workspace and README. "No" means not found in the reviewed public docs, not proof of absence.

### Comparable OSS projects

| Project | GitHub | License | Last activity | Why it is the comparison |
|---|---|---|---|---|
| Appium | https://github.com/appium/appium | Apache-2.0 | Last push 2026-10-24; latest npm release 2026-09-24 | Broadest OSS mobile automation substrate: W3C WebDriver with iOS/Android drivers and many client languages. The default alternative for cross-platform device coverage. |
| Maestro | https://github.com/mobile-dev-inc/maestro | Apache-2.0 | cli-2.11.0 released 2026-09-29; last push 2026-10-24 | YAML flow runner with built-in flakiness tolerance, MCP server in the CLI, and record-capable Studio. Closest low-ceremony alternative for agentic UI testing. |
| mobile-mcp | https://github.com/mobile-next/mobile-mcp | Apache-2.0 | v1.0.12 released 2026-10-18 | MCP-native, accessibility-first iOS/Android automation on simulators, emulators, and real devices. Closest direct competitor to OmniPilot's MCP surface. |

### Capability comparison

| Capability | Ours | Appium | Maestro | mobile-mcp | Gap |
|---|---|---|---|---|---|
| iOS simulator automation | Yes | Yes | Yes | Yes | None. |
| Android emulator automation | No | Yes | Yes | Yes | Absent; Android is planned in ROADMAP Phase 3. |
| Physical iOS device automation | No | Yes | No | Yes | Absent; README says physical-device runner is still ahead. |
| Physical Android device automation | No | Yes | Yes | Yes | Absent; no Android adapter exists yet. |
| MCP-native server | Yes | No | Yes | Yes | Parity with Maestro/mobile-mcp; ahead of Appium core. |
| Verification from fresh post-action observation | Yes | No | Partial | No | Ours is ahead: `verified` requires criteria plus a fresh final observation. |
| Per-task evidence bundle | Yes | Partial | Partial | Partial | Ours is ahead on bundled hierarchy, screenshots, `task.json`, and `events.jsonl`. |
| Record/replay authoring | No | Partial | Partial | No | Absent; Appium Inspector and Maestro Studio record, both outside the core runner. |
| Automatic flakiness tolerance/idle wait | Partial | Partial | Yes | No | Weaker; we retry within budgets but do not auto-synchronize on app idle. |
| Typed, versioned plan schema | Yes | No | No | No | Ours is ahead; plans are validated JSON rather than free-form scripts. |
| Policy/approval gate before actions | Yes | No | No | No | Ours is ahead; app allowlists and risk approval are in the engine. |
| Client language coverage | No | Yes | Partial | Partial | Weaker; ours is Rust/CLI/MCP plus JSON plans, Appium has many official clients. |
| JUnit/CI report output | Partial | Partial | Partial | No | Weaker; we emit JUnit and non-zero exits, but Apple/Android CI is still unchecked. |
| Parallel device execution | No | Yes | Partial | No | Absent and a stated non-goal; Appium Grid and Maestro Cloud cover scale. |
| Local-first with no mandatory cloud | Yes | Yes | Partial | Partial | Ours is ahead or at parity; Maestro and mobile-mcp offer optional paid cloud. |

### Gaps that matter (ranked)

1. **Android is absent, not merely weaker.** Evidence: README badge says "Android: planned"; ROADMAP Phase 3 has every Android item unchecked; the Cargo workspace has no Android crate. Appium, Maestro, and mobile-mcp all automate Android emulators today. Hurts any team with both platforms, which is most mobile QA groups. Effort: L.
2. **Physical devices are unsupported.** Evidence: README says "Physical-device support is still ahead"; the iOS device build only "compiles with signing disabled". Maestro supports physical Android; Appium and mobile-mcp support both real iOS and Android. Hurts release verification on real hardware, radios, and OS versions. Effort: M-L.
3. **Flakiness handling is thinner than Maestro's.** Evidence: our engine owns retries and budgets (docs/ARCHITECTURE.md), but there is no documented app-idle/settling synchronization; Maestro advertises "built-in flakiness tolerance and automatic waiting". Hurts dynamic UIs where a fresh observation arrives before rendering settles. Effort: M.
4. **No authoring or record/replay path.** Evidence: run_task requires explicit typed steps and/or success criteria; README says natural-language planning is disabled. Appium Inspector and Maestro Studio record interactions, and Maestro ships an MCP server with a viewer. Hurts manual QA and teams migrating existing scripts. Effort: M.
5. **CI and packaging are incomplete for a test tool.** Evidence: JUnit output exists, but ROADMAP has "CI for Rust, Apple, and Android" and "Packaged releases" unchecked. Appium and Maestro have established CI integrations. Hurts adoption in release pipelines that will not build Rust 1.90 locally. Effort: S-M.
6. **Language/ecosystem lock-in to Rust and JSON.** Evidence: clients are the CLI and MCP; there is no Python/JS/Java SDK. Appium's multi-language clients and Maestro's YAML/JS ecosystem lower switching cost. Hurts teams with existing test code. Effort: L.

### Where we are differentiated

- Verification-first semantics: a task is `verified` only when explicit success criteria pass against a fresh final observation; no criteria means completed but unverified (docs/ARCHITECTURE.md). Appium and mobile-mcp leave this to the caller; Maestro asserts but does not emit a provenance status.
- Evidence is a first-class, locally stored artifact: screenshots, hierarchy snapshots, `task.json`, and `events.jsonl` under `traces/<task_id>/`, plus Markdown/JUnit/JSON reports.
- Typed, versioned task/action/assertion schema with an allowlisted action surface and no arbitrary shell escape hatch.
- Policy gate with app allowlists and risk approval runs before the device is touched, with blocked-task outcomes.
- Honest capability negotiation: unsupported actions are reported as unsupported instead of silently degrading.
- MCP-native on the mock and iOS simulator paths with a loopback-only, per-session-token adapter protocol.

### Disposition

- **Build:** Android UI Automator adapter, then physical-device runners; add idle/settling synchronization and a minimal recorder that emits typed plans. Publish Linux/macOS binaries and a CI action.
- **Integrate/reuse:** keep XCUITest as the iOS substrate; adopt Appium drivers only if physical-device coverage demands it rather than reimplementing transport. Reuse the existing NDJSON protocol for new adapters.
- **Defer/reject:** device-fleet management and parallel execution remain non-goals; defer a natural-language planner until typed plans and approval are proven; reject unrestricted shell/OS bypasses.

### Sources

- https://github.com/appium/appium (accessed 2026-10-24)
- https://github.com/mobile-dev-inc/maestro (accessed 2026-10-24)
- https://github.com/mobile-next/mobile-mcp (accessed 2026-10-24)
- Repo files inspected: README.md, docs/ARCHITECTURE.md, docs/ROADMAP.md, Cargo.toml, pyproject.toml, USE_CASES.md
