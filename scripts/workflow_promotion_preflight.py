"""Workflow promotion inventory and evidence verifier for issue #6689.

The inventory deliberately recognizes only a narrow, provable comment-only
subset. Anything that cannot be proven to preserve executable YAML is reported
as executable and therefore needs candidate-bound evidence.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from collections.abc import Iterable, Mapping
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


class PromotionPolicyError(RuntimeError):
    """The candidate cannot be proven safe for workflow promotion."""


@dataclass(frozen=True)
class _SemanticLine:
    text: str
    path: tuple[str, ...]
    block_body: bool = False


_MAPPING_KEY = re.compile(
    r"^\s*(?:-\s+)?(?P<key>[A-Za-z0-9_.-]+)\s*:\s*(?P<value>.*)$"
)
_BLOCK_SCALAR = re.compile(r"^[>|](?:[+-]?[1-9]?|[1-9][+-]?)?$", re.ASCII)
_FULL_SHA = re.compile(r"^[0-9a-fA-F]{40}$", re.ASCII)
_FULL_SHA256 = re.compile(r"^[0-9a-fA-F]{64}$", re.ASCII)
_GITHUB_RUN_URL = re.compile(
    r"^https://github\.com/(?P<owner>[^/]+)/(?P<repo>[^/]+)/actions/runs/"
    r"(?P<id>[1-9][0-9]*)(?:/.*)?$",
    re.ASCII,
)
_DEFAULT_ALLOWED_EVENTS = frozenset({"push", "pull_request", "workflow_dispatch"})
_EXECUTION_MODES = frozenset({"direct", "contracts-only", "verify-only"})

_RISK_ORDER = (
    "trigger",
    "routing",
    "permissions",
    "secrets",
    "matrix",
    "action-ref",
    "cache",
    "artifact",
    "timeout",
    "concurrency",
    "job-command",
    "deployment",
    "security",
    "action-code",
)


def _run_git(repo_root: Path, *args: str, text: bool = False) -> bytes | str:
    try:
        result = subprocess.run(
            ["git", *args],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=text,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        stderr = getattr(error, "stderr", b"")
        if isinstance(stderr, bytes):
            stderr = stderr.decode("utf-8", errors="replace")
        detail = str(stderr).strip() or str(error)
        raise PromotionPolicyError(f"git 명령 실패: {' '.join(args)}: {detail}") from error
    return result.stdout


def _resolve_commit(repo_root: Path, reference: str) -> str:
    value = str(_run_git(repo_root, "rev-parse", f"{reference}^{{commit}}", text=True)).strip()
    if not _FULL_SHA.fullmatch(value):
        raise PromotionPolicyError(f"commit SHA를 확인할 수 없다: {reference}")
    return value.lower()


def _git_blob(repo_root: Path, commit: str, path: str) -> str:
    value = str(_run_git(repo_root, "rev-parse", f"{commit}:{path}", text=True)).strip()
    if not _FULL_SHA.fullmatch(value):
        raise PromotionPolicyError(f"Git blob을 확인할 수 없다: {commit}:{path}")
    return value.lower()


def _git_file(repo_root: Path, commit: str, path: str) -> bytes:
    result = _run_git(repo_root, "show", f"{commit}:{path}")
    if isinstance(result, str):
        return result.encode("utf-8")
    return result


def _strip_inline_yaml_comment(line: str) -> str:
    """Strip a YAML comment only when it is outside quoted scalar text."""

    in_single = False
    in_double = False
    escaped = False
    index = 0
    while index < len(line):
        char = line[index]
        if in_double:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_double = False
        elif in_single:
            if char == "'":
                if index + 1 < len(line) and line[index + 1] == "'":
                    index += 1
                else:
                    in_single = False
        elif char == '"':
            in_double = True
        elif char == "'":
            in_single = True
        elif char == "#" and (index == 0 or line[index - 1].isspace()):
            return line[:index].rstrip()
        index += 1
    return line.rstrip()


def _indent_width(line: str) -> int:
    prefix = line[: len(line) - len(line.lstrip(" \t"))]
    if "\t" in prefix:
        # YAML indentation with tabs is invalid or ambiguous. Keeping each tab
        # visibly wider makes fingerprints conservative instead of erasing it.
        return sum(8 if char == "\t" else 1 for char in prefix)
    return len(prefix)


def _semantic_lines(content: bytes) -> list[_SemanticLine]:
    text = content.decode("utf-8", errors="surrogateescape")
    records: list[_SemanticLine] = []
    stack: list[tuple[int, str]] = []
    block_indent: int | None = None
    block_path: tuple[str, ...] = ()

    for raw_line in text.splitlines():
        if block_indent is not None:
            if not raw_line.strip():
                records.append(_SemanticLine(raw_line.rstrip(), block_path, True))
                continue
            if _indent_width(raw_line) > block_indent:
                records.append(_SemanticLine(raw_line.rstrip(), block_path, True))
                continue
            block_indent = None
            block_path = ()

        line = _strip_inline_yaml_comment(raw_line)
        if not line.strip():
            continue

        indent = _indent_width(line)
        while stack and stack[-1][0] >= indent:
            stack.pop()

        match = _MAPPING_KEY.match(line)
        if match is None:
            path = tuple(key for _, key in stack)
            records.append(_SemanticLine(line, path))
            continue

        key = match.group("key")
        value = match.group("value").strip()
        path = (*[item[1] for item in stack], key)
        records.append(_SemanticLine(line, path))

        if _BLOCK_SCALAR.fullmatch(value):
            block_indent = indent
            block_path = path
        elif value == "":
            stack.append((indent, key))

    return records


def _semantic_fingerprint(content: bytes) -> str:
    return "\n".join(record.text for record in _semantic_lines(content))


def _axis_signature(records: list[_SemanticLine], axis: str) -> list[tuple[tuple[str, ...], str]]:
    def has_key(*keys: str) -> bool:
        return any(key in record.path for key in keys)

    selected: list[tuple[tuple[str, ...], str]] = []
    for record in records:
        lower = record.text.lower()
        include = False
        if axis == "trigger":
            include = bool(record.path and record.path[0] == "on")
        elif axis == "routing":
            include = has_key("paths", "paths-ignore", "branches", "tags", "types", "if") or any(
                marker in lower
                for marker in (
                    "rendercontractpaths",
                    "isallowedreviewpath",
                    "pdfprefixes",
                    "filename.startswith",
                )
            )
        elif axis == "permissions":
            include = has_key("permissions")
        elif axis == "secrets":
            include = has_key("secrets") or "secrets." in lower
        elif axis == "matrix":
            include = has_key("matrix", "strategy")
        elif axis == "action-ref":
            include = bool(record.path and record.path[-1] == "uses")
        elif axis == "cache":
            include = "cache" in lower or has_key("cache")
        elif axis == "artifact":
            include = "artifact" in lower or has_key("artifact", "retention-days")
        elif axis == "timeout":
            include = has_key("timeout-minutes") or "-timeout=" in lower
        elif axis == "concurrency":
            include = has_key("concurrency", "cancel-in-progress")
        elif axis == "job-command":
            include = has_key("run", "script", "shell", "working-directory")
        elif axis == "deployment":
            include = any(
                marker in lower
                for marker in (
                    "pages: write",
                    "id-token: write",
                    "deploy-pages",
                    "npm publish",
                    "gh release",
                    "action-gh-release",
                )
            )
        elif axis == "security":
            include = any(
                marker in lower
                for marker in (
                    "security-events",
                    "codeql",
                    "pull_request_target",
                    "oidc",
                )
            )
        if include:
            selected.append((record.path, record.text))
    return selected


def _risk_axes(
    before: bytes | None,
    after: bytes | None,
    *,
    path: str,
    classification: str,
) -> list[str]:
    if classification == "comment-only":
        return []
    if not path.endswith((".yml", ".yaml")):
        return ["action-code"] if path.startswith(".github/actions/") else []

    before_records = _semantic_lines(before or b"")
    after_records = _semantic_lines(after or b"")
    found = {
        axis
        for axis in _RISK_ORDER
        if axis not in {"security", "action-code"}
        and _axis_signature(before_records, axis) != _axis_signature(after_records, axis)
    }
    if "codeql" in Path(path).name.lower() and classification == "executable":
        found.add("security")
    if path.startswith(".github/actions/"):
        found.add("action-code")
    return [axis for axis in _RISK_ORDER if axis in found]


def _uses_values(content: bytes | None) -> list[str]:
    if content is None:
        return []
    values: list[str] = []
    for record in _semantic_lines(content):
        if not record.path or record.path[-1] != "uses":
            continue
        match = _MAPPING_KEY.match(record.text)
        if match is None:
            continue
        value = match.group("value").strip().strip("'\"")
        if value:
            values.append(value)
    return values


def _unpinned_external_actions(
    before: bytes | None,
    after: bytes | None,
) -> list[str]:
    before_values = Counter(_uses_values(before))
    added_values = Counter(_uses_values(after)) - before_values
    violations: list[str] = []
    for value in sorted(added_values.elements()):
        if value.startswith(("./", "docker://")):
            continue
        if "@" not in value:
            violations.append(value)
            continue
        _, reference = value.rsplit("@", maxsplit=1)
        if not _FULL_SHA.fullmatch(reference):
            violations.append(value)
    return violations


def _parse_name_status(raw: bytes) -> list[tuple[str, str | None, str]]:
    tokens = raw.decode("utf-8", errors="surrogateescape").split("\0")
    if tokens and tokens[-1] == "":
        tokens.pop()
    changes: list[tuple[str, str | None, str]] = []
    index = 0
    while index < len(tokens):
        status_token = tokens[index]
        index += 1
        status = status_token[0] if status_token else ""
        if status in {"R", "C"}:
            if index + 1 >= len(tokens):
                raise PromotionPolicyError("rename/copy diff 출력이 불완전하다")
            old_path = tokens[index]
            new_path = tokens[index + 1]
            index += 2
            changes.append(("renamed" if status == "R" else "copied", old_path, new_path))
            continue
        if index >= len(tokens):
            raise PromotionPolicyError("name-status diff 출력이 불완전하다")
        path = tokens[index]
        index += 1
        names = {"A": "added", "D": "deleted", "M": "modified", "T": "type-changed"}
        changes.append((names.get(status, "unknown"), None, path))
    return changes


def _blob_record(repo_root: Path, commit: str, path: str) -> tuple[dict[str, str], bytes]:
    content = _git_file(repo_root, commit, path)
    return {
        "gitBlob": _git_blob(repo_root, commit, path),
        "sha256": hashlib.sha256(content).hexdigest(),
    }, content


def _canonical_sha256(value: Mapping[str, Any]) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def _run_sort_id(run: Mapping[str, Any]) -> int:
    run_id = run.get("id")
    if isinstance(run_id, int) and not isinstance(run_id, bool) and run_id > 0:
        return run_id
    return -1


def build_inventory(
    repo_root: Path | str,
    base_sha: str,
    candidate_sha: str,
) -> dict[str, Any]:
    """Return the deterministic workflow/action diff envelope."""

    root = Path(repo_root).resolve()
    if not (root / ".git").exists():
        # Worktrees use a .git file, while ordinary repositories use a directory.
        if not (root / ".git").is_file():
            raise PromotionPolicyError(f"Git 저장소가 아니다: {root}")
    base = _resolve_commit(root, base_sha)
    candidate = _resolve_commit(root, candidate_sha)
    merge_base = str(_run_git(root, "merge-base", base, candidate, text=True)).strip().lower()
    raw = _run_git(
        root,
        "diff",
        "--name-status",
        "-z",
        "--find-renames=50%",
        base,
        candidate,
        "--",
        ".github/workflows",
        ".github/actions",
    )
    if isinstance(raw, str):
        raw = raw.encode("utf-8", errors="surrogateescape")

    entries: list[dict[str, Any]] = []
    violations: set[str] = set()
    for status, old_path, path in _parse_name_status(raw):
        before_path = old_path if old_path is not None else path
        before_record: dict[str, str] | None = None
        after_record: dict[str, str] | None = None
        before_content: bytes | None = None
        after_content: bytes | None = None
        if status not in {"added", "copied"}:
            before_record, before_content = _blob_record(root, base, before_path)
        elif status == "copied" and old_path is not None:
            before_record, before_content = _blob_record(root, base, old_path)
        if status != "deleted":
            after_record, after_content = _blob_record(root, candidate, path)

        if (
            status == "modified"
            and path.endswith((".yml", ".yaml"))
            and before_content is not None
            and after_content is not None
            and _semantic_fingerprint(before_content) == _semantic_fingerprint(after_content)
        ):
            classification = "comment-only"
        else:
            classification = "executable"

        entry: dict[str, Any] = {
            "path": path,
            "status": status,
            "classification": classification,
            "before": before_record,
            "after": after_record,
            "riskAxes": _risk_axes(
                before_content,
                after_content,
                path=path,
                classification=classification,
            ),
        }
        if old_path is not None:
            entry["oldPath"] = old_path
        entries.append(entry)
        violations.update(_unpinned_external_actions(before_content, after_content))

    entries.sort(key=lambda item: (item["path"], item.get("oldPath", "")))
    envelope: dict[str, Any] = {
        "schemaVersion": 1,
        "baseSha": base,
        "candidateSha": candidate,
        "mergeBase": merge_base,
        "entries": entries,
        "policyViolations": sorted(violations),
    }
    envelope["inventorySha256"] = _canonical_sha256(envelope)
    return envelope


def _policy_string_list(
    config: Mapping[str, Any],
    field: str,
    *,
    path: str,
    allow_empty: bool = False,
) -> list[str]:
    value = config.get(field)
    if not isinstance(value, list) or (not value and not allow_empty):
        raise PromotionPolicyError(f"invalid-workflow-policy:{path}:{field}")
    normalized = [str(item) for item in value]
    if any(not item for item in normalized) or len(set(normalized)) != len(normalized):
        raise PromotionPolicyError(f"invalid-workflow-policy:{path}:{field}")
    return normalized


def apply_execution_policy(
    inventory: Mapping[str, Any],
    policy: Mapping[str, Any],
) -> dict[str, Any]:
    """Bind execution requirements to an inventory and renew its digest."""

    if policy.get("schemaVersion") != 1:
        raise PromotionPolicyError("invalid-workflow-policy:schema-version")
    repository = str(policy.get("repository", ""))
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise PromotionPolicyError("invalid-workflow-policy:repository")
    workflow_policies = policy.get("workflows")
    if not isinstance(workflow_policies, Mapping):
        raise PromotionPolicyError("invalid-workflow-policy:workflows")

    enriched = copy.deepcopy(dict(inventory))
    entries = enriched.get("entries")
    if not isinstance(entries, list):
        raise PromotionPolicyError("invalid-inventory:entries")
    raw_violations = enriched.get("policyViolations", [])
    if not isinstance(raw_violations, list):
        raise PromotionPolicyError("invalid-inventory:policy-violations")
    violations = {str(item) for item in raw_violations if str(item)}

    for entry in entries:
        if not isinstance(entry, dict):
            raise PromotionPolicyError("invalid-inventory:entry")
        changed_axes = entry.get("riskAxes", [])
        if not isinstance(changed_axes, list):
            raise PromotionPolicyError("invalid-inventory:risk-axes")
        entry["changedAxes"] = list(changed_axes)
        if entry.get("classification") == "comment-only":
            continue

        path = str(entry.get("path", ""))
        config = workflow_policies.get(path)
        if not isinstance(config, Mapping):
            violations.add(f"missing-workflow-policy:{path or '<missing-path>'}")
            continue

        execution_mode = str(config.get("executionMode", ""))
        if execution_mode not in _EXECUTION_MODES:
            raise PromotionPolicyError(f"invalid-workflow-policy:{path}:executionMode")
        evidence_path = str(config.get("evidencePath", path))
        if not evidence_path.startswith(".github/workflows/"):
            raise PromotionPolicyError(f"invalid-workflow-policy:{path}:evidencePath")

        entry["executionMode"] = execution_mode
        entry["evidencePath"] = evidence_path
        entry["sensitiveSurfaces"] = _policy_string_list(
            config,
            "sensitiveSurfaces",
            path=path,
            allow_empty=True,
        )
        entry["requiredJobs"] = _policy_string_list(
            config,
            "requiredJobs",
            path=path,
        )
        entry["requiredSkippedJobs"] = _policy_string_list(
            config,
            "requiredSkippedJobs",
            path=path,
            allow_empty=True,
        )
        entry["requiredArtifacts"] = _policy_string_list(
            config,
            "requiredArtifacts",
            path=path,
            allow_empty=True,
        )
        entry["allowedEvents"] = _policy_string_list(
            config,
            "allowedEvents",
            path=path,
        )
        entry["allowedActors"] = _policy_string_list(
            config,
            "allowedActors",
            path=path,
        )
        required_verdict = config.get("requiredVerdictArtifact")
        if required_verdict is not None:
            if not isinstance(required_verdict, Mapping):
                raise PromotionPolicyError(
                    f"invalid-workflow-policy:{path}:requiredVerdictArtifact"
                )
            name = str(required_verdict.get("name", ""))
            required_path = str(required_verdict.get("requiredPath", ""))
            accepted_verdicts = required_verdict.get("acceptedVerdicts")
            if (
                not name
                or not required_path
                or not isinstance(accepted_verdicts, list)
                or not accepted_verdicts
                or any(not str(item) for item in accepted_verdicts)
            ):
                raise PromotionPolicyError(
                    f"invalid-workflow-policy:{path}:requiredVerdictArtifact"
                )
            entry["requiredVerdictArtifact"] = {
                "name": name,
                "requiredPath": required_path,
                "acceptedVerdicts": [str(item) for item in accepted_verdicts],
            }

    enriched["repository"] = repository
    enriched["policySha256"] = _canonical_sha256(policy)
    enriched["policyViolations"] = sorted(violations)
    enriched.pop("inventorySha256", None)
    enriched["inventorySha256"] = _canonical_sha256(enriched)
    return enriched


def verify_evidence(
    inventory: Mapping[str, Any],
    runs: Iterable[Mapping[str, Any]],
    waivers: Iterable[Mapping[str, Any]] = (),
    *,
    now: datetime,
    trusted_maintainers: frozenset[str],
) -> dict[str, Any]:
    """Return a fail-closed verdict for candidate-bound run evidence."""

    candidate_sha = str(inventory.get("candidateSha", "")).lower()
    inventory_sha = str(inventory.get("inventorySha256", ""))
    repository = str(inventory.get("repository", ""))
    errors: list[str] = []
    accepted_runs: list[dict[str, Any]] = []
    accepted_waivers: list[dict[str, Any]] = []
    run_list: list[dict[str, Any]] = []
    if isinstance(runs, (Mapping, str, bytes)) or not isinstance(runs, Iterable):
        errors.append("invalid-evidence:runs")
    else:
        for index, item in enumerate(runs):
            if isinstance(item, Mapping):
                run_list.append(dict(item))
            else:
                errors.append(f"invalid-run-record:{index}")

    waiver_list: list[dict[str, Any]] = []
    if isinstance(waivers, (Mapping, str, bytes)) or not isinstance(waivers, Iterable):
        errors.append("invalid-evidence:waivers")
    else:
        for index, item in enumerate(waivers):
            if isinstance(item, Mapping):
                waiver_list.append(dict(item))
            else:
                errors.append(f"invalid-waiver-record:{index}")

    if not _FULL_SHA.fullmatch(candidate_sha):
        errors.append("invalid-candidate-sha")
    unsigned_inventory = dict(inventory)
    unsigned_inventory.pop("inventorySha256", None)
    if (
        not _FULL_SHA256.fullmatch(inventory_sha)
        or _canonical_sha256(unsigned_inventory) != inventory_sha.lower()
    ):
        errors.append("invalid-inventory-sha256")

    policy_violations = inventory.get("policyViolations", [])
    if not isinstance(policy_violations, list):
        errors.append("invalid-inventory:policy-violations")
    else:
        errors.extend(
            f"policy-violation:{violation}"
            for violation in map(str, policy_violations)
            if violation
        )
    if now.tzinfo is None:
        errors.append("invalid-now:timezone-required")

    entries = inventory.get("entries")
    if not isinstance(entries, list):
        entries = []
        errors.append("invalid-inventory:entries")
    else:
        errors.extend(
            f"invalid-inventory-entry:{index}"
            for index, entry in enumerate(entries)
            if not isinstance(entry, Mapping)
        )

    for raw_entry in sorted(
        (item for item in entries if isinstance(item, Mapping)),
        key=lambda item: str(item.get("path", "")),
    ):
        if raw_entry.get("classification") == "comment-only":
            continue
        path = str(raw_entry.get("path", ""))
        evidence_path = str(raw_entry.get("evidencePath", path))
        after = raw_entry.get("after")
        before = raw_entry.get("before")
        evidence_blob = after if isinstance(after, Mapping) else before
        expected_hash = (
            str(evidence_blob.get("sha256", ""))
            if isinstance(evidence_blob, Mapping)
            else ""
        )
        if not path or not _FULL_SHA256.fullmatch(expected_hash):
            errors.append(f"invalid-entry:{path or '<missing-path>'}")
            continue

        matching_path = [run for run in run_list if str(run.get("path", "")) == evidence_path]
        exact_runs = [
            run
            for run in matching_path
            if str(run.get("headSha", "")).lower() == candidate_sha
            and str(run.get("workflowSha256", "")).lower() == expected_hash.lower()
        ]
        exact_runs.sort(key=_run_sort_id, reverse=True)

        run_errors: list[str] = []
        accepted_run: Mapping[str, Any] | None = None
        for run in exact_runs:
            current_errors: list[str] = []
            run_id = run.get("id")
            if not isinstance(run_id, int) or isinstance(run_id, bool) or run_id <= 0:
                current_errors.append(f"invalid-run-id:{run_id}")
            run_url = str(run.get("url", ""))
            url_match = _GITHUB_RUN_URL.fullmatch(run_url)
            if url_match is None or str(run_id) != url_match.group("id"):
                current_errors.append(f"invalid-run-url:{run_id}")
            elif repository and (
                f"{url_match.group('owner')}/{url_match.group('repo')}".lower()
                != repository.lower()
            ):
                current_errors.append(f"run-repository-mismatch:{run_id}")

            configured_events = raw_entry.get("allowedEvents")
            if configured_events is None:
                allowed_events = _DEFAULT_ALLOWED_EVENTS
            elif isinstance(configured_events, list) and configured_events:
                allowed_events = frozenset(map(str, configured_events))
            else:
                allowed_events = frozenset()
                current_errors.append(f"invalid-entry-allowed-events:{path}")
            event = str(run.get("event", ""))
            if event not in allowed_events:
                current_errors.append(f"run-event-not-allowed:{event or '<missing>'}")

            expected_mode = raw_entry.get("executionMode")
            if expected_mode is not None:
                observed_mode = str(run.get("executionMode", ""))
                if observed_mode != str(expected_mode):
                    current_errors.append(
                        f"execution-mode-mismatch:{observed_mode or '<missing>'}:{expected_mode}"
                    )

            configured_actors = raw_entry.get("allowedActors")
            if configured_actors is not None:
                if isinstance(configured_actors, list) and configured_actors:
                    actor = str(run.get("actor", ""))
                    if actor not in set(map(str, configured_actors)):
                        current_errors.append(f"run-actor-not-allowed:{actor or '<missing>'}")
                else:
                    current_errors.append(f"invalid-entry-allowed-actors:{path}")

            if run.get("paginationComplete") is not True:
                current_errors.append(f"incomplete-job-pagination:{path}")

            status = str(run.get("status", "missing"))
            conclusion = str(run.get("conclusion", "missing"))
            if status != "completed" or conclusion != "success":
                current_errors.append(f"run-not-green:{path}:{status}:{conclusion}")

            jobs = run.get("jobs")
            job_list = jobs if isinstance(jobs, list) else []
            jobs_by_name = {
                str(job.get("name", "")): job
                for job in job_list
                if isinstance(job, Mapping) and job.get("name")
            }
            for required_job in raw_entry.get("requiredJobs", []):
                name = str(required_job)
                job = jobs_by_name.get(name)
                if job is None:
                    current_errors.append(f"missing-job:{name}")
                    continue
                job_status = str(job.get("status", "missing"))
                job_conclusion = str(job.get("conclusion", "missing"))
                if job_status != "completed" or job_conclusion != "success":
                    current_errors.append(f"job-not-green:{name}:{job_conclusion}")
            for required_skipped_job in raw_entry.get("requiredSkippedJobs", []):
                name = str(required_skipped_job)
                job = jobs_by_name.get(name)
                if job is None:
                    current_errors.append(f"missing-skipped-job:{name}")
                    continue
                job_status = str(job.get("status", "missing"))
                job_conclusion = str(job.get("conclusion", "missing"))
                if job_status != "completed" or job_conclusion != "skipped":
                    current_errors.append(
                        f"job-not-skipped:{name}:{job_conclusion}"
                    )

            artifacts = run.get("artifacts")
            artifact_list = artifacts if isinstance(artifacts, list) else []
            artifacts_by_name = {
                str(item.get("name", "")): item
                for item in artifact_list
                if isinstance(item, Mapping) and item.get("name")
            }
            artifact_names = {
                str(item.get("name", "")) if isinstance(item, Mapping) else str(item)
                for item in artifact_list
            }
            for required_artifact in raw_entry.get("requiredArtifacts", []):
                if str(required_artifact) not in artifact_names:
                    current_errors.append(f"missing-artifact:{required_artifact}")

            required_verdict = raw_entry.get("requiredVerdictArtifact")
            if isinstance(required_verdict, Mapping):
                verdict_name = str(required_verdict.get("name", ""))
                verdict_artifact = artifacts_by_name.get(verdict_name)
                if verdict_artifact is None:
                    current_errors.append(f"missing-verdict-artifact:{verdict_name}")
                else:
                    verdict_sha = str(verdict_artifact.get("sha256", ""))
                    if not _FULL_SHA256.fullmatch(verdict_sha):
                        current_errors.append(
                            f"invalid-verdict-artifact-sha256:{verdict_name}"
                        )
                    verdict_value = str(verdict_artifact.get("verdict", ""))
                    accepted_verdicts = {
                        str(item) for item in required_verdict.get("acceptedVerdicts", [])
                    }
                    if verdict_value not in accepted_verdicts:
                        current_errors.append(
                            f"verdict-not-accepted:{verdict_name}:{verdict_value or '<missing>'}"
                        )
                    required_path = str(required_verdict.get("requiredPath", ""))
                    files = verdict_artifact.get("files")
                    artifact_files = (
                        {str(item) for item in files} if isinstance(files, list) else set()
                    )
                    if required_path not in artifact_files:
                        current_errors.append(
                            f"missing-verdict-file:{verdict_name}:{required_path}"
                        )
            elif required_verdict:
                if str(required_verdict) not in artifact_names:
                    current_errors.append(f"missing-verdict-artifact:{required_verdict}")

            if not current_errors:
                accepted_run = run
                break
            if not run_errors:
                run_errors = current_errors

        if accepted_run is not None:
            accepted_runs.append(
                {
                    "path": path,
                    "id": accepted_run.get("id"),
                    "url": accepted_run.get("url", ""),
                    "executionMode": accepted_run.get("executionMode", ""),
                }
            )
            continue

        if exact_runs:
            errors.extend(run_errors or [f"run-not-green:{path}:unknown"])
            continue

        waiver = next(
            (
                item
                for item in waiver_list
                if str(item.get("path", "")) == path
                and str(item.get("candidateSha", "")).lower() == candidate_sha
                and str(item.get("workflowSha256", "")).lower() == expected_hash.lower()
            ),
            None,
        )
        if waiver is not None and _valid_waiver(
            waiver,
            entry=raw_entry,
            now=now,
            trusted_maintainers=trusted_maintainers,
        ):
            accepted_waivers.append(
                {
                    "path": path,
                    "approvedBy": waiver.get("approvedBy"),
                    "url": waiver.get("url"),
                    "expiresAt": waiver.get("expiresAt"),
                }
            )
            continue

        if waiver is not None:
            errors.append(f"invalid-waiver:{path}")
        errors.append(
            f"missing-run:{path}" if not matching_path else f"no-exact-run:{path}"
        )

    result = {
        "schemaVersion": 1,
        "candidateSha": candidate_sha,
        "inventorySha256": inventory_sha,
        "policySha256": inventory.get("policySha256", ""),
        "ok": not errors,
        "errors": sorted(set(errors)),
        "acceptedRuns": sorted(accepted_runs, key=lambda item: item["path"]),
        "acceptedWaivers": sorted(accepted_waivers, key=lambda item: item["path"]),
    }
    result["verdictSha256"] = _canonical_sha256(result)
    return result


def _parse_timestamp(value: object) -> datetime | None:
    if not isinstance(value, str) or not value:
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    return parsed if parsed.tzinfo is not None else None


def _valid_waiver(
    waiver: Mapping[str, Any],
    *,
    entry: Mapping[str, Any],
    now: datetime,
    trusted_maintainers: frozenset[str],
) -> bool:
    allowed_scopes = {
        "workflow-dispatch-registration",
        "github-hosted-runner-unavailable",
        "safe-equivalent-adapter",
    }
    scopes = waiver.get("scope")
    expiry = _parse_timestamp(waiver.get("expiresAt"))
    protected_surfaces = {
        "permissions",
        "secret",
        "secrets",
        "security",
        "deployment",
    }
    entry_surfaces = {
        str(value)
        for field in ("riskAxes", "sensitiveSurfaces")
        for value in (entry.get(field) if isinstance(entry.get(field), list) else [])
    }
    return bool(
        not entry_surfaces.intersection(protected_surfaces)
        and waiver.get("approvedBy") in trusted_maintainers
        and isinstance(waiver.get("reason"), str)
        and str(waiver.get("reason")).strip()
        and isinstance(scopes, list)
        and scopes
        and set(map(str, scopes)).issubset(allowed_scopes)
        and expiry is not None
        and expiry.astimezone(UTC) > now.astimezone(UTC)
        and isinstance(waiver.get("url"), str)
        and str(waiver.get("url")).startswith("https://github.com/")
    )


def render_inventory_markdown(inventory: Mapping[str, Any]) -> str:
    """Render the inventory without changing its deterministic entry order."""

    lines = [
        "# Workflow promotion inventory",
        "",
        f"- Base SHA: `{inventory.get('baseSha', '')}`",
        f"- Candidate SHA: `{inventory.get('candidateSha', '')}`",
        f"- Merge base: `{inventory.get('mergeBase', '')}`",
        f"- Inventory SHA-256: `{inventory.get('inventorySha256', '')}`",
        f"- Policy SHA-256: `{inventory.get('policySha256', '')}`",
        "",
        "| Status | Classification | Path | Changed axes | Mode | Sensitive surfaces |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for entry in inventory.get("entries", []):
        axes = ", ".join(entry.get("riskAxes", [])) or "none"
        surfaces = ", ".join(entry.get("sensitiveSurfaces", [])) or "none"
        path = str(entry.get("path", ""))
        if entry.get("oldPath"):
            path = f"{entry['oldPath']} -> {path}"
        lines.append(
            f"| {entry.get('status', '')} | {entry.get('classification', '')} | "
            f"`{path}` | {axes} | {entry.get('executionMode', '')} | {surfaces} |"
        )
    violations = inventory.get("policyViolations", [])
    if violations:
        lines.extend(["", "## Policy violations", ""])
        lines.extend(f"- `{value}`" for value in violations)
    return "\n".join(lines) + "\n"


def _write_json(value: Mapping[str, Any]) -> None:
    json.dump(value, sys.stdout, ensure_ascii=False, sort_keys=True, indent=2)
    sys.stdout.write("\n")


def _load_json(path: str) -> Any:
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise PromotionPolicyError(f"JSON을 읽을 수 없다: {path}: {error}") from error


def _main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    inventory_parser = subparsers.add_parser("inventory")
    inventory_parser.add_argument("--repo", default=".")
    inventory_parser.add_argument("--base-sha", required=True)
    inventory_parser.add_argument("--candidate-sha", required=True)
    inventory_parser.add_argument("--policy")
    inventory_parser.add_argument("--format", choices=("json", "markdown"), default="json")

    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("--inventory", required=True)
    verify_parser.add_argument("--runs", required=True)
    verify_parser.add_argument("--waivers")
    verify_parser.add_argument("--now")
    verify_parser.add_argument("--trusted-maintainer", action="append", default=[])

    args = parser.parse_args(argv)
    try:
        if args.command == "inventory":
            result = build_inventory(args.repo, args.base_sha, args.candidate_sha)
            if args.policy:
                policy = _load_json(args.policy)
                if not isinstance(policy, Mapping):
                    raise PromotionPolicyError("workflow policy는 JSON object여야 한다")
                result = apply_execution_policy(result, policy)
            if args.format == "markdown":
                sys.stdout.write(render_inventory_markdown(result))
            else:
                _write_json(result)
            return 0

        inventory = _load_json(args.inventory)
        runs = _load_json(args.runs)
        waivers = _load_json(args.waivers) if args.waivers else []
        observed_now = _parse_timestamp(args.now) if args.now else datetime.now(UTC)
        if observed_now is None:
            raise PromotionPolicyError("--now은 timezone이 포함된 ISO-8601이어야 한다")
        result = verify_evidence(
            inventory,
            runs,
            waivers,
            now=observed_now,
            trusted_maintainers=frozenset(args.trusted_maintainer),
        )
        _write_json(result)
        return 0 if result["ok"] else 1
    except PromotionPolicyError as error:
        _write_json({"schemaVersion": 1, "ok": False, "errors": [str(error)]})
        return 1


if __name__ == "__main__":
    raise SystemExit(_main())
