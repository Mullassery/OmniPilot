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

//! Append-only task traces. The JSONL format is intentionally simple so a
//! future `inspect` command can stream it without loading a whole task.

use omnipilot_core::TaskEventRecord;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;

pub trait Tracer: Send + Sync {
    fn record(&self, record: TaskEventRecord);
}

#[derive(Debug, Default)]
pub struct NoopTracer;

impl Tracer for NoopTracer {
    fn record(&self, _record: TaskEventRecord) {}
}

#[derive(Debug, Default)]
pub struct MemoryTracer {
    events: Mutex<Vec<TaskEventRecord>>,
}

impl MemoryTracer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<TaskEventRecord> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }
}

impl Tracer for MemoryTracer {
    fn record(&self, record: TaskEventRecord) {
        if let Ok(mut events) = self.events.lock() {
            events.push(record);
        }
    }
}

pub struct JsonlTracer {
    writer: Mutex<BufWriter<File>>,
}

impl JsonlTracer {
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
        })
    }
}

impl Tracer for JsonlTracer {
    fn record(&self, record: TaskEventRecord) {
        let Ok(mut writer) = self.writer.lock() else {
            return;
        };
        if let Err(error) = serde_json::to_writer(&mut *writer, &record) {
            tracing::warn!(%error, "failed to serialize task event");
            return;
        }
        if let Err(error) = writer.write_all(b"\n").and_then(|_| writer.flush()) {
            tracing::warn!(%error, "failed to write task event");
        }
    }
}
