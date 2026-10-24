#!/usr/bin/env python3
"""Minimal OmniPilot MCP stdio client (stdlib only).

Build the CLI first:
    cargo build --release -p omnipilot-cli

Then run:
    python3 examples/mcp/client.py

It launches `omnipilot mcp`, performs the MCP initialize handshake, lists the
available tools, runs the built-in mock demo task, and polls `get_task` until
the result is verified.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


class McpClient:
    def __init__(self, command: list[str]) -> None:
        self._process = subprocess.Popen(
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        self._next_id = 0

    def close(self) -> None:
        if self._process.stdin:
            self._process.stdin.close()
        self._process.terminate()
        try:
            self._process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self._process.kill()

    def _send(self, message: dict[str, Any]) -> None:
        assert self._process.stdin is not None
        self._process.stdin.write(json.dumps(message) + "\n")
        self._process.stdin.flush()

    def _read_response(self, request_id: int) -> dict[str, Any]:
        assert self._process.stdout is not None
        while True:
            line = self._process.stdout.readline()
            if not line:
                stderr = self._process.stderr.read() if self._process.stderr else ""
                raise RuntimeError(f"MCP server exited before responding.\n{stderr}")
            message = json.loads(line)
            if message.get("id") == request_id:
                if "error" in message:
                    raise RuntimeError(json.dumps(message["error"], indent=2))
                return message["result"]

    def request(self, method: str, params: dict[str, Any] | None = None) -> Any:
        self._next_id += 1
        request_id = self._next_id
        self._send(
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": method,
                "params": params or {},
            }
        )
        return self._read_response(request_id)

    def notify(self, method: str, params: dict[str, Any] | None = None) -> None:
        self._send(
            {"jsonrpc": "2.0", "method": method, "params": params or {}}
        )

    def call_tool(self, name: str, arguments: dict[str, Any]) -> Any:
        result = self.request(
            "tools/call", {"name": name, "arguments": arguments}
        )
        texts = [
            block["text"]
            for block in result.get("content", [])
            if block.get("type") == "text"
        ]
        if result.get("isError"):
            raise RuntimeError("\n".join(texts))
        return json.loads(texts[0]) if texts else None


def demo_run_task_arguments() -> dict[str, Any]:
    return {
        "goal": "Open the demo Settings screen and turn on dark mode",
        "platform": "mock",
        "device_id": "mock-device-1",
        "steps": [
            {"type": "launch_app", "app_id": "com.omnipilot.demo"},
            {
                "type": "tap",
                "target": {
                    "kind": "element",
                    "selector": {"kind": "identifier", "value": "home.settings"},
                },
            },
            {
                "type": "tap",
                "target": {
                    "kind": "element",
                    "selector": {
                        "kind": "identifier",
                        "value": "settings.dark_mode",
                    },
                },
            },
        ],
        "success_criteria": [
            {
                "type": "element_value_equals",
                "selector": {
                    "kind": "identifier",
                    "value": "settings.dark_mode",
                },
                "value": "On",
            },
            {"type": "text_present", "text": "Dark mode"},
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        default=str(Path(__file__).resolve().parents[2] / "target/release/omnipilot"),
        help="Path to the omnipilot binary",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=30.0,
        help="Seconds to wait for the demo task to finish",
    )
    args = parser.parse_args()

    client = McpClient([args.binary, "mcp"])
    try:
        initialized = client.request(
            "initialize",
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "omnipilot-example", "version": "0.1.0"},
            },
        )
        print(
            "Connected to",
            initialized["serverInfo"]["name"],
            initialized["serverInfo"]["version"],
        )
        client.notify("notifications/initialized")

        tools = client.request("tools/list")["tools"]
        print("Tools:", ", ".join(sorted(tool["name"] for tool in tools)))

        devices = client.call_tool("list_devices", {})
        print("Devices:", ", ".join(device["id"] for device in devices["devices"]))

        queued = client.call_tool("run_task", demo_run_task_arguments())
        task_id = queued["task_id"]
        print("Queued task", task_id)

        deadline = time.monotonic() + args.timeout
        while time.monotonic() < deadline:
            record = client.call_tool("get_task", {"task_id": task_id})
            status = record["status"]
            if status in {"completed", "failed", "cancelled"}:
                result = record.get("result") or {}
                print(
                    f"Finished: status={status} verified={result.get('verified')}"
                )
                print(result.get("summary", "No summary recorded"))
                return 0 if result.get("verified") else 1
            time.sleep(0.2)
        print("Timed out waiting for the task", file=sys.stderr)
        return 1
    finally:
        client.close()


if __name__ == "__main__":
    raise SystemExit(main())
