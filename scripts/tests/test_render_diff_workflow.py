from __future__ import annotations

import hashlib
import importlib.util
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from zipfile import Path as ZipPath, ZipFile


WORKFLOW_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/render-diff.yml"
EVIDENCE_SPEC = importlib.util.spec_from_file_location(
    "render_diff_evidence", Path(__file__).resolve().parents[1] / "render_diff_evidence.py"
)
EVIDENCE = importlib.util.module_from_spec(EVIDENCE_SPEC)
EVIDENCE_SPEC.loader.exec_module(EVIDENCE)


class RenderDiffTriggerPolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_review_records_do_not_trigger_canvas_visual_diff(self) -> None:
        pull_request_trigger = self.workflow.split("  workflow_dispatch:", maxsplit=1)[0]

        self.assertIn("  pull_request:\n", pull_request_trigger)
        self.assertIn("      - 'src/renderer/**'", pull_request_trigger)
        self.assertIn("      - 'rhwp-studio/**'", pull_request_trigger)
        self.assertIn("      - 'scripts/ci-impact-classifier.cjs'", pull_request_trigger)
        self.assertIn("      - 'scripts/generate_exact_face_collection_fixture.py'", pull_request_trigger)
        self.assertIn("      - 'scripts/generate_exact_kerning_fixture.py'", pull_request_trigger)
        self.assertIn(
            "      - 'mydocs/tech/canvaskit-parity-implementation.md'",
            pull_request_trigger,
        )
        self.assertIn("      - 'mydocs/tech/text-ir-v2.md'", pull_request_trigger)
        self.assertNotIn("'mydocs/**'", pull_request_trigger)

    def test_font_registration_contract_precedes_pinned_measurement_browser(self) -> None:
        # 설치 Chrome용 Ubuntu sandbox 경로를 쓰며 측정 Chromium의 pin은 유지한다.
        contract = "      - name: Validate Visual Sweep Canvas font registration\n"
        install = "      - name: Install Chromium\n"
        self.assertEqual(self.workflow.count(contract), 1)
        self.assertLess(self.workflow.index(contract), self.workflow.index(install))
        step = self.workflow.split(contract, 1)[1].split("      - name:", 1)[0]
        self.assertIn(
            "node --test scripts/tests/visual-sweep-font-registration-browser.test.mjs", step
        )
        self.assertNotIn("VISUAL_SWEEP_CHROME", step)
        self.assertNotIn("--no-sandbox", step)
        self.assertIn('chromium@"${RHWP_CHROMIUM_BUILD_ID}"', self.workflow)

    def test_canvas_uses_the_base_classifier_render_axis(self) -> None:
        self.assertIn(
            "ref: ${{ github.event_name == 'pull_request' "
            "&& github.event.pull_request.base.sha || github.sha }}",
            self.workflow,
        )
        self.assertIn("persist-credentials: false", self.workflow)
        self.assertIn("sparse-checkout: scripts/ci-impact-classifier.cjs", self.workflow)
        self.assertIn(
            "render_required: ${{ steps.impact.outputs.render_required || 'true' }}",
            self.workflow,
        )
        self.assertIn(
            "needs.preflight.outputs.render_required == 'true'",
            self.workflow,
        )

    def test_cli_render_boundaries_follow_direct_workflow_consumers(self) -> None:
        pull_request_trigger = self.workflow.split("  workflow_dispatch:", maxsplit=1)[0]

        self.assertIn("      - 'src/cli/document_io.rs'", pull_request_trigger)
        self.assertIn("      - 'src/cli/outputs/mod.rs'", pull_request_trigger)
        self.assertIn("      - 'src/cli/outputs/pdf.rs'", pull_request_trigger)
        self.assertNotIn(
            "      - 'src/cli/commands/caption_validation.rs'",
            pull_request_trigger,
        )
        self.assertNotIn("      - 'src/cli/outputs/raster.rs'", pull_request_trigger)
        self.assertNotIn("      - 'src/cli/outputs/vector.rs'", pull_request_trigger)
        self.assertNotIn("      - 'src/cli/queries/structure.rs'", pull_request_trigger)
        self.assertNotIn("      - 'src/main.rs'", pull_request_trigger)

    def test_label_events_do_not_restart_render_diff_and_manual_dispatch_is_full(self) -> None:
        self.assertIn(
            "types: [opened, reopened, synchronize, ready_for_review, converted_to_draft]",
            self.workflow,
        )
        self.assertNotIn("labeled, unlabeled", self.workflow)
        self.assertNotIn("label.name === 'ci:full'", self.workflow)
        self.assertIn("forceFullReason: 'manual-or-unsupported-event'", self.workflow)

    def test_render_classifier_failures_default_to_full(self) -> None:
        self.assertIn("continue-on-error: true", self.workflow)
        self.assertIn("'fail-closed:impact-unavailable'", self.workflow)
        self.assertIn("forceFullReason: 'collection-error'", self.workflow)

    def test_pdf_raster_timeout_skips_only_the_unavailable_runtime_lane(self) -> None:
        canvas_job = self.workflow.split("  canvas-visual-diff:\n", maxsplit=1)[1]
        install = canvas_job.split("      - name: Install PDF raster tools\n", maxsplit=1)[1].split(
            "      - name: Install wasm-pack\n", maxsplit=1
        )[0]

        self.assertIn("id: pdf-raster-runtime", install)
        self.assertEqual(1, install.count("timeout 180 apt-get"))
        self.assertIn('if [[ "${status}" -eq 124 ]]', install)
        self.assertIn('echo "available=false" >> "${GITHUB_OUTPUT}"', install)
        self.assertIn('echo "available=true" >> "${GITHUB_OUTPUT}"', install)
        self.assertIn("skipping Canvas visual diff", install)
        self.assertIn(
            "steps.pdf-raster-runtime.outputs.available == 'true'",
            canvas_job,
        )


class WasmBuildMeasurementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location(
            "measurement", WORKFLOW_PATH.parents[2] / "scripts/measure_render_diff_wasm.py"
        )
        cls.measurement = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.measurement)

    def test_build_modes_resolve_to_distinct_cache_and_lto_policies(self):
        dev = self.measurement.build_configuration("dev", "auto")
        standard = self.measurement.build_configuration("release", "standard")
        cdylib = self.measurement.build_configuration("release", "cdylib")
        self.assertEqual(dev["cdylib_only"], "0")
        self.assertEqual(standard["crate_types"], "rlib+cdylib")
        self.assertEqual(standard["lto_policy"], "manifest")
        self.assertEqual(cdylib["cdylib_only"], "1")
        self.assertEqual(cdylib["crate_types"], "cdylib")
        self.assertEqual(cdylib["lto_policy"], "override-false")
        self.assertEqual(standard, self.measurement.build_configuration("release", "auto"))
        self.assertEqual(len({item["cache_variant"] for item in (dev, standard, cdylib)}), 3)
        with self.assertRaisesRegex(ValueError, "release"):
            self.measurement.build_configuration("dev", "cdylib")

    def test_cache_restore_cannot_cross_build_modes_and_save_reuses_primary_key(self):
        workflow = WORKFLOW_PATH.read_text()
        restore = workflow.split("      - name: Restore cargo registry & build cache\n")[1].split(
            "      - name: Build WASM package\n")[0]
        prefix = "${{ runner.os }}-ubuntu-24.04-render-diff-cargo-v2-${{ steps.wasm_configuration.outputs.cache_variant }}-"
        self.assertIn(f"key: {prefix}", restore)
        self.assertIn(f"restore-keys: |\n            {prefix}\n", restore)
        self.assertNotIn("${{ runner.os }}-render-diff-cargo-\n", restore)
        self.assertNotIn("${{ runner.os }}-render-diff-cargo-v2-", restore)
        self.assertIn("key: ${{ steps.render_diff_cargo_cache_restore.outputs.cache-primary-key }}", workflow)
        self.assertIn("github.event_name == 'workflow_dispatch' && inputs['wasm-build-mode'] || 'standard'", workflow)
        self.assertIn("github.ref == 'refs/heads/devel' || github.ref == 'refs/heads/main'", workflow)

    def test_release_rejects_missing_optimization_and_failed_build(self):
        with self.assertRaisesRegex(RuntimeError, "wasm-opt"):
            self.measurement.validate("release", 0, {})
        with self.assertRaisesRegex(RuntimeError, "exit code 3"):
            self.measurement.validate("release", 3, {"wasm-opt": ["/bin/wasm-opt"]})
        self.measurement.validate("dev", 0, {})
        self.measurement.validate("release", 0, {"wasm-opt": ["/bin/wasm-opt"]})

    def test_log_boundaries_and_actual_command(self):
        events, commands = {}, {}
        for elapsed, line in [
            (1, "[INFO]: Compiling to Wasm..."),
            (11, "[INFO]: Installing wasm-bindgen..."),
            (14, "[INFO]: Optimizing wasm binaries with `wasm-opt`..."),
            (14.1, '[INFO wasm_pack::child] Running "/cache/wasm-opt" "pkg/rhwp_bg.wasm" "-o" "pkg/tmp.wasm" "-O"'),
            (34, "[INFO]: Done in 34s"),
        ]:
            self.measurement.observe(line, elapsed, events, commands)
        self.measurement.observe('[INFO wasm_pack::child] Running "/cache/wasm-opt" "--version"',
                                 35, events, commands)
        self.assertEqual(commands["wasm-opt"][-1], "-O")
        self.assertEqual(self.measurement.intervals(events, 35), {
            "preparation": 1, "cargo": 10, "bindgen_setup_and_run": 3,
            "wasm_opt_and_finalize": 20, "after_package": 1,
        })
        self.assertTrue(all(value is None for name, value in
                            self.measurement.intervals({}, 1).items()))

    def test_release_workflow_preserves_development_and_deployment_routes(self):
        workflow = WORKFLOW_PATH.read_text()
        self.assertIn('python3 scripts/measure_render_diff_wasm.py --profile "$RHWP_WASM_PROFILE"', workflow)
        self.assertIn("github.event_name == 'workflow_dispatch' && inputs['wasm-profile'] || 'release'", workflow)
        self.assertIn("default: 'release'\n        type: choice", workflow)
        self.assertIn("RHWP_WASM_BUILD_MANIFEST:", workflow)
        self.assertIn("output/render-diff-build/", workflow)
        self.assertIn("'scripts/measure_render_diff_wasm.py'", workflow)
        repo = WORKFLOW_PATH.parents[2]
        ci = (repo / ".github/workflows/ci.yml").read_text()
        self.assertIn("wasm-pack build --target web --dev", ci)
        for name in ("deploy-pages", "npm-publish"):
            self.assertIn("python3 scripts/measure_render_diff_wasm.py --profile release --build-mode standard",
                          (repo / f".github/workflows/{name}.yml").read_text())


