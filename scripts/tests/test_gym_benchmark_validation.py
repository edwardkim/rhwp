"""Gym benchmark workflow의 제품 CI·릴리즈 비개입 계약."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPO_ROOT / ".github/workflows/gym-release-gate.yml"
RELEASE_WORKFLOWS = (
    REPO_ROOT / ".github/workflows/release-binary.yml",
    REPO_ROOT / ".github/workflows/npm-publish.yml",
)


class GymBenchmarkWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_workflow_identity_is_benchmark_not_release_gate(self) -> None:
        self.assertIn("name: Gym Benchmark Validation", self.workflow)
        self.assertNotIn("name: Gym Release Gate", self.workflow)

    def test_no_push_or_tag_trigger(self) -> None:
        trigger = self.workflow.split("permissions:", maxsplit=1)[0]
        self.assertIn("workflow_dispatch:", trigger)
        self.assertIn("pull_request:", trigger)
        self.assertNotIn("  push:", trigger)
        self.assertNotIn("tags:", trigger)

    def test_pr_trigger_is_scoped_to_gym_changes(self) -> None:
        trigger = self.workflow.split("permissions:", maxsplit=1)[0]
        self.assertIn("'gym/**'", trigger)
        self.assertIn("'scripts/tests/test_gym_*.py'", trigger)
        self.assertNotIn("'src/**'", trigger)

    def test_contract_suite_is_explicit_and_bounded(self) -> None:
        contracts = self.workflow.split("  contracts:", maxsplit=1)[1].split(
            "  full-benchmark:", maxsplit=1
        )[0]
        self.assertIn("scripts.tests.test_gym_benchmark_validation", contracts)
        self.assertIn("scripts.tests.test_gym_coverage", contracts)
        self.assertIn("scripts.tests.test_gym_profiles", contracts)
        self.assertIn("scripts.tests.test_gym_schema", contracts)
        self.assertIn("scripts.tests.test_gym_pack_health", contracts)
        self.assertIn("scripts.tests.test_gym_build_baseline", contracts)
        self.assertIn("scripts.tests.test_gym_discriminate", contracts)
        self.assertIn("scripts.tests.test_gym_trajectory", contracts)
        self.assertIn("scripts.tests.test_gym_authority_ledger", contracts)
        self.assertIn("scripts.tests.test_gym_competitive_bench", contracts)
        self.assertIn("scripts.tests.test_gym_tutorial", contracts)
        self.assertNotIn("unittest discover", contracts)

    def test_full_audits_are_manual_only(self) -> None:
        full = self.workflow.split("  full-benchmark:", maxsplit=1)[1]
        self.assertIn("needs: contracts", full)
        self.assertIn(
            "if: ${{ github.event_name == 'workflow_dispatch' && inputs.mode == 'full' }}",
            full,
        )
        for tool in (
            "authority_ledger.py",
            "oracle_probe.py",
            "build_baseline.py",
            "discriminate.py",
            "trajectory.py",
        ):
            self.assertIn(tool, full)
        self.assertNotIn("release_gate.py", full)
        self.assertNotIn("release_diff.py", full)

    def test_evidence_is_benchmark_scoped(self) -> None:
        self.assertIn("gym-benchmark-evidence", self.workflow)
        self.assertIn("authority-ledger.json", self.workflow)
        self.assertIn("authority_ledger=${authority_exit}", self.workflow)
        self.assertIn("gym-benchmark-validation-${{ github.sha }}", self.workflow)
        self.assertIn("if: always()", self.workflow)
        self.assertIn("if-no-files-found: error", self.workflow)

    def test_release_publishers_do_not_consume_gym(self) -> None:
        for path in RELEASE_WORKFLOWS:
            text = path.read_text(encoding="utf-8")
            with self.subTest(path=path.name):
                self.assertNotIn("gym-certification", text)
                self.assertNotIn("Gym certification", text)
                self.assertNotIn("verify_gym_release_certification.py", text)

    def test_workflow_has_read_only_permissions(self) -> None:
        self.assertIn("permissions:\n  contents: read", self.workflow)
        for permission in ("contents: write", "pull-requests: write", "id-token: write"):
            self.assertNotIn(permission, self.workflow)


if __name__ == "__main__":
    unittest.main()
