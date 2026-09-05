"""Workflow promotion inventory and evidence contract for issue #6689.

Stage 1 intentionally fixes the public Python interface before implementing it.
The RED tests in ``scripts/tests/test_workflow_promotion_preflight.py`` define
the accepted result envelope and fail-closed behavior. Stage 2 replaces these
sentinels with the implementation; callers must not treat this scaffold as a
working preflight.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping
from datetime import datetime
from pathlib import Path
from typing import Any


class PromotionPolicyError(RuntimeError):
    """The candidate cannot be proven safe for workflow promotion."""


def build_inventory(
    repo_root: Path | str,
    base_sha: str,
    candidate_sha: str,
) -> dict[str, Any]:
    """Return the deterministic workflow/action diff envelope."""

    raise NotImplementedError("Stage 2: workflow promotion inventory")


def verify_evidence(
    inventory: Mapping[str, Any],
    runs: Iterable[Mapping[str, Any]],
    waivers: Iterable[Mapping[str, Any]] = (),
    *,
    now: datetime,
    trusted_maintainers: frozenset[str],
) -> dict[str, Any]:
    """Return a fail-closed verdict for candidate-bound run evidence."""

    raise NotImplementedError("Stage 2: workflow promotion evidence verifier")
