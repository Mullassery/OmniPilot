# Audit Report: OmniPilot

## Overview
- **Type**: Mobile automation framework (iOS/Android) with Rust core + Python orchestration
- **Languages**: Rust, Python, Swift (apple/)
- **Version**: 1.0 (pyproject) / workspace
- **License**: Apache-2.0
- **Purpose**: Verification-first mobile automation, MCP-native

## Architecture
- **Rust crates**: Core engine, adapters
- **Python**: Orchestration (LangGraph/LangChain), CLI, MCP server
- **iOS**: XCUITest integration (apple/ directory)
- **Android**: Planned (UIAutomator)
- **MCP**: Native MCP server

## Entry Points
- CLI: `artemis` command
- MCP server
- Python API

## Dependencies
- Heavy Python deps (LangChain/LangGraph, FastAPI, OpenCV, etc.)
- Rust deps
- adbutils/uiautomator2 for Android

## Security Observations
- Local-first design
- Policy gates, allowlists mentioned
- No shell escape hatch (claimed)
- Evidence stored locally
- Needs review of device control permissions

## Tests/CI
- Rust CI workflow present
- Tests likely exist

## Findings

| Severity | Finding | Evidence | Impact | Root Cause Hypothesis | Recommendation |
|----------|---------|----------|--------|----------------------|---------------|
| Medium | Mixed Rust/Python architecture increases complexity | Both languages, multiple packages | More surface area to maintain/secure | Feature richness | Clear boundaries, document interfaces |
| Medium | Heavy ML/LLM dependencies | LangChain ecosystem deps | Large dependency tree, supply chain risk | AI agent capabilities | Pin versions, audit deps, consider minimal install |
| Medium | Android support incomplete/planned | README states planned | Feature gap | Development stage | Document clearly |
| Low | Alpha status | README states | Not production ready | Maturity | Set expectations |

## Strengths
- Verification-first approach is solid
- Native APIs vs OCR-only
- MCP integration
- Evidence-based results
- Good documentation of status

## Limitations
- Alpha, iOS more mature than Android
- Complex dependency stack
- Requires device/simulator setup

## Prioritized Recommendations
1. **High**: Security review of device control paths
2. **Medium**: Dependency audit (LangChain supply chain)
3. **Medium**: Complete Android adapter or clearly scope
4. **Low**: Add integration tests
