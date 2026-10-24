// Copyright 2026 OmniPilot contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A deterministic in-memory device used for tests, protocol conformance, and
//! the `omnipilot demo` command. It is deliberately small: three screens, one
//! toggle, one text field, and explicit logs. It never pretends to be a real
//! Android or iOS device.

use async_trait::async_trait;
use chrono::Utc;
use omnipilot_core::{
    Action, ActionId, ActionOutcome, ActionStatus, AdapterInfo, AdapterLimits, AppState, Assertion,
    Bounds, Capability, DeviceId, DeviceInfo, DeviceStatus, Element, ErrorCode, LogEntry,
    Observation, OmniError, Platform, Screenshot, Size, SnapshotId, SupportLevel, Target, TaskSpec,
};
use omnipilot_protocol::{
    ActParams, AdapterService, CancelParams, DeviceAdapter, InitializeParams, InitializeResult,
    LogsParams, LogsResult, ObserveParams, RpcError, SimpleResult,
};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub const DEMO_APP: &str = "com.omnipilot.demo";
pub const DEMO_DEVICE_ID: &str = "mock-device-1";
pub const DEFAULT_TOKEN: &str = "omnipilot-mock-token";
const SCREEN_WIDTH: u32 = 390;
const SCREEN_HEIGHT: u32 = 844;
const ONE_PIXEL_PNG: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    Home,
    Settings,
    About,
}

impl Screen {
    fn activity(&self) -> &'static str {
        match self {
            Self::Home => ".HomeActivity",
            Self::Settings => ".SettingsActivity",
            Self::About => ".AboutActivity",
        }
    }
}

#[derive(Debug)]
struct MockState {
    screen: Screen,
    app_id: Option<String>,
    dark_mode: bool,
    input: String,
    seq: u64,
    logs: Vec<LogEntry>,
    available: bool,
    initialized: bool,
    session_id: Option<omnipilot_core::SessionId>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            app_id: None,
            dark_mode: false,
            input: String::new(),
            seq: 0,
            logs: Vec::new(),
            available: true,
            initialized: false,
            session_id: None,
        }
    }
}

/// Cloneable mock adapter. Clones share the same state, which makes it usable
/// both in-process and as an out-of-process protocol service in tests.
#[derive(Clone)]
pub struct MockAdapter {
    state: Arc<Mutex<MockState>>,
    token: Arc<String>,
    device: DeviceInfo,
}

