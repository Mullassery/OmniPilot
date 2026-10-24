# OmniPilot - Use Cases

## Overview

<p align="center"> <img src="docs/assets/omnipilot-banner.svg" alt="OmniPilot — verification-first mobile automation for iOS and Android" width="100%" />

## Use Cases

OmniPilot is for workflows where an action returning “success” is not enough.
The common pattern is: a typed plan runs, the adapter acts, the engine captures a
fresh observation, and the result includes evidence that can be inspected later.

| Use case | Who it helps | Proof it produces | Available today |
| --- | --- | --- | --- |
| AI agents that must prove completion | Agent and assistant builders | Final hierarchy, screenshot, assertion result, trace | Mock and opt-in iOS via MCP |
| iOS release smoke tests | iOS QA and release engineers | Deterministic step results plus screenshots and hierarchy | iOS simulator |
| Flaky UI test triage | QA and app developers | Failed assertion next to the actual final screen state | Mock and iOS |
| UI contract and identifier coverage | Mobile platform teams | Role, label, value, identifier, bounds, enabled, clickable | iOS |
| Privacy-sensitive local automation | Regulated and security-conscious teams | Local evidence, loopback-only transport, policy decisions | Mock and iOS CLI |
| MCP tooling for coding assistants | Developers using MCP clients | Task records, logs, and evidence through MCP tools | Mock and opt-in iOS simulator |
| Adapter conformance and Android expansion | Platform and adapter engineers | Shared protocol tests and engine behavior across adapters | Mock and iOS; Android planned |
| Evidence-rich bug reports | Support, QA, and engineering | Reproducible task result with screenshots, hierarchy, and trace | Mock and iOS |
| Agent evaluation sandbox | AI researchers and evaluators | Repeatable task outcomes without device flakiness | Deterministic mock |
| Approval-gated automation | Teams with risky or restricted flows | Explicit allow/deny decisions before actions execute | Engine and policy today |

## Common Scenarios

- Development and testing with OmniPilot
- Integration into existing workflows
- Exploration and prototyping

