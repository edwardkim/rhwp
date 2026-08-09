"""Docker publish workflow release-source contract."""

from __future__ import annotations

import unittest
from pathlib import Path


WORKFLOW_PATH = (
    Path(__file__).resolve().parents[2] / ".github/workflows/docker-publish.yml"
)


class DockerPublishWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_dispatch_builds_requested_tag_and_other_events_keep_their_ref(self) -> None:
        self.assertIn(
            "ref: ${{ github.event_name == 'workflow_dispatch' && inputs.tag || github.ref }}",
            self.workflow,
        )
        self.assertIn('git rev-parse --verify "refs/tags/$TAG^{commit}"', self.workflow)
        self.assertIn('CARGO_VERSION="$(awk -F', self.workflow)

    def test_prerelease_does_not_move_latest(self) -> None:
        self.assertIn('if [[ "$VERSION" == *-* ]]', self.workflow)
        self.assertIn("PUBLISH_LATEST: ${{ steps.ver.outputs.publish_latest }}", self.workflow)
        latest_tag = self.workflow.index('docker tag rhwp-cli:local "$IMAGE:latest"')
        latest_guard = self.workflow.index('if [[ "$PUBLISH_LATEST" == "true" ]]')
        self.assertGreater(latest_tag, latest_guard)


if __name__ == "__main__":
    unittest.main()
