#!/usr/bin/env python3
"""Collect exact-head GitHub Actions evidence for workflow promotion.

The collector is deliberately read-only. It never dispatches workflows and it
does not interpret a green run as sufficient; the offline verifier remains the
authority for policy decisions.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from collections.abc import Mapping
from pathlib import Path
from typing import Any


MAX_API_BYTES = 10 * 1024 * 1024
MAX_ARTIFACT_BYTES = 1024 * 1024
MAX_VERDICT_BYTES = 64 * 1024
MAX_PAGES = 10
PAGE_SIZE = 100
WAIVER_MARKER = "<!-- rhwp-workflow-promotion-waiver:v1 -->"
WAIVER_PATTERN = re.compile(
    re.escape(WAIVER_MARKER) + r"\s*```json\s*(\{.*?\})\s*```",
    re.DOTALL,
)
FULL_SHA = re.compile(r"^[0-9a-f]{40}$")
FULL_SHA256 = re.compile(r"^[0-9a-f]{64}$")


class PromotionEvidenceError(RuntimeError):
    """Evidence could not be collected without weakening a boundary."""


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file_pointer, code, message, headers, new_url):
        return None


def _read_limited(response, *, max_bytes: int) -> bytes:
    content_length = response.headers.get("Content-Length")
    if content_length is not None:
        try:
            if int(content_length) > max_bytes:
                raise PromotionEvidenceError("response exceeds size limit")
        except ValueError as error:
            raise PromotionEvidenceError("invalid Content-Length") from error
    payload = response.read(max_bytes + 1)
    if len(payload) > max_bytes:
        raise PromotionEvidenceError("response exceeds size limit")
    return payload


class GitHubApiSource:
    """Small GitHub REST client with bounded pagination and downloads."""

    def __init__(self, repository: str, token: str, api_url: str) -> None:
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
            raise PromotionEvidenceError("invalid repository")
        if not token:
            raise PromotionEvidenceError("GitHub token is missing")
        self.repository = repository
        self.api_url = api_url.rstrip("/")
        self.headers = {
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "User-Agent": "rhwp-workflow-promotion-evidence/1",
            "X-GitHub-Api-Version": "2022-11-28",
        }
        self.opener = urllib.request.build_opener(_NoRedirect())

    def _url(self, path: str, query: Mapping[str, object] | None = None) -> str:
        url = f"{self.api_url}/{path.lstrip('/')}"
        if query:
            url = f"{url}?{urllib.parse.urlencode(query)}"
        return url

    def _request(self, path: str, *, max_bytes: int = MAX_API_BYTES) -> bytes:
        request = urllib.request.Request(self._url(path), headers=self.headers)
        try:
            with self.opener.open(request, timeout=30) as response:
                return _read_limited(response, max_bytes=max_bytes)
        except urllib.error.HTTPError as error:
            raise PromotionEvidenceError(
                f"GitHub API request failed: {error.code}: {path}"
            ) from error
        except urllib.error.URLError as error:
            raise PromotionEvidenceError(f"GitHub API request failed: {path}") from error

    def _json(
        self,
        path: str,
        query: Mapping[str, object] | None = None,
    ) -> Any:
        url = self._url(path, query)
        request = urllib.request.Request(url, headers=self.headers)
        try:
            with self.opener.open(request, timeout=30) as response:
                payload = _read_limited(response, max_bytes=MAX_API_BYTES)
        except urllib.error.HTTPError as error:
            raise PromotionEvidenceError(
                f"GitHub API request failed: {error.code}: {path}"
            ) from error
        except urllib.error.URLError as error:
            raise PromotionEvidenceError(f"GitHub API request failed: {path}") from error
        try:
            return json.loads(payload)
        except json.JSONDecodeError as error:
            raise PromotionEvidenceError(f"GitHub API returned invalid JSON: {path}") from error

    def _paginate_object(
        self,
        path: str,
        key: str,
        query: Mapping[str, object] | None = None,
    ) -> tuple[list[dict[str, Any]], bool]:
        items: list[dict[str, Any]] = []
        expected_total: int | None = None
        for page in range(1, MAX_PAGES + 1):
            parameters = dict(query or {})
            parameters.update({"per_page": PAGE_SIZE, "page": page})
            payload = self._json(path, parameters)
            if not isinstance(payload, Mapping):
                raise PromotionEvidenceError(f"invalid paginated payload: {path}")
            total = payload.get("total_count")
            batch = payload.get(key)
            if not isinstance(total, int) or total < 0 or not isinstance(batch, list):
                raise PromotionEvidenceError(f"invalid pagination fields: {path}")
            if expected_total is None:
                expected_total = total
            elif total != expected_total:
                raise PromotionEvidenceError(f"pagination total changed: {path}")
            if any(not isinstance(item, Mapping) for item in batch):
                raise PromotionEvidenceError(f"invalid pagination item: {path}")
            items.extend(dict(item) for item in batch)
            if len(items) >= expected_total:
                return items, len(items) == expected_total
            if not batch:
                return items, False
        return items, False

    def _paginate_list(self, path: str) -> tuple[list[dict[str, Any]], bool]:
        items: list[dict[str, Any]] = []
        for page in range(1, MAX_PAGES + 1):
            payload = self._json(path, {"per_page": PAGE_SIZE, "page": page})
            if not isinstance(payload, list) or any(
                not isinstance(item, Mapping) for item in payload
            ):
                raise PromotionEvidenceError(f"invalid paginated list: {path}")
            items.extend(dict(item) for item in payload)
            if len(payload) < PAGE_SIZE:
                return items, True
        return items, False

    def list_runs(self, candidate_sha: str):
        return self._paginate_object(
            f"repos/{self.repository}/actions/runs",
            "workflow_runs",
            {"head_sha": candidate_sha},
        )

    def list_jobs(self, run_id: int):
        return self._paginate_object(
            f"repos/{self.repository}/actions/runs/{run_id}/jobs",
            "jobs",
            {"filter": "latest"},
        )

    def list_artifacts(self, run_id: int):
        return self._paginate_object(
            f"repos/{self.repository}/actions/runs/{run_id}/artifacts",
            "artifacts",
        )

    def list_issue_comments(self, issue_number: int):
        return self._paginate_list(
            f"repos/{self.repository}/issues/{issue_number}/comments"
        )

    def download_artifact(self, artifact_id: int, *, max_bytes: int) -> bytes:
        path = f"repos/{self.repository}/actions/artifacts/{artifact_id}/zip"
        request = urllib.request.Request(self._url(path), headers=self.headers)
        try:
            with self.opener.open(request, timeout=30) as response:
                return _read_limited(response, max_bytes=max_bytes)
        except urllib.error.HTTPError as error:
            if error.code not in {301, 302, 303, 307, 308}:
                raise PromotionEvidenceError(
                    f"artifact download failed: {error.code}: {artifact_id}"
                ) from error
            location = error.headers.get("Location")
            if not location:
                raise PromotionEvidenceError("artifact redirect has no location") from error
            redirected = urllib.request.Request(
                location,
                headers={"User-Agent": self.headers["User-Agent"]},
            )
            try:
                with urllib.request.urlopen(redirected, timeout=30) as response:
                    return _read_limited(response, max_bytes=max_bytes)
            except (urllib.error.HTTPError, urllib.error.URLError) as redirect_error:
                raise PromotionEvidenceError(
                    f"artifact download failed after redirect: {artifact_id}"
                ) from redirect_error
        except urllib.error.URLError as error:
            raise PromotionEvidenceError(
                f"artifact download failed: {artifact_id}"
            ) from error


def _structured_verdict(
    raw: bytes,
    *,
    required_path: str,
) -> tuple[str, list[str]]:
    try:
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            members = archive.infolist()
            if len(members) > 16:
                raise PromotionEvidenceError("verdict artifact has too many files")
            names = [member.filename for member in members]
            if required_path not in names:
                raise PromotionEvidenceError(
                    f"verdict artifact is missing {required_path}"
                )
            member = archive.getinfo(required_path)
            if member.is_dir() or member.file_size > MAX_VERDICT_BYTES:
                raise PromotionEvidenceError(f"{required_path} exceeds size limit")
            payload = archive.read(member)
    except (zipfile.BadZipFile, KeyError, RuntimeError) as error:
        if isinstance(error, PromotionEvidenceError):
            raise
        raise PromotionEvidenceError("invalid verdict artifact ZIP") from error
    try:
        verdict = json.loads(payload)
    except json.JSONDecodeError as error:
        raise PromotionEvidenceError("verdict artifact contains invalid JSON") from error
    if not isinstance(verdict, Mapping) or not isinstance(verdict.get("verdict"), str):
        raise PromotionEvidenceError("verdict artifact has no string verdict")
    return str(verdict["verdict"]), names


def _collect_waivers(
    comments: list[dict[str, Any]],
    *,
    trusted_maintainers: frozenset[str],
) -> list[dict[str, Any]]:
    waivers: list[dict[str, Any]] = []
    for comment in comments:
        user = comment.get("user")
        author = str(user.get("login", "")) if isinstance(user, Mapping) else ""
        if author not in trusted_maintainers:
            continue
        body = comment.get("body")
        url = comment.get("html_url")
        if not isinstance(body, str) or WAIVER_MARKER not in body:
            continue
        matches = list(WAIVER_PATTERN.finditer(body))
        if not matches:
            raise PromotionEvidenceError("trusted waiver comment is malformed")
        for match in matches:
            try:
                payload = json.loads(match.group(1))
            except json.JSONDecodeError as error:
                raise PromotionEvidenceError("trusted waiver JSON is malformed") from error
            if not isinstance(payload, Mapping):
                raise PromotionEvidenceError("trusted waiver must be a JSON object")
            waiver = {
                key: payload.get(key)
                for key in (
                    "path",
                    "candidateSha",
                    "workflowSha256",
                    "reason",
                    "scope",
                    "expiresAt",
                )
            }
            waiver["approvedBy"] = author
            waiver["url"] = str(url or "")
            waivers.append(waiver)
    return sorted(
        waivers,
        key=lambda item: (str(item.get("path", "")), str(item.get("url", ""))),
    )


def collect_evidence(
    inventory: Mapping[str, Any],
    source,
    *,
    issue_number: int | None = None,
    trusted_maintainers: frozenset[str] = frozenset(),
) -> dict[str, Any]:
    """Collect API records without making a promotion decision."""

    repository = str(inventory.get("repository", ""))
    candidate_sha = str(inventory.get("candidateSha", "")).lower()
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise PromotionEvidenceError("inventory repository is invalid")
    if not FULL_SHA.fullmatch(candidate_sha):
        raise PromotionEvidenceError("inventory candidate SHA is invalid")
    raw_entries = inventory.get("entries")
    if not isinstance(raw_entries, list):
        raise PromotionEvidenceError("inventory entries are invalid")

    entries: dict[str, Mapping[str, Any]] = {}
    for entry in raw_entries:
        if not isinstance(entry, Mapping) or entry.get("classification") == "comment-only":
            continue
        path = str(entry.get("evidencePath", entry.get("path", "")))
        after = entry.get("after")
        before = entry.get("before")
        evidence_blob = after if isinstance(after, Mapping) else before
        workflow_sha = (
            str(evidence_blob.get("sha256", ""))
            if isinstance(evidence_blob, Mapping)
            else ""
        )
        if not path or not FULL_SHA256.fullmatch(workflow_sha):
            raise PromotionEvidenceError(f"executable entry has no evidence hash: {path}")
        entries[path] = entry

    raw_runs, runs_complete = source.list_runs(candidate_sha)
    if not isinstance(raw_runs, list):
        raise PromotionEvidenceError("run listing is invalid")
    collected_runs: list[dict[str, Any]] = []
    for raw_run in raw_runs:
        if not isinstance(raw_run, Mapping):
            raise PromotionEvidenceError("run record is invalid")
        path = str(raw_run.get("path", ""))
        if path not in entries or str(raw_run.get("head_sha", "")).lower() != candidate_sha:
            continue
        run_id = raw_run.get("id")
        if not isinstance(run_id, int) or isinstance(run_id, bool) or run_id <= 0:
            raise PromotionEvidenceError("run ID is invalid")
        jobs, jobs_complete = source.list_jobs(run_id)
        artifacts, artifacts_complete = source.list_artifacts(run_id)
        entry = entries[path]
        required_verdict = entry.get("requiredVerdictArtifact")
        evidence_artifacts: list[dict[str, Any]] = []
        for artifact in sorted(
            artifacts,
            key=lambda item: int(item.get("id", 0)) if isinstance(item, Mapping) else 0,
        ):
            if not isinstance(artifact, Mapping) or not artifact.get("name"):
                raise PromotionEvidenceError("artifact record is invalid")
            record: dict[str, Any] = {"name": str(artifact["name"])}
            if (
                isinstance(required_verdict, Mapping)
                and record["name"] == str(required_verdict.get("name", ""))
                and artifact.get("expired") is not True
            ):
                artifact_id = artifact.get("id")
                size = artifact.get("size_in_bytes")
                digest = str(artifact.get("digest", ""))
                if (
                    not isinstance(artifact_id, int)
                    or not isinstance(size, int)
                    or size < 0
                    or size > MAX_ARTIFACT_BYTES
                    or not digest.startswith("sha256:")
                ):
                    raise PromotionEvidenceError("verdict artifact metadata is invalid")
                raw = source.download_artifact(
                    artifact_id,
                    max_bytes=MAX_ARTIFACT_BYTES,
                )
                actual_digest = hashlib.sha256(raw).hexdigest()
                if actual_digest != digest.removeprefix("sha256:"):
                    raise PromotionEvidenceError("verdict artifact digest mismatch")
                verdict, files = _structured_verdict(
                    raw,
                    required_path=str(required_verdict.get("requiredPath", "")),
                )
                record.update(
                    {"sha256": actual_digest, "verdict": verdict, "files": files}
                )
            evidence_artifacts.append(record)

        actor = raw_run.get("actor")
        collected_runs.append(
            {
                "id": run_id,
                "url": str(raw_run.get("html_url", "")),
                "path": path,
                "event": str(raw_run.get("event", "")),
                "actor": str(actor.get("login", "")) if isinstance(actor, Mapping) else "",
                "headSha": candidate_sha,
                "workflowSha256": str(
                    (entry.get("after") or entry.get("before") or {}).get("sha256", "")
                ),
                "executionMode": str(entry.get("executionMode", "")),
                "paginationComplete": bool(
                    runs_complete and jobs_complete and artifacts_complete
                ),
                "status": str(raw_run.get("status", "")),
                "conclusion": str(raw_run.get("conclusion", "")),
                "jobs": [
                    {
                        "name": str(job.get("name", "")),
                        "status": str(job.get("status", "")),
                        "conclusion": str(job.get("conclusion", "")),
                    }
                    for job in jobs
                    if isinstance(job, Mapping)
                ],
                "artifacts": evidence_artifacts,
            }
        )

    comments: list[dict[str, Any]] = []
    comments_complete = True
    if issue_number is not None:
        comments, comments_complete = source.list_issue_comments(issue_number)
        if not comments_complete:
            raise PromotionEvidenceError("waiver comment pagination is incomplete")
    return {
        "schemaVersion": 1,
        "repository": repository,
        "candidateSha": candidate_sha,
        "runsComplete": bool(runs_complete),
        "commentsComplete": comments_complete,
        "runs": sorted(collected_runs, key=lambda item: (item["path"], item["id"])),
        "waivers": _collect_waivers(
            comments,
            trusted_maintainers=trusted_maintainers,
        ),
    }


def _load_json(path: str) -> Any:
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise PromotionEvidenceError(f"cannot read JSON: {path}") from error


def _write_json(path: str, value: object) -> None:
    Path(path).write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def _main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", required=True)
    parser.add_argument("--runs-output", required=True)
    parser.add_argument("--waivers-output", required=True)
    parser.add_argument("--issue-number", type=int)
    parser.add_argument("--trusted-maintainer", action="append", default=[])
    parser.add_argument("--api-url", default=os.environ.get("GITHUB_API_URL", "https://api.github.com"))
    parser.add_argument("--token-env", default="GITHUB_TOKEN")
    args = parser.parse_args(argv)
    try:
        inventory = _load_json(args.inventory)
        if not isinstance(inventory, Mapping):
            raise PromotionEvidenceError("inventory must be a JSON object")
        token = os.environ.get(args.token_env, "")
        source = GitHubApiSource(str(inventory.get("repository", "")), token, args.api_url)
        evidence = collect_evidence(
            inventory,
            source,
            issue_number=args.issue_number,
            trusted_maintainers=frozenset(args.trusted_maintainer),
        )
        _write_json(args.runs_output, evidence["runs"])
        _write_json(args.waivers_output, evidence["waivers"])
        json.dump(
            {
                "schemaVersion": 1,
                "ok": True,
                "runCount": len(evidence["runs"]),
                "waiverCount": len(evidence["waivers"]),
                "runsComplete": evidence["runsComplete"],
                "commentsComplete": evidence["commentsComplete"],
            },
            sys.stdout,
            ensure_ascii=False,
            sort_keys=True,
        )
        sys.stdout.write("\n")
        return 0
    except PromotionEvidenceError as error:
        json.dump(
            {"schemaVersion": 1, "ok": False, "errors": [str(error)]},
            sys.stdout,
            ensure_ascii=False,
            sort_keys=True,
        )
        sys.stdout.write("\n")
        return 1


if __name__ == "__main__":
    raise SystemExit(_main())
