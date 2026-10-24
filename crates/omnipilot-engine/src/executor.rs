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

//! The execution loop: observe, authorize, act, re-observe, assert. A task is
//! only `completed` and `verified` when its assertions passed against a fresh
//! observation, never merely because every command returned success.

use crate::evidence::EvidenceStore;
use crate::policy::Policy;
use crate::trace::Tracer;
use chrono::Utc;
use omnipilot_core::{
    Action, ActionStatus, Assertion, AssertionResult, ErrorCode, EvidenceRef, Observation,
    OmniError, SnapshotId, StepResult, StepStatus, SupportLevel, TaskEvent, TaskEventRecord,
    TaskId, TaskMetrics, TaskResult, TaskSpec, TaskStatus,
};
use omnipilot_protocol::DeviceAdapter;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct ExecutionContext {
    pub adapter: Arc<dyn DeviceAdapter>,
    pub policy: Policy,
    pub evidence: Arc<dyn EvidenceStore>,
    pub tracer: Arc<dyn Tracer>,
}

pub struct Executor {
    context: ExecutionContext,
}

struct RunState {
    task_id: TaskId,
    spec: TaskSpec,
    cancel: CancellationToken,
    deadline: Instant,
    seq: u64,
    status: TaskStatus,
    summary: String,
    verified: bool,
    steps: Vec<StepResult>,
    assertions: Vec<AssertionResult>,
    evidence: Vec<EvidenceRef>,
    limitations: Vec<String>,
    error: Option<OmniError>,
    initial: Option<Observation>,
    final_observation: Option<Observation>,
    metrics: TaskMetrics,
}

struct TaskFailure {
    status: TaskStatus,
    error: OmniError,
}

impl TaskFailure {
    fn new(status: TaskStatus, error: OmniError) -> Self {
        Self { status, error }
    }

    fn cancelled() -> Self {
        Self::new(
            TaskStatus::Cancelled,
            OmniError::new(ErrorCode::Cancelled, "task was cancelled"),
        )
    }

    fn timed_out() -> Self {
        Self::new(
            TaskStatus::TimedOut,
            OmniError::new(ErrorCode::Timeout, "task exceeded its time budget"),
        )
    }

    fn from_error(error: OmniError) -> Self {
        let status = match error.code {
            ErrorCode::PolicyDenied | ErrorCode::UnsupportedAction => TaskStatus::Blocked,
            ErrorCode::Timeout => TaskStatus::TimedOut,
            ErrorCode::Cancelled => TaskStatus::Cancelled,
            ErrorCode::DeviceUnavailable | ErrorCode::AdapterUnavailable => {
                TaskStatus::Inconclusive
            }
            _ => TaskStatus::Failed,
        };
        Self::new(status, error)
    }
}

impl Executor {
    pub fn new(context: ExecutionContext) -> Self {
        Self { context }
    }

    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    pub async fn execute(
        &self,
        task_id: TaskId,
        spec: TaskSpec,
        cancel: CancellationToken,
    ) -> TaskResult {
        let started_at = Utc::now();
        if let Err(error) = spec.validate() {
            return TaskResult::failed(task_id, spec.goal, error, started_at);
        }

        let deadline = Instant::now() + Duration::from_millis(spec.budgets.max_duration_ms);
        let mut state = RunState {
            task_id,
            spec,
            cancel,
            deadline,
            seq: 0,
            status: TaskStatus::Queued,
            summary: String::new(),
            verified: false,
            steps: Vec::new(),
            assertions: Vec::new(),
            evidence: Vec::new(),
            limitations: Vec::new(),
            error: None,
            initial: None,
            final_observation: None,
            metrics: TaskMetrics::default(),
        };

        if let Err(failure) = self.run(&mut state).await {
            state.summary = failure.error.message.clone();
            state.status = failure.status;
            state.error = Some(failure.error);
        }

        let finished_at = Utc::now();
        state.metrics.steps = state.steps.len() as u32;
        state.metrics.duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;

        let finished_status = state.status;
        let finished_verified = state.verified;
        self.emit(
            &mut state,
            TaskEvent::Finished {
                status: finished_status,
                verified: finished_verified,
            },
        );

        TaskResult {
            task_id,
            status: state.status,
            goal: state.spec.goal,
            summary: state.summary,
            verified: state.verified && state.status == TaskStatus::Completed,
            assertions: state.assertions,
            steps: state.steps,
            evidence: state.evidence,
            limitations: state.limitations,
            error: state.error,
            started_at,
            finished_at,
            metrics: state.metrics,
        }
    }

