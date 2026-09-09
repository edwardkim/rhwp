from __future__ import annotations

import unittest
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


if __name__ == "__main__":
    unittest.main()
