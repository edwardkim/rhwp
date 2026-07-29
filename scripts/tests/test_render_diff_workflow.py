from __future__ import annotations

import unittest
from pathlib import Path


WORKFLOW_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/render-diff.yml"


class RenderDiffTriggerPolicyTests(unittest.TestCase):
    def test_review_records_do_not_trigger_canvas_visual_diff(self) -> None:
        workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        pull_request_trigger = workflow.split("  workflow_dispatch:", maxsplit=1)[0]

        self.assertIn("  pull_request:\n", pull_request_trigger)
        self.assertIn("      - 'src/renderer/**'", pull_request_trigger)
        self.assertNotIn("'mydocs/**'", pull_request_trigger)


if __name__ == "__main__":
    unittest.main()
