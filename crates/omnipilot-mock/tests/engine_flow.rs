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

use omnipilot_core::{
    Action, Assertion, ErrorCode, Selector, Target, TaskId, TaskSpec, TaskStatus,
};
use omnipilot_engine::{
    ExecutionContext, Executor, MemoryEvidenceStore, MemoryTracer, Policy, PolicyConfig, Runtime,
    RuntimeConfig, TaskRecord,
};
use omnipilot_mock::{MockAdapter, DEMO_APP};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

fn context(policy: Policy) -> ExecutionContext {
    ExecutionContext {
        adapter: Arc::new(MockAdapter::new()),
        policy,
        evidence: Arc::new(MemoryEvidenceStore::new()),
        tracer: Arc::new(MemoryTracer::new()),
    }
}

async fn run(spec: TaskSpec) -> omnipilot_core::TaskResult {
    Executor::new(context(Policy::default()))
        .execute(TaskId::new(), spec, CancellationToken::new())
        .await
}

fn tap_missing() -> Action {
    Action::Tap {
        target: Target::Element {
            selector: Selector::Identifier {
                value: "does.not.exist".to_string(),
            },
        },
        times: None,
        duration_ms: None,
    }
}

#[tokio::test]
async fn demo_task_is_completed_and_verified() {
    let result = run(MockAdapter::demo_task()).await;
    assert_eq!(result.status, TaskStatus::Completed);
    assert!(result.verified);
    assert_eq!(result.steps.len(), 3);
    assert_eq!(result.assertions.len(), 2);
    assert!(result.assertions.iter().all(|assertion| assertion.passed));
    assert!(!result.evidence.is_empty());
}

#[tokio::test]
async fn success_without_criteria_is_explicitly_unverified() {
    let mut spec = MockAdapter::demo_task();
    spec.success_criteria.clear();

    let result = run(spec).await;
    assert_eq!(result.status, TaskStatus::Completed);
    assert!(!result.verified);
    assert!(!result.limitations.is_empty());
    assert!(result.summary.contains("unverified"));
}

#[tokio::test]
async fn failed_assertion_is_a_failed_task() {
    let mut spec = MockAdapter::demo_task();
    spec.success_criteria = vec![Assertion::TextPresent {
        text: "This text is not on the screen".to_string(),
    }];

    let result = run(spec).await;
    assert_eq!(result.status, TaskStatus::Failed);
    assert!(!result.verified);
    assert_eq!(
        result.error.as_ref().map(|error| error.code),
        Some(ErrorCode::VerificationFailed)
    );
}

#[tokio::test]
async fn policy_denial_blocks_the_task() {
    let mut spec = TaskSpec::new("launch a forbidden app", omnipilot_core::Platform::Mock);
    spec.steps = vec![Action::LaunchApp {
        app_id: DEMO_APP.to_string(),
    }];
    spec.budgets.max_steps = 5;

    let policy = Policy::new(PolicyConfig {
        allowed_apps: vec!["com.example.someone.else".to_string()],
        ..PolicyConfig::default()
    });
    let result = Executor::new(context(policy))
        .execute(TaskId::new(), spec, CancellationToken::new())
        .await;

    assert_eq!(result.status, TaskStatus::Blocked);
    assert_eq!(
        result.error.as_ref().map(|error| error.code),
        Some(ErrorCode::PolicyDenied)
    );
}

#[tokio::test]
async fn retryable_action_is_attempted_twice() {
    let mut spec = TaskSpec::new("tap a missing element", omnipilot_core::Platform::Mock);
    spec.steps = vec![
        Action::LaunchApp {
            app_id: DEMO_APP.to_string(),
        },
        tap_missing(),
    ];
    spec.budgets.max_steps = 5;
    spec.budgets.max_retries = 1;

    let result = run(spec).await;
    assert_eq!(result.status, TaskStatus::Failed);
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[1].attempts, 2);
    assert_eq!(result.metrics.retries, 1);
}

#[tokio::test]
async fn time_budget_produces_timed_out() {
    let mut spec = TaskSpec::new("wait too long", omnipilot_core::Platform::Mock);
    spec.steps = vec![Action::Wait { duration_ms: 500 }];
    spec.budgets.max_steps = 5;
    spec.budgets.max_duration_ms = 50;

    let result = run(spec).await;
    assert_eq!(result.status, TaskStatus::TimedOut);
    assert_eq!(
        result.error.as_ref().map(|error| error.code),
        Some(ErrorCode::Timeout)
    );
}

#[tokio::test]
async fn cancellation_produces_cancelled() {
    let mut spec = TaskSpec::new("wait until cancelled", omnipilot_core::Platform::Mock);
    spec.steps = vec![Action::Wait { duration_ms: 5_000 }];
    spec.budgets.max_steps = 5;
    spec.budgets.max_duration_ms = 10_000;

    let cancel = CancellationToken::new();
    let task_cancel = cancel.clone();
    let executor = Executor::new(context(Policy::default()));
    let task =
        tokio::spawn(async move { executor.execute(TaskId::new(), spec, task_cancel).await });
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    cancel.cancel();
    let result = task.await.expect("task join");
    assert_eq!(result.status, TaskStatus::Cancelled);
}

#[tokio::test]
async fn persists_task_record_under_evidence_root() {
    let root = std::env::temp_dir().join(format!("omnipilot-task-{}", TaskId::new()));
    let runtime = Runtime::new(
        vec![Arc::new(MockAdapter::new())],
        RuntimeConfig {
            evidence_root: root.clone(),
            policy: Policy::default(),
        },
    );

    let result = runtime.run_task(MockAdapter::demo_task()).await;
    assert!(result.verified);

    let path = root.join(result.task_id.to_string()).join("task.json");
    let bytes = std::fs::read(&path).expect("task record was persisted");
    let record: TaskRecord = serde_json::from_slice(&bytes).expect("task record is valid JSON");
    assert_eq!(record.status, TaskStatus::Completed);
    assert_eq!(
        record.result.as_ref().map(|result| result.task_id),
        Some(result.task_id)
    );

    std::fs::remove_dir_all(&root).expect("temporary task directory was removed");
}
