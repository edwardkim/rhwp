"""#6689 GitHub Actions 증적 collector의 실패-폐쇄 계약."""

from __future__ import annotations

import hashlib
import io
import json
import unittest
import zipfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TARGET = REPO_ROOT / "scripts/workflow_promotion_evidence.py"


def load_target():
    import importlib.util

    spec = importlib.util.spec_from_file_location("workflow_promotion_evidence", TARGET)
    if spec is None or spec.loader is None:
        raise RuntimeError("collector module을 불러올 수 없다")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verdict_zip(verdict: str = "completed", *, padding: int = 0) -> bytes:
    payload = json.dumps({"verdict": verdict, "padding": "x" * padding}).encode()
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("verdict.json", payload)
    return output.getvalue()


class FakeSource:
    def __init__(self, archive: bytes | None = None) -> None:
        self.archive = archive or verdict_zip()
        self.jobs_complete = True
        self.comments = []

    def list_runs(self, candidate_sha: str):
        return ([{
            "id": 42,
            "html_url": "https://github.com/edwardkim/rhwp/actions/runs/42",
            "path": ".github/workflows/oracle-public-advisory.yml",
            "event": "workflow_dispatch",
            "actor": {"login": "edwardkim"},
            "head_sha": candidate_sha,
            "status": "completed",
            "conclusion": "success",
        }], True)

    def list_jobs(self, run_id: int):
        return ([{
            "name": "oracle-public-compare-advisory",
            "status": "completed",
            "conclusion": "success",
        }], self.jobs_complete)

    def list_artifacts(self, run_id: int):
        return ([{
            "id": 91,
            "name": "oracle-public-advisory-verdict",
            "expired": False,
            "size_in_bytes": len(self.archive),
            "digest": f"sha256:{hashlib.sha256(self.archive).hexdigest()}",
        }], True)

    def download_artifact(self, artifact_id: int, *, max_bytes: int):
        if len(self.archive) > max_bytes:
            raise RuntimeError("download limit exceeded")
        return self.archive

    def list_issue_comments(self, issue_number: int):
        return (self.comments, True)


class WorkflowPromotionEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.module = load_target()

    def inventory(self) -> dict:
        return {
            "schemaVersion": 1,
            "repository": "edwardkim/rhwp",
            "candidateSha": "b" * 40,
            "entries": [{
                "path": ".github/workflows/oracle-public-advisory.yml",
                "classification": "executable",
                "after": {"sha256": "c" * 64},
                "executionMode": "direct",
                "allowedActors": ["edwardkim"],
                "requiredVerdictArtifact": {
                    "name": "oracle-public-advisory-verdict",
                    "requiredPath": "verdict.json",
                    "acceptedVerdicts": ["completed"],
                },
            }],
        }

    def test_collects_exact_run_jobs_and_structured_verdict(self) -> None:
        source = FakeSource()
        evidence = self.module.collect_evidence(self.inventory(), source)
        self.assertEqual(evidence["waivers"], [])
        self.assertEqual(len(evidence["runs"]), 1)
        run = evidence["runs"][0]
        self.assertEqual(run["headSha"], "b" * 40)
        self.assertEqual(run["workflowSha256"], "c" * 64)
        self.assertTrue(run["paginationComplete"])
        artifact = run["artifacts"][0]
        self.assertEqual(artifact["verdict"], "completed")
        self.assertEqual(artifact["files"], ["verdict.json"])
        self.assertEqual(artifact["sha256"], hashlib.sha256(source.archive).hexdigest())

    def test_incomplete_job_pagination_stays_visible(self) -> None:
        source = FakeSource()
        source.jobs_complete = False
        evidence = self.module.collect_evidence(self.inventory(), source)
        self.assertFalse(evidence["runs"][0]["paginationComplete"])

    def test_verdict_archive_digest_must_match_api(self) -> None:
        source = FakeSource()
        original = source.list_artifacts

        def tampered(run_id: int):
            artifacts, complete = original(run_id)
            artifacts[0]["digest"] = "sha256:" + "0" * 64
            return artifacts, complete

        source.list_artifacts = tampered
        with self.assertRaisesRegex(self.module.PromotionEvidenceError, "digest"):
            self.module.collect_evidence(self.inventory(), source)

    def test_large_uncompressed_verdict_is_rejected(self) -> None:
        source = FakeSource(verdict_zip(padding=70_000))
        with self.assertRaisesRegex(self.module.PromotionEvidenceError, "verdict.json"):
            self.module.collect_evidence(self.inventory(), source)

    def test_waiver_identity_and_url_come_from_github_comment(self) -> None:
        source = FakeSource()
        source.comments = [{
            "user": {"login": "edwardkim"},
            "html_url": "https://github.com/edwardkim/rhwp/pull/99#issuecomment-7",
            "body": """<!-- rhwp-workflow-promotion-waiver:v1 -->
```json
{"path":".github/workflows/oracle-public-advisory.yml","candidateSha":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","workflowSha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","approvedBy":"forged","url":"https://example.invalid","reason":"runner unavailable","scope":["github-hosted-runner-unavailable"],"expiresAt":"2026-09-06T00:00:00Z"}
```
""",
        }]
        evidence = self.module.collect_evidence(
            self.inventory(),
            source,
            issue_number=99,
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        waiver = evidence["waivers"][0]
        self.assertEqual(waiver["approvedBy"], "edwardkim")
        self.assertEqual(
            waiver["url"],
            "https://github.com/edwardkim/rhwp/pull/99#issuecomment-7",
        )


if __name__ == "__main__":
    unittest.main()
