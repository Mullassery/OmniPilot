# Copyright 2026 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0

"""Exceptions raised by the Artemis remote client."""

from __future__ import annotations

from typing import Any


class OmniPilotClientError(Exception):
    """Base class for all errors raised by this package."""


class NetworkError(OmniPilotClientError):
    """The Artemis host could not be reached or the connection failed."""


class ProtocolError(OmniPilotClientError):
    """The server returned a response that does not satisfy the API contract."""


class ApiError(OmniPilotClientError):
    """The Artemis host returned an unsuccessful HTTP response."""

    def __init__(
        self,
        status_code: int,
        detail: str,
        *,
        payload: Any = None,
    ) -> None:
        self.status_code = status_code
        self.detail = detail
        self.payload = payload
        super().__init__(f"Artemis API returned HTTP {status_code}: {detail}")


class AuthenticationError(ApiError):
    """Authentication or authorization failed."""


class NotFoundError(ApiError):
    """The requested Artemis resource does not exist."""


class ConflictError(ApiError):
    """The request conflicts with the current server or device state."""


class TaskRejectedError(OmniPilotClientError):
    """The server accepted the request but rejected task admission."""


class TaskTimeoutError(OmniPilotClientError):
    """Waiting for an Artemis task exceeded the caller's deadline."""

    def __init__(self, task_id: str, timeout: float) -> None:
        self.task_id = task_id
        self.timeout = timeout
        super().__init__(f"Task {task_id} did not finish within {timeout:g} seconds")