    async fn run(&self, state: &mut RunState) -> Result<(), TaskFailure> {
        self.transition(state, TaskStatus::Running);

        let info = self.context.adapter.info();
        if info.platform != state.spec.platform {
            return Err(TaskFailure::new(
                TaskStatus::Blocked,
                OmniError::unsupported(format!(
                    "device {} is {} but the task targets {}",
                    info.id,
                    info.platform.as_str(),
                    state.spec.platform.as_str()
                )),
            ));
        }

        let capabilities = self
            .await_with_stop(self.context.adapter.capabilities(), state)
            .await?;
        let capabilities: HashMap<String, SupportLevel> = capabilities
            .into_iter()
            .map(|capability| (capability.name, capability.support))
            .collect();

        let initial = self.observe(state, true).await?;
        self.capture(state, &initial, true)?;
        state.initial = Some(initial);

        let steps = state.spec.steps.clone();
        for (index, action) in steps.iter().enumerate() {
            self.check_stop(state)?;

            match capabilities.get(action.name()).copied() {
                Some(SupportLevel::Unsupported) | None => {
                    let message = format!("adapter does not support action {}", action.name());
                    self.push_step(
                        state,
                        index as u32,
                        action,
                        StepStatus::Unsupported,
                        message.clone(),
                        0,
                        None,
                    );
                    return Err(TaskFailure::new(
                        TaskStatus::Blocked,
                        OmniError::unsupported(message),
                    ));
                }
                _ => {}
            }

            if let Err(error) = self
                .context
                .policy
                .authorize(action, &state.spec.approved_risks)
            {
                self.push_step(
                    state,
                    index as u32,
                    action,
                    StepStatus::Denied,
                    error.message.clone(),
                    0,
                    None,
                );
                return Err(TaskFailure::from_error(error));
            }

            self.emit(
                state,
                TaskEvent::StepStarted {
                    index: index as u32,
                    action: action.clone(),
                },
            );

            let mut attempts = 0u32;
            loop {
                attempts += 1;
                let result = self
                    .await_with_stop(self.context.adapter.execute(action), state)
                    .await;

                match result {
                    Err(failure) => {
                        self.push_step(
                            state,
                            index as u32,
                            action,
                            StepStatus::Skipped,
                            failure.error.message.clone(),
                            attempts,
                            None,
                        );
                        return Err(failure);
                    }
                    Ok(Err(error)) => {
                        if error.retryable && attempts <= state.spec.budgets.max_retries {
                            state.metrics.retries += 1;
                            continue;
                        }
                        self.push_step(
                            state,
                            index as u32,
                            action,
                            StepStatus::Failed,
                            error.message.clone(),
                            attempts,
                            None,
                        );
                        return Err(TaskFailure::from_error(error));
                    }
                    Ok(Ok(outcome)) => {
                        if let Some(observation) = &outcome.observation {
                            self.capture(state, observation, false)?;
                        }
                        match outcome.status {
                            ActionStatus::Executed => {
                                self.push_step(
                                    state,
                                    index as u32,
                                    action,
                                    StepStatus::Executed,
                                    outcome.message,
                                    attempts,
                                    outcome
                                        .observation
                                        .as_ref()
                                        .map(|observation| observation.id),
                                );
                                break;
                            }
                            ActionStatus::Unsupported => {
                                self.push_step(
                                    state,
                                    index as u32,
                                    action,
                                    StepStatus::Unsupported,
                                    outcome.message.clone(),
                                    attempts,
                                    None,
                                );
                                return Err(TaskFailure::new(
                                    TaskStatus::Blocked,
                                    OmniError::unsupported(outcome.message),
                                ));
                            }
                            ActionStatus::Denied => {
                                self.push_step(
                                    state,
                                    index as u32,
                                    action,
                                    StepStatus::Denied,
                                    outcome.message.clone(),
                                    attempts,
                                    None,
                                );
                                return Err(TaskFailure::new(
                                    TaskStatus::Blocked,
                                    OmniError::new(ErrorCode::PolicyDenied, outcome.message),
                                ));
                            }
                            ActionStatus::Failed => {
                                if attempts <= state.spec.budgets.max_retries {
                                    state.metrics.retries += 1;
                                    continue;
                                }
                                let message = format!(
                                    "step failed after {attempts} attempt(s): {}",
                                    outcome.message
                                );
                                self.push_step(
                                    state,
                                    index as u32,
                                    action,
                                    StepStatus::Failed,
                                    message.clone(),
                                    attempts,
                                    None,
                                );
                                return Err(TaskFailure::new(
                                    TaskStatus::Failed,
                                    OmniError::new(ErrorCode::Internal, message),
                                ));
                            }
                        }
                    }
                }
            }
        }

        self.check_stop(state)?;
        self.transition(state, TaskStatus::Verifying);

        let final_observation = self.observe(state, true).await?;
        self.capture(state, &final_observation, true)?;
        let initial_ref = state.initial.clone().expect("initial observation stored");
        self.evaluate(state, &final_observation, &initial_ref);
        state.final_observation = Some(final_observation);
        Ok(())
    }

