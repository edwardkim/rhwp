from __future__ import annotations

import os
import re
import subprocess
import textwrap
import unittest
from pathlib import Path


WORKFLOW_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/ci.yml"
CLASSIFIER_PATH = Path(__file__).resolve().parents[1] / "ci-impact-classifier.cjs"
WORKER_MARKER = "  # [#2393] 기본 테스트 병렬화"


class CiImpactWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        cls.preflight, cls.workers = cls.workflow.split(WORKER_MARKER, maxsplit=1)

    def _step(self, name: str, source: str | None = None) -> str:
        workflow = source or self.workflow
        step = workflow.split(f"      - name: {name}", maxsplit=1)[1]
        boundary = re.search(r"(?m)^(?:      - name:|  [A-Za-z0-9_-]+:)\s*", step)
        return step[: boundary.start()] if boundary else step

    def _job(self, name: str) -> str:
        match = re.search(
            rf"(?ms)^  {re.escape(name)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)",
            self.workflow,
        )
        self.assertIsNotNone(match, name)
        return match.group(0) if match else ""

    def _run_aggregate(self, **overrides: str) -> subprocess.CompletedProcess[str]:
        step = self._step("Check Build & Test worker results")
        script = textwrap.dedent(step.split("        run: |\n", maxsplit=1)[1])
        env = {
            **os.environ,
            "PREFLIGHT_RESULT": "success",
            "FAST_PASS": "false",
            "RUST_REQUIRED": "false",
            "NATIVE_SKIA_REQUIRED": "false",
            "FRONTEND_MODE": "unit",
            "IMPACT_REASON": "classified:studio-unit",
            "BUILD_SLOW_RESULT": "skipped",
            "BUILD_A_RESULT": "skipped",
            "BUILD_B_RESULT": "skipped",
            "TEST_SLOW_RESULT": "skipped",
            "TEST_REGULAR_1_RESULT": "skipped",
            "TEST_REGULAR_2_RESULT": "skipped",
            "TEST_REGULAR_3_RESULT": "skipped",
            "LINT_RESULT": "skipped",
            "NATIVE_SKIA_RESULT": "skipped",
            "FRONTEND_UNIT_RESULT": "success",
            "FRONTEND_PACKAGE_RESULT": "skipped",
            **overrides,
        }
        return subprocess.run(
            ["bash", "-e", "-o", "pipefail", "-c", script],
            check=False,
            capture_output=True,
            env=env,
            text=True,
        )

    def test_preflight_exposes_every_axis_with_fail_closed_defaults(self) -> None:
        expected_defaults = {
            "rust_required": "'true'",
            "frontend_mode": "'package'",
            "render_required": "'true'",
            "native_skia_required": "'true'",
            "codeql_languages": "'javascript-typescript,python,rust'",
            "classification_status": "'full'",
            "classifier_version": "'unavailable'",
            "impact_reason": "'fail-closed:impact-unavailable'",
            "impact_authority": "'unavailable'",
        }
        for output, default in expected_defaults.items():
            with self.subTest(output=output):
                self.assertIn(f"      {output}:", self.preflight)
                self.assertIn(default, self.preflight)

    def test_classifier_uses_pr_base_sha_without_checkout_credentials(self) -> None:
        step = self._step("Check out trusted CI impact classifier", self.preflight)
        self.assertIn(
            "ref: ${{ github.event_name == 'pull_request' "
            "&& github.event.pull_request.base.sha || github.sha }}",
            step,
        )
        self.assertIn("persist-credentials: false", step)
        self.assertIn("sparse-checkout: scripts/ci-impact-classifier.cjs", step)
        self.assertIn("sparse-checkout-cone-mode: false", step)
        self.assertIn("id: checkout-impact-classifier", step)
        self.assertIn("Classify CI impact", self.preflight)
        self.assertIn(
            "Stage 4 activates frontend_mode, render_required, rust_required, "
            "and native_skia_required",
            self.preflight,
        )
        self.assertIn("pr-base-trusted", self.preflight)
        self.assertNotIn("pr-base-trusted-shadow", self.preflight)

    def test_missing_classifier_checkout_cannot_claim_trusted_authority(self) -> None:
        self.assertIn(
            "const classifierPath = path.join(\n"
            "              workspace,\n"
            "              'scripts',\n"
            "              'ci-impact-classifier.cjs',",
            self.preflight,
        )
        self.assertIn(
            "const checkoutSucceeded = "
            "process.env.CLASSIFIER_CHECKOUT_OUTCOME === 'success'\n"
            "              && fs.existsSync(classifierPath);",
            self.preflight,
        )
        self.assertIn(
            "const authority = !checkoutSucceeded\n"
            "              ? 'unavailable'",
            self.preflight,
        )

    def test_review_only_fast_pass_does_not_pay_classifier_cost(self) -> None:
        for step_name in (
            "Check out trusted CI impact classifier",
            "Collect CI impact input",
            "Classify CI impact",
        ):
            with self.subTest(step=step_name):
                self.assertIn(
                    "if: ${{ steps.finalize.outputs.fast_pass != 'true' }}",
                    self._step(step_name, self.preflight),
                )

    def test_label_events_do_not_restart_ci_and_manual_dispatch_forces_full(self) -> None:
        self.assertIn(
            "types: [opened, reopened, synchronize]",
            self.workflow,
        )
        self.assertNotIn("labeled, unlabeled", self.workflow)
        collect = self._step("Collect CI impact input", self.preflight)
        self.assertNotIn("label.name === 'ci:full'", collect)
        self.assertIn("context.eventName === 'workflow_dispatch'", collect)
        self.assertIn("? 'manual-or-tag'", collect)

    def test_stage4_consumes_frontend_rust_and_native_axes_but_defers_codeql(self) -> None:
        self.assertIn("needs.preflight.outputs.frontend_mode", self.workers)
        for active_axis in (
            "needs.preflight.outputs.rust_required",
            "needs.preflight.outputs.native_skia_required",
        ):
            with self.subTest(axis=active_axis):
                self.assertIn(active_axis, self.workers)
        self.assertNotIn("needs.preflight.outputs.codeql_languages", self.workers)

    def test_unit_and_package_jobs_are_mutually_exclusive(self) -> None:
        unit = self._job("frontend-unit-gates")
        package = self._job("frontend-package-gates")
        self.assertIn("needs.preflight.outputs.frontend_mode == 'unit'", unit)
        self.assertIn("npx tsc --project tsconfig.ci-unit.json --noEmit", unit)
        self.assertIn("npm --prefix rhwp-studio run test", unit)
        self.assertNotIn("wasm-pack build", unit)
        self.assertIn("needs.preflight.outputs.frontend_mode == 'package'", package)
        self.assertIn("wasm-pack build --target web --dev", package)
        self.assertIn("npm --prefix rhwp-studio run test", package)
        self.assertIn("npm --prefix rhwp-studio run build", package)

    def test_rust_lint_and_archive_builders_require_rust_axis(self) -> None:
        lint = self._job("lint")
        self.assertIn("needs.preflight.outputs.rust_required == 'true'", lint)

        for job_name in (
            "build-test-archive-slow",
            "build-test-archive-a",
            "build-test-archive-b",
        ):
            with self.subTest(job=job_name):
                job = self._job(job_name)
                self.assertIn("needs.preflight.outputs.rust_required == 'true'", job)
                self.assertIn("needs.lint.result == 'success'", job)
                self.assertIn("frontend-unit-gates", job)
                self.assertIn("frontend-package-gates", job)
                self.assertIn("frontend_mode == 'none'", job)
                self.assertIn("frontend_mode == 'unit'", job)
                self.assertIn("frontend_mode == 'package'", job)

    def test_native_skia_accepts_expected_lint_state_for_each_rust_lane(self) -> None:
        native = self._job("native-skia-tests")
        self.assertIn("needs.preflight.outputs.native_skia_required == 'true'", native)
        self.assertIn("needs.preflight.outputs.rust_required == 'true'", native)
        self.assertIn("needs.lint.result == 'success'", native)
        self.assertIn("needs.preflight.outputs.rust_required == 'false'", native)
        self.assertIn("needs.lint.result == 'skipped'", native)
        self.assertIn("frontend-unit-gates", native)
        self.assertIn("frontend-package-gates", native)
        self.assertIn("frontend_mode == 'none'", native)
        self.assertIn("frontend_mode == 'unit'", native)
        self.assertIn("frontend_mode == 'package'", native)
        self.assertNotIn("build-test-archive-", native)
        self.assertNotIn("test-regular-shard", native)
        self.assertNotIn("test-slow-shard", native)

    def test_aggregate_harness_stops_at_the_next_job_boundary(self) -> None:
        step = self._step("Check Build & Test worker results")
        script = textwrap.dedent(step.split("        run: |\n", maxsplit=1)[1])
        self.assertNotIn("wasm-build:", script)
        self.assertNotIn("startsWith(github.ref", script)

    def test_native_skia_integration_targets_are_classifier_inputs(self) -> None:
        native_step = self._step("Native Skia tests")
        classifier = CLASSIFIER_PATH.read_text(encoding="utf-8")
        targets = set(re.findall(r"--test ([A-Za-z0-9_]+)", native_step))
        self.assertTrue(targets)
        for target in targets:
            with self.subTest(target=target):
                self.assertIn(f"'tests/{target}.rs'", classifier)

    def test_rust_workers_wait_only_for_their_test_archive(self) -> None:
        expected_archives = {
            "test-slow-shard": "build-test-archive-slow",
            "test-regular-shard-1": "build-test-archive-a",
            "test-regular-shard-2": "build-test-archive-slow",
            "test-regular-shard-3": "build-test-archive-b",
        }
        for job_name, archive in expected_archives.items():
            with self.subTest(job=job_name):
                job = self._job(job_name)
                self.assertIn("needs.preflight.outputs.rust_required == 'true'", job)
                self.assertIn(f"needs: [preflight, {archive}]", job)
                self.assertIn(f"needs['{archive}'].result == 'success'", job)
                self.assertNotIn("native-skia-tests", job)
                self.assertNotIn("native_skia_required", job)

    def test_aggregate_validates_expected_success_and_skipped_states(self) -> None:
        aggregate = self._job("build-and-test")
        self.assertIn("- frontend-unit-gates", aggregate)
        self.assertIn("- frontend-package-gates", aggregate)
        self.assertIn("- native-skia-tests", aggregate)
        self.assertIn("RUST_REQUIRED:", aggregate)
        self.assertIn("NATIVE_SKIA_REQUIRED:", aggregate)
        self.assertIn("Rust lane expected success", aggregate)
        self.assertIn("Rust lane expected skipped", aggregate)
        self.assertIn("Native Skia lane expected success", aggregate)
        self.assertIn("Native Skia lane expected skipped", aggregate)
        self.assertIn("Unknown rust_required", aggregate)
        self.assertIn("Unknown native_skia_required", aggregate)
        self.assertIn("Frontend none lane expected skipped/skipped", aggregate)
        self.assertIn("Frontend unit lane expected success/skipped", aggregate)
        self.assertIn("Frontend package lane expected skipped/success", aggregate)
        self.assertIn("Unknown frontend mode", aggregate)

    def test_shard_count_artifacts_are_downloaded_only_for_rust_lane(self) -> None:
        aggregate = self._job("build-and-test")
        for step_name in (
            "Download shard counts",
            "Download archive expected counts",
            "Verify shard totals",
        ):
            with self.subTest(step=step_name):
                self.assertIn(
                    "needs.preflight.outputs.rust_required == 'true'",
                    self._step(step_name, aggregate),
                )

    def test_aggregate_accepts_every_supported_stage4_lane(self) -> None:
        rust_success = {
            "RUST_REQUIRED": "true",
            "LINT_RESULT": "success",
            "BUILD_SLOW_RESULT": "success",
            "BUILD_A_RESULT": "success",
            "BUILD_B_RESULT": "success",
            "TEST_SLOW_RESULT": "success",
            "TEST_REGULAR_1_RESULT": "success",
            "TEST_REGULAR_2_RESULT": "success",
            "TEST_REGULAR_3_RESULT": "success",
        }
        cases = {
            "frontend-only": {},
            "rust-non-render": {
                **rust_success,
                "FRONTEND_MODE": "none",
                "FRONTEND_UNIT_RESULT": "skipped",
            },
            "rust-render": {
                **rust_success,
                "NATIVE_SKIA_REQUIRED": "true",
                "NATIVE_SKIA_RESULT": "success",
                "FRONTEND_MODE": "none",
                "FRONTEND_UNIT_RESULT": "skipped",
            },
            "non-rust-native-input": {
                "NATIVE_SKIA_REQUIRED": "true",
                "NATIVE_SKIA_RESULT": "success",
                "FRONTEND_MODE": "package",
                "FRONTEND_UNIT_RESULT": "skipped",
                "FRONTEND_PACKAGE_RESULT": "success",
            },
        }
        for name, env in cases.items():
            with self.subTest(lane=name):
                result = self._run_aggregate(**env)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_aggregate_rejects_axis_result_mismatches(self) -> None:
        cases = {
            "unexpected-rust-worker": {"LINT_RESULT": "success"},
            "missing-native-worker": {
                "NATIVE_SKIA_REQUIRED": "true",
                "NATIVE_SKIA_RESULT": "skipped",
            },
            "unexpected-native-worker": {"NATIVE_SKIA_RESULT": "success"},
            "frontend-mismatch": {"FRONTEND_UNIT_RESULT": "skipped"},
            "unknown-rust-axis": {"RUST_REQUIRED": "maybe"},
            "unknown-native-axis": {"NATIVE_SKIA_REQUIRED": "maybe"},
        }
        for name, env in cases.items():
            with self.subTest(lane=name):
                result = self._run_aggregate(**env)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_aggregate_fast_pass_still_accepts_skipped_heavy_jobs(self) -> None:
        result = self._run_aggregate(
            FAST_PASS="true",
            RUST_REQUIRED="true",
            NATIVE_SKIA_REQUIRED="true",
            FRONTEND_MODE="package",
            FRONTEND_UNIT_RESULT="skipped",
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_classifier_failures_remain_fail_closed_without_failing_preflight(self) -> None:
        for step_name in (
            "Check out trusted CI impact classifier",
            "Collect CI impact input",
            "Classify CI impact",
            "Summarize CI impact classification",
        ):
            with self.subTest(step=step_name):
                self.assertIn("continue-on-error: true", self._step(step_name, self.preflight))


if __name__ == "__main__":
    unittest.main()
