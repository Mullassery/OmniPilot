# Copyright 2026 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Artemis Diagnostics & System Readiness Package."""

from omnipilot.core.diagnostics.adb_keys import (
    AdbKeyStatus,
    get_adb_key_paths,
    heal_adb_keys,
    inspect_adb_keys,
)
from omnipilot.core.diagnostics.device_smoke import smoke_test_device
from omnipilot.core.diagnostics.emulator_manager import (
    EmulatorLaunchStage,
    EmulatorLaunchState,
    emulator_manager,
)
from omnipilot.core.diagnostics.engine import ReadinessEngine, readiness_engine
from omnipilot.core.diagnostics.probes.adb_probe import AdbDeviceProbe
from omnipilot.core.diagnostics.probes.base import BaseProbe
from omnipilot.core.diagnostics.probes.credentials_probe import (
    LLMCredentialsProbe,
    VisionOCRProbe,
)
from omnipilot.core.diagnostics.probes.host_probe import IntegrationHostProbe
from omnipilot.core.diagnostics.probes.runtime_probe import (
    PythonRuntimeProbe,
    SystemConfigProbe,
)
from omnipilot.core.diagnostics.probes.toolchain_probe import ToolchainProbe
from omnipilot.core.diagnostics.readiness import (
    CHECK_ORDER,
    Verdict,
    adb_keys_corrupted,
    base_verdict,
    collect_readiness,
    sort_by_fix_order,
)
from omnipilot.core.diagnostics.schema import (
    DeviceInfo,
    ProbeAction,
    ProbeCategory,
    ProbeResult,
    ProbeStatus,
    SystemReadinessReport,
)

__all__ = [
    "ReadinessEngine",
    "readiness_engine",
    "emulator_manager",
    "EmulatorLaunchStage",
    "EmulatorLaunchState",
    "BaseProbe",
    "IntegrationHostProbe",
    "PythonRuntimeProbe",
    "SystemConfigProbe",
    "AdbDeviceProbe",
    "LLMCredentialsProbe",
    "VisionOCRProbe",
    "ToolchainProbe",
    "DeviceInfo",
    "ProbeAction",
    "ProbeCategory",
    "ProbeResult",
    "ProbeStatus",
    "SystemReadinessReport",
    "AdbKeyStatus",
    "get_adb_key_paths",
    "inspect_adb_keys",
    "heal_adb_keys",
    "smoke_test_device",
    "CHECK_ORDER",
    "Verdict",
    "collect_readiness",
    "base_verdict",
    "adb_keys_corrupted",
    "sort_by_fix_order",
]
