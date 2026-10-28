# Roadmap: OmniPilot

> Living roadmap based on the current codebase and alpha state.

## Current State
- **Rust core**: Engine, protocol, runtime, CLI, MCP server, evidence store, reporting.
- **iOS**: XCUITest-based adapter + long-lived adapter working on simulator; physical device in progress.
- **Android**: ADB/UIAutomator driver (Python) + Rust adapter scaffolding added; integration/testing ongoing.
- **Python package**: Orchestrator/UI/CLI (`omnipilot`), MCP tools, rebranded from Artemis.
- **Local-first**: No cloud dependency required.

## Priorities

### Now (0-30 days)
| Item | Priority | Status | Notes |
|---|---|---|---|
| Android Rust adapter integration | P0 | In Progress | Wire `omnipilot-android` into CLI/runtime; align with core models. |
| Android CLI path & tests | P0 | In Progress | Validate `--platform android` flows, ADB discovery. |
| PyPI release | P0 | Ready | Bump version to 1.1.0, publish package. |
| Docs polish | P0 | In Progress | Update README/ROADMAP to reflect Android progress. |

### Next (30-90 days)
| Item | Priority | Status | Notes |
|---|---|---|---|
| Android UIAutomator v2 backend | P0 | Planned | Implement real ADB/UIAutomator actions (tap/swipe/input/keys/app launch). |
| Emulator/device detection | P0 | Planned | Robust `adb` discovery, serial selection, health checks. |
| Evidence capture for Android | P0 | Planned | Screenshots/hierarchy capture via ADB/UIAutomator. |
| Android conformance tests | P0 | Planned | Add adapter conformance tests mirroring iOS/mock. |
| Physical iOS device support | P1 | Planned | Code signing/device provisioning flows. |

### Later (90+ days)
| Item | Priority | Status | Notes |
|---|---|---|---|
| Natural-language planning | P1 | Planned | Safer planner with verification loops. |
| CI matrix (macOS/Linux/Windows) | P1 | Planned | Rust + Python tests, packaging, artifacts. |
| Packaged releases | P1 | Planned | CLI binaries, PyPI, install scripts. |
| MCP server hardening | P2 | Planned | Better schemas, auth, examples. |
| Plugin/extension model | P2 | Planned | Community adapters. |

## Milestones
- **M1: Android alpha** - CLI can run basic tasks on emulator via ADB/UIAutomator with evidence.
- **M2: Cross-platform parity** - Android reaches core conformance for observe/act/assert.
- **M3: Beta release** - Stable CLI/PyPI, CI, docs, and packaging.

## Success Metrics
| Metric | Target |
|---|---|
| Android adapter conformance | Pass core protocol tests |
| Test coverage (Rust) | >70% on core/engine/protocol |
| PyPI installs | Tracked |
| CI passing | 100% on main |

