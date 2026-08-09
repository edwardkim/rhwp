"""Installer release workflow and portable installer checksum contracts."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class ReleaseInstallersWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = (
            ROOT / ".github/workflows/release-installers.yml"
        ).read_text(encoding="utf-8")
        cls.installer = (ROOT / "contrib/install/install.sh").read_text(
            encoding="utf-8"
        )

    def test_dispatch_resolves_tag_to_an_immutable_validated_source(self) -> None:
        self.assertIn(
            "ref: ${{ github.event_name == 'workflow_dispatch' && inputs.tag || github.ref }}",
            self.workflow,
        )
        self.assertIn("source_sha: ${{ steps.v.outputs.source_sha }}", self.workflow)
        self.assertEqual(
            self.workflow.count("ref: ${{ needs.version.outputs.source_sha }}"),
            3,
        )
        self.assertIn('git rev-parse --verify "refs/tags/$TAG^{commit}"', self.workflow)
        self.assertIn('if [[ "$VERSION" != "$CARGO_VERSION" ]]', self.workflow)

    def test_installer_has_stock_macos_checksum_fallback(self) -> None:
        self.assertIn("command -v sha256sum", self.installer)
        self.assertIn("command -v shasum", self.installer)
        self.assertIn("shasum -a 256 -c SHA256SUMS.asset.txt", self.installer)
        self.assertNotIn("--ignore-missing", self.installer)


if __name__ == "__main__":
    unittest.main()
