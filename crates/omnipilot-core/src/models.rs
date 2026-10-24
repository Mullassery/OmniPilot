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

//! Domain vocabulary for devices, observations, actions, assertions, tasks,
//! evidence, and protocol handshakes.

use crate::error::OmniError;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            Serialize,
            Deserialize,
            JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(value)?))
            }
        }
    };
}

uuid_id!(SessionId);
uuid_id!(TaskId);
uuid_id!(ActionId);
uuid_id!(SnapshotId);
uuid_id!(EvidenceId);

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct DeviceId(pub String);

impl DeviceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for DeviceId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Android,
    Ios,
    Mock,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Mock => "mock",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Ready,
    Busy,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    pub fn contains(&self, point: &Point) -> bool {
        let x = point.x as i32;
        let y = point.y as i32;
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.width as i32)
            && y < self.y.saturating_add(self.height as i32)
    }
}

/// A target point in a 0..=1000 normalized coordinate space. Keeping targets
/// normalized lets a plan move between devices with different resolutions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Point {
    pub x: u32,
    pub y: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Point {
    pub fn is_valid(&self) -> bool {
        self.x <= 1000 && self.y <= 1000
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selector {
    Identifier { value: String },
    Text { value: String },
    TextContains { value: String },
    RoleAndText { role: String, text: String },
    Label { value: String },
    Index { value: u32 },
}

impl Selector {
    pub fn describe(&self) -> String {
        match self {
            Self::Identifier { value } => format!("identifier={value}"),
            Self::Text { value } => format!("text={value}"),
            Self::TextContains { value } => format!("text~={value}"),
            Self::RoleAndText { role, text } => format!("role={role} text={text}"),
            Self::Label { value } => format!("label={value}"),
            Self::Index { value } => format!("index={value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    Element { selector: Selector },
    Point { point: Point },
}

impl Target {
    pub fn element(selector: Selector) -> Self {
        Self::Element { selector }
    }

    pub fn describe(&self) -> String {
        match self {
            Self::Element { selector } => selector.describe(),
            Self::Point { point } => format!("point({},{})", point.x, point.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    Back,
    Home,
    Enter,
    Delete,
    AppSwitch,
    VolumeUp,
    VolumeDown,
    Power,
    Custom(String),
}

/// An action an adapter can execute. There is intentionally no arbitrary shell
/// or ADB command here: OmniPilot exposes only device operations it can model,
/// authorize, and verify.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    LaunchApp {
        app_id: String,
    },
    TerminateApp {
        app_id: String,
    },
    Tap {
        target: Target,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        times: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
    LongPress {
        target: Target,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
    Swipe {
        direction: SwipeDirection,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        distance: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
    InputText {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<Target>,
        text: String,
        #[serde(default)]
        clear_existing: bool,
    },
    PressKey {
        key: Key,
    },
    Wait {
        duration_ms: u64,
    },
    Screenshot,
}

impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LaunchApp { .. } => "launch_app",
            Self::TerminateApp { .. } => "terminate_app",
            Self::Tap { .. } => "tap",
            Self::LongPress { .. } => "long_press",
            Self::Swipe { .. } => "swipe",
            Self::InputText { .. } => "input_text",
            Self::PressKey { .. } => "press_key",
            Self::Wait { .. } => "wait",
            Self::Screenshot => "screenshot",
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Self::LaunchApp { app_id } => format!("launch {app_id}"),
            Self::TerminateApp { app_id } => format!("terminate {app_id}"),
            Self::Tap { target, .. } => format!("tap {}", target.describe()),
            Self::LongPress { target, .. } => format!("long-press {}", target.describe()),
            Self::Swipe { direction, .. } => format!("swipe {direction:?}"),
            Self::InputText { text, .. } => format!("input {} chars", text.chars().count()),
            Self::PressKey { key } => format!("press {key:?}"),
            Self::Wait { duration_ms } => format!("wait {duration_ms}ms"),
            Self::Screenshot => "screenshot".to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), OmniError> {
        match self {
            Self::LaunchApp { app_id } | Self::TerminateApp { app_id } if app_id.is_empty() => {
                Err(OmniError::invalid("app_id must not be empty"))
            }
            Self::Tap {
                target,
                times,
                duration_ms,
            } => {
                validate_target(target)?;
                if times == &Some(0) {
                    return Err(OmniError::invalid("tap times must be at least 1"));
                }
                if duration_ms == &Some(0) {
                    return Err(OmniError::invalid("tap duration_ms must be positive"));
                }
                Ok(())
            }
            Self::LongPress {
                target,
                duration_ms,
            } => {
                validate_target(target)?;
                if duration_ms == &Some(0) {
                    return Err(OmniError::invalid(
                        "long_press duration_ms must be positive",
                    ));
                }
                Ok(())
            }
            Self::Swipe {
                distance,
                duration_ms,
                ..
            } => {
                if let Some(distance) = distance {
                    if !(0.0..=1.0).contains(distance) {
                        return Err(OmniError::invalid(
                            "swipe distance must be between 0.0 and 1.0",
                        ));
                    }
                }
                if duration_ms == &Some(0) {
                    return Err(OmniError::invalid("swipe duration_ms must be positive"));
                }
                Ok(())
            }
            Self::InputText { target, .. } => {
                if let Some(target) = target {
                    validate_target(target)?;
                }
                Ok(())
            }
            Self::Wait { duration_ms } if *duration_ms == 0 => {
                Err(OmniError::invalid("wait duration_ms must be positive"))
            }
            _ => Ok(()),
        }
    }
}

fn validate_target(target: &Target) -> Result<(), OmniError> {
    match target {
        Target::Element { selector } => {
            let empty = match selector {
                Selector::Identifier { value }
                | Selector::Text { value }
                | Selector::TextContains { value }
                | Selector::Label { value } => value.is_empty(),
                Selector::RoleAndText { role, text } => role.is_empty() || text.is_empty(),
                Selector::Index { .. } => false,
            };
            if empty {
                Err(OmniError::invalid("selector value must not be empty"))
            } else {
                Ok(())
            }
        }
        Target::Point { point } => {
            if point.is_valid() {
                Ok(())
            } else {
                Err(OmniError::invalid(
                    "point coordinates must be in the 0..=1000 range",
                ))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Element {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub bounds: Bounds,
    pub enabled: bool,
    pub focused: bool,
    pub clickable: bool,
}

impl Element {
    pub fn matches(&self, selector: &Selector) -> bool {
        match selector {
            Selector::Identifier { value } => self.identifier.as_deref() == Some(value),
            Selector::Text { value } => {
                self.text.as_deref() == Some(value) || self.label.as_deref() == Some(value)
            }
            Selector::TextContains { value } => {
                let needle = value.to_lowercase();
                [
                    self.text.as_deref(),
                    self.label.as_deref(),
                    self.value.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|candidate| candidate.to_lowercase().contains(&needle))
            }
            Selector::RoleAndText { role, text } => {
                self.role.eq_ignore_ascii_case(role)
                    && (self.text.as_deref() == Some(text.as_str())
                        || self.label.as_deref() == Some(text.as_str()))
            }
            Selector::Label { value } => self.label.as_deref() == Some(value),
            Selector::Index { .. } => false,
        }
    }

    /// Every string field a user or model might reasonably search for.
    pub fn searchable_text(&self) -> impl Iterator<Item = &str> {
        [
            self.text.as_deref(),
            self.label.as_deref(),
            self.value.as_deref(),
            self.identifier.as_deref(),
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AppState {
    pub app_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    pub foreground: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Screenshot {
    pub mime: String,
    pub base64: String,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Observation {
    pub id: SnapshotId,
    pub captured_at: DateTime<Utc>,
    pub platform: Platform,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<AppState>,
    pub screen: Size,
    pub elements: Vec<Element>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<Screenshot>,
    #[serde(default)]
    pub hierarchy_truncated: bool,
}

impl Observation {
    pub fn find(&self, selector: &Selector) -> Option<&Element> {
        match selector {
            Selector::Index { value } => self.elements.get(*value as usize),
            _ => self
                .elements
                .iter()
                .find(|element| element.matches(selector)),
        }
    }

    pub fn find_all<'a>(&'a self, selector: &'a Selector) -> Vec<&'a Element> {
        match selector {
            Selector::Index { value } => self.elements.get(*value as usize).into_iter().collect(),
            _ => self
                .elements
                .iter()
                .filter(|element| element.matches(selector))
                .collect(),
        }
    }

    pub fn contains_text(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        self.elements
            .iter()
            .flat_map(Element::searchable_text)
            .any(|candidate| candidate.to_lowercase().contains(&needle))
    }

    pub fn element_by_id(&self, id: &str) -> Option<&Element> {
        self.elements.iter().find(|element| element.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub tag: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Supported,
    RequiresPermission,
    SimulatorOnly,
    Experimental,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Capability {
    pub name: String,
    pub support: SupportLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub platform: Platform,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    pub screen: Size,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AdapterLimits {
    pub max_observation_bytes: u64,
    pub max_log_entries: u32,
    pub supports_cancellation: bool,
    pub supports_screenshot: bool,
    pub supports_hierarchy: bool,
}

impl Default for AdapterLimits {
    fn default() -> Self {
        Self {
            max_observation_bytes: 4 * 1024 * 1024,
            max_log_entries: 500,
            supports_cancellation: true,
            supports_screenshot: true,
            supports_hierarchy: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AdapterInfo {
    pub name: String,
    pub version: String,
    pub platform: Platform,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub session_id: SessionId,
    pub token: String,
    pub client: ClientInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub adapter: AdapterInfo,
    pub device: DeviceInfo,
    pub capabilities: Vec<Capability>,
    pub limits: AdapterLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    ReadOnly,
    UiMutation,
    ExternalSideEffect,
    Privileged,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Budgets {
    pub max_steps: u32,
    pub max_duration_ms: u64,
    pub max_retries: u32,
}

impl Default for Budgets {
    fn default() -> Self {
        Self {
            max_steps: 20,
            max_duration_ms: 120_000,
            max_retries: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TaskSpec {
    pub goal: String,
    pub platform: Platform,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<DeviceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(default)]
    pub steps: Vec<Action>,
    #[serde(default)]
    pub success_criteria: Vec<Assertion>,
    #[serde(default)]
    pub budgets: Budgets,
    #[serde(default)]
    pub approved_risks: Vec<RiskLevel>,
}

impl TaskSpec {
    pub fn new(goal: impl Into<String>, platform: Platform) -> Self {
        Self {
            goal: goal.into(),
            platform,
            device_id: None,
            app_id: None,
            steps: Vec::new(),
            success_criteria: Vec::new(),
            budgets: Budgets::default(),
            approved_risks: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), OmniError> {
        if self.goal.trim().is_empty() {
            return Err(OmniError::invalid("goal must not be empty"));
        }
        if self.steps.len() > self.budgets.max_steps as usize {
            return Err(OmniError::invalid(format!(
                "plan has {} steps but max_steps is {}",
                self.steps.len(),
                self.budgets.max_steps
            )));
        }
        for (index, step) in self.steps.iter().enumerate() {
            step.validate()
                .map_err(|error| error.with_details(serde_json::json!({ "step_index": index })))?;
        }
        if self.budgets.max_duration_ms == 0 {
            return Err(OmniError::invalid("max_duration_ms must be positive"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Assertion {
    TextPresent {
        text: String,
    },
    TextAbsent {
        text: String,
    },
    ElementPresent {
        selector: Selector,
    },
    ElementAbsent {
        selector: Selector,
    },
    ElementEnabled {
        selector: Selector,
        enabled: bool,
    },
    ElementValueEquals {
        selector: Selector,
        value: String,
    },
    AppForeground {
        app_id: String,
    },
    ScreenChanged {
        #[serde(default)]
        from: Option<SnapshotId>,
    },
}

impl Assertion {
    pub fn describe(&self) -> String {
        match self {
            Self::TextPresent { text } => format!("text present: {text}"),
            Self::TextAbsent { text } => format!("text absent: {text}"),
            Self::ElementPresent { selector } => {
                format!("element present: {}", selector.describe())
            }
            Self::ElementAbsent { selector } => format!("element absent: {}", selector.describe()),
            Self::ElementEnabled { selector, enabled } => {
                format!("element enabled={enabled}: {}", selector.describe())
            }
            Self::ElementValueEquals { selector, value } => {
                format!("element value={value}: {}", selector.describe())
            }
            Self::AppForeground { app_id } => format!("app foreground: {app_id}"),
            Self::ScreenChanged { .. } => "screen changed".to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), OmniError> {
        match self {
            Self::TextPresent { text } | Self::TextAbsent { text } if text.is_empty() => {
                Err(OmniError::invalid("assertion text must not be empty"))
            }
            Self::ElementPresent { selector }
            | Self::ElementAbsent { selector }
            | Self::ElementEnabled { selector, .. }
            | Self::ElementValueEquals { selector, .. } => validate_target(&Target::Element {
                selector: selector.clone(),
            }),
            Self::AppForeground { app_id } if app_id.is_empty() => {
                Err(OmniError::invalid("assertion app_id must not be empty"))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Running,
    Verifying,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Blocked,
    Inconclusive,
}

impl TaskStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Failed
                | Self::Cancelled
                | Self::TimedOut
                | Self::Blocked
                | Self::Inconclusive
        )
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Verifying => "verifying",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Blocked => "blocked",
            Self::Inconclusive => "inconclusive",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Executed,
    Failed,
    Unsupported,
    Denied,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Executed,
    Failed,
    Unsupported,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionOutcome {
    pub action_id: ActionId,
    pub status: ActionStatus,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<Observation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StepResult {
    pub index: u32,
    pub action: Action,
    pub status: StepStatus,
    pub message: String,
    pub attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<SnapshotId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Screenshot,
    Hierarchy,
    Logs,
    Trace,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceRef {
    pub id: EvidenceId,
    pub kind: EvidenceKind,
    pub mime: String,
    pub uri: String,
    pub bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub created_at: DateTime<Utc>,
    pub redacted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TaskMetrics {
    pub steps: u32,
    pub retries: u32,
    pub model_calls: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub goal: String,
    pub summary: String,
    /// True only when every supplied success criterion was evaluated and passed.
    pub verified: bool,
    #[serde(default)]
    pub assertions: Vec<AssertionResult>,
    #[serde(default)]
    pub steps: Vec<StepResult>,
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    #[serde(default)]
    pub limitations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<OmniError>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    #[serde(default)]
    pub metrics: TaskMetrics,
}

impl TaskResult {
    pub fn failed(
        task_id: TaskId,
        goal: impl Into<String>,
        error: OmniError,
        started_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            task_id,
            status: match error.code {
                crate::error::ErrorCode::PolicyDenied => TaskStatus::Blocked,
                crate::error::ErrorCode::UnsupportedAction => TaskStatus::Blocked,
                crate::error::ErrorCode::Timeout => TaskStatus::TimedOut,
                crate::error::ErrorCode::Cancelled => TaskStatus::Cancelled,
                _ => TaskStatus::Failed,
            },
            goal: goal.into(),
            summary: error.message.clone(),
            verified: false,
            assertions: Vec::new(),
            steps: Vec::new(),
            evidence: Vec::new(),
            limitations: Vec::new(),
            error: Some(error),
            started_at,
            finished_at: now,
            metrics: TaskMetrics::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskEvent {
    StateChanged {
        from: TaskStatus,
        to: TaskStatus,
    },
    StepStarted {
        index: u32,
        action: Action,
    },
    StepFinished {
        index: u32,
        status: StepStatus,
        message: String,
        attempts: u32,
    },
    AssertionEvaluated {
        assertion: Assertion,
        passed: bool,
        message: String,
    },
    EvidenceCreated {
        evidence: EvidenceRef,
    },
    Limitation {
        message: String,
    },
    Log {
        level: String,
        message: String,
    },
    Finished {
        status: TaskStatus,
        verified: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TaskEventRecord {
    pub seq: u64,
    pub at: DateTime<Utc>,
    pub task_id: TaskId,
    pub event: TaskEvent,
}

/// Structured output from a one-off observation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservationEnvelope {
    pub observation: Observation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceRef>,
}

/// JSON value used for open-ended protocol metadata.
pub type Metadata = Value;
