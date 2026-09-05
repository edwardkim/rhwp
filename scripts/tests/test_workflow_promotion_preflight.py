"""RED contracts for the #6689 workflow promotion preflight."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from datetime import UTC, datetime
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TARGET = REPO_ROOT / "scripts/workflow_promotion_preflight.py"
POLICY_PATH = REPO_ROOT / "scripts/workflow_promotion_policy.json"
DEPLOY_WORKFLOW = REPO_ROOT / ".github/workflows/deploy-pages.yml"
GYM_WORKFLOW = REPO_ROOT / ".github/workflows/gym-release-gate.yml"
ORACLE_WORKFLOW = REPO_ROOT / ".github/workflows/oracle-public-advisory.yml"
SPEC = importlib.util.spec_from_file_location("workflow_promotion_preflight", TARGET)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"모듈을 불러올 수 없다: {TARGET}")
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
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


def canonical_sha256(value: dict) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


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

    def test_preexisting_unpinned_action_is_not_a_new_policy_violation(self) -> None:
        before = """name: Existing\non: [push]\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: echo before\n"""
        after = before.replace("echo before", "echo after")
        base = self.repo.commit({".github/workflows/existing.yml": before}, "base")
        candidate = self.repo.commit({".github/workflows/existing.yml": after}, "candidate")

        result = MODULE.build_inventory(self.repo.root, base, candidate)
        self.assertEqual(result["policyViolations"], [])

    def test_cli_json_and_markdown_share_the_inventory_identity(self) -> None:
        before = "name: CLI\non: [push]\njobs: {}\n"
        after = before.replace("[push]", "[workflow_dispatch]")
        base = self.repo.commit({".github/workflows/cli.yml": before}, "base")
        candidate = self.repo.commit({".github/workflows/cli.yml": after}, "candidate")
        command = [
            sys.executable,
            str(TARGET),
            "inventory",
            "--repo",
            str(self.repo.root),
            "--base-sha",
            base,
            "--candidate-sha",
            candidate,
        ]
        json_run = subprocess.run(command, check=True, capture_output=True, text=True)
        markdown_run = subprocess.run(
            [*command, "--format", "markdown"],
            check=True,
            capture_output=True,
            text=True,
        )
        inventory = json.loads(json_run.stdout)
        self.assertIn(inventory["inventorySha256"], markdown_run.stdout)
        self.assertIn(".github/workflows/cli.yml", markdown_run.stdout)

    def test_add_delete_and_rename_are_deterministic(self) -> None:
        workflow = "name: A\non: [push]\njobs: {}\n"
        deleted_workflow = "name: Deleted\n" + "\n".join(
            f"delete-key-{index}: alpha-{index}" for index in range(20)
        )
        added_workflow = "name: Added\n" + "\n".join(
            f"added-key-{index}: omega-value-with-a-different-shape-{index}"
            for index in range(20)
        )
        base = self.repo.commit(
            {
                ".github/workflows/delete.yml": deleted_workflow,
                ".github/workflows/old.yml": workflow.replace("A", "Old"),
            },
            "base",
        )
        git(self.repo.root, "mv", ".github/workflows/old.yml", ".github/workflows/new.yml")
        (self.repo.root / ".github/workflows/delete.yml").unlink()
        candidate = self.repo.commit(
            {".github/workflows/add.yml": added_workflow},
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
        inventory = {
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
        inventory["inventorySha256"] = canonical_sha256(inventory)
        return inventory

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
            "actor": "edwardkim",
            "paginationComplete": True,
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

    def test_inventory_digest_and_policy_violations_fail_closed(self) -> None:
        inventory = self.inventory()
        inventory["policyViolations"] = ["actions/checkout@v4"]
        inventory["inventorySha256"] = canonical_sha256(
            {key: value for key, value in inventory.items() if key != "inventorySha256"}
        )
        policy_verdict = MODULE.verify_evidence(
            inventory,
            [self.candidate_run()],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(policy_verdict["ok"])
        self.assertIn("policy-violation:actions/checkout@v4", policy_verdict["errors"])

        inventory["entries"][0]["path"] = ".github/workflows/tampered.yml"
        tampered = MODULE.verify_evidence(
            inventory,
            [],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertIn("invalid-inventory-sha256", tampered["errors"])

    def test_unapproved_event_and_invalid_run_url_fail_closed(self) -> None:
        run = self.candidate_run()
        run["event"] = "schedule"
        run["url"] = "https://example.invalid/run/42"
        verdict = self.verify([run])
        self.assertFalse(verdict["ok"])
        self.assertIn("run-event-not-allowed:schedule", verdict["errors"])
        self.assertIn("invalid-run-url:42", verdict["errors"])

    def test_actor_and_pagination_contract_fail_closed(self) -> None:
        inventory = self.inventory()
        inventory["entries"][0]["allowedActors"] = ["edwardkim"]
        inventory["inventorySha256"] = canonical_sha256(
            {key: value for key, value in inventory.items() if key != "inventorySha256"}
        )
        run = self.candidate_run()
        run["actor"] = "untrusted-user"
        run["paginationComplete"] = False
        verdict = MODULE.verify_evidence(
            inventory,
            [run],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(verdict["ok"])
        self.assertIn("run-actor-not-allowed:untrusted-user", verdict["errors"])
        self.assertIn(
            "incomplete-job-pagination:.github/workflows/fuzz-smoke.yml",
            verdict["errors"],
        )

    def test_malformed_run_id_and_snapshot_shape_fail_closed(self) -> None:
        malformed_run = self.candidate_run()
        malformed_run["id"] = "not-an-id"
        verdict = self.verify([malformed_run])
        self.assertFalse(verdict["ok"])
        self.assertIn("invalid-run-id:not-an-id", verdict["errors"])

        malformed_snapshot = MODULE.verify_evidence(
            self.inventory(),
            {},
            {},
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(malformed_snapshot["ok"])
        self.assertIn("invalid-evidence:runs", malformed_snapshot["errors"])
        self.assertIn("invalid-evidence:waivers", malformed_snapshot["errors"])

    def test_cli_verify_exit_code_follows_verdict(self) -> None:
        inventory = self.inventory()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inventory_path = root / "inventory.json"
            runs_path = root / "runs.json"
            inventory_path.write_text(json.dumps(inventory), encoding="utf-8")
            runs_path.write_text(json.dumps([self.candidate_run()]), encoding="utf-8")
            command = [
                sys.executable,
                str(TARGET),
                "verify",
                "--inventory",
                str(inventory_path),
                "--runs",
                str(runs_path),
                "--now",
                "2026-09-05T02:00:00Z",
                "--trusted-maintainer",
                "edwardkim",
            ]
            accepted = subprocess.run(command, check=False, capture_output=True, text=True)
            self.assertEqual(accepted.returncode, 0, accepted.stdout)
            self.assertTrue(json.loads(accepted.stdout)["ok"])

            stale = self.candidate_run(head_sha="c" * 40)
            runs_path.write_text(json.dumps([stale]), encoding="utf-8")
            rejected = subprocess.run(command, check=False, capture_output=True, text=True)
            self.assertEqual(rejected.returncode, 1, rejected.stdout)
            self.assertFalse(json.loads(rejected.stdout)["ok"])

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

    def test_waiver_cannot_cover_permissions_or_security_surfaces(self) -> None:
        inventory = self.inventory()
        inventory["entries"][0]["riskAxes"] = ["permissions"]
        inventory["inventorySha256"] = canonical_sha256(
            {key: value for key, value in inventory.items() if key != "inventorySha256"}
        )
        waiver = {
            "path": ".github/workflows/fuzz-smoke.yml",
            "candidateSha": self.candidate,
            "workflowSha256": self.workflow_hash,
            "approvedBy": "edwardkim",
            "reason": "runner unavailable",
            "scope": ["github-hosted-runner-unavailable"],
            "expiresAt": "2026-09-06T00:00:00Z",
            "url": "https://github.com/edwardkim/rhwp/issues/6689#issuecomment-2",
        }
        verdict = MODULE.verify_evidence(
            inventory,
            [],
            [waiver],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(verdict["ok"])
        self.assertIn("invalid-waiver:.github/workflows/fuzz-smoke.yml", verdict["errors"])

    def test_deleted_workflow_waiver_is_bound_to_before_hash(self) -> None:
        inventory = self.inventory()
        entry = inventory["entries"][0]
        entry["status"] = "deleted"
        entry["before"] = {"sha256": self.workflow_hash}
        entry["after"] = None
        entry["riskAxes"] = ["trigger"]
        inventory["inventorySha256"] = canonical_sha256(
            {key: value for key, value in inventory.items() if key != "inventorySha256"}
        )
        waiver = {
            "path": ".github/workflows/fuzz-smoke.yml",
            "candidateSha": self.candidate,
            "workflowSha256": self.workflow_hash,
            "approvedBy": "edwardkim",
            "reason": "deleted workflow has no candidate run",
            "scope": ["safe-equivalent-adapter"],
            "expiresAt": "2026-09-06T00:00:00Z",
            "url": "https://github.com/edwardkim/rhwp/issues/6689#issuecomment-3",
        }
        verdict = MODULE.verify_evidence(
            inventory,
            [],
            [waiver],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertTrue(verdict["ok"], verdict["errors"])


class WorkflowPromotionExecutionPolicyTests(unittest.TestCase):
    expected_workflows = {
        ".github/workflows/adapter-diff.yml",
        ".github/workflows/ci.yml",
        ".github/workflows/codeql.yml",
        ".github/workflows/deploy-pages.yml",
        ".github/workflows/gym-release-gate.yml",
        ".github/workflows/oracle-public-advisory.yml",
        ".github/workflows/proptest-roundtrip.yml",
        ".github/workflows/release-binary.yml",
        ".github/workflows/npm-publish.yml",
        ".github/workflows/render-diff.yml",
    }

    @classmethod
    def setUpClass(cls) -> None:
        cls.policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))

    def test_policy_covers_baseline_and_is_bound_to_inventory_hash(self) -> None:
        self.assertEqual(set(self.policy["workflows"]), self.expected_workflows)
        inventory = {
            "schemaVersion": 1,
            "baseSha": "a" * 40,
            "candidateSha": "b" * 40,
            "mergeBase": "a" * 40,
            "entries": [
                {
                    "path": path,
                    "classification": "executable",
                    "after": {"sha256": sha256(path)},
                    "riskAxes": ["trigger"],
                }
                for path in sorted(self.expected_workflows)
            ],
            "policyViolations": [],
        }
        inventory["inventorySha256"] = canonical_sha256(inventory)
        enriched = MODULE.apply_execution_policy(inventory, self.policy)
        self.assertRegex(enriched["policySha256"], r"^[0-9a-f]{64}$")
        self.assertNotEqual(enriched["inventorySha256"], inventory["inventorySha256"])
        self.assertEqual(enriched["repository"], "edwardkim/rhwp")
        for entry in enriched["entries"]:
            self.assertEqual(entry["changedAxes"], entry["riskAxes"])
            self.assertIn(
                entry["executionMode"],
                {"direct", "contracts-only", "verify-only"},
            )
            self.assertTrue(entry["requiredJobs"])
            self.assertIn("requiredSkippedJobs", entry)
            expected_events = (
                ["push", "workflow_dispatch"]
                if entry["path"] == ".github/workflows/oracle-public-advisory.yml"
                else ["workflow_dispatch"]
            )
            self.assertEqual(entry["allowedEvents"], expected_events)
            self.assertEqual(entry["allowedActors"], ["edwardkim"])
        self.assertEqual(enriched["policyViolations"], [])

    def test_missing_policy_and_execution_mode_mismatch_fail_closed(self) -> None:
        inventory = {
            "schemaVersion": 1,
            "baseSha": "a" * 40,
            "candidateSha": "b" * 40,
            "mergeBase": "a" * 40,
            "entries": [
                {
                    "path": ".github/workflows/unknown.yml",
                    "classification": "executable",
                    "after": {"sha256": "c" * 64},
                    "riskAxes": ["trigger"],
                }
            ],
            "policyViolations": [],
        }
        inventory["inventorySha256"] = canonical_sha256(inventory)
        enriched = MODULE.apply_execution_policy(inventory, self.policy)
        self.assertIn(
            "missing-workflow-policy:.github/workflows/unknown.yml",
            enriched["policyViolations"],
        )

        candidate_run = {
            "id": 77,
            "url": "https://github.com/edwardkim/rhwp/actions/runs/77",
            "path": ".github/workflows/deploy-pages.yml",
            "event": "workflow_dispatch",
            "actor": "edwardkim",
            "headSha": "b" * 40,
            "workflowSha256": sha256(".github/workflows/deploy-pages.yml"),
            "executionMode": "direct",
            "paginationComplete": True,
            "status": "completed",
            "conclusion": "success",
            "jobs": [{"name": "Build", "status": "completed", "conclusion": "success"}],
            "artifacts": [{"name": "github-pages"}],
        }
        baseline = {
            **inventory,
            "entries": [
                {
                    "path": ".github/workflows/deploy-pages.yml",
                    "classification": "executable",
                    "after": {"sha256": sha256(".github/workflows/deploy-pages.yml")},
                    "riskAxes": ["trigger"],
                }
            ],
            "policyViolations": [],
        }
        baseline["inventorySha256"] = canonical_sha256(
            {key: value for key, value in baseline.items() if key != "inventorySha256"}
        )
        deploy_inventory = MODULE.apply_execution_policy(baseline, self.policy)
        verdict = MODULE.verify_evidence(
            deploy_inventory,
            [candidate_run],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(verdict["ok"])
        self.assertIn("execution-mode-mismatch:direct:verify-only", verdict["errors"])

    def test_verify_only_requires_deploy_job_to_be_skipped(self) -> None:
        inventory = {
            "schemaVersion": 1,
            "baseSha": "a" * 40,
            "candidateSha": "b" * 40,
            "mergeBase": "a" * 40,
            "entries": [
                {
                    "path": ".github/workflows/deploy-pages.yml",
                    "classification": "executable",
                    "after": {"sha256": "c" * 64},
                    "riskAxes": ["permissions"],
                }
            ],
            "policyViolations": [],
        }
        inventory["inventorySha256"] = canonical_sha256(inventory)
        enriched = MODULE.apply_execution_policy(inventory, self.policy)
        run = {
            "id": 78,
            "url": "https://github.com/edwardkim/rhwp/actions/runs/78",
            "path": ".github/workflows/deploy-pages.yml",
            "event": "workflow_dispatch",
            "actor": "edwardkim",
            "headSha": "b" * 40,
            "workflowSha256": "c" * 64,
            "executionMode": "verify-only",
            "paginationComplete": True,
            "status": "completed",
            "conclusion": "success",
            "jobs": [
                {"name": "Build", "status": "completed", "conclusion": "success"},
                {"name": "Deploy", "status": "completed", "conclusion": "success"},
            ],
            "artifacts": [{"name": "github-pages"}],
        }
        verdict = MODULE.verify_evidence(
            enriched,
            [run],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(verdict["ok"])
        self.assertIn("job-not-skipped:Deploy:success", verdict["errors"])

    def test_pages_manual_run_builds_but_never_deploys(self) -> None:
        workflow = DEPLOY_WORKFLOW.read_text(encoding="utf-8")
        global_permissions = workflow.split("concurrency:", maxsplit=1)[0]
        self.assertIn("permissions:\n  contents: read", global_permissions)
        self.assertNotIn("pages: write", global_permissions)
        self.assertNotIn("id-token: write", global_permissions)
        deploy = workflow.split("  deploy:\n", maxsplit=1)[1]
        self.assertIn(
            "if: ${{ github.event_name == 'push' && github.ref == 'refs/heads/main' }}",
            deploy,
        )
        self.assertIn("permissions:\n      pages: write\n      id-token: write", deploy)

    def test_gym_dispatch_defaults_to_contracts_only_and_actions_are_pinned(self) -> None:
        workflow = GYM_WORKFLOW.read_text(encoding="utf-8")
        trigger = workflow.split("permissions:", maxsplit=1)[0]
        self.assertIn("mode:", trigger)
        self.assertIn("default: contracts", trigger)
        full = workflow.split("  full-benchmark:\n", maxsplit=1)[1]
        self.assertIn("inputs.mode == 'full'", full)
        self.assertNotIn("actions/checkout@v4", workflow)
        self.assertNotIn("dtolnay/rust-toolchain@stable", workflow)

    def test_oracle_emits_a_machine_verdict_artifact(self) -> None:
        workflow = ORACLE_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("id: pack", workflow)
        self.assertIn("- name: Finalize advisory verdict", workflow)
        self.assertIn("oracle-advisory/verdict.json", workflow)
        self.assertIn("name: oracle-public-advisory-verdict", workflow)
        self.assertIn("if-no-files-found: error", workflow)

    def test_oracle_policy_accepts_bootstrap_push_but_rejects_other_events(self) -> None:
        config = self.policy["workflows"][
            ".github/workflows/oracle-public-advisory.yml"
        ]
        self.assertEqual(config["allowedEvents"], ["push", "workflow_dispatch"])
        self.assertNotIn("pull_request", config["allowedEvents"])

    def test_oracle_structured_verdict_must_be_completed(self) -> None:
        inventory = {
            "schemaVersion": 1,
            "baseSha": "a" * 40,
            "candidateSha": "b" * 40,
            "mergeBase": "a" * 40,
            "entries": [
                {
                    "path": ".github/workflows/oracle-public-advisory.yml",
                    "classification": "executable",
                    "after": {"sha256": "c" * 64},
                    "riskAxes": ["job-command"],
                }
            ],
            "policyViolations": [],
        }
        inventory["inventorySha256"] = canonical_sha256(inventory)
        enriched = MODULE.apply_execution_policy(inventory, self.policy)
        run = {
            "id": 88,
            "url": "https://github.com/edwardkim/rhwp/actions/runs/88",
            "path": ".github/workflows/oracle-public-advisory.yml",
            "event": "workflow_dispatch",
            "actor": "edwardkim",
            "headSha": "b" * 40,
            "workflowSha256": "c" * 64,
            "executionMode": "direct",
            "paginationComplete": True,
            "status": "completed",
            "conclusion": "success",
            "jobs": [
                {
                    "name": "oracle-public-compare-advisory",
                    "status": "completed",
                    "conclusion": "success",
                }
            ],
            "artifacts": [
                {
                    "name": "oracle-public-advisory-verdict",
                    "sha256": "d" * 64,
                    "verdict": "completed",
                    "files": ["verdict.json"],
                }
            ],
        }
        accepted = MODULE.verify_evidence(
            enriched,
            [run],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertTrue(accepted["ok"], accepted["errors"])

        run["artifacts"][0]["verdict"] = "skipped"
        rejected = MODULE.verify_evidence(
            enriched,
            [run],
            [],
            now=datetime(2026, 9, 5, 2, 0, tzinfo=UTC),
            trusted_maintainers=frozenset({"edwardkim"}),
        )
        self.assertFalse(rejected["ok"])
        self.assertIn(
            "verdict-not-accepted:oracle-public-advisory-verdict:skipped",
            rejected["errors"],
        )

    def test_release_workflows_require_verify_only_publish_verdict(self) -> None:
        for path in (
            ".github/workflows/release-binary.yml",
            ".github/workflows/npm-publish.yml",
        ):
            with self.subTest(path=path):
                config = self.policy["workflows"][path]
                self.assertEqual(config["executionMode"], "verify-only")
                self.assertIn("Publish channel aggregate", "\n".join(config["requiredJobs"]))
                self.assertIn(
                    "Publish VS Code Marketplace extension",
                    "\n".join(config["requiredSkippedJobs"]),
                )
                self.assertIn("release-publish-evidence", config["requiredArtifacts"])
                self.assertEqual(
                    config["requiredVerdictArtifact"],
                    {
                        "name": "release-publish-evidence",
                        "requiredPath": "release-publish-evidence.json",
                        "acceptedVerdicts": ["completed"],
                    },
                )

        release = self.policy["workflows"][".github/workflows/release-binary.yml"]
        caller_name = "Publish packages after binary release"
        self.assertEqual(
            release["requiredJobs"][5:],
            [
                f"{caller_name} / Validate release source",
                f"{caller_name} / Build WASM",
                f"{caller_name} / Build VSIX once",
                f"{caller_name} / Publish channel aggregate",
            ],
        )
        self.assertEqual(
            release["requiredSkippedJobs"][1:],
            [
                f"{caller_name} / Publish @rhwp/core",
                f"{caller_name} / Publish @rhwp/editor",
                f"{caller_name} / Publish VS Code Marketplace extension",
                f"{caller_name} / Publish Open VSX extension",
            ],
        )


if __name__ == "__main__":
    unittest.main()
