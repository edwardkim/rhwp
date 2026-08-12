"""Nextest archive profile·timeout·cache 운영 계약."""

from __future__ import annotations

import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = REPO_ROOT / ".github/workflows/ci.yml"
BUILD_ARCHIVE_WORKFLOW = REPO_ROOT / ".github/workflows/build-nextest-archives.yml"
RELEASE_BINARY_WORKFLOW = REPO_ROOT / ".github/workflows/release-binary.yml"


def job_body(workflow: str, job_name: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(job_name)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
    )
    if match is None:
        raise AssertionError(f"workflow에 {job_name} job이 없다")
    return match.group(0)


def step_body(workflow: str, step_name: str) -> str:
    marker = f"      - name: {step_name}\n"
    if marker not in workflow:
        raise AssertionError(f"workflow에 {step_name} step이 없다")
    body = workflow.split(marker, maxsplit=1)[1]
    boundary = re.search(
        r"(?m)^(?:      - (?:name:|uses:)|  [A-Za-z0-9_-]+:)\s*", body
    )
    return body[: boundary.start()] if boundary else body


def run_script(script: str, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "-e", "-o", "pipefail", "-c", script],
        check=False,
        capture_output=True,
        env={**os.environ, **env},
        text=True,
    )


class NextestArchiveWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ci = CI_WORKFLOW.read_text(encoding="utf-8")
        cls.builder = BUILD_ARCHIVE_WORKFLOW.read_text(encoding="utf-8")
        cls.release_binary = RELEASE_BINARY_WORKFLOW.read_text(encoding="utf-8")

    def test_manual_release_grade_input_is_explicit_boolean_opt_in(self) -> None:
        trigger = self.ci.split("  workflow_dispatch:\n", maxsplit=1)[1].split(
            "\nenv:", maxsplit=1
        )[0]
        self.assertIn("    inputs:\n      release_grade:", trigger)
        self.assertIn("        type: boolean", trigger)
        self.assertIn("        required: true", trigger)
        self.assertIn("        default: false", trigger)

    def test_policy_router_covers_fast_release_and_fail_closed_paths(self) -> None:
        step = step_body(self.ci, "Select test profile policy")
        script = step.split("        run: |\n", maxsplit=1)[1]
        script = "\n".join(line.removeprefix("          ") for line in script.splitlines())

        cases = [
            ("pull_request", "refs/pull/1/merge", "false", "release-test", "30"),
            ("push", "refs/heads/devel", "false", "release-test", "30"),
            ("workflow_dispatch", "refs/heads/feature", "false", "release-test", "30"),
            ("push", "refs/heads/main", "false", "release", "60"),
            ("push", "refs/tags/v1.2.3", "false", "release", "60"),
            ("workflow_dispatch", "refs/heads/feature", "true", "release", "60"),
            ("push", "refs/heads/unexpected", "false", "release", "60"),
            ("workflow_dispatch", "refs/heads/feature", "invalid", "release", "60"),
        ]
        for event, ref, requested, expected_profile, expected_timeout in cases:
            with self.subTest(event=event, ref=ref, requested=requested):
                with tempfile.TemporaryDirectory() as directory:
                    output = Path(directory) / "output"
                    summary = Path(directory) / "summary"
                    result = run_script(
                        script,
                        {
                            "GITHUB_EVENT_NAME": event,
                            "GITHUB_REF": ref,
                            "RELEASE_GRADE": requested,
                            "GITHUB_OUTPUT": str(output),
                            "GITHUB_STEP_SUMMARY": str(summary),
                        },
                    )
                    self.assertEqual(result.returncode, 0, result.stderr)
                    outputs = dict(
                        line.split("=", maxsplit=1)
                        for line in output.read_text(encoding="utf-8").splitlines()
                    )
                    self.assertEqual(outputs["cargo_profile"], expected_profile)
                    self.assertEqual(outputs["timeout_minutes"], expected_timeout)
                    summary_text = summary.read_text(encoding="utf-8")
                    self.assertIn(expected_profile, summary_text)
                    self.assertIn(expected_timeout, summary_text)

    def test_preflight_exposes_fail_closed_archive_policy_outputs(self) -> None:
        preflight = job_body(self.ci, "preflight")
        self.assertIn(
            "test_profile: ${{ steps.test-policy.outputs.cargo_profile "
            "|| 'release' }}",
            preflight,
        )
        self.assertIn(
            "test_archive_timeout_minutes: ${{ "
            "steps.test-policy.outputs.timeout_minutes || '60' }}",
            preflight,
        )

    def test_all_archive_builders_receive_the_same_policy(self) -> None:
        for name in (
            "build-test-archive-slow",
            "build-test-archive-a",
            "build-test-archive-b",
        ):
            with self.subTest(job=name):
                job = job_body(self.ci, name)
                self.assertIn(
                    "cargo_profile: ${{ needs.preflight.outputs.test_profile "
                    "|| 'release' }}",
                    job,
                )
                self.assertIn(
                    "timeout_minutes: ${{ fromJSON("
                    "needs.preflight.outputs.test_archive_timeout_minutes || '60') }}",
                    job,
                )

    def test_native_skia_uses_the_same_test_profile_policy(self) -> None:
        native = job_body(self.ci, "native-skia-tests")
        self.assertIn(
            "TEST_PROFILE: ${{ needs.preflight.outputs.test_profile || 'release' }}",
            native,
        )
        step = step_body(self.ci, "Native Skia tests")
        self.assertIn('case "${TEST_PROFILE}" in', step)
        self.assertIn("release-test)", step)
        self.assertIn("release)", step)
        self.assertIn("Unknown test profile", step)
        self.assertNotIn('"${GITHUB_EVENT_NAME}" == "pull_request"', step)

    def test_reusable_builder_accepts_explicit_policy_and_uses_dynamic_timeout(self) -> None:
        self.assertIn("      cargo_profile:\n", self.builder)
        self.assertIn("      timeout_minutes:\n", self.builder)
        self.assertIn("        type: string", self.builder)
        self.assertIn("        type: number", self.builder)
        self.assertIn("    timeout-minutes: ${{ inputs.timeout_minutes }}", self.builder)
        self.assertIn(
            '--cargo-profile "${{ inputs.cargo_profile }}"',
            self.builder,
        )
        self.assertNotIn("- name: Select cargo profile", self.builder)
        self.assertNotIn("steps.profile.outputs.cargo_profile", self.builder)

    def test_reusable_builder_rejects_profile_timeout_mismatches(self) -> None:
        step = step_body(self.builder, "Validate test archive policy")
        script = step.split("        run: |\n", maxsplit=1)[1]
        script = "\n".join(line.removeprefix("          ") for line in script.splitlines())

        for profile, timeout, expected in (
            ("release-test", "30", 0),
            ("release", "60", 0),
            ("release-test", "60", 1),
            ("release", "30", 1),
            ("debug", "60", 1),
        ):
            with self.subTest(profile=profile, timeout=timeout):
                result = run_script(
                    script,
                    {"CARGO_PROFILE": profile, "TIMEOUT_MINUTES": timeout},
                )
                self.assertEqual(result.returncode, expected, result.stderr)

    def test_builder_summary_exposes_policy_and_cache_state(self) -> None:
        self.assertIn("id: rust-cache", self.builder)
        summary = step_body(self.builder, "Summarize test archive policy")
        for field in (
            "event",
            "ref",
            "cargo_profile",
            "timeout_minutes",
            "cache_exact_hit",
            "cache_save_eligible",
        ):
            with self.subTest(field=field):
                self.assertIn(field, summary)
        self.assertIn("steps.rust-cache.outputs.cache-hit", summary)
        self.assertIn("if: ${{ always() }}", summary)
        self.assertIn(
            "save-if: ${{ github.event_name == 'push' && "
            "(github.ref == 'refs/heads/devel' || github.ref == 'refs/heads/main') }}",
            self.builder,
        )

    def test_required_check_and_release_artifact_contracts_stay_stable(self) -> None:
        self.assertIn("name: Build & Test", self.ci)
        self.assertIn(
            "cargo build --release --bin rhwp --target ${{ matrix.target }}",
            self.release_binary,
        )
        self.assertIn("wasm-pack build --target web --release", self.ci)


if __name__ == "__main__":
    unittest.main()
