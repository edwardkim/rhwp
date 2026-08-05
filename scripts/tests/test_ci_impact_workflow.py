from __future__ import annotations

import re
import unittest
from pathlib import Path


WORKFLOW_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/ci.yml"
CACHE_SWEEP_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/cache-generation-sweep.yml"
ARCHIVE_RUN_WORKFLOW_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/run-nextest-archives.yml"
WORKER_MARKER = "  # [#2393] 기본 테스트 병렬화"


class CiImpactWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        cls.preflight, cls.workers = cls.workflow.split(WORKER_MARKER, maxsplit=1)

    def _step(self, name: str, source: str | None = None) -> str:
        workflow = source or self.workflow
        step = workflow.split(f"      - name: {name}", maxsplit=1)[1]
        return step.split("\n      - name:", maxsplit=1)[0]

    def _job(self, name: str) -> str:
        match = re.search(
            rf"(?ms)^  {re.escape(name)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)",
            self.workflow,
        )
        self.assertIsNotNone(match, name)
        return match.group(0) if match else ""

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
        self.assertIn("Stage 3 activates frontend_mode and render_required", self.preflight)
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
                    "if: ${{ steps.detect.outputs.fast_pass != 'true' }}",
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

    def test_stage3_consumes_only_frontend_axis(self) -> None:
        self.assertIn("needs.preflight.outputs.frontend_mode", self.workers)
        for deferred_axis in (
            "needs.preflight.outputs.rust_required",
            "needs.preflight.outputs.native_skia_required",
            "needs.preflight.outputs.codeql_languages",
        ):
            with self.subTest(axis=deferred_axis):
                self.assertNotIn(deferred_axis, self.workers)

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

    def test_rust_workers_require_the_frontend_truth_table(self) -> None:
        for job_name in (
            "build-test-archive-slow",
            "build-test-archive-a",
            "build-test-archive-b",
            "native-skia-tests",
        ):
            with self.subTest(job=job_name):
                job = self._job(job_name)
                self.assertIn("frontend-unit-gates", job)
                self.assertIn("frontend-package-gates", job)
                self.assertIn("frontend_mode == 'none'", job)
                self.assertIn("frontend_mode == 'unit'", job)
                self.assertIn("frontend_mode == 'package'", job)

    def test_aggregate_validates_expected_success_and_skipped_states(self) -> None:
        aggregate = self._job("build-and-test")
        self.assertIn("- frontend-unit-gates", aggregate)
        self.assertIn("- frontend-package-gates", aggregate)
        self.assertIn("Frontend none lane expected skipped/skipped", aggregate)
        self.assertIn("Frontend unit lane expected success/skipped", aggregate)
        self.assertIn("Frontend package lane expected skipped/success", aggregate)
        self.assertIn("Unknown frontend mode", aggregate)

    def test_nextest_shards_do_not_wait_for_native_skia(self) -> None:
        for job_name in (
            "test-slow-shard",
            "test-regular-shard-1",
            "test-regular-shard-2",
            "test-regular-shard-3",
        ):
            with self.subTest(job=job_name):
                job = self._job(job_name)
                self.assertNotIn("native-skia-tests", job)
                self.assertIn("actions: read", job)

    def test_cost_model_publication_allows_only_trusted_full_pr_or_devel_push(self) -> None:
        publisher = self._job("publish-nextest-cost-model")
        self.assertIn("github.event_name == 'push'", publisher)
        self.assertIn("github.ref == 'refs/heads/devel'", publisher)
        self.assertIn("github.event_name == 'pull_request'", publisher)
        self.assertIn("github.event.pull_request.head.repo.full_name == github.repository", publisher)
        self.assertIn("github.event.pull_request.author_association", publisher)
        self.assertIn("needs.preflight.outputs.fast_pass != 'true'", publisher)
        self.assertIn("needs['build-and-test'].result == 'success'", publisher)
        self.assertIn("actions: write", publisher)
        self.assertIn("Keep only latest nextest cost model cache", publisher)

    def test_trusted_cost_collection_is_explicit_worker_input(self) -> None:
        worker = ARCHIVE_RUN_WORKFLOW_PATH.read_text(encoding="utf-8")
        self.assertIn("collect_costs:", worker)
        self.assertIn("COLLECT_NEXTEST_COSTS: $" + "{{ inputs.collect_costs", worker)
        self.assertIn("if: $" + "{{ inputs.collect_costs }}", worker)
        self.assertIn('>"$events" 2>"$log" &', worker)
        for job_name in (
            "test-slow-shard",
            "test-regular-shard-1",
            "test-regular-shard-2",
            "test-regular-shard-3",
        ):
            with self.subTest(job=job_name):
                job = self._job(job_name)
                self.assertIn("collect_costs:", job)
                self.assertIn("github.event.pull_request.head.repo.full_name == github.repository", job)
                self.assertIn("github.event.pull_request.author_association", job)
                self.assertIn("needs.preflight.outputs.fast_pass != 'true'", job)

    def test_periodic_cache_sweep_excludes_self_managed_nextest_cost_model(self) -> None:
        sweep = CACHE_SWEEP_PATH.read_text(encoding="utf-8")
        self.assertIn("isSelfManagedNextestCostModel", sweep)
        self.assertIn("key.includes('-nextest-cost-model-v1-')", sweep)
        self.assertIn("if (isSelfManagedNextestCostModel(c.key)) continue;", sweep)

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
