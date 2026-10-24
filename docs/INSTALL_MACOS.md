# macOS install (CLI + MCP server)

OmniPilot is a Rust binary that also hosts the MCP stdio server. There is no
separate MCP executable.

## Requirements

- macOS 13 or newer
- Rust stable 1.90+ (`rustup` is the simplest install)
- For iOS work only: Xcode 26+ and an installed simulator runtime

## Build

```bash
cd /path/to/OmniPilot
cargo build --release -p omnipilot-cli
```

The binary is `target/release/omnipilot`. Verify it:

```bash
target/release/omnipilot --version
target/release/omnipilot demo
```

`demo` runs the built-in mock scenario and must report a verified result.

## Install on PATH

Copy the binary somewhere on `PATH` (no installer or daemon is required):

```bash
sudo install -m 0755 target/release/omnipilot /usr/local/bin/omnipilot
```

If you prefer a per-user install:

```bash
mkdir -p "$HOME/.local/bin"
install -m 0755 target/release/omnipilot "$HOME/.local/bin/omnipilot"
# Ensure "$HOME/.local/bin" is on PATH in your shell profile.
```

## MCP server

No extra install step: point your MCP client at the same binary.

```json
{
  "mcpServers": {
    "omnipilot": {
      "command": "/usr/local/bin/omnipilot",
      "args": ["mcp"]
    }
  }
}
```

For a booted iOS simulator:

```json
{
  "mcpServers": {
    "omnipilot-ios": {
      "command": "/usr/local/bin/omnipilot",
      "args": ["mcp", "--ios", "--ios-device", "<simulator-udid>"]
    }
  }
}
```

Find a UDID with `omnipilot ios-devices`. See [`MCP.md`](MCP.md) for the tool
reference and a runnable client.

## Gatekeeper and signing

The locally built binary is not code-signed or notarized. Running it directly
from Terminal is fine. If you distribute the binary to another Mac, sign and
notarize it first:

```bash
codesign --force --options runtime --sign "Developer ID Application: ..." \
  target/release/omnipilot
xcrun notarytool submit <archive.zip> --keychain-profile <profile> --wait
```

For a quick local ad-hoc signature (no notarization):

```bash
codesign --force --sign - target/release/omnipilot
```

## Uninstall

```bash
sudo rm /usr/local/bin/omnipilot
# or: rm "$HOME/.local/bin/omnipilot"
```

Task records and evidence live under `traces/` (or `$OMNIPILOT_TRACES`).
Remove that directory only if you no longer need the evidence.