impl Default for MockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockAdapter {
    pub fn new() -> Self {
        Self::with_token(DEFAULT_TOKEN)
    }

    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState::default())),
            token: Arc::new(token.into()),
            device: DeviceInfo {
                id: DeviceId::new(DEMO_DEVICE_ID),
                platform: Platform::Mock,
                name: "OmniPilot Mock Phone".to_string(),
                model: Some("MockPhone1".to_string()),
                os_version: Some("1.0".to_string()),
                screen: Size {
                    width: SCREEN_WIDTH,
                    height: SCREEN_HEIGHT,
                },
                status: DeviceStatus::Ready,
            },
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, MockState>, OmniError> {
        self.state
            .lock()
            .map_err(|_| OmniError::internal("mock adapter state lock poisoned"))
    }

    fn ensure_available(state: &MockState) -> Result<(), OmniError> {
        if state.available {
            Ok(())
        } else {
            Err(OmniError::new(
                ErrorCode::DeviceUnavailable,
                "mock adapter has been shut down",
            ))
        }
    }

    fn log(state: &mut MockState, level: &str, tag: &str, message: impl Into<String>) {
        state.logs.push(LogEntry {
            timestamp: Utc::now(),
            level: level.to_string(),
            tag: tag.to_string(),
            message: message.into(),
        });
        if state.logs.len() > 500 {
            let excess = state.logs.len() - 500;
            state.logs.drain(0..excess);
        }
    }

    fn elements(state: &MockState) -> Vec<Element> {
        let Some(app_id) = state.app_id.as_deref() else {
            return Vec::new();
        };
        if app_id != DEMO_APP {
            return Vec::new();
        }
        match state.screen {
            Screen::Home => vec![
                text_element("home.title", "heading", "OmniPilot Demo", 40, 90, 310, 48),
                button("home.settings", "Settings", 40, 220, 310, 64),
                button("home.about", "About", 40, 320, 310, 64),
            ],
            Screen::Settings => vec![
                button("settings.back", "Back", 20, 48, 96, 48),
                text_element("settings.title", "heading", "Settings", 40, 120, 310, 48),
                Element {
                    id: "settings.dark_mode".to_string(),
                    parent_id: None,
                    role: "switch".to_string(),
                    label: Some("Dark mode".to_string()),
                    value: Some(if state.dark_mode { "On" } else { "Off" }.to_string()),
                    identifier: Some("settings.dark_mode".to_string()),
                    text: Some("Dark mode".to_string()),
                    bounds: Bounds {
                        x: 40,
                        y: 220,
                        width: 310,
                        height: 64,
                    },
                    enabled: true,
                    focused: false,
                    clickable: true,
                },
                Element {
                    id: "settings.search".to_string(),
                    parent_id: None,
                    role: "text_field".to_string(),
                    label: Some("Search settings".to_string()),
                    value: Some(state.input.clone()),
                    identifier: Some("settings.search".to_string()),
                    text: Some("Search settings".to_string()),
                    bounds: Bounds {
                        x: 40,
                        y: 330,
                        width: 310,
                        height: 56,
                    },
                    enabled: true,
                    focused: false,
                    clickable: true,
                },
            ],
            Screen::About => vec![
                button("about.back", "Back", 20, 48, 96, 48),
                text_element(
                    "about.title",
                    "heading",
                    "About OmniPilot",
                    40,
                    120,
                    310,
                    48,
                ),
                text_element(
                    "about.body",
                    "text",
                    "A verification-first mobile automation platform.",
                    40,
                    200,
                    310,
                    96,
                ),
            ],
        }
    }

    fn render(&self, state: &mut MockState, include_screenshot: bool) -> Observation {
        state.seq += 1;
        let app = state.app_id.as_ref().map(|app_id| AppState {
            app_id: app_id.clone(),
            activity: Some(state.screen.activity().to_string()),
            foreground: true,
        });
        Observation {
            id: SnapshotId::new(),
            captured_at: Utc::now(),
            platform: Platform::Mock,
            app,
            screen: Size {
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
            },
            elements: Self::elements(state),
            screenshot: include_screenshot.then(|| Screenshot {
                mime: "image/png".to_string(),
                base64: ONE_PIXEL_PNG.to_string(),
                width: 1,
                height: 1,
                redacted: false,
            }),
            hierarchy_truncated: false,
        }
    }

    fn resolve_target(state: &MockState, target: &Target) -> Result<String, OmniError> {
        let observation_elements = Self::elements(state);
        match target {
            Target::Element { selector } => {
                let found = match selector {
                    omnipilot_core::Selector::Index { value } => {
                        observation_elements.get(*value as usize)
                    }
                    _ => observation_elements
                        .iter()
                        .find(|element| element.matches(selector)),
                };
                found.map(|element| element.id.clone()).ok_or_else(|| {
                    OmniError::new(
                        ErrorCode::ElementNotFound,
                        format!("no mock element matches {}", selector.describe()),
                    )
                    .with_retryable(true)
                })
            }
            Target::Point { point } => {
                if !point.is_valid() {
                    return Err(OmniError::invalid(
                        "point coordinates must be in the 0..=1000 range",
                    ));
                }
                let x = point.x * SCREEN_WIDTH / 1000;
                let y = point.y * SCREEN_HEIGHT / 1000;
                observation_elements
                    .iter()
                    .find(|element| {
                        element.bounds.contains(&omnipilot_core::Point {
                            x,
                            y,
                            description: None,
                        })
                    })
                    .map(|element| element.id.clone())
                    .ok_or_else(|| {
                        OmniError::new(ErrorCode::ElementNotFound, "no element at that point")
                    })
            }
        }
    }

    fn perform_locked(&self, state: &mut MockState, action: &Action) -> Result<String, OmniError> {
        Self::ensure_available(state)?;
        action.validate()?;
        if !state.initialized {
            // In-process use is allowed without a handshake; the flag only
            // records that a protocol client has connected.
        }
        match action {
            Action::LaunchApp { app_id } => {
                if app_id != DEMO_APP {
                    return Err(OmniError::new(
                        ErrorCode::AppNotFound,
                        format!("mock app not installed: {app_id}"),
                    ));
                }
                state.app_id = Some(app_id.clone());
                state.screen = Screen::Home;
                state.input.clear();
                Self::log(state, "info", "activity", "launched OmniPilot Demo");
                Ok("launched com.omnipilot.demo".to_string())
            }
            Action::TerminateApp { app_id } => {
                if state.app_id.as_deref() == Some(app_id.as_str()) {
                    state.app_id = None;
                    Self::log(state, "info", "activity", format!("terminated {app_id}"));
                    Ok(format!("terminated {app_id}"))
                } else {
                    Ok(format!("{app_id} was not running"))
                }
            }
            Action::Tap { target, .. } | Action::LongPress { target, .. } => {
                let id = Self::resolve_target(state, target)?;
                let message = match id.as_str() {
                    "home.settings" => {
                        state.screen = Screen::Settings;
                        "opened Settings".to_string()
                    }
                    "home.about" => {
                        state.screen = Screen::About;
                        "opened About".to_string()
                    }
                    "settings.back" | "about.back" => {
                        state.screen = Screen::Home;
                        "returned Home".to_string()
                    }
                    "settings.dark_mode" => {
                        state.dark_mode = !state.dark_mode;
                        format!(
                            "dark mode is now {}",
                            if state.dark_mode { "On" } else { "Off" }
                        )
                    }
                    "settings.search" => "focused the search field".to_string(),
                    other => format!("tapped {other}"),
                };
                Self::log(state, "info", "input", message.clone());
                Ok(message)
            }
            Action::Swipe { direction, .. } => match (state.screen.clone(), direction) {
                (Screen::Home, omnipilot_core::SwipeDirection::Up) => {
                    state.screen = Screen::Settings;
                    Ok("swiped up to Settings".to_string())
                }
                (Screen::Settings, omnipilot_core::SwipeDirection::Down) => {
                    state.screen = Screen::Home;
                    Ok("swiped down to Home".to_string())
                }
                _ => Ok(format!("swiped {direction:?}")),
            },
            Action::InputText {
                target,
                text,
                clear_existing,
            } => {
                if let Some(target) = target {
                    let _ = Self::resolve_target(state, target)?;
                }
                if *clear_existing {
                    state.input.clear();
                }
                state.input.push_str(text);
                Self::log(
                    state,
                    "info",
                    "input",
                    format!("entered {} chars", text.chars().count()),
                );
                Ok("entered text".to_string())
            }
            Action::PressKey { key } => {
                let message = match key {
                    omnipilot_core::Key::Back => {
                        state.screen = Screen::Home;
                        "pressed Back".to_string()
                    }
                    other => format!("pressed {other:?}"),
                };
                Self::log(state, "info", "input", message.clone());
                Ok(message)
            }
            Action::Wait { .. } => Ok("waited".to_string()),
            Action::Screenshot => Ok("captured screenshot".to_string()),
        }
    }

    /// A small, fully deterministic end-to-end scenario used by the CLI demo
    /// and integration tests.
    pub fn demo_task() -> TaskSpec {
        let mut spec = TaskSpec::new(
            "Open the demo Settings screen and turn on dark mode",
            Platform::Mock,
        );
        spec.device_id = Some(DeviceId::new(DEMO_DEVICE_ID));
        spec.app_id = Some(DEMO_APP.to_string());
        spec.steps = vec![
            Action::LaunchApp {
                app_id: DEMO_APP.to_string(),
            },
            Action::Tap {
                target: Target::Element {
                    selector: omnipilot_core::Selector::Identifier {
                        value: "home.settings".to_string(),
                    },
                },
                times: None,
                duration_ms: None,
            },
            Action::Tap {
                target: Target::Element {
                    selector: omnipilot_core::Selector::Identifier {
                        value: "settings.dark_mode".to_string(),
                    },
                },
                times: None,
                duration_ms: None,
            },
        ];
        spec.success_criteria = vec![
            Assertion::ElementValueEquals {
                selector: omnipilot_core::Selector::Identifier {
                    value: "settings.dark_mode".to_string(),
                },
                value: "On".to_string(),
            },
            Assertion::TextPresent {
                text: "Dark mode".to_string(),
            },
        ];
        spec.budgets.max_steps = 10;
        spec
    }
}

