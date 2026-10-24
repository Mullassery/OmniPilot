# Upstream provenance

> This file is a legal and provenance record, not OmniPilot product branding.
> It is retained for Apache-2.0 compliance and accurate attribution.

## Source

OmniPilot started from the Google ARTEMIS repository:

- Repository: https://github.com/google/artemis
- Upstream commit: `351ca8422f7b5b54e80a9c1ce03a222e02415b6b`
- Commit date: 2026-09-28
- Upstream license: Apache License 2.0
- Upstream copyright: Copyright 2026 Google LLC

The original English and Chinese README files were moved, unchanged, to:

- `docs/upstream/ARTEMIS_README.md`
- `docs/upstream/ARTEMIS_README_CN.md`

Their original relative links and images may not resolve from the new location.
They are retained for provenance, not as the current project documentation.

## Third-party attribution

The upstream checkout includes `third_party/mobile_use/NOTICE`, which requests
attribution to Minitap, Inc. for Mobile-Use and its architectural methodologies.
That attribution is preserved in the root `NOTICE`.

## What changed

OmniPilot is a new Rust implementation with its own crate structure, protocol,
execution engine, CLI, MCP server, mock adapter, and Apple demo harness. It is
not a fork that keeps the upstream Python application as its runtime.

The upstream Python tree remains in the working directory during the transition
so that behavior and test coverage can be compared. It is not part of the
OmniPilot release build.

## Publishing checklist

- [ ] Replace `origin` with the intended OmniPilot repository.
- [ ] Confirm the final repository name and owner.
- [ ] Keep `LICENSE` and `NOTICE` in the published tree.
- [ ] Add a short attribution line to user-facing docs and the future About UI.
- [ ] Do not imply endorsement by Google LLC or Minitap, Inc.
