"""Release package workflow source-selection contract."""

from __future__ import annotations

import unittest
from pathlib import Path


WORKFLOW_PATH = (
    Path(__file__).resolve().parents[2] / ".github/workflows/release-packages.yml"
)


class ReleasePackagesWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_dispatch_checkout_uses_requested_tag_and_push_keeps_event_ref(self) -> None:
        self.assertIn(
            "ref: ${{ github.event_name == 'workflow_dispatch' && inputs.tag || github.ref }}",
            self.workflow,
        )
        self.assertIn(
            "REQUESTED_TAG: ${{ github.event_name == 'workflow_dispatch' "
            "&& inputs.tag || github.ref_name }}",
            self.workflow,
        )

    def test_all_package_build_checkouts_use_validated_source_sha(self) -> None:
        self.assertEqual(
            self.workflow.count("ref: ${{ needs.version-gate.outputs.source_sha }}"),
            3,
        )
        self.assertIn("source_sha: ${{ steps.ver.outputs.source_sha }}", self.workflow)
        self.assertIn('git rev-parse --verify "refs/tags/$TAG^{commit}"', self.workflow)
        self.assertIn('python3 tools/set_package_version.py "${VERSION}" --check', self.workflow)


if __name__ == "__main__":
    unittest.main()
