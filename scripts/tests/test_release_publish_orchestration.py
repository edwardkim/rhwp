"""#6634 Release Binary와 package publish 사이의 보호 계약."""

from __future__ import annotations

import json
import re
import unittest
from datetime import datetime
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
BINARY_WORKFLOW = REPO_ROOT / ".github/workflows/release-binary.yml"
PACKAGE_WORKFLOW = REPO_ROOT / ".github/workflows/npm-publish.yml"
PROMOTION_POLICY = REPO_ROOT / "scripts/workflow_promotion_policy.json"
LINEAGE_FIXTURE = (
    REPO_ROOT
    / "mydocs/tech/investigations/issue-6634/release_publish_lineage.json"
)


def instant(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def job_block(workflow: str, job_id: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(job_id)}:\s*$.*?(?=^  [a-zA-Z0-9_-]+:\s*$|\Z)",
        workflow,
    )
    if match is None:
        raise AssertionError(f"workflow job이 없다: {job_id}")
    return match.group(0)


class ReleasePublishLineageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(LINEAGE_FIXTURE.read_text(encoding="utf-8"))

    def test_fixture_has_expected_release_cohorts(self):
        releases = self.fixture["releases"]
        self.assertEqual(
            [release["tag"] for release in releases],
            ["v0.8.0", "v0.8.1", "v0.8.2", "v0.8.3", "v0.8.4", "v0.8.6"],
        )
        self.assertEqual(
            [release["tag"] for release in releases if release["automaticPackageRun"]],
            ["v0.8.0", "v0.8.1", "v0.8.2", "v0.8.3"],
        )
        self.assertEqual(
            [release["tag"] for release in releases if not release["automaticPackageRun"]],
            ["v0.8.4", "v0.8.6"],
        )

    def test_successful_release_event_preceded_binary_attachment(self):
        for release in self.fixture["releases"][:4]:
            published = instant(release["releasePublishedAt"])
            attachment = instant(release["binaryRun"]["releaseJobStartedAt"])
            package = release["automaticPackageRun"]
            self.assertLess(published, attachment, release["tag"])
            self.assertEqual(package["event"], "release", release["tag"])
            self.assertLess(instant(package["createdAt"]), attachment, release["tag"])

    def test_suppressed_release_event_was_published_inside_attachment_job(self):
        for release in self.fixture["releases"][4:]:
            published = instant(release["releasePublishedAt"])
            started = instant(release["binaryRun"]["releaseJobStartedAt"])
            completed = instant(release["binaryRun"]["releaseJobCompletedAt"])
            self.assertLessEqual(started, published, release["tag"])
            self.assertLessEqual(published, completed, release["tag"])
            self.assertIsNone(release["automaticPackageRun"], release["tag"])
            self.assertEqual(
                release["publicationPath"],
                "github-token-inside-binary-attachment",
                release["tag"],
            )

    def test_v086_manual_recovery_did_not_use_release_tag_sha(self):
        release = self.fixture["releases"][-1]
        recovery = release["recoveryPackageRun"]
        self.assertEqual(release["tag"], "v0.8.6")
        self.assertEqual(recovery["event"], "workflow_dispatch")
        self.assertNotEqual(recovery["headSha"], release["tagSha"])


class ReleasePublishOrchestrationRedTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.binary = BINARY_WORKFLOW.read_text(encoding="utf-8")
        cls.package = PACKAGE_WORKFLOW.read_text(encoding="utf-8")
        cls.policy = json.loads(PROMOTION_POLICY.read_text(encoding="utf-8"))

    def test_package_workflow_does_not_depend_on_release_published_event(self):
        self.assertNotRegex(self.package, r"(?m)^  release:\s*$")
        self.assertNotIn("types: [published]", self.package)

    def test_package_workflow_is_reusable(self):
        self.assertRegex(self.package, r"(?m)^  workflow_call:\s*$")

    def test_manual_dispatch_defaults_to_non_publishing_mode(self):
        self.assertRegex(
            self.package,
            r"(?ms)^      publish:\s*$.*?^        type: boolean\s*$.*?^        default: false\s*$",
        )

    def test_release_binary_calls_same_commit_package_workflow_after_release(self):
        caller = job_block(self.binary, "publish-packages")
        self.assertIn("needs: release", caller)
        self.assertIn("uses: ./.github/workflows/npm-publish.yml", caller)
        self.assertIn("secrets: inherit", caller)

    def test_production_publish_invokes_exact_release_source_guard(self):
        self.assertIn("scripts/release_publish_guard.py", self.package)
        self.assertRegex(self.package, r"(?m)^  validate-release-source:\s*$")

    def test_extension_channels_are_independent_jobs(self):
        marketplace = job_block(self.package, "publish-vscode-marketplace")
        open_vsx = job_block(self.package, "publish-open-vsx")
        self.assertNotIn("ovsx publish", marketplace)
        self.assertNotIn("vsce publish", open_vsx)

    def test_publish_completion_is_an_explicit_aggregate(self):
        aggregate = job_block(self.package, "publish-complete")
        for dependency in (
            "publish-npm-core",
            "publish-npm-editor",
            "publish-vscode-marketplace",
            "publish-open-vsx",
        ):
            self.assertIn(dependency, aggregate)

    def test_promotion_policy_covers_both_release_workflows(self):
        workflows = self.policy["workflows"]
        self.assertIn(".github/workflows/release-binary.yml", workflows)
        self.assertIn(".github/workflows/npm-publish.yml", workflows)


if __name__ == "__main__":
    unittest.main()
