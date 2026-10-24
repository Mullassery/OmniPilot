# Apple build and iOS adapter

## Current scope

The Apple-first milestone has three working parts:

1. **macOS host**: the Rust engine, CLI, and MCP server build and run natively.
2. **iOS harness**: a SwiftUI demo app with stable accessibility identifiers and
   a fast XCUITest suite.
3. **iOS adapter**: a long-lived XCUITest process that serves the OmniPilot
   protocol over loopback, launched and managed by the Rust host.

The adapter is intentionally separate from the demo UI tests. The
`OmniPilotAdapter` scheme runs one long-lived test, while `OmniPilotDemo` keeps
the normal UI tests fast and deterministic.

## Environment used

- macOS 26.5.2 on Apple silicon
- Xcode 26.6
- iOS 26.5 simulator runtime
- iPhone 17 simulator
- XcodeGen 2.46.0
- Swift 6.3.3
- Rust 1.99

`xcode-select` points at Command Line Tools on this host. The commands below
therefore set `DEVELOPER_DIR` explicitly, and the Rust host falls back to
`/Applications/Xcode.app/Contents/Developer` when it is present.

## Generate the project

`apple/project.yml` is the source of truth. Regenerate after changing targets or
build settings:

```bash
brew install xcodegen
cd apple
xcodegen generate
```

The generated project is `apple/OmniPilot.xcodeproj`. It contains two shared
schemes:

- `OmniPilotDemo` — demo app plus the two UI tests.
- `OmniPilotAdapter` — demo app plus `OmniPilotAdapterTests`.

## Run the demo UI tests

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  xcodebuild -project apple/OmniPilot.xcodeproj \
  -scheme OmniPilotDemo \
  -destination 'platform=iOS Simulator,name=iPhone 17' \
  test
```

Verified on 2026-10-24:

- `testDarkModeFlow` passed
- `testAboutAndBack` passed
- 2 tests, 0 failures

## Run the iOS adapter

Build the host once:

```bash
cargo build --release
```

List available iOS simulators:

```bash
target/release/omnipilot ios-devices
```

Boot a simulator and wait until it is ready, or shut it down:

```bash
target/release/omnipilot ios-boot --device C6DD0C15-8C4D-43C0-B171-B2BDCE2838A7
target/release/omnipilot ios-shutdown --device C6DD0C15-8C4D-43C0-B171-B2BDCE2838A7
```

Both commands are idempotent and report whether they changed the simulator
state. `ios-boot` uses `simctl bootstatus -b`, so it waits for boot to finish.

Run the built-in iOS demo through the real engine:

```bash
target/release/omnipilot ios-demo
```

Select a specific simulator by UDID:

```bash
target/release/omnipilot ios-demo \
  --device C6DD0C15-8C4D-43C0-B171-B2BDCE2838A7
```

Run an explicit typed task:

```bash
target/release/omnipilot run \
  --platform ios \
  --device C6DD0C15-8C4D-43C0-B171-B2BDCE2838A7 \
  --goal 'Open Settings and enable dark mode on iOS' \
  --app com.omnipilot.demo \
  --step '{"type":"launch_app","app_id":"com.omnipilot.demo"}' \
  --step '{"type":"tap","target":{"kind":"element","selector":{"kind":"identifier","value":"home.settings"}}}' \
  --step '{"type":"tap","target":{"kind":"element","selector":{"kind":"identifier","value":"settings.dark_mode"}}}' \
  --assert-text 'Dark mode is On'
```

Add `--junit reports/ios.xml` to any task command to write a JUnit XML report.

The host launches `xcodebuild` with the `OmniPilotAdapter` scheme. If no
`--device` is supplied, the default destination is
`platform=iOS Simulator,name=iPhone 17`. Override the whole destination with
`--ios-destination` if needed.

Serve the mock adapter plus the iOS simulator over MCP:

```bash
target/release/omnipilot mcp \
  --ios \
  --ios-device C6DD0C15-8C4D-43C0-B171-B2BDCE2838A7
