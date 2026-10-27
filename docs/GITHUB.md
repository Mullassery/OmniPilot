# GitHub repository settings

Copy-paste metadata for the repository settings page.

## About

**Short description (About field):**

> Verification-first mobile automation for iOS and Android. Drive real apps with native APIs, then prove success with fresh observations and evidence. Rust engine, MCP-native, local-first, open source.

**Website:**

Leave blank until there is a documentation site. The README is the current
landing page.

## Topics

Use these 20 topics, in this order:

```text
mobile-automation
ios
android
xcuitest
uiautomator
mcp
model-context-protocol
rust
swift
ai-agents
agentic-ai
test-automation
mobile-testing
verification
local-first
appium-alternative
developer-tools
automation
llm
open-source
```

## Social preview

Use `docs/assets/omnipilot-social.png` as the 1280x640 social preview. The
editable source is `docs/assets/omnipilot-social.svg`; the README banner is
`docs/assets/omnipilot-banner.svg`.

Primary line:

> AI agents can tap. OmniPilot can prove it worked.

Secondary line:

> Verification-first mobile automation for iOS and Android.

## Extended About

Use this for the organization profile, launch post, or pinned discussion.

> OmniPilot is a verification-first mobile automation platform for iOS and
> Android. It lets AI agents, test suites, and developers drive real apps through
> native APIs, then proves the result with fresh observations and evidence.
>
> Most mobile automation stops at “the action executed.” OmniPilot closes the
> loop: observe, authorize, act, observe again, verify, and store the evidence.
> A task is never marked verified just because a tap returned success.
>
> The project is built in Rust, speaks MCP natively, runs local-first, and keeps
> the adapter protocol open. iOS uses XCUITest; Android will use UI Automator.
> The current alpha includes a working Rust engine, CLI, MCP server, deterministic
> mock device, passing iOS UI tests, and a long-lived XCUITest adapter that has
> completed verified iOS tasks with fresh hierarchy and screenshot evidence.
> The Android adapter, physical-device runner, and planner are the next
> milestones.

## Launch post

Short version:

> Mobile agents can tap. They usually can’t prove it worked.
>
> I’m building OmniPilot: verification-first mobile automation for iOS and
> Android. Native APIs, typed plans, policy gates, fresh evidence, and a
> versioned adapter protocol. Rust engine, MCP-native, local-first, open source.
>
> The mock demo is verified end to end. iOS now has a long-lived XCUITest
> adapter: the host launches the runner, drives a real simulator app, and proves
> the result with fresh hierarchy and screenshot evidence. Android is next.
> Follow the build: ../OmniPilot

## Apply with GitHub CLI

If the remote and repository are configured:

```bash
gh repo edit Mullassery/OmniPilot \
  --description "Verification-first mobile automation for iOS and Android. Drive real apps with native APIs, then prove success with fresh observations and evidence. Rust engine, MCP-native, local-first, open source." \
  --add-topic mobile-automation \
  --add-topic ios \
  --add-topic android \
  --add-topic xcuitest \
  --add-topic uiautomator \
  --add-topic mcp \
  --add-topic model-context-protocol \
  --add-topic rust \
  --add-topic swift \
  --add-topic ai-agents \
  --add-topic agentic-ai \
  --add-topic test-automation \
  --add-topic mobile-testing \
  --add-topic verification \
  --add-topic local-first \
  --add-topic appium-alternative \
  --add-topic developer-tools \
  --add-topic automation \
  --add-topic llm \
  --add-topic open-source
```

## Launch checklist

- [ ] Replace the upstream `origin` with the OmniPilot repository.
- [ ] Set the About description and all 20 topics.
- [x] Generate the 1280x640 social preview PNG.
- [ ] Upload `docs/assets/omnipilot-social.png` in repository settings.
- [ ] Pin the README and enable Discussions.
- [ ] Add the CI badge to the README after the first green run.
- [ ] Publish the first alpha release with the verified mock demo.
- [ ] Keep the “not implemented yet” section honest until the Android adapter
      and physical-device runner land.
