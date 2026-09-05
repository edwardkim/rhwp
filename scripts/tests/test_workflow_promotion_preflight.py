"""RED contracts for the #6689 workflow promotion preflight."""

from __future__ import annotations

import hashlib
import importlib.util
import subprocess
import tempfile
import unittest
from datetime import UTC, datetime
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TARGET = REPO_ROOT / "scripts/workflow_promotion_preflight.py"
SPEC = importlib.util.spec_from_file_location("workflow_promotion_preflight", TARGET)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"모듈을 불러올 수 없다: {TARGET}")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def git(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


class WorkflowRepo:
    def __init__(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        git(self.root, "init", "-b", "main")
        git(self.root, "config", "user.name", "Workflow Contract")
        git(self.root, "config", "user.email", "workflow-contract@example.invalid")

    def close(self) -> None:
        self.temp.cleanup()

    def commit(self, files: dict[str, str], message: str) -> str:
        for relative, content in files.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        git(self.root, "add", "-A")
        git(self.root, "commit", "-m", message)
        return git(self.root, "rev-parse", "HEAD")


def inventory_entry(inventory: dict, path: str) -> dict:
    return next(entry for entry in inventory["entries"] if entry["path"] == path)


def sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


class WorkflowPromotionInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = WorkflowRepo()

    def tearDown(self) -> None:
        self.repo.close()

    def test_only_yaml_comments_and_blank_lines_are_comment_only(self) -> None:
        before = """name: Safe\n# old note\non:\n  workflow_dispatch:\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n"""
        after = """name: Safe\n\n# new note with more detail\non:\n  workflow_dispatch:\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok # command note changed\n"""
        base = self.repo.commit({".github/workflows/safe.yml": before}, "base")
        candidate = self.repo.commit({".github/workflows/safe.yml": after}, "comments")

        result = MODULE.build_inventory(self.repo.root, base, candidate)
        entry = inventory_entry(result, ".github/workflows/safe.yml")
        self.assertEqual(entry["classification"], "comment-only")
        self.assertEqual(entry["riskAxes"], [])

    def test_block_scalar_comment_change_remains_executable(self) -> None:
        before = """name: Script\non: [push]\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n          # shell setup v1\n          echo ok\n"""
        after = before.replace("shell setup v1", "shell setup v2")
        base = self.repo.commit({".github/workflows/script.yml": before}, "base")
        candidate = self.repo.commit({".github/workflows/script.yml": after}, "script")

        result = MODULE.build_inventory(self.repo.root, base, candidate)
        entry = inventory_entry(result, ".github/workflows/script.yml")
        self.assertEqual(entry["classification"], "executable")
        self.assertIn("job-command", entry["riskAxes"])

    def test_high_risk_axes_and_unpinned_action_are_reported(self) -> None:
        before = """name: Risk\non:\n  workflow_dispatch:\npermissions:\n  contents: read\nconcurrency:\n  group: risk\njobs:\n  check:\n    timeout-minutes: 5\n    strategy:\n      matrix:\n        target: [a]\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@0123456789012345678901234567890123456789\n      - run: echo ok\n"""
        after = """name: Risk\non:\n  schedule:\n    - cron: '0 0 * * *'\npermissions:\n  contents: write\nconcurrency:\n  group: risk-${{ github.ref }}\njobs:\n  check:\n    timeout-minutes: 30\n    strategy:\n      matrix:\n        target: [a, b]\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/cache@v4\n      - uses: actions/upload-artifact@v4\n      - run: echo ${{ secrets.DEPLOY_TOKEN }}\n"""
        base = self.repo.commit({".github/workflows/risk.yml": before}, "base")
        candidate = self.repo.commit({".github/workflows/risk.yml": after}, "risk")

        result = MODULE.build_inventory(self.repo.root, base, candidate)
        axes = set(inventory_entry(result, ".github/workflows/risk.yml")["riskAxes"])
        self.assertTrue(
            {"trigger", "permissions", "secrets", "matrix", "action-ref", "cache", "artifact", "timeout", "concurrency"}.issubset(axes)
        )
        self.assertIn("actions/checkout@v4", result["policyViolations"])

    def test_add_delete_and_rename_are_deterministic(self) -> None:
        workflow = "name: A\non: [push]\njobs: {}\n"
        base = self.repo.commit(
            {
                ".github/workflows/delete.yml": workflow,
                ".github/workflows/old.yml": workflow.replace("A", "Old"),
            },
            "base",
        )
        git(self.repo.root, "mv", ".github/workflows/old.yml", ".github/workflows/new.yml")
        (self.repo.root / ".github/workflows/delete.yml").unlink()
        candidate = self.repo.commit(
            {".github/workflows/add.yml": workflow.replace("A", "Added")},
            "candidate",
        )

        first = MODULE.build_inventory(self.repo.root, base, candidate)
        second = MODULE.build_inventory(self.repo.root, base, candidate)
        self.assertEqual(first, second)
        self.assertEqual(
            [(entry["status"], entry["path"]) for entry in first["entries"]],
            [
                ("added", ".github/workflows/add.yml"),
                ("deleted", ".github/workflows/delete.yml"),
                ("renamed", ".github/workflows/new.yml"),
            ],
        )
        self.assertEqual(first["entries"][2]["oldPath"], ".github/workflows/old.yml")


class WorkflowPromotionEvidenceTests(unittest.TestCase):
    candidate = "b" * 40
    workflow_text = "name: Fuzz smoke\non: [push]\njobs: {}\n"
    workflow_hash = sha256(workflow_text)

    def inventory(self) -> dict:
        return {
            "schemaVersion": 1,
            "baseSha": "a" * 40,
            "candidateSha": self.candidate,
            "entries": [
                {
                    "path": ".github/workflows/fuzz-smoke.yml",
                    "classification": "executable",
                    "after": {"sha256": self.workflow_hash},
                    "requiredJobs": [
                        "fuzz parse_hwp",
                        "fuzz parse_hwp3",
                        "fuzz parse_hwpx",
                        "fuzz parse_hml",
                        "fuzz parse_wmf",
                        "fuzz parse_ooxml_chart",
                    ],
                }
            ],
        }

    def candidate_run(
        self, *, head_sha: str | None = None, conclusion: str = "success"
    ) -> dict:
        return {
            "id": 42,
            "url": "https://github.com/edwardkim/rhwp/actions/runs/42",
            "path": ".github/workflows/fuzz-smoke.yml",
            "event": "workflow_dispatch",
            "headSha": head_sha or self.candidate,
            "workflowSha256": self.workflow_hash,
            "status": "completed",
            "conclusion": conclusion,
            "jobs": [
                {"name": name, "status": "completed", "conclusion": "success"}
                for name in self.inventory()["entries"][0]["requiredJobs"]
            ],
        }

    def verify(self, runs: list[dict], waivers: list[dict] | None = None) -> dict:
        return MODULE.verify_evidence(
            self.inventory(),
            runs,
            waivers or [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )

    def test_exact_candidate_and_all_six_jobs_pass(self) -> None:
        verdict = self.verify([self.candidate_run()])
        self.assertTrue(verdict["ok"])
        self.assertEqual(verdict["errors"], [])

    def test_legacy_green_general_ci_does_not_replace_fuzz_run(self) -> None:
        general_ci = self.candidate_run()
        general_ci["path"] = ".github/workflows/ci.yml"
        general_ci["jobs"] = [{"name": "Build & Test", "status": "completed", "conclusion": "success"}]
        verdict = self.verify([general_ci])
        self.assertFalse(verdict["ok"])
        self.assertIn("missing-run:.github/workflows/fuzz-smoke.yml", verdict["errors"])

    def test_stale_head_or_workflow_hash_fails_closed(self) -> None:
        stale = self.candidate_run(head_sha="c" * 40)
        stale_hash = self.candidate_run()
        stale_hash["workflowSha256"] = "d" * 64
        verdict = self.verify([stale, stale_hash])
        self.assertFalse(verdict["ok"])
        self.assertIn("no-exact-run:.github/workflows/fuzz-smoke.yml", verdict["errors"])

    def test_missing_or_skipped_required_job_is_not_green(self) -> None:
        run = self.candidate_run()
        run["jobs"][-1]["conclusion"] = "skipped"
        verdict = self.verify([run])
        self.assertFalse(verdict["ok"])
        self.assertIn("job-not-green:fuzz parse_ooxml_chart:skipped", verdict["errors"])

    def test_continue_on_error_requires_verdict_artifact(self) -> None:
        inventory = self.inventory()
        inventory["entries"][0]["requiredVerdictArtifact"] = "oracle-advisory-verdict"
        run = self.candidate_run()
        verdict = MODULE.verify_evidence(
            inventory,
            [run],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(verdict["ok"])
        self.assertIn("missing-verdict-artifact:oracle-advisory-verdict", verdict["errors"])

    def test_waiver_requires_maintainer_exact_scope_and_future_expiry(self) -> None:
        waiver = {
            "path": ".github/workflows/fuzz-smoke.yml",
            "candidateSha": self.candidate,
            "workflowSha256": self.workflow_hash,
            "approvedBy": "edwardkim",
            "reason": "default branch 등록 전 동등 adapter만 실행 가능",
            "scope": ["workflow-dispatch-registration"],
            "expiresAt": "2026-09-06T00:00:00Z",
            "url": "https://github.com/edwardkim/rhwp/issues/6689#issuecomment-1",
        }
        accepted = self.verify([], [waiver])
        self.assertTrue(accepted["ok"])

        waiver["approvedBy"] = "untrusted-user"
        rejected = self.verify([], [waiver])
        self.assertFalse(rejected["ok"])
        self.assertIn("invalid-waiver:.github/workflows/fuzz-smoke.yml", rejected["errors"])


if __name__ == "__main__":
    unittest.main()
