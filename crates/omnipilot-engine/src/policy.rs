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

//! Deny-by-default policy for side effects. The protocol has no arbitrary shell
//! command, so policy focuses on app scope and the risk of each modeled action.

use omnipilot_core::{Action, ErrorCode, OmniError, RiskLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// If non-empty, app lifecycle actions are limited to these bundle ids.
    #[serde(default)]
    pub allowed_apps: Vec<String>,
    #[serde(default = "default_true")]
    pub allow_ui_mutations: bool,
    #[serde(default)]
    pub allow_external_side_effects: bool,
    #[serde(default)]
    pub allow_privileged: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allowed_apps: Vec::new(),
            allow_ui_mutations: true,
            allow_external_side_effects: false,
            allow_privileged: false,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    RequireApproval(RiskLevel),
    Deny(String),
}

#[derive(Debug, Clone)]
pub struct Policy {
    config: PolicyConfig,
}

impl Default for Policy {
    fn default() -> Self {
        Self::new(PolicyConfig::default())
    }
}

impl Policy {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &PolicyConfig {
        &self.config
    }

    pub fn risk(action: &Action) -> RiskLevel {
        match action {
            Action::Wait { .. } | Action::Screenshot => RiskLevel::ReadOnly,
            Action::LaunchApp { .. }
            | Action::TerminateApp { .. }
            | Action::Tap { .. }
            | Action::LongPress { .. }
            | Action::Swipe { .. }
            | Action::InputText { .. }
            | Action::PressKey { .. } => RiskLevel::UiMutation,
        }
    }

    pub fn evaluate(&self, action: &Action, approved: &[RiskLevel]) -> PolicyDecision {
        let risk = Self::risk(action);
        let approved = approved.contains(&risk);

        match risk {
            RiskLevel::ReadOnly => PolicyDecision::Allow,
            RiskLevel::UiMutation => {
                if !self.config.allow_ui_mutations && !approved {
                    PolicyDecision::RequireApproval(risk)
                } else {
                    PolicyDecision::Allow
                }
            }
            RiskLevel::ExternalSideEffect => {
                if self.config.allow_external_side_effects || approved {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::RequireApproval(risk)
                }
            }
            RiskLevel::Privileged => {
                if self.config.allow_privileged || approved {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny("privileged actions are disabled".to_string())
                }
            }
        }
    }

    pub fn authorize(&self, action: &Action, approved: &[RiskLevel]) -> Result<(), OmniError> {
        if let Action::LaunchApp { app_id } | Action::TerminateApp { app_id } = action {
            if !self.config.allowed_apps.is_empty()
                && !self
                    .config
                    .allowed_apps
                    .iter()
                    .any(|allowed| allowed == app_id)
            {
                return Err(OmniError::new(
                    ErrorCode::PolicyDenied,
                    format!("app {app_id} is not in the policy allowlist"),
                ));
            }
        }

        match self.evaluate(action, approved) {
            PolicyDecision::Allow => Ok(()),
            PolicyDecision::RequireApproval(risk) => Err(OmniError::new(
                ErrorCode::PolicyDenied,
                format!("action requires explicit approval for risk level {risk:?}"),
            )),
            PolicyDecision::Deny(reason) => Err(OmniError::new(ErrorCode::PolicyDenied, reason)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omnipilot_core::{Selector, Target};

    fn tap() -> Action {
        Action::Tap {
            target: Target::Element {
                selector: Selector::Text {
                    value: "Continue".to_string(),
                },
            },
            times: None,
            duration_ms: None,
        }
    }

    #[test]
    fn read_only_is_allowed_by_default() {
        assert_eq!(
            Policy::default().evaluate(&Action::Screenshot, &[]),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn ui_mutation_requires_approval_when_disabled() {
        let policy = Policy::new(PolicyConfig {
            allow_ui_mutations: false,
            ..PolicyConfig::default()
        });
        assert!(matches!(
            policy.evaluate(&tap(), &[]),
            PolicyDecision::RequireApproval(RiskLevel::UiMutation)
        ));
        assert_eq!(
            policy.evaluate(&tap(), &[RiskLevel::UiMutation]),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn app_allowlist_is_enforced() {
        let policy = Policy::new(PolicyConfig {
            allowed_apps: vec!["com.example.allowed".to_string()],
            ..PolicyConfig::default()
        });
        let error = policy
            .authorize(
                &Action::LaunchApp {
                    app_id: "com.example.denied".to_string(),
                },
                &[],
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::PolicyDenied);
    }
}
