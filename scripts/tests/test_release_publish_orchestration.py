"""#6634 Release Binary와 package publish 사이의 보호 계약."""

from __future__ import annotations

import json
import re
import unittest
from datetime import datetime
from pathlib import Path

from scripts.release_publish_guard import evaluate_release_source


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
        binary_header = self.binary.split("\njobs:", maxsplit=1)[0]
        package_header = self.package.split("\njobs:", maxsplit=1)[0]
        release = job_block(self.binary, "release")
        caller = job_block(self.binary, "publish-packages")
        npm_core = job_block(self.package, "publish-npm-core")
        npm_editor = job_block(self.package, "publish-npm-editor")
        self.assertNotIn("contents: write", binary_header)
        self.assertNotIn("id-token: write", package_header)
        self.assertIn(
            "if: github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')",
            release,
        )
        self.assertIn("contents: write", release)
        self.assertIn("needs: [build, release]", caller)
        self.assertIn("needs.build.result == 'success'", caller)
        self.assertIn("needs.release.result == 'success'", caller)
        self.assertIn("needs.release.result == 'skipped'", caller)
        self.assertIn("id-token: write", caller)
        self.assertIn("uses: ./.github/workflows/npm-publish.yml", caller)
        self.assertNotIn("secrets: inherit", caller)
        self.assertIn("VSCE_PAT: ${{ secrets.VSCE_PAT }}", caller)
        self.assertIn("OVSX_PAT: ${{ secrets.OVSX_PAT }}", caller)
        self.assertIn("id-token: write", npm_core)
        self.assertIn("id-token: write", npm_editor)

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


class ReleasePublishGuardTests(unittest.TestCase):
    def valid_context(self, *, mode: str = "publish") -> dict[str, object]:
        sha = "a" * 40
        return {
            "mode": mode,
            "eventName": "push" if mode == "publish" else "workflow_dispatch",
            "ref": "refs/tags/v0.8.6" if mode == "publish" else "refs/heads/devel",
            "refType": "tag" if mode == "publish" else "branch",
            "refName": "v0.8.6" if mode == "publish" else "devel",
            "githubSha": sha,
            "checkoutSha": sha,
            "tagSha": sha if mode == "publish" else None,
            "versions": {
                "cargo": "0.8.6",
                "npmEditor": "0.8.6",
                "vscode": "0.8.6",
            },
            "release": (
                {
                    "tag_name": "v0.8.6",
                    "draft": False,
                    "prerelease": False,
                    "published_at": "2026-09-02T03:00:31Z",
                }
                if mode == "publish"
                else None
            ),
        }

    def test_exact_stable_release_is_accepted(self):
        verdict = evaluate_release_source(self.valid_context())
        self.assertTrue(verdict["accepted"], verdict["errors"])

    def test_verify_mode_accepts_branch_without_release_metadata(self):
        verdict = evaluate_release_source(self.valid_context(mode="verify"))
        self.assertTrue(verdict["accepted"], verdict["errors"])
        self.assertEqual(verdict["mode"], "verify")

    def test_publish_rejects_non_tag_ref(self):
        context = self.valid_context()
        context.update(
            {"ref": "refs/heads/main", "refType": "branch", "refName": "main"}
        )
        verdict = evaluate_release_source(context)
        self.assertFalse(verdict["accepted"])
        self.assertIn("publish-ref-not-tag", verdict["errors"])

    def test_publish_rejects_tag_sha_mismatch(self):
        context = self.valid_context()
        context["tagSha"] = "b" * 40
        verdict = evaluate_release_source(context)
        self.assertFalse(verdict["accepted"])
        self.assertIn("tag-sha-mismatch", verdict["errors"])

    def test_publish_rejects_package_version_mismatch(self):
        context = self.valid_context()
        context["versions"]["npmEditor"] = "0.8.5"
        verdict = evaluate_release_source(context)
        self.assertFalse(verdict["accepted"])
        self.assertIn("version-mismatch:npmEditor", verdict["errors"])

    def test_publish_rejects_draft_or_prerelease(self):
        for field in ("draft", "prerelease"):
            with self.subTest(field=field):
                context = self.valid_context()
                context["release"][field] = True
                verdict = evaluate_release_source(context)
                self.assertFalse(verdict["accepted"])
                self.assertIn(f"release-is-{field}", verdict["errors"])


if __name__ == "__main__":
    unittest.main()
