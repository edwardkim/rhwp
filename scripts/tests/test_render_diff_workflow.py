from __future__ import annotations

import unittest
import importlib.util
from pathlib import Path


WORKFLOW_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/render-diff.yml"


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
            "types: [opened, reopened, synchronize]",
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
        cdylib = self.measurement.build_configuration("release", "auto")
        self.assertEqual(dev["cdylib_only"], "0")
        self.assertEqual(standard["crate_types"], "rlib+cdylib")
        self.assertEqual(standard["lto_policy"], "manifest")
        self.assertEqual(cdylib["cdylib_only"], "1")
        self.assertEqual(cdylib["crate_types"], "cdylib")
        self.assertEqual(cdylib["lto_policy"], "override-false")
        self.assertEqual(cdylib, self.measurement.build_configuration("release", "cdylib"))
        self.assertEqual(len({item["cache_variant"] for item in (dev, standard, cdylib)}), 3)
        with self.assertRaisesRegex(ValueError, "release"):
            self.measurement.build_configuration("dev", "cdylib")

    def test_cache_restore_cannot_cross_build_modes_and_save_reuses_primary_key(self):
        workflow = WORKFLOW_PATH.read_text()
        restore = workflow.split("      - name: Restore cargo registry & build cache\n")[1].split(
            "      - name: Build WASM package\n")[0]
        prefix = "${{ runner.os }}-render-diff-cargo-v2-${{ steps.wasm_configuration.outputs.cache_variant }}-"
        self.assertIn(f"key: {prefix}", restore)
        self.assertIn(f"restore-keys: |\n            {prefix}\n", restore)
        self.assertNotIn("${{ runner.os }}-render-diff-cargo-\n", restore)
        self.assertIn("key: ${{ steps.render_diff_cargo_cache_restore.outputs.cache-primary-key }}", workflow)
        self.assertIn("github.event_name == 'workflow_dispatch' && inputs['wasm-build-mode'] || 'cdylib'", workflow)
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
        for name in ("deploy-pages", "npm-publish", "full-renderer-sweep"):
            self.assertIn("wasm-pack build --target web --release",
                          (repo / f".github/workflows/{name}.yml").read_text())


if __name__ == "__main__":
    unittest.main()