    fn evaluate(
        &self,
        state: &mut RunState,
        final_observation: &Observation,
        initial: &Observation,
    ) {
        let mut all_passed = true;
        let criteria = state.spec.success_criteria.clone();

        for assertion in &criteria {
            let result = evaluate_assertion(assertion, final_observation, initial);
            self.emit(
                state,
                TaskEvent::AssertionEvaluated {
                    assertion: assertion.clone(),
                    passed: result.passed,
                    message: result.message.clone(),
                },
            );
            if !result.passed {
                all_passed = false;
            }
            state.assertions.push(result);
        }

        if criteria.is_empty() {
            state.status = TaskStatus::Completed;
            state.verified = false;
            state.summary =
                "Completed without success criteria; the result is unverified.".to_string();
            let limitation =
                "No success criteria were supplied; actions ran but goal achievement was not proven."
                    .to_string();
            state.limitations.push(limitation.clone());
            self.emit(
                state,
                TaskEvent::Limitation {
                    message: limitation,
                },
            );
        } else if all_passed {
            state.status = TaskStatus::Completed;
            state.verified = true;
            let noun = if criteria.len() == 1 {
                "criterion"
            } else {
                "criteria"
            };
            state.summary = format!("Completed and verified {} success {noun}.", criteria.len());
        } else {
            state.status = TaskStatus::Failed;
            state.verified = false;
            state.summary =
                "Actions completed, but one or more success criteria failed.".to_string();
            state.error = Some(OmniError::new(
                ErrorCode::VerificationFailed,
                "one or more success criteria did not hold on the final observation",
            ));
        }
    }

    async fn observe(
        &self,
        state: &mut RunState,
        include_screenshot: bool,
    ) -> Result<Observation, TaskFailure> {
        let result = self
            .await_with_stop(self.context.adapter.observe(include_screenshot), state)
            .await?;
        result.map_err(TaskFailure::from_error)
    }

    async fn await_with_stop<F, T>(&self, future: F, state: &RunState) -> Result<T, TaskFailure>
    where
        F: Future<Output = T>,
    {
        tokio::select! {
            _ = state.cancel.cancelled() => Err(TaskFailure::cancelled()),
            _ = tokio::time::sleep_until(state.deadline) => Err(TaskFailure::timed_out()),
            value = future => Ok(value),
        }
    }

    fn check_stop(&self, state: &RunState) -> Result<(), TaskFailure> {
        if state.cancel.is_cancelled() {
            return Err(TaskFailure::cancelled());
        }
        if Instant::now() >= state.deadline {
            return Err(TaskFailure::timed_out());
        }
        Ok(())
    }

    fn capture(
        &self,
        state: &mut RunState,
        observation: &Observation,
        include_hierarchy: bool,
    ) -> Result<(), TaskFailure> {
        if let Some(screenshot) = &observation.screenshot {
            let reference = self
                .context
                .evidence
                .put_screenshot(screenshot)
                .map_err(TaskFailure::from_error)?;
            self.emit(
                state,
                TaskEvent::EvidenceCreated {
                    evidence: reference.clone(),
                },
            );
            state.evidence.push(reference);
        }

        if include_hierarchy {
            let reference = self
                .context
                .evidence
                .put_hierarchy(observation)
                .map_err(TaskFailure::from_error)?;
            self.emit(
                state,
                TaskEvent::EvidenceCreated {
                    evidence: reference.clone(),
                },
            );
            state.evidence.push(reference);
        }

        Ok(())
    }

    fn transition(&self, state: &mut RunState, to: TaskStatus) {
        if state.status == to {
            return;
        }
        let from = state.status;
        state.status = to;
        self.emit(state, TaskEvent::StateChanged { from, to });
    }

    #[allow(clippy::too_many_arguments)]
    fn push_step(
        &self,
        state: &mut RunState,
        index: u32,
        action: &Action,
        status: StepStatus,
        message: String,
        attempts: u32,
        observation: Option<SnapshotId>,
    ) {
        state.steps.push(StepResult {
            index,
            action: action.clone(),
            status,
            message: message.clone(),
            attempts,
            observation,
        });
        self.emit(
            state,
            TaskEvent::StepFinished {
                index,
                status,
                message,
                attempts,
            },
        );
    }

