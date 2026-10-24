# Security Policy

## Supported versions

OmniPilot is alpha software. Only the latest release and the `main` branch
receive fixes.

## Reporting a vulnerability

Please do not open a public issue for a security vulnerability. Use GitHub's
[private vulnerability reporting](https://github.com/Mullassery/OmniPilot/security/advisories/new)
for this repository. If that is unavailable, open an issue titled
"Security contact request" with no vulnerability details.

There is no guaranteed response time or SLA.

## Security model

- **Local-first by default.** The engine, MCP server, and evidence stay on the
  host. Transport is loopback unless explicitly configured otherwise.
- **Policy-gated actions.** UI mutations can be restricted by app allowlists and
  approval gates. The policy decision is recorded with the task.
- **Evidence is sensitive.** Screenshots, hierarchy dumps, and logs can contain
  personal or business data. Protect the trace directory like any other test
  artifact and redact before sharing.
- **No arbitrary shell escape.** OmniPilot exposes device operations it can
  model and verify, not a general shell.
- **Physical devices and future adapters are not independently audited.** Treat
  capability claims in the README as the current boundary.

## Scope

The Rust engine, CLI, MCP server, mock adapter, and iOS adapter are in scope.
Third-party apps under automation, device firmware, and your host environment
are out of scope.
