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

//! In-process runtime that owns devices, task records, cancellation, and
//! evidence directories. CLI and MCP are thin layers over this API.

use crate::evidence::{EvidenceStore, FileEvidenceStore};
use crate::executor::{ExecutionContext, Executor};
use crate::policy::Policy;
use crate::trace::{JsonlTracer, NoopTracer, Tracer};
use chrono::{DateTime, Utc};
use omnipilot_core::{
    Capability, DeviceId, DeviceInfo, ErrorCode, EvidenceId, EvidenceRef, LogEntry, Observation,
    OmniError, TaskId, TaskResult, TaskSpec, TaskStatus,
};
use omnipilot_protocol::DeviceAdapter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub evidence_root: PathBuf,
    pub policy: Policy,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            evidence_root: PathBuf::from("traces"),
            policy: Policy::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub task_id: TaskId,
    pub spec: TaskSpec,
    pub status: TaskStatus,
    pub result: Option<TaskResult>,
    pub trace_uri: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Runtime {
    devices: HashMap<DeviceId, Arc<dyn DeviceAdapter>>,
    tasks: Mutex<HashMap<TaskId, TaskRecord>>,
    cancellations: Mutex<HashMap<TaskId, CancellationToken>>,
    config: RuntimeConfig,
}

impl Runtime {
    pub fn new(adapters: Vec<Arc<dyn DeviceAdapter>>, config: RuntimeConfig) -> Self {
        let devices = adapters
            .into_iter()
            .map(|adapter| (adapter.info().id.clone(), adapter))
            .collect();
        Self {
            devices,
            tasks: Mutex::new(HashMap::new()),
            cancellations: Mutex::new(HashMap::new()),
            config,
        }
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn devices(&self) -> Vec<DeviceInfo> {
        let mut devices: Vec<DeviceInfo> = self
            .devices
            .values()
            .map(|adapter| adapter.info())
            .collect();
        devices.sort_by(|left, right| left.id.cmp(&right.id));
        devices
    }

    pub fn device(&self, id: &DeviceId) -> Option<DeviceInfo> {
        self.devices.get(id).map(|adapter| adapter.info())
    }

    pub async fn capabilities(&self, id: &DeviceId) -> Result<Vec<Capability>, OmniError> {
        let adapter = self.adapter(id)?;
        Ok(adapter.capabilities().await)
    }

    pub async fn observe(
        &self,
        id: &DeviceId,
        include_screenshot: bool,
    ) -> Result<Observation, OmniError> {
        let adapter = self.adapter(id)?;
        adapter.observe(include_screenshot).await
    }

    pub async fn logs(&self, id: &DeviceId, limit: usize) -> Result<Vec<LogEntry>, OmniError> {
        let adapter = self.adapter(id)?;
        adapter.logs(limit).await
    }

    /// Run a task to completion. Used by the CLI, where the caller wants the
    /// final result in one call.
    pub async fn run_task(&self, spec: TaskSpec) -> TaskResult {
        let started_at = Utc::now();
        if let Err(error) = spec.validate() {
            return TaskResult::failed(TaskId::new(), spec.goal, error, started_at);
        }

        let adapter = match self.select_adapter(&spec) {
            Ok(adapter) => adapter,
            Err(error) => {
                return TaskResult::failed(TaskId::new(), spec.goal, error, started_at);
            }
        };

        let task_id = TaskId::new();
        let cancel = CancellationToken::new();
        self.insert_record(TaskRecord {
            task_id,
            spec: spec.clone(),
            status: TaskStatus::Queued,
            result: None,
            trace_uri: None,
            created_at: started_at,
            updated_at: started_at,
        });

        let result = self.execute_task(task_id, spec, adapter, cancel).await;
        self.update_record(&result, None);
        result
    }

    /// Start a task in the background and return its id immediately. Used by
    /// MCP so long tasks do not block the stdio session.
    pub fn spawn_task(self: &Arc<Self>, spec: TaskSpec) -> Result<TaskId, OmniError> {
        spec.validate()?;
        let adapter = self.select_adapter(&spec)?;
        let task_id = TaskId::new();
        let now = Utc::now();
        self.insert_record(TaskRecord {
            task_id,
            spec: spec.clone(),
            status: TaskStatus::Queued,
            result: None,
            trace_uri: None,
            created_at: now,
            updated_at: now,
        });

        let cancel = CancellationToken::new();
        self.cancellations
            .lock()
            .map_err(|_| OmniError::internal("cancellation lock poisoned"))?
            .insert(task_id, cancel.clone());

        let runtime = Arc::clone(self);
        tokio::spawn(async move {
            runtime.set_record_status(task_id, TaskStatus::Running);
            let result = runtime.execute_task(task_id, spec, adapter, cancel).await;
            runtime.update_record(&result, None);
            if let Ok(mut cancellations) = runtime.cancellations.lock() {
                cancellations.remove(&task_id);
            }
        });

        Ok(task_id)
    }

    pub fn task(&self, id: &TaskId) -> Option<TaskRecord> {
        self.tasks
            .lock()
            .ok()
            .and_then(|tasks| tasks.get(id).cloned())
    }

    pub fn tasks(&self) -> Vec<TaskRecord> {
        let mut tasks: Vec<TaskRecord> = self
            .tasks
            .lock()
            .map(|tasks| tasks.values().cloned().collect())
            .unwrap_or_default();
        tasks.sort_by_key(|task| task.created_at);
        tasks
    }

    pub fn cancel_task(&self, id: &TaskId) -> bool {
        self.cancellations
            .lock()
            .ok()
            .and_then(|cancellations| cancellations.get(id).cloned())
            .map(|token| {
                token.cancel();
                true
            })
            .unwrap_or(false)
    }

    pub fn evidence(
        &self,
        task_id: &TaskId,
        evidence_id: &EvidenceId,
    ) -> Result<(EvidenceRef, Vec<u8>), OmniError> {
        let record = self
            .task(task_id)
            .ok_or_else(|| OmniError::invalid(format!("task {task_id} was not found")))?;
        let result = record.result.ok_or_else(|| {
            OmniError::new(
                ErrorCode::EvidenceUnavailable,
                "task has not finished; evidence is not available yet",
            )
        })?;
        let reference = result
            .evidence
            .iter()
            .find(|reference| reference.id == *evidence_id)
            .cloned()
            .ok_or_else(|| {
                OmniError::new(
                    ErrorCode::EvidenceUnavailable,
                    format!("evidence {evidence_id} does not belong to task {task_id}"),
                )
            })?;
        let store = FileEvidenceStore::new(self.task_dir(task_id));
        let bytes = store.get(evidence_id)?;
        Ok((reference, bytes))
    }

    async fn execute_task(
        &self,
        task_id: TaskId,
        spec: TaskSpec,
        adapter: Arc<dyn DeviceAdapter>,
        cancel: CancellationToken,
    ) -> TaskResult {
        let task_dir = self.task_dir(&task_id);
        let trace_path = task_dir.join("events.jsonl");
        let trace_uri = Some(format!("file://{}", trace_path.display()));
        let tracer: Arc<dyn Tracer> = match JsonlTracer::new(&trace_path) {
            Ok(tracer) => Arc::new(tracer),
            Err(error) => {
                tracing::warn!(%error, %task_id, "falling back to no-op tracer");
                Arc::new(NoopTracer)
            }
        };
        let evidence: Arc<dyn EvidenceStore> = Arc::new(FileEvidenceStore::new(&task_dir));
        let context = ExecutionContext {
            adapter,
            policy: self.config.policy.clone(),
            evidence,
            tracer,
        };

        let result = Executor::new(context).execute(task_id, spec, cancel).await;
        self.update_record(&result, trace_uri);
        result
    }

    fn select_adapter(&self, spec: &TaskSpec) -> Result<Arc<dyn DeviceAdapter>, OmniError> {
        if let Some(device_id) = &spec.device_id {
            return self.adapter(device_id);
        }
        self.devices
            .values()
            .find(|adapter| adapter.info().platform == spec.platform)
            .cloned()
            .ok_or_else(|| {
                OmniError::new(
                    ErrorCode::DeviceUnavailable,
                    format!("no {} device is available", spec.platform.as_str()),
                )
            })
    }

    fn adapter(&self, id: &DeviceId) -> Result<Arc<dyn DeviceAdapter>, OmniError> {
        self.devices.get(id).cloned().ok_or_else(|| {
            OmniError::new(
                ErrorCode::DeviceUnavailable,
                format!("device {id} is not connected"),
            )
        })
    }

    fn task_dir(&self, task_id: &TaskId) -> PathBuf {
        self.config.evidence_root.join(task_id.to_string())
    }

    fn insert_record(&self, record: TaskRecord) {
        self.persist_record(&record);
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.insert(record.task_id, record);
        }
    }

