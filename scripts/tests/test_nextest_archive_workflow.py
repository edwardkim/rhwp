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
RUN_ARCHIVE_WORKFLOW = REPO_ROOT / ".github/workflows/run-nextest-archives.yml"
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
        cls.runner = RUN_ARCHIVE_WORKFLOW.read_text(encoding="utf-8")
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

    def test_two_archive_builders_split_lib_and_integration_targets(self):
        from pathlib import Path
        root = Path(__file__).resolve().parents[2]
        ci = (root / ".github/workflows/ci.yml").read_text()
        builder = (root / ".github/workflows/build-nextest-archives.yml").read_text()
        runner = (root / ".github/workflows/run-nextest-archives.yml").read_text()
        self.assertIn("build-test-archive-a:", ci); self.assertIn("build-test-archive-b:", ci); self.assertIn("build-test-archive-c:", ci)
        self.assertNotIn("test-slow-shard:", ci); self.assertEqual(1, ci.count('partition: "hash:1/2"')); self.assertEqual(1, ci.count('partition: "hash:2/2"')); self.assertEqual(2, ci.count('partition: "hash:1/1"'))
        self.assertIn("target_group: lib", ci); self.assertIn("target_group: integration-b", ci); self.assertIn("target_group: integration-c", ci)
        self.assertIn("cargo metadata --no-deps --format-version 1", builder)
        self.assertIn("scripts/select-nextest-archive-targets.mjs", builder)
        self.assertIn(
            "duration_policy='tests/suites/nextest-target-duration-policy.json'",
            builder,
        )
        self.assertIn('--policy "${duration_policy}"', builder)
        self.assertNotIn("index % 2", builder)
        self.assertIn("cargo_target_args+=(--lib)", builder)
        self.assertIn('cargo_target_args+=(--test "${target}")', builder)
        self.assertNotIn("--tests", builder)
        self.assertIn("--profile ci-duration-observation", runner)
        self.assertIn("scripts/collect-nextest-target-durations.mjs", runner)
        self.assertIn("nextest-target-durations-${{ github.run_id }}-${{ inputs.archive_label }}", runner)

    def test_duration_policy_collects_devel_and_same_repository_pr_b_and_c_measurements(self):
        from pathlib import Path
        root = Path(__file__).resolve().parents[2]
        policy = (root / "tests/suites/nextest-target-duration-policy.json").read_text()
        config = (root / ".config/nextest.toml").read_text()
        runner = (root / ".github/workflows/run-nextest-archives.yml").read_text()
        selector = (root / "scripts/select-nextest-archive-targets.mjs").read_text()
        collector = (root / "scripts/collect-nextest-target-durations.mjs").read_text()
        refresh = (root / "scripts/refresh-nextest-target-duration-policy.mjs").read_text()

        self.assertIn('"schema_version": 1', policy)
        self.assertIn('"fallback_seconds": 1', policy)
        self.assertIn("[profile.ci-duration-observation.junit]", config)
        self.assertIn("github.event_name == 'push'", runner)
        self.assertIn("github.ref == 'refs/heads/devel'", runner)
        self.assertIn("github.event_name == 'pull_request'", runner)
        self.assertIn(
            "github.event.pull_request.head.repo.full_name == github.repository",
            runner,
        )
        self.assertIn("inputs.archive_label == 'b'", runner)
        self.assertIn("inputs.archive_label == 'c'", runner)
        self.assertIn("retention-days: 3", runner)
        self.assertIn("retention-days: 30", runner)
        self.assertIn("estimatedSeconds", selector)
        self.assertIn("<testcase", collector)
        self.assertIn("JUnit report contains no target durations", collector)
        self.assertIn("exactly one B report and one C report", refresh)
        self.assertIn("identical run, ref, and sha provenance", refresh)

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

    def test_reusable_builder_isolates_partition_artifacts(self):
        from pathlib import Path
        root = Path(__file__).resolve().parents[2]
        ci = (root / ".github/workflows/ci.yml").read_text()
        builder = (root / ".github/workflows/build-nextest-archives.yml").read_text()
        runner = (root / ".github/workflows/run-nextest-archives.yml").read_text()
        self.assertIn("inputs.target_group", builder)
        self.assertIn("test-archive-${{ github.run_id }}-${{ inputs.archive_label }}", builder)
        self.assertIn("archive-expected-${{ github.run_id }}-${{ inputs.archive_label }}", builder)
        self.assertIn("test-archive-${{ github.run_id }}-${{ inputs.archive_label }}", runner)

    def test_builder_prepares_derived_suites_before_compiling_the_archive(self) -> None:
        prepare = "node scripts/rust-test-suite-manifest.mjs --prepare"
        archive = "cargo nextest archive"
        self.assertIn(prepare, self.builder)
        self.assertIn(archive, self.builder)
        self.assertLess(self.builder.index(prepare), self.builder.index(archive))

    def test_four_workers_validate_each_archive_coverage(self):
        from pathlib import Path
        root = Path(__file__).resolve().parents[2]
        ci = (root / ".github/workflows/ci.yml").read_text()
        builder = (root / ".github/workflows/build-nextest-archives.yml").read_text()
        runner = (root / ".github/workflows/run-nextest-archives.yml").read_text()
        for name in ("test-archive-a-shard-1:", "test-archive-a-shard-2:", "test-archive-b-shard-1:", "test-archive-c-shard-1:"):
            self.assertIn(name, ci)
        self.assertIn("Archive A shard total mismatch", ci); self.assertIn("Archive B shard total mismatch", ci); self.assertIn("Archive C shard total mismatch", ci)
        self.assertIn("name: Build & Test", ci)

    def test_reusable_builder_rejects_profile_timeout_mismatches(self) -> None:
        step = step_body(self.builder, "Validate test archive policy")
        script = step.split("        run: |\n", maxsplit=1)[1]
        script = "\n".join(line.removeprefix("          ") for line in script.splitlines())

        for profile, timeout, target_group, expected in (
            ("release-test", "30", "lib", 0),
            ("release", "60", "integration-b", 0),
            ("release", "60", "integration-c", 0),
            ("release-test", "60", "lib", 1),
            ("release", "30", "integration-b", 1),
            ("debug", "60", "lib", 1),
            ("release", "60", "integration", 1),
            ("release-test", "30", "unknown", 1),
        ):
            with self.subTest(profile=profile, timeout=timeout, target_group=target_group):
                result = run_script(
                    script,
                    {
                        "CARGO_PROFILE": profile,
                        "TIMEOUT_MINUTES": timeout,
                        "TARGET_GROUP": target_group,
                    },
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
            "target_group",
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

    def test_internal_workspace_crates_have_a_required_execution_gate(self) -> None:
        step = step_body(self.ci, "Test internal Rust crates")
        self.assertIn("cargo test --workspace", step)
        for package in (
            "rhwp",
            "rhwp-subsecond",
            "rhwp-native-ffi",
            "batch-convert",
        ):
            with self.subTest(package=package):
                self.assertIn(f"--exclude {package}", step)
        self.assertIn("--lib", step)

    def test_required_check_and_release_artifact_contracts_stay_stable(self) -> None:
        self.assertIn("name: Build & Test", self.ci)
        self.assertIn(
            "cargo build --release --bin rhwp --target ${{ matrix.target }}",
            self.release_binary,
        )
        self.assertIn("wasm-pack build --target web --release", self.ci)

    def test_duration_policy_is_pinned_for_prs_and_refreshed_only_from_devel(self) -> None:
        nextest = (REPO_ROOT / ".config/nextest.toml").read_text()

        self.assertIn('[profile.ci-duration-observation.junit]\npath = "junit.xml"', nextest)
        self.assertNotIn('path = "target/nextest/ci-duration-observation/junit.xml"', nextest)
        self.assertIn("resolve-nextest-duration-policy:", self.ci)
        self.assertIn("refresh-nextest-target-duration-data:", self.ci)
        self.assertIn("ci-metrics/nextest-target-durations", self.ci)
        self.assertIn("github.ref == 'refs/heads/devel'", self.ci)
        self.assertIn("duration_policy_sha:", self.builder)
        self.assertIn(
            "duration_policy_ref='ci-metrics/nextest-target-durations'",
            self.builder,
        )
        self.assertIn(
            'refs/heads/${duration_policy_ref}:refs/remotes/origin/${duration_policy_ref}',
            self.builder,
        )
        self.assertIn(
            'git merge-base --is-ancestor "${{ inputs.duration_policy_sha }}"',
            self.builder,
        )
        self.assertIn(
            'git show "${{ inputs.duration_policy_sha }}:nextest-target-duration-policy.json"',
            self.builder,
        )
        self.assertIn("duration_policy_source=metrics-ref", self.builder)
        self.assertIn(
            "duration_policy_source=fallback:metrics-policy-unavailable",
            self.builder,
        )
        self.assertNotIn(
            'git fetch --depth=1 origin "${{ inputs.duration_policy_sha }}"',
            self.builder,
        )


if __name__ == "__main__":
    unittest.main()