    fn emit(&self, state: &mut RunState, event: TaskEvent) {
        state.seq += 1;
        self.context.tracer.record(TaskEventRecord {
            seq: state.seq,
            at: Utc::now(),
            task_id: state.task_id,
            event,
        });
    }
}

fn evaluate_assertion(
    assertion: &Assertion,
    observation: &Observation,
    initial: &Observation,
) -> AssertionResult {
    match assertion {
        Assertion::TextPresent { text } => {
            let passed = observation.contains_text(text);
            assertion_result(
                assertion,
                passed,
                passed.then(|| "present".to_string()),
                if passed {
                    format!("text was present: {text}")
                } else {
                    format!("text was not present: {text}")
                },
            )
        }
        Assertion::TextAbsent { text } => {
            let passed = !observation.contains_text(text);
            assertion_result(
                assertion,
                passed,
                passed.then(|| "absent".to_string()),
                if passed {
                    format!("text was absent: {text}")
                } else {
                    format!("text was unexpectedly present: {text}")
                },
            )
        }
        Assertion::ElementPresent { selector } => {
            let element = observation.find(selector);
            let passed = element.is_some();
            assertion_result(
                assertion,
                passed,
                element.map(|element| element.id.clone()),
                if passed {
                    format!("element was present: {}", selector.describe())
                } else {
                    format!("element was not present: {}", selector.describe())
                },
            )
        }
        Assertion::ElementAbsent { selector } => {
            let element = observation.find(selector);
            let passed = element.is_none();
            assertion_result(
                assertion,
                passed,
                element.map(|element| element.id.clone()),
                if passed {
                    format!("element was absent: {}", selector.describe())
                } else {
                    format!("element was unexpectedly present: {}", selector.describe())
                },
            )
        }
        Assertion::ElementEnabled { selector, enabled } => {
            let element = observation.find(selector);
            let actual = element.map(|element| element.enabled);
            let passed = actual == Some(*enabled);
            assertion_result(
                assertion,
                passed,
                actual.map(|value| value.to_string()),
                match actual {
                    Some(value) if value == *enabled => {
                        format!("element enabled={value} as expected")
                    }
                    Some(value) => format!("expected enabled={enabled}, observed {value}"),
                    None => format!("element not found: {}", selector.describe()),
                },
            )
        }
        Assertion::ElementValueEquals { selector, value } => {
            let element = observation.find(selector);
            let actual = element.and_then(|element| element.value.clone());
            let passed = actual.as_deref() == Some(value.as_str());
            assertion_result(
                assertion,
                passed,
                actual.clone(),
                match &actual {
                    Some(actual) if actual == value => {
                        format!("element value was {actual}")
                    }
                    Some(actual) => format!("expected value {value}, observed {actual}"),
                    None => format!("element not found or had no value: {}", selector.describe()),
                },
            )
        }
        Assertion::AppForeground { app_id } => {
            let actual = observation.app.as_ref();
            let passed = actual
                .map(|app| app.app_id == *app_id && app.foreground)
                .unwrap_or(false);
            assertion_result(
                assertion,
                passed,
                actual.map(|app| app.app_id.clone()),
                if passed {
                    format!("{app_id} was foreground")
                } else {
                    format!("{app_id} was not foreground")
                },
            )
        }
        Assertion::ScreenChanged { from } => {
            let changed = match from {
                Some(from) if *from != initial.id => observation.id != *from,
                _ => screen_signature(initial) != screen_signature(observation),
            };
            assertion_result(
                assertion,
                changed,
                Some(changed.to_string()),
                if changed {
                    "screen changed".to_string()
                } else {
                    "screen did not change".to_string()
                },
            )
        }
    }
}

fn assertion_result(
    assertion: &Assertion,
    passed: bool,
    observed: Option<String>,
    message: String,
) -> AssertionResult {
    AssertionResult {
        assertion: assertion.clone(),
        passed,
        observed,
        message,
    }
}

fn screen_signature(observation: &Observation) -> Vec<String> {
    let mut signature = vec![observation
        .app
        .as_ref()
        .and_then(|app| app.activity.clone())
        .unwrap_or_default()];
    signature.extend(observation.elements.iter().map(|element| {
        format!(
            "{}={}",
            element.id,
            element.value.clone().unwrap_or_default()
        )
    }));
    signature
}