    fn set_record_status(&self, task_id: TaskId, status: TaskStatus) {
        let record = self.tasks.lock().ok().and_then(|mut tasks| {
            let record = tasks.get_mut(&task_id)?;
            record.status = status;
            record.updated_at = Utc::now();
            Some(record.clone())
        });
        if let Some(record) = record {
            self.persist_record(&record);
        }
    }

    fn update_record(&self, result: &TaskResult, trace_uri: Option<String>) {
        let record = self.tasks.lock().ok().and_then(|mut tasks| {
            let record = tasks.get_mut(&result.task_id)?;
            record.status = result.status;
            record.result = Some(result.clone());
            record.updated_at = Utc::now();
            if trace_uri.is_some() {
                record.trace_uri = trace_uri;
            }
            Some(record.clone())
        });
        if let Some(record) = record {
            self.persist_record(&record);
        }
    }

    fn persist_record(&self, record: &TaskRecord) {
        let task_dir = self.task_dir(&record.task_id);
        if let Err(error) = std::fs::create_dir_all(&task_dir) {
            tracing::warn!(%error, task_id = %record.task_id, "could not create task directory");
            return;
        }
        let path = task_dir.join("task.json");
        match serde_json::to_vec_pretty(record) {
            Ok(bytes) => {
                if let Err(error) = std::fs::write(&path, bytes) {
                    tracing::warn!(%error, path = %path.display(), "could not persist task record");
                }
            }
            Err(error) => {
                tracing::warn!(%error, task_id = %record.task_id, "could not serialize task record");
            }
        }
    }
}