```

The server starts the XCUITest adapter before accepting MCP requests. After
that, `list_devices` returns both devices and `run_task` infers the platform
from `device_id`. Pass `--ios-project`, `--ios-scheme`, or `--ios-destination`
to override the adapter launch configuration.

Verified on 2026-10-24:

- `omnipilot ios-demo`: `completed`, `verified=true`, 3 steps, 1 criterion,
  4 evidence items, ~6.1 seconds.
- `omnipilot run --platform ios`: `completed`, `verified=true`, same task.
- Adapter host smoke: initialize, launch, observe, tap, observe, toggle, verify,
  shutdown.
- MCP iOS smoke: `list_devices` returned the simulator and mock device; a
  typed `run_task` completed and verified with 4 evidence items.
- Lifecycle smoke on iPhone 17e: boot changed `Shutdown` to `Booted`, a second
  boot was unchanged, shutdown changed it back, and a second shutdown was
  unchanged.

## How the adapter works

1. `omnipilot-ios` starts `xcodebuild test` for
   `OmniPilotAdapterTests/OmniPilotAdapterTests/testServeAdapter`.
2. The test starts an `NWListener` on `127.0.0.1` and prints one line:

   ```text
   OMNIPILOT_ADAPTER_READY port=<port> token=<session-token>
   ```

3. The host parses the line, redacts the token from logs, connects over TCP, and
   sends `initialize` with the protocol version and token.
4. The host sends `observe` and `act` requests. The Swift runner maps them to
   XCUITest hierarchy snapshots and gestures.
5. The engine captures fresh observations, evaluates assertions, and writes
   evidence under `traces/<task_id>/`.
6. The host sends `shutdown`; the test exits and `xcodebuild` terminates.

The token is generated per session, never logged, and the listener is bound to
loopback only. The runner has no arbitrary shell command.

### SwiftUI switch handling

Tapping the center of a SwiftUI `Toggle` row can hit the label instead of the
control. The adapter detects switch/toggle elements and taps at normalized
offset `(0.9, 0.5)`, matching the working demo UI test.

## Accessibility identifiers

The UI tests and adapter use stable identifiers rather than labels:

- `home.title`, `home.settings`, `home.about`
- `settings.back`, `settings.title`, `settings.dark_mode`
- `settings.dark_mode.state`, `settings.search`
- `about.back`, `about.title`, `about.body`

These identifiers are the contract the XCUITest adapter resolves into OmniPilot
selectors.

## Build for a device

The demo app compiles for a generic iOS device with signing disabled:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  xcodebuild -project apple/OmniPilot.xcodeproj \
  -scheme OmniPilotDemo \
  -destination 'generic/platform=iOS' \
  build CODE_SIGNING_ALLOWED=NO
```

Running the adapter on a physical device still requires a provisioned runner and
explicit signing.

## Current limitations

- The host can boot and shut down a simulator by UDID; it does not yet manage a
  device pool or choose destinations automatically.
- The adapter targets the simulator first. Physical-device support needs signing
  and provisioning work.
- Action coverage is intentionally small: launch, terminate, tap, long press,
  swipe, text input, a few keys, wait, and screenshots.
- The MCP server exposes the mock adapter by default. iOS requires the opt-in
  `--ios` flags and a booted simulator.
- There is no Apple CI job yet.

## Next Apple steps

1. Add a provisioned physical-device runner.
2. Broaden action and log/cancellation conformance coverage.
3. Add an Apple CI job that compiles the adapter and runs the fast UI tests.

## Troubleshooting

- If Xcode cannot find a simulator, run `xcrun simctl list devices available`
  with `DEVELOPER_DIR` set, or use `omnipilot ios-devices`.
- If the adapter times out before printing its ready line, run the adapter build
  directly to surface compile errors:

  ```bash
  DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
    xcodebuild -project apple/OmniPilot.xcodeproj \
    -scheme OmniPilotAdapter \
    -destination 'platform=iOS Simulator,name=iPhone 17' \
    build-for-testing
  ```

- Running the `OmniPilotAdapter` test directly without a host will wait 60
  seconds and fail by design. Use `omnipilot ios-demo` to drive it.
- If a UI test target fails because it has no `Info.plist`, keep
  `GENERATE_INFOPLIST_FILE: YES` in `apple/project.yml` and regenerate.
- Do not hand-edit `project.pbxproj`; edit `project.yml` and regenerate.