class RenderDiffEvidenceTests(unittest.TestCase):
    def index_zip(self, entries, **options):
        buffer = io.BytesIO()
        with ZipFile(buffer, "w") as archive:
            for name, data in entries.items():
                archive.writestr(name, data if isinstance(data, str) else json.dumps(data))
        with ZipFile(buffer) as archive:
            return EVIDENCE.render_index(ZipPath(archive), **options)

    def test_report_roles_use_recorded_results_and_only_measured_pages(self):
        canvas = {"results": [{"fixture": "a.hwp", "pageIndex": 0, "pass": False}]}
        index = self.index_zip({
            f"{EVIDENCE.DIFF}/results.json": canvas,
            f"{EVIDENCE.DIFF}/pdf-results.json": {
                "config": {"directPdfEnabled": True, "directPdfGate": True},
                "results": [
                    {"fixture": "a.hwp", "pageIndex": 0, "exceedsReportThreshold": True,
                     "reportComparison": {"diffRatio": 0.9}, "directExpected": True,
                     "directComparison": {"diffRatio": 0.01}, "directGateFailed": False},
                    {"fixture": "missing.hwp", "pageIndex": 0, "reportError": "load-fixture",
                     "directExpected": True, "directError": "load-fixture"},
                ],
            },
            f"{EVIDENCE.READINESS}/browser/browser-baseline-report.json": {
                "canvaskitReadinessGate": {"summary": {"passed": 1, "failed": 0, "missing": 1}},
                "results": [{"file": "selected.hwp", "page": 2, "readinessGateRequired": True},
                            {"file": "reference.hwp", "page": 2, "readinessGateRequired": False}],
            },
            f"{EVIDENCE.BUILD}/build.json": {"source_sha": "built", "artifacts": {
                "rhwp_bg.wasm": {"sha256": "built-wasm"},
            }},
            f"{EVIDENCE.BUILD}/consumption/browser.json": {"matched": False},
        }, pr_head="submitted")
        self.assertIn("Render Diff gate | 0 passed, 1 failed", index)
        self.assertIn("Reference | 1 warnings, 1 errors", index)
        self.assertIn("Render Diff gate | 1/2 compared, 1 failures", index)
        self.assertIn("Readiness gate | 1 passed, 0 failed, 1 missing | <code>selected.hwp</code> p3", index)
        self.assertNotIn("missing.hwp", index)
        self.assertNotIn("reference.hwp", index)
        canvas_digest = hashlib.sha256(json.dumps(canvas).encode()).hexdigest()
        self.assertIn(f"results.json</code> <code>{canvas_digest}", index)
        self.assertIn("PR head <code>submitted", index)
        self.assertIn("WASM build source <code>built", index)
        self.assertIn("Browser WASM consumption (mismatched)", index)
        self.assertIn("Native PDF build record Unavailable", index)
        self.assertNotIn("silhouette.tsv", index)

    def test_machine_index_and_actions_share_gate_and_reference_results(self):
        entries = {
            f"{EVIDENCE.DIFF}/pdf-results.json": {
                "config": {"directPdfEnabled": True, "directPdfGate": True, "directMaxDiffRatio": 0.02},
                "results": [{"fixture": "a.hwp", "pageIndex": 0, "directExpected": True,
                             "exceedsReportThreshold": True, "reportComparison": {"diffRatio": 0.56},
                             "directGateFailed": True, "directComparison": {"diffRatio": 0.0241}}]},
            f"{EVIDENCE.READINESS}/browser/browser-baseline-report.json": {
                "canvaskitReadinessGate": {"summary": {"total": 2, "evaluated": 1, "passed": 1,
                                                       "failed": 0, "missing": 1},
                                          "checks": [{"sampleId": "A", "activeBackend": "canvaskit"}]}},
            f"{EVIDENCE.READINESS}/baseline-report.json": {
                "manifest": {"samples": [{"id": k, "canvaskitReadinessGate": True} for k in ("A","B")]}}
        }
        buffer = io.BytesIO()
        with ZipFile(buffer, "w") as archive:
            for name, value in entries.items():
                archive.writestr(name, json.dumps(value))
        with ZipFile(buffer) as archive:
            index, markdown = EVIDENCE.build_index(ZipPath(archive),
                run_url="https://github.com/example/test/actions/runs/12/attempts/2",
                pr_head="a"*40, tested_sha="b"*40)
        self.assertEqual((index["schema_version"], index["run_id"], index["run_attempt"]), (1,12,2))
        rows = index["comparisons"]
        self.assertEqual(rows["direct"]["result"], "1/1쪽 기준 초과")
        self.assertIn("0.410%p 초과", rows["direct"]["point"])
        self.assertEqual(rows["browser"]["role"], "reference")
        self.assertEqual(rows["canvas"]["result"], "결과 미확인")
        self.assertEqual(rows["readiness"]["metrics"]["missing_samples"], ["B"])
        for row in rows.values():
            self.assertIn(EVIDENCE.code(row["result"]), markdown)
            self.assertIn("<br>".join(map(EVIDENCE.code, row["point"].splitlines())), markdown)
            self.assertTrue(set(row["evidence"]).issubset(index["files"]))

    def test_cli_records_actual_merge_objects_in_a_shallow_checkout(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo, shallow = root / 'history', root / 'shallow'
            subprocess.run(['git', 'init', '-q', '-b', 'test', str(repo)], check=True)

            def git(*args):
                return subprocess.check_output([
                    'git', '-C', str(repo), '-c', 'user.name=Fixture',
                    '-c', 'user.email=fixture@example.com', '-c', 'commit.gpgsign=false', *args
                ], text=True).strip()

            def run_index(*args):
                output = root / 'evidence.json'
                result = subprocess.check_output([
                    sys.executable, EVIDENCE.__file__, str(root), '--json-output', str(output), *args
                ], text=True)
                return json.loads(output.read_text(encoding='utf-8')), result

            tree = git('write-tree')
            event_base = git('commit-tree', tree, '-m', 'Event base')
            actual_base = git('commit-tree', tree, '-p', event_base, '-m', 'Advanced base')
            head = git('commit-tree', tree, '-p', event_base, '-m', 'PR head')
            (repo / 'test.txt').write_text('Merged content\n', encoding='utf-8')
            git('add', 'test.txt')
            tested_tree = git('write-tree')
            self.assertNotEqual(tested_tree, tree)
            merge = git('commit-tree', tested_tree, '-p', actual_base, '-p', head, '-m', 'Tested merge')
            git('update-ref', 'HEAD', merge)
            subprocess.run(['git', 'clone', '-q', '--depth=1', repo.as_uri(), str(shallow)], check=True)
            self.assertEqual(subprocess.check_output(
                ['git', '-C', str(shallow), 'rev-list', '--count', 'HEAD'], text=True).strip(), '1')

            index, markdown = run_index(
                '--checkout', str(shallow), '--pr-head', head, '--pr-event-base', event_base
            )
            self.assertEqual(index['tested_sha'], merge)
            self.assertEqual(index['tested_tree'], tested_tree)
            self.assertEqual(index['pr_event_base'], event_base)
            self.assertEqual(index['tested_merge_first_parent'], actual_base)
            self.assertIn(f'PR event base {EVIDENCE.code(event_base)}', markdown)
            self.assertIn(f'Tested PR merge first parent {EVIDENCE.code(actual_base)}', markdown)

            non_pr, _ = run_index('--checkout', str(shallow))
            self.assertIsNone(non_pr['tested_merge_first_parent'])
            self.assertNotIn('pr_event_base', non_pr)
            git('update-ref', 'HEAD', head)
            single_parent, _ = run_index('--checkout', str(repo), '--pr-head', head, '--pr-event-base', event_base)
            self.assertIsNone(single_parent['tested_merge_first_parent'])

            replay, _ = run_index('--tested-sha', merge, '--pr-head', head)
            self.assertEqual(replay['tested_sha'], merge)
            self.assertNotIn('tested_tree', replay)
            self.assertNotIn('tested_merge_first_parent', replay)

    def test_print_sweep_needs_explicit_complete_matching_evidence(self):
        manifest = {"mode": "silhouette_only", "run_state": "complete", "completed_pages": [1, 3],
                    "metrics": [{"page": 1, "silhouette_raw_match_percent": 99.8999},
                                {"page": 3, "silhouette_raw_match_percent": 98.5}],
                    "pr_review_gate": {"status": "not_evaluated"}, "provenance": {
                        "comparison_profile": "print", "raster_backend": "native-png",
                        "hwp": {"path": "original.hwpx", "sha256": "d" * 64},
                        "pdf": {"path": "print.pdf", "sha256": "e" * 64},
                    }}
        key = "output/native"
        tsv = "page\tsilhouette_raw_match_percent\n1\t99.90\n3\t98.50\n"
        manifest["provenance"]["git_head"] = "b" * 40
        manifest["tsv_sha256"] = hashlib.sha256(tsv.encode()).hexdigest()
        entries = {f"{key}/silhouette_manifest.json": manifest, f"{key}/silhouette.tsv": tsv}
        self.assertNotIn("silhouette.tsv", self.index_zip(entries))
        self.assertNotIn("silhouette.tsv", self.index_zip(entries, sweep_dirs=[key], pr_head="c" * 40))
        linked = self.index_zip(entries, sweep_dirs=[key], pr_head="b" * 40)
        self.assertIn("<code>not_evaluated</code> | <code>original.hwpx</code> p1, 3", linked)
        self.assertIn(f"{key}/silhouette.tsv</code> <code>{hashlib.sha256(tsv.encode()).hexdigest()}", linked)
        for bad_entries in (
            {f"{key}/silhouette_manifest.json": manifest},
            {**entries, f"{key}/silhouette.tsv": tsv.replace("3\t", "2\t")},
            {**entries, f"{key}/silhouette.tsv": tsv.replace("98.50", "99.50")},
            {**entries, f"{key}/silhouette_manifest.json": {**manifest, "run_state": "incomplete"}},
            {**entries, f"{key}/silhouette_manifest.json": {**manifest, "provenance": {}}},
        ):
            self.assertNotIn("silhouette.tsv", self.index_zip(bad_entries, sweep_dirs=[key], pr_head="b" * 40))

    def test_summary_retains_detailed_reports_with_and_without_index(self):
        import os
        import subprocess
        import tempfile
        import textwrap

        step = WORKFLOW_PATH.read_text().split("      - name: Add render diff summary\n", 1)[1]
        command = textwrap.dedent(step.split("        run: |\n", 1)[1])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reports = ["rhwp-studio/e2e/screenshots/render-diff/summary.md",
                       "rhwp-studio/e2e/screenshots/render-diff/pdf-summary.md",
                       "output/renderer-baseline/canvaskit-readiness/baseline-report.md",
                       "output/render-diff-build/summary.md"]
            for path in reports:
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                (root / path).write_text(path)
            target = root / "summary"
            for indexed in (False, True):
                target.write_text("")
                if indexed:
                    (root / "output/render-diff-build/evidence.md").write_text("INDEX")
                subprocess.run(["bash", "-eu", "-c", command], check=True, cwd=root,
                               env={**os.environ, "ARTIFACT_URL": "", "GITHUB_STEP_SUMMARY": str(target)})
                summary = target.read_text()
                folded = summary.split("<details>", 1)[1].split("</details>", 1)[0]
                for path in reports:
                    self.assertIn(path, folded)
                self.assertEqual("INDEX" in summary.split("<details>", 1)[0], indexed)

    def test_workflow_uses_the_uploaded_index_and_keeps_native_record_advisory(self):
        workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        native = workflow.split("      - name: Record native CLI build evidence\n", 1)[1].split("\n\n", 1)[0]
        index = workflow.split("      - name: Index existing visual comparison evidence\n", 1)[1].split("\n\n", 1)[0]
        upload = workflow.split("      - name: Upload render diff artifacts\n", 1)[1].split("\n\n", 1)[0]
        summary = workflow.split("      - name: Add render diff summary\n", 1)[1]
        self.assertIn("continue-on-error: true", native)
        self.assertIn("RHWP_RENDER_DIFF_RHWP_BIN", native)
        self.assertIn("continue-on-error: true", index)
        self.assertIn("$GITHUB_RUN_ATTEMPT", index)
        self.assertNotIn("--print-sweep", index)
        self.assertIn("output/render-diff-build/evidence.md", index)
        self.assertIn("output/render-diff-build/", upload)
        self.assertIn('cat output/render-diff-build/evidence.md >> "$GITHUB_STEP_SUMMARY"', summary)
        self.assertIn("steps.render-artifacts.outputs.artifact-url", summary)


if __name__ == "__main__":
    unittest.main()
