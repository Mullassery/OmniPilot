# Adapter protocol

Protocol version: `omnipilot.adapter/1`

Transport: newline-delimited JSON-RPC 2.0 over a loopback TCP connection. Each
request and response is one UTF-8 JSON object followed by `\n`. The production
transport is local-only; do not expose the adapter port to a network.

## Handshake

The client sends `initialize` first:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocol_version": "omnipilot.adapter/1",
    "session_id": "uuid",
    "token": "per-session-secret",
    "client": { "name": "omnipilot", "version": "0.1.0" }
  }
}
```

The adapter validates the protocol version and token, then returns adapter and
device metadata, capabilities, and limits. Every other method is rejected until
the handshake succeeds. A second `initialize` is rejected.

## Methods

| Method | Params | Result |
| --- | --- | --- |
| `initialize` | `InitializeParams` | `InitializeResult` |
| `observe` | `ObserveParams` | `Observation` |
| `act` | `ActParams` | `ActionOutcome` |
| `logs` | `LogsParams` | `LogsResult` |
| `cancel` | `CancelParams` | `SimpleResult` |
| `shutdown` | none | `SimpleResult` |

`observe` can include a screenshot and hierarchy. `act` accepts a typed `Action`
such as `launch_app`, `tap`, `input_text`, `swipe`, `press_key`, `wait`, or
`screenshot`. Unsupported actions return an `UNSUPPORTED` error rather than a
fake success.

## Errors

Standard JSON-RPC codes are used for parse, request, method, and parameter
errors. OmniPilot adds:

| Code | Meaning |
| --- | --- |
| `-32000` | Application/device error |
| `-32001` | Unauthorized or policy denial |
| `-32002` | Unsupported action or capability |

## Schema

Print the complete versioned schema bundle:

```bash
target/release/omnipilot protocol schema
```

The schema covers core task, action, assertion, observation, result, evidence,
and protocol types. It is the source of truth for MCP tool inputs and future
client SDKs.

## Security invariants

- The token is generated per session and must never be logged.
- The transport binds to loopback only.
- The adapter cannot execute arbitrary shell commands.
- The engine applies policy before sending an action.
- The adapter must not claim capabilities it does not implement.
