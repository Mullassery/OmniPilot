# MCP server and client examples

OmniPilot exposes its engine over the Model Context Protocol (MCP) via stdio.
The server is the same release binary: `omnipilot mcp` (mock adapter) or
`omnipilot mcp --ios --ios-device <udid>` (mock + a booted iOS simulator).

Build it first:

```bash
cargo build --release -p omnipilot-cli
```

## Client configuration

Claude Desktop, Cursor, and other MCP clients accept the standard
`mcpServers` shape:

```json
{
  "mcpServers": {
    "omnipilot": {
      "command": "/absolute/path/to/OmniPilot/target/release/omnipilot",
      "args": ["mcp"]
    }
  }
}
```

For iOS, add the simulator flags. The server launches the XCUITest adapter
before accepting MCP requests:

```json
{
  "mcpServers": {
    "omnipilot-ios": {
      "command": "/absolute/path/to/OmniPilot/target/release/omnipilot",
      "args": ["mcp", "--ios", "--ios-device", "<simulator-udid>"]
    }
  }
}
```

Use `omnipilot ios-devices` to discover a UDID, or `--ios-destination` to
override the whole `xcodebuild` destination.

A dependency-free, runnable example client lives at
[`examples/mcp/client.py`](../examples/mcp/client.py). It performs the
initialize handshake, lists tools, runs the built-in mock demo, and polls the
task to a verified result:

```bash
python3 examples/mcp/client.py
```

## Tools

All tool responses are JSON text content. Task-mutating tools take typed
arguments only; there is no free-text planning path in this build.

### `list_devices`

No arguments. Returns the protocol version and every device the process can
use (the mock device by default, plus the iOS simulator when configured).

### `get_capabilities`

```json
{ "device_id": "mock-device-1" }
```

Returns the device's honest capability matrix.

### `observe`

```json
{ "device_id": "mock-device-1", "include_screenshot": false }
```

Returns the current screen and accessibility hierarchy. Screenshot bytes are
included only when `include_screenshot` is true.

### `get_logs`

```json
{ "device_id": "mock-device-1", "limit": 100 }
```

Returns recent device logs. `limit` is capped at 500.

### `run_task`

Starts an explicit, typed plan and returns a task id immediately. Poll
`get_task` for the verified result.

```json
{
  "goal": "Open the demo Settings screen and turn on dark mode",
  "platform": "mock",
  "device_id": "mock-device-1",
  "app_id": "com.omnipilot.demo",
  "steps": [
    { "type": "launch_app", "app_id": "com.omnipilot.demo" },
    {
      "type": "tap",
      "target": {
        "kind": "element",
        "selector": { "kind": "identifier", "value": "home.settings" }
      }
    },
    {
      "type": "tap",
      "target": {
        "kind": "element",
        "selector": { "kind": "identifier", "value": "settings.dark_mode" }
      }
    }
  ],
  "success_criteria": [
    {
      "type": "element_value_equals",
      "selector": { "kind": "identifier", "value": "settings.dark_mode" },
      "value": "On"
    },
    { "type": "text_present", "text": "Dark mode" }
  ],
  "max_steps": 20,
  "max_duration_ms": 120000,
  "approve_ui": false
}
```

`steps` and `success_criteria` may be used together; supplying neither is an
invalid request. `approve_ui` adds the UI-mutation risk approval required by
the default policy. The adapter still verifies the final observation before a
result is marked successful.

Supported action `type` values: `launch_app`, `terminate_app`, `tap`,
`long_press`, `swipe`, `input_text`, `press_key`, `wait`, `screenshot`.

Supported assertion `type` values: `text_present`, `text_absent`,
`element_present`, `element_absent`, `element_enabled`,
`element_value_equals`, `app_foreground`, `screen_changed`.

Selectors use `{"kind": "identifier" | "text" | "text_contains" |
"role_and_text" | "label" | "index", ...}`. Targets use
`{"kind": "element", "selector": {...}}` or
`{"kind": "point", "point": {"x": 10, "y": 20}}`.

### `get_task`

```json
{ "task_id": "<task-id>" }
```

Returns the persisted task record: status, steps, assertion results,
limitations, errors, and evidence references.

### `cancel_task`

```json
{ "task_id": "<task-id>" }
```

Requests cancellation and reports whether the task was still cancellable.

### `get_evidence`

```json
{
  "task_id": "<task-id>",
  "evidence_id": "<evidence-id>",
  "include_content": false
}
```

Returns evidence metadata, optionally with the raw bytes base64-encoded.

## Example task flow

The MCP server is asynchronous for tasks:

1. `run_task` returns `{ "task_id": ..., "status": "queued" }`.
2. Poll `get_task` until `status` is `completed`, `failed`, or `cancelled`.
3. Read `result.verified`, assertion results, and evidence from the record.
4. Use `get_evidence` for a specific artifact if you need the bytes.

`examples/mcp/client.py` implements exactly this flow and exits non-zero if the
demo task is not verified.

## Security

- MCP runs over stdio, not a network listener.
- The default policy requires approval for UI mutations. Set `approve_ui` only
  for plans you have reviewed.
- The adapter protocol token is per-session and never logged.
- The adapter cannot execute arbitrary shell commands.