fn button(id: &str, label: &str, x: i32, y: i32, width: u32, height: u32) -> Element {
    Element {
        id: id.to_string(),
        parent_id: None,
        role: "button".to_string(),
        label: Some(label.to_string()),
        value: None,
        identifier: Some(id.to_string()),
        text: Some(label.to_string()),
        bounds: Bounds {
            x,
            y,
            width,
            height,
        },
        enabled: true,
        focused: false,
        clickable: true,
    }
}

fn text_element(
    id: &str,
    role: &str,
    text: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Element {
    Element {
        id: id.to_string(),
        parent_id: None,
        role: role.to_string(),
        label: None,
        value: None,
        identifier: Some(id.to_string()),
        text: Some(text.to_string()),
        bounds: Bounds {
            x,
            y,
            width,
            height,
        },
        enabled: true,
        focused: false,
        clickable: false,
    }
}

fn tokens_match(expected: &str, actual: &str) -> bool {
    if expected.len() != actual.len() {
        return false;
    }
    expected
        .as_bytes()
        .iter()
        .zip(actual.as_bytes())
        .fold(0u8, |acc, (left, right)| acc | (left ^ right))
        == 0
}

#[async_trait]
impl AdapterService for MockAdapter {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, RpcError> {
        omnipilot_protocol::validate_initialize(&params.protocol_version)?;
        if !tokens_match(&self.token, &params.token) {
            return Err(RpcError::unauthorized("invalid session token"));
        }
        let capabilities = self.capabilities().await;
        let mut state = self
            .lock()
            .map_err(|error| RpcError::internal(error.to_string()))?;
        if !state.available {
            return Err(RpcError::application("mock adapter is shut down"));
        }
        state.initialized = true;
        state.session_id = Some(params.session_id);
        Ok(InitializeResult {
            protocol_version: omnipilot_core::PROTOCOL_VERSION.to_string(),
            adapter: AdapterInfo {
                name: "omnipilot-mock".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                platform: Platform::Mock,
            },
            device: self.device.clone(),
            capabilities,
            limits: AdapterLimits::default(),
        })
    }

    async fn observe(&self, params: ObserveParams) -> Result<Observation, RpcError> {
        let mut state = self
            .lock()
            .map_err(|error| RpcError::internal(error.to_string()))?;
        Self::ensure_available(&state).map_err(RpcError::from_omni)?;
        let mut observation = self.render(&mut state, params.include_screenshot);
        if !params.include_hierarchy {
            observation.elements.clear();
        }
        Ok(observation)
    }

    async fn act(&self, params: ActParams) -> Result<ActionOutcome, RpcError> {
        let action_id = params.action_id.unwrap_or_default();
        let mut outcome = self
            .execute(&params.action)
            .await
            .map_err(RpcError::from_omni)?;
        outcome.action_id = action_id;
        Ok(outcome)
    }

    async fn logs(&self, params: LogsParams) -> Result<LogsResult, RpcError> {
        let limit = params.limit.unwrap_or(100).min(500) as usize;
        let entries = DeviceAdapter::logs(self, limit)
            .await
            .map_err(RpcError::from_omni)?;
        Ok(LogsResult { entries })
    }

    async fn cancel(&self, params: CancelParams) -> Result<SimpleResult, RpcError> {
        let mut state = self
            .lock()
            .map_err(|error| RpcError::internal(error.to_string()))?;
        let reason = params
            .reason
            .unwrap_or_else(|| "client cancelled".to_string());
        Self::log(
            &mut state,
            "warn",
            "session",
            format!("cancel requested: {reason}"),
        );
        Ok(SimpleResult {
            ok: true,
            message: reason,
        })
    }

    async fn shutdown(&self) -> Result<SimpleResult, RpcError> {
        let mut state = self
            .lock()
            .map_err(|error| RpcError::internal(error.to_string()))?;
        state.available = false;
        Self::log(&mut state, "info", "session", "shutdown");
        Ok(SimpleResult::ok())
    }
}

#[async_trait]
impl DeviceAdapter for MockAdapter {
    fn info(&self) -> DeviceInfo {
        let mut device = self.device.clone();
        if let Ok(state) = self.state.lock() {
            device.status = if state.available {
                DeviceStatus::Ready
            } else {
                DeviceStatus::Unavailable
            };
        }
        device
    }

    async fn capabilities(&self) -> Vec<Capability> {
        [
            "launch_app",
            "terminate_app",
            "tap",
            "long_press",
            "swipe",
            "input_text",
            "press_key",
            "wait",
            "screenshot",
            "observe",
            "logs",
        ]
        .into_iter()
        .map(|name| Capability {
            name: name.to_string(),
            support: SupportLevel::Supported,
            note: None,
        })
        .collect()
    }

    async fn observe(&self, include_screenshot: bool) -> Result<Observation, OmniError> {
        let mut state = self.lock()?;
        Self::ensure_available(&state)?;
        Ok(self.render(&mut state, include_screenshot))
    }

    async fn execute(&self, action: &Action) -> Result<ActionOutcome, OmniError> {
        if let Action::Wait { duration_ms } = action {
            action.validate()?;
            if *duration_ms > 0 {
                sleep(Duration::from_millis(*duration_ms)).await;
            }
            let state = self.lock()?;
            Self::ensure_available(&state)?;
            return Ok(ActionOutcome {
                action_id: ActionId::new(),
                status: ActionStatus::Executed,
                message: format!("waited {duration_ms}ms"),
                observation: None,
            });
        }

        let mut state = self.lock()?;
        let message = self.perform_locked(&mut state, action)?;
        let observation = self.render(&mut state, true);
        Ok(ActionOutcome {
            action_id: ActionId::new(),
            status: ActionStatus::Executed,
            message,
            observation: Some(observation),
        })
    }

    async fn logs(&self, limit: usize) -> Result<Vec<LogEntry>, OmniError> {
        let state = self.lock()?;
        let start = state.logs.len().saturating_sub(limit);
        Ok(state.logs[start..].to_vec())
    }

    async fn shutdown(&self) -> Result<(), OmniError> {
        let mut state = self.lock()?;
        state.available = false;
        Self::log(&mut state, "info", "session", "shutdown");
        Ok(())
    }
}

/// A fresh session handshake for tests and embedding.
pub fn mock_handshake() -> (InitializeParams, omnipilot_core::SessionId, String) {
    let (session_id, _) = omnipilot_protocol::new_session();
    (
        InitializeParams {
            protocol_version: omnipilot_core::PROTOCOL_VERSION.to_string(),
            session_id,
            token: DEFAULT_TOKEN.to_string(),
            client: omnipilot_core::ClientInfo {
                name: "omnipilot-test".to_string(),
                version: "0.1.0".to_string(),
            },
        },
        session_id,
        DEFAULT_TOKEN.to_string(),
    )
}

/// Helper for tests that need a unique request id.
pub fn request_id() -> u64 {
    Uuid::new_v4().as_u128() as u64
}
