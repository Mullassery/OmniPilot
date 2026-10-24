# OmniPilot Apple targets

This directory contains the Apple-first demo, UI tests, and long-lived XCUITest
protocol adapter.

- `project.yml` is the XcodeGen source of truth.
- `OmniPilotDemo/` is a small SwiftUI app with stable accessibility identifiers.
- `OmniPilotUITests/` contains the fast demo UI tests.
- `OmniPilotAdapterTests/` contains the loopback JSON-RPC adapter server.
- `OmniPilot.xcodeproj` is generated; do not hand-edit it.

Generate the project:

```bash
cd apple
xcodegen generate
```

Run the fast demo tests from the repository root:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  xcodebuild -project apple/OmniPilot.xcodeproj \
  -scheme OmniPilotDemo \
  -destination 'platform=iOS Simulator,name=iPhone 17' \
  test
```

Run the adapter through the Rust host:

```bash
cargo build --release
target/release/omnipilot ios-devices
target/release/omnipilot ios-boot --device <simulator-udid>
target/release/omnipilot ios-demo
```

The adapter test is long-lived and waits for the host. Do not run
`OmniPilotAdapter` directly unless you are testing the timeout path; use
`omnipilot ios-demo` instead. See `../docs/APPLE.md` for architecture, commands,
and current limitations.
