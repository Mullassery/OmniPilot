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

//! Durable evidence storage. Screenshots and hierarchy snapshots are content
//! addressed by SHA-256 and referenced from `TaskResult`.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::Utc;
use omnipilot_core::{EvidenceId, EvidenceKind, EvidenceRef, Observation, OmniError, Screenshot};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub trait EvidenceStore: Send + Sync {
    fn put(
        &self,
        kind: EvidenceKind,
        mime: &str,
        bytes: &[u8],
        redacted: bool,
    ) -> Result<EvidenceRef, OmniError>;

    fn get(&self, id: &EvidenceId) -> Result<Vec<u8>, OmniError>;

    fn put_json(
        &self,
        kind: EvidenceKind,
        value: &Value,
        redacted: bool,
    ) -> Result<EvidenceRef, OmniError> {
        let bytes = serde_json::to_vec_pretty(value).map_err(|error| {
            OmniError::internal(format!("could not serialize evidence: {error}"))
        })?;
        self.put(kind, "application/json", &bytes, redacted)
    }

    fn put_screenshot(&self, screenshot: &Screenshot) -> Result<EvidenceRef, OmniError> {
        let bytes = STANDARD
            .decode(screenshot.base64.as_bytes())
            .map_err(|error| {
                OmniError::internal(format!(
                    "adapter returned invalid screenshot base64: {error}"
                ))
            })?;
        self.put(
            EvidenceKind::Screenshot,
            &screenshot.mime,
            &bytes,
            screenshot.redacted,
        )
    }

    fn put_hierarchy(&self, observation: &Observation) -> Result<EvidenceRef, OmniError> {
        let mut copy = observation.clone();
        copy.screenshot = None;
        let value = serde_json::to_value(copy)
            .map_err(|error| OmniError::internal(format!("hierarchy encode failed: {error}")))?;
        self.put_json(EvidenceKind::Hierarchy, &value, false)
    }
}

#[derive(Debug, Clone)]
pub struct FileEvidenceStore {
    root: PathBuf,
}

impl FileEvidenceStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl EvidenceStore for FileEvidenceStore {
    fn put(
        &self,
        kind: EvidenceKind,
        mime: &str,
        bytes: &[u8],
        redacted: bool,
    ) -> Result<EvidenceRef, OmniError> {
        fs::create_dir_all(&self.root)?;
        let id = EvidenceId::new();
        let extension = extension_for(mime);
        let path = self.root.join(format!("{id}.{extension}"));
        fs::write(&path, bytes)?;

        let digest = Sha256::digest(bytes);
        Ok(EvidenceRef {
            id,
            kind,
            mime: mime.to_string(),
            uri: format!("file://{}", path.display()),
            bytes: bytes.len() as u64,
            sha256: Some(hex::encode(digest)),
            created_at: Utc::now(),
            redacted,
        })
    }

    fn get(&self, id: &EvidenceId) -> Result<Vec<u8>, OmniError> {
        let prefix = id.to_string();
        let entries = fs::read_dir(&self.root).map_err(|error| {
            OmniError::new(
                omnipilot_core::ErrorCode::EvidenceUnavailable,
                format!("cannot read evidence directory: {error}"),
            )
        })?;
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with(&prefix) {
                return Ok(fs::read(entry.path())?);
            }
        }
        Err(OmniError::new(
            omnipilot_core::ErrorCode::EvidenceUnavailable,
            format!("evidence {id} was not found"),
        ))
    }
}

#[derive(Debug, Default)]
pub struct MemoryEvidenceStore {
    entries: Mutex<HashMap<EvidenceId, (EvidenceRef, Vec<u8>)>>,
}

impl MemoryEvidenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries
            .lock()
            .map(|entries| entries.len())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl EvidenceStore for MemoryEvidenceStore {
    fn put(
        &self,
        kind: EvidenceKind,
        mime: &str,
        bytes: &[u8],
        redacted: bool,
    ) -> Result<EvidenceRef, OmniError> {
        let id = EvidenceId::new();
        let digest = Sha256::digest(bytes);
        let reference = EvidenceRef {
            id,
            kind,
            mime: mime.to_string(),
            uri: format!("memory://{id}"),
            bytes: bytes.len() as u64,
            sha256: Some(hex::encode(digest)),
            created_at: Utc::now(),
            redacted,
        };
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| OmniError::internal("evidence store lock poisoned"))?;
        entries.insert(id, (reference.clone(), bytes.to_vec()));
        Ok(reference)
    }

    fn get(&self, id: &EvidenceId) -> Result<Vec<u8>, OmniError> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| OmniError::internal("evidence store lock poisoned"))?;
        entries
            .get(id)
            .map(|(_, bytes)| bytes.clone())
            .ok_or_else(|| {
                OmniError::new(
                    omnipilot_core::ErrorCode::EvidenceUnavailable,
                    format!("evidence {id} was not found"),
                )
            })
    }
}

fn extension_for(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "application/json" => "json",
        "text/plain" => "txt",
        _ => "bin",
    }
}
