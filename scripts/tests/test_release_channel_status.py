"""#6634 외부 배포 채널 상태와 부분 재시도 보호 계약."""

from __future__ import annotations

import importlib.util
import json
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_TARGET = REPO_ROOT / "scripts/release_channel_status.py"
EVIDENCE_TARGET = REPO_ROOT / "scripts/release_publish_evidence.py"
FIXTURE = (
    REPO_ROOT
    / "mydocs/tech/investigations/issue-6634/release_channel_status_cases.json"
)
WORKFLOW = REPO_ROOT / ".github/workflows/npm-publish.yml"


def load_module(name: str, path: Path):
    if not path.exists():
        return None
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"module을 읽을 수 없다: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


STATUS = load_module("release_channel_status", STATUS_TARGET)
EVIDENCE = load_module("release_publish_evidence", EVIDENCE_TARGET)


def job_block(workflow: str, job_id: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(job_id)}:\s*$.*?(?=^  [a-zA-Z0-9_-]+:\s*$|\Z)",
        workflow,
    )
    if match is None:
        raise AssertionError(f"workflow job이 없다: {job_id}")
    return match.group(0)


class Stage3SourcePresenceTests(unittest.TestCase):
    def test_channel_status_probe_exists(self):
        self.assertTrue(STATUS_TARGET.exists())

    def test_publish_evidence_aggregator_exists(self):
        self.assertTrue(EVIDENCE_TARGET.exists())


@unittest.skipUnless(STATUS is not None, "RED: channel status probe 미구현")
class ChannelStatusFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_public_registry_responses_are_fail_closed(self):
        for case in self.fixture["probeCases"]:
            with self.subTest(case=case["id"]):
                if "expectedError" in case:
                    with self.assertRaisesRegex(
                        STATUS.ChannelStatusError, case["expectedError"]
                    ):
                        STATUS.interpret_response(
                            case["channel"],
                            case["identifier"],
                            case["version"],
                            case["httpStatus"],
                            case["payload"],
                        )
                else:
                    present = STATUS.interpret_response(
                        case["channel"],
                        case["identifier"],
                        case["version"],
                        case["httpStatus"],
                        case["payload"],
                    )
                    self.assertEqual(present, case["expectedPresent"])


@unittest.skipUnless(EVIDENCE is not None, "RED: publish evidence 집계 미구현")
class PublishEvidenceFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))

    def test_partial_retry_and_lookup_failure_cases(self):
        gates = {
            "validate-release-source": "success",
            "build-wasm": "success",
            "build-vsix": "success",
        }
        for case in self.fixture["aggregateCases"]:
            with self.subTest(case=case["id"]):
                verdict = EVIDENCE.evaluate_publish_evidence(
                    {
                        "mode": case["mode"],
                        "extensionsRequested": case["extensionsRequested"],
                        "githubSha": "a" * 40,
                        "refName": "v0.8.6",
                        "gates": gates,
                        "channels": case["channels"],
                    }
                )
                self.assertEqual(verdict["accepted"], case["expectedAccepted"])
                self.assertEqual(
                    verdict["verdict"],
                    "completed" if case["expectedAccepted"] else "failed",
                )
                if case["mode"] == "verify":
                    self.assertEqual(
                        {item["state"] for item in verdict["channels"].values()},
                        {"verify-only"},
                    )

    def test_failed_build_gate_is_rejected(self):
        verdict = EVIDENCE.evaluate_publish_evidence(
            {
                "mode": "verify",
                "extensionsRequested": True,
                "githubSha": "a" * 40,
                "refName": "devel",
                "gates": {
                    "validate-release-source": "success",
                    "build-wasm": "failure",
                    "build-vsix": "skipped",
                },
                "channels": {
                    name: {"jobResult": "skipped", "state": ""}
                    for name in EVIDENCE.EXPECTED_CHANNELS
                },
            }
        )
        self.assertFalse(verdict["accepted"])
        self.assertIn("gate-not-success:build-wasm:failure", verdict["errors"])

    def test_extensions_can_be_explicitly_excluded_from_manual_recovery(self):
        verdict = EVIDENCE.evaluate_publish_evidence(
            {
                "mode": "publish",
                "extensionsRequested": False,
                "githubSha": "a" * 40,
                "refName": "v0.8.6",
                "gates": {
                    "validate-release-source": "success",
                    "build-wasm": "success",
                    "build-vsix": "success",
                },
                "channels": {
                    "npm-core": {
                        "jobResult": "success",
                        "state": "already-present",
                    },
                    "npm-editor": {"jobResult": "success", "state": "published"},
                    "vscode-marketplace": {"jobResult": "skipped", "state": ""},
                    "open-vsx": {"jobResult": "skipped", "state": ""},
                },
            }
        )
        self.assertTrue(verdict["accepted"], verdict["errors"])
        self.assertEqual(
            verdict["channels"]["vscode-marketplace"]["state"], "not-requested"
        )
        self.assertEqual(verdict["channels"]["open-vsx"]["state"], "not-requested")


class PublishWorkflowStage3Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_vsix_is_built_once_and_consumed_by_independent_jobs(self):
        build = job_block(self.workflow, "build-vsix")
        marketplace = job_block(self.workflow, "publish-vscode-marketplace")
        open_vsx = job_block(self.workflow, "publish-open-vsx")
        self.assertEqual(build.count("vsce package"), 1)
        self.assertIn("upload-artifact", build)
        self.assertIn("download-artifact", marketplace)
        self.assertIn("download-artifact", open_vsx)
        self.assertIn("vsce publish", marketplace)
        self.assertNotIn("ovsx publish", marketplace)
        self.assertIn("ovsx publish", open_vsx)
        self.assertNotIn("vsce publish", open_vsx)

    def test_each_publish_channel_has_status_output(self):
        for job_id in (
            "publish-npm-core",
            "publish-npm-editor",
            "publish-vscode-marketplace",
            "publish-open-vsx",
        ):
            with self.subTest(job=job_id):
                block = job_block(self.workflow, job_id)
                self.assertIn("outputs:", block)
                self.assertIn("release_channel_status.py", block)
                self.assertIn("already-present", block)
                self.assertIn("published", block)
        self.assertIn(
            "[[ \"${package_name}\" == '@rhwp/core' ]]",
            job_block(self.workflow, "publish-npm-core"),
        )
        self.assertIn(
            "[[ \"${package_name}\" == '@rhwp/editor' ]]",
            job_block(self.workflow, "publish-npm-editor"),
        )
        for job_id in ("publish-vscode-marketplace", "publish-open-vsx"):
            self.assertIn(
                "[[ \"${extension_id}\" == 'edwardkim.rhwp-vscode' ]]",
                job_block(self.workflow, job_id),
            )

    def test_publish_completion_is_fail_closed_and_evidenced(self):
        aggregate = job_block(self.workflow, "publish-complete")
        self.assertIn("if: always()", aggregate)
        self.assertIn("release_publish_evidence.py", aggregate)
        self.assertIn("release-publish-evidence", aggregate)
        for dependency in (
            "validate-release-source",
            "build-wasm",
            "build-vsix",
            "publish-npm-core",
            "publish-npm-editor",
            "publish-vscode-marketplace",
            "publish-open-vsx",
        ):
            self.assertIn(dependency, aggregate)

    def test_same_ref_publish_runs_are_serialized(self):
        header = self.workflow.split("\njobs:", maxsplit=1)[0]
        self.assertIn("concurrency:", header)
        self.assertIn("github.ref", header)
        self.assertIn("cancel-in-progress: false", header)


if __name__ == "__main__":
    unittest.main()
