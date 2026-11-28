"""Nextest archive profile·timeout·cache 운영 계약."""

from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import textwrap
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
        r"(?m)^(?:      - (?:name:|uses:)|  (?:[A-Za-z0-9_-]+:|#))\s*", body
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
            ("pull_request", "refs/pull/1/merge", "false", "release-test", "60"),
            ("push", "refs/heads/devel", "false", "release-test", "60"),
            ("workflow_dispatch", "refs/heads/feature", "false", "release-test", "60"),
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
        self.assertIn(
            "security_sweep_samples_json: ${{ "
            "steps.collect-impact.outputs.security_sweep_samples_json || '[]' }}",
            preflight,
        )

    def test_new_sample_security_sweep_list_flows_to_archive_workers(self) -> None:
        self.assertIn("security_sweep_samples_json:", self.runner)
        self.assertIn(
            "RHWP_SECURITY_SWEEP_SAMPLES_JSON: "
            "${{ inputs.security_sweep_samples_json }}",
            self.runner,
        )
        for job_name in (
            "test-archive-a-shard-1",
            "test-archive-b-shard-1",
            "test-archive-c-shard-1",
            "test-archive-d-shard-1",
            "test-archive-e-shard-1",
        ):
            with self.subTest(job=job_name):
                worker = job_body(self.ci, job_name)
                self.assertIn(
                    "security_sweep_samples_json: ${{ "
                    "needs.preflight.outputs.security_sweep_samples_json || '[]' }}",
                    worker,
                )

    def test_archive_builders_split_lib_and_four_integration_targets(self):
        from pathlib import Path
        root = Path(__file__).resolve().parents[2]
        ci = (root / ".github/workflows/ci.yml").read_text()
        builder = (root / ".github/workflows/build-nextest-archives.yml").read_text()
        runner = (root / ".github/workflows/run-nextest-archives.yml").read_text()
        self.assertIn("build-test-archive-a:", ci); self.assertIn("build-test-archive-b:", ci); self.assertIn("build-test-archive-c:", ci); self.assertIn("build-test-archive-d:", ci)
        self.assertNotIn("test-slow-shard:", ci); self.assertNotIn('partition: "hash:1/2"', ci); self.assertNotIn('partition: "hash:2/2"', ci); self.assertEqual(5, ci.count('partition: "hash:1/1"'))
        self.assertIn("target_group: lib", ci); self.assertIn("target_group: integration-b", ci); self.assertIn("target_group: integration-c", ci); self.assertIn("target_group: integration-d", ci)
        self.assertIn("cargo metadata --no-deps --format-version 1", builder)
        self.assertIn("scripts/select-nextest-archive-targets.mjs", builder)
        self.assertIn(
            "duration_policy='tests/suites/nextest-target-duration-policy.json'",
            builder,
        )
        self.assertIn('--policy "${duration_policy}"', builder)
        self.assertIn('--manifest tests/suites/manifest.json', builder)
        self.assertIn('--rebalance-by-duration "${duration_policy}"', builder)
        self.assertNotIn("index % 2", builder)
        self.assertIn("cargo_target_args+=(--lib)", builder)
        self.assertIn('cargo_target_args+=(--test "${target}")', builder)
        self.assertNotIn("--tests", builder)
        self.assertIn("--profile ci-duration-observation", runner)
        self.assertIn("scripts/collect-nextest-target-durations.mjs", runner)
        self.assertIn("nextest-target-durations-${{ github.run_id }}-${{ inputs.archive_label }}", runner)

    def test_duration_policy_collects_devel_and_same_repository_pr_b_c_d_measurements(self):
        from pathlib import Path
        root = Path(__file__).resolve().parents[2]
        policy = (root / "tests/suites/nextest-target-duration-policy.json").read_text()
        config = (root / ".config/nextest.toml").read_text()
        runner = (root / ".github/workflows/run-nextest-archives.yml").read_text()
        selector = (root / "scripts/select-nextest-archive-targets.mjs").read_text()
        collector = (root / "scripts/collect-nextest-target-durations.mjs").read_text()
        refresh = (root / "scripts/refresh-nextest-target-duration-policy.mjs").read_text()

        self.assertIn('"schema_version": 2', policy)
        self.assertIn('"fallback_seconds_per_test": 5', policy)
        self.assertIn('"parallelism_factor": 4', policy)
        self.assertIn('"cases": {}', policy)
        self.assertIn('"test_cases": {}', policy)
        self.assertIn("[profile.ci-duration-observation.junit]", config)
        self.assertIn("github.event_name == 'push'", runner)
        self.assertIn("github.ref == 'refs/heads/devel'", runner)
        self.assertIn("github.event_name == 'pull_request'", runner)
        self.assertIn(
            "github.event.pull_request.base.repo.id == github.repository_id",
            runner,
        )
        self.assertIn("github.repository == 'edwardkim/rhwp'", runner)
        self.assertIn("github.event.pull_request.base.ref == 'devel'", runner)
        self.assertIn("-attempt-${{ github.run_attempt }}-", runner)
        self.assertNotIn("github.event.pull_request.head.repo.full_name == github.repository", runner)
        self.assertIn("inputs.archive_label == 'b'", runner)
        self.assertIn("inputs.archive_label == 'c'", runner)
        self.assertIn("inputs.archive_label == 'd'", runner)
        self.assertIn("retention-days: 3", runner)
        self.assertIn("retention-days: 30", runner)
        self.assertIn("estimatedSeconds", selector)
        self.assertIn("estimatedWallSeconds", selector)
        self.assertIn("maxTestcaseSeconds", selector)
        self.assertIn("parallelismFactor", selector)
        self.assertIn("<testcase", collector)
        self.assertIn("JUnit report contains no target durations", collector)
        self.assertIn("collectDurationMeasurement", collector)
        self.assertIn("case_durations", collector)
        self.assertIn("const refreshed = refreshDurationPolicy(policy, measurements)", refresh)
        self.assertIn("identical run, ref, and sha provenance", refresh)

    def test_native_skia_uses_release_test_for_tag_fallback_only(self) -> None:
        native = job_body(self.ci, "native-skia-tests")
        self.assertIn(
            "TEST_PROFILE: ${{ github.event_name == 'push' && "
            "startsWith(github.ref, 'refs/tags/v') && 'release-test' || "
            "needs.preflight.outputs.test_profile || 'release' }}",
            native,
        )
        self.assertIn("needs.preflight.outputs.native_skia_reuse_run_id == ''", native)
        step = step_body(self.ci, "Native Skia tests")
        self.assertIn('case "${TEST_PROFILE}" in', step)
        self.assertIn("release-test)", step)
        self.assertIn("release)", step)
        self.assertIn("Unknown test profile", step)
        self.assertNotIn('"${GITHUB_EVENT_NAME}" == "pull_request"', step)

    def test_native_skia_promotion_receipt_follows_the_test(self) -> None:
        native = job_body(self.ci, "native-skia-tests")
        capture = step_body(native, "Record Native Skia promotion evidence")
        self.assertIn("github.base_ref == 'main'", capture)
        self.assertIn("github.head_ref == 'devel'", capture)
        self.assertIn("github.event.pull_request.head.repo.full_name == github.repository", capture)
        self.assertLess(native.index("- name: Native Skia tests"), native.index(capture))
        upload = step_body(native, "Upload Native Skia promotion evidence")
        self.assertIn("steps.native-skia-promotion-evidence.outputs.artifact_name", upload)
        self.assertIn("if-no-files-found: error", upload)
        script = textwrap.dedent(capture.split("        run: |\n", 1)[1])
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "output"
            sha, tree = subprocess.check_output(
                ["git", "rev-parse", "HEAD", "HEAD^{tree}"],
                cwd=REPO_ROOT, text=True,
            ).splitlines()
            env = {
                "GITHUB_OUTPUT": str(output), "GITHUB_SHA": sha,
                "PR_NUMBER": "123", "GITHUB_RUN_ATTEMPT": "2", "RUNNER_TEMP": directory,
            }
            result = run_script(script, env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(output.read_text(), f"artifact_name=native-skia-promotion-v1-123-2-{sha}-{tree}\n")
            result = run_script(script, {**env, "GITHUB_SHA": "0" * 40})
            self.assertNotEqual(result.returncode, 0)

    def test_native_skia_tag_requires_exact_successful_promotion_evidence(self) -> None:
        step = step_body(job_body(self.ci, "preflight"), "Verify release tag Native Skia evidence")
        self.assertIn("github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')", step)
        script_body = textwrap.dedent(step.split("          script: |\n", 1)[1])
        driver = r"""
            import assert from 'node:assert/strict';
            const verify = new (Object.getPrototypeOf(async function () {}).constructor)(
              'github', 'context', 'core', SCRIPT_BODY,
            );
            const taggedSha = 'a'.repeat(40);
            const testedSha = 'b'.repeat(40);
            const tree = 'c'.repeat(40);
            const baseSha = 'd'.repeat(40);
            const headSha = 'e'.repeat(40);
            const repository = { id: 42 };
            const ref = (name, sha) => ({ ref: name, sha, repo: repository });
            const beforeMerge = '2026-11-18T14:00:00Z';
            const mergedAt = '2026-11-18T15:00:00Z';
            const afterMerge = '2026-11-18T16:00:00Z';
            const context = { sha: taggedSha, repo: { owner: 'test', repo: 'rhwp' } };
            const data = (value) => ({ data: value });
            const successful = { status: 'completed', conclusion: 'success' };

            function fixture() {
              const commit = (sha) => ({ sha, tree: { sha: tree }, parents: [{ sha: baseSha }, { sha: headSha }] });
              const step = (name) => ({ ...successful, name });
              return {
                tagged: commit(taggedSha),
                tested: commit(testedSha),
                pulls: [{
                  number: 123, state: 'closed', merged_at: mergedAt, merge_commit_sha: taggedSha,
                  base: ref('main', baseSha), head: ref('devel', headSha),
                }],
                run: {
                  ...successful, id: 456, run_attempt: 2, event: 'pull_request',
                  path: '.github/workflows/ci.yml', head_branch: 'devel', head_sha: headSha,
                  repository, head_repository: repository, created_at: beforeMerge, pull_requests: [],
                },
                jobs: ['Native Skia tests', 'Build & Test', 'Workflow promotion preflight'].map((name) => ({
                  ...successful, name, run_id: 456, head_sha: headSha, completed_at: beforeMerge,
                  steps: [step('Native Skia tests'), step('Upload Native Skia promotion evidence')],
                })),
                artifacts: [{
                  name: `native-skia-promotion-v1-123-2-${testedSha}-${tree}`,
                  expired: false, created_at: beforeMerge,
                }],
              };
            }

            async function verifyCase(label, change, shouldReuse = false) {
              const evidence = fixture();
              change(evidence);
              const output = {};
              const github = {
                rest: {
                  git: { getCommit: async ({ commit_sha }) => {
                    if (evidence.apiError) throw new Error('API unavailable');
                    if (commit_sha === taggedSha) return data(evidence.tagged);
                    assert.equal(commit_sha, testedSha);
                    return data(evidence.tested);
                  } },
                  pulls: { list: () => evidence.pulls },
                  actions: {
                    listWorkflowRuns: async (params) => {
                      assert.equal(params.event, 'pull_request');
                      assert.equal(params.workflow_id, 'ci.yml');
                      assert.equal(params.head_sha, headSha);
                      return data({ workflow_runs: [evidence.run] });
                    },
                    listWorkflowRunArtifacts: () => evidence.artifacts,
                  },
                },
                paginate: async (request, params) => {
                  if (typeof request === 'function') return request();
                  assert.match(request, /\/attempts\/\{attempt_number\}\/jobs$/);
                  assert.equal(params.run_id, 456);
                  assert.equal(params.attempt_number, evidence.run.run_attempt);
                  return evidence.jobs;
                },
              };
              await verify(github, context, {
                notice: () => {},
                setOutput: (key, value) => { output[key] = value; },
              });
              assert.deepEqual(output, shouldReuse ? { source_run_id: '456' } : {}, label);
            }

            await verifyCase('exact promotion', () => {}, true);
            await verifyCase('unassociated tag', (f) => { f.pulls = []; });
            await verifyCase('foreign PR', (f) => { f.pulls[0].head.repo = { id: 1 }; });
            await verifyCase('foreign run', (f) => { f.run.head_repository = { id: 1 }; });
            await verifyCase('failed run', (f) => { f.run.conclusion = 'failure'; });
            await verifyCase('skipped native job', (f) => { f.jobs[0].conclusion = 'skipped'; });
            await verifyCase('skipped tests in green native job', (f) => { f.jobs[0].steps[0].conclusion = 'skipped'; });
            await verifyCase('missing promotion check', (f) => { f.jobs.pop(); });
            await verifyCase('late aggregate', (f) => { f.jobs[1].completed_at = afterMerge; });
            await verifyCase('missing receipt', (f) => { f.artifacts = []; });
            await verifyCase('expired receipt', (f) => { f.artifacts[0].expired = true; });
            await verifyCase('different attempt', (f) => { f.run.run_attempt = 3; });
            await verifyCase('different tree', (f) => { f.tested.tree.sha = 'f'.repeat(40); });
            await verifyCase('different tested parent', (f) => { f.tested.parents.reverse(); });
            await verifyCase('API error', (f) => { f.apiError = true; });
        """
        result = subprocess.run(
            ["node", "--input-type=module"],
            input=driver.replace("SCRIPT_BODY", json.dumps(script_body)),
            env={**os.environ, "GITHUB_REPOSITORY_ID": "42"},
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_build_and_test_accepts_native_skia_reuse_only_for_a_release_tag(self) -> None:
        step = step_body(self.ci, "Check Build & Test worker results")
        script = textwrap.dedent(step.split("        run: |\n", 1)[1])
        worker_names = re.findall(r"^          ([A-Z0-9_]+_RESULT):", step, re.M)
        base_env = {
            **dict.fromkeys(worker_names, "success"),
            "PREFLIGHT_RESULT": "success", "FAST_PASS": "false",
            "RUST_REQUIRED": "true", "NATIVE_SKIA_REQUIRED": "true",
            "FRONTEND_MODE": "package", "FRONTEND_UNIT_RESULT": "skipped",
            "CANONICAL_PROMOTION": "false", "PROMOTION_RESULT": "skipped",
        }
        for event, ref, receipt, native, expected in (
            ("push", "refs/tags/v1.2.3", "456", "skipped", 0),
            ("push", "refs/tags/v1.2.3", "456", "failure", 1),
            ("push", "refs/tags/v1.2.3", "invalid", "skipped", 1),
            ("push", "refs/tags/v1.2.3", "", "success", 0),
            ("push", "refs/tags/v1.2.3", "", "skipped", 1),
            ("pull_request", "refs/pull/123/merge", "456", "skipped", 1),
            ("workflow_dispatch", "refs/tags/v1.2.3", "456", "skipped", 1),
        ):
            with self.subTest(event=event, receipt=receipt, native=native):
                result = run_script(script, {
                    **base_env, "GITHUB_EVENT_NAME": event, "GITHUB_REF": ref,
                    "NATIVE_SKIA_RESULT": native, "NATIVE_SKIA_REUSE_RUN_ID": receipt,
                })
                self.assertEqual(result.returncode, expected, result.stdout + result.stderr)

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
        for name in ("test-archive-a-shard-1:", "test-archive-b-shard-1:", "test-archive-c-shard-1:", "test-archive-d-shard-1:"):
            self.assertIn(name, ci)
        self.assertIn("Archive A shard total mismatch", ci); self.assertIn("Archive B shard total mismatch", ci); self.assertIn("Archive C shard total mismatch", ci); self.assertIn("Archive D shard total mismatch", ci)
        self.assertIn("|| 'Build & Test' }}", ci)

    def test_reusable_builder_rejects_profile_timeout_mismatches(self) -> None:
        step = step_body(self.builder, "Validate test archive policy")
        script = step.split("        run: |\n", maxsplit=1)[1]
        script = "\n".join(line.removeprefix("          ") for line in script.splitlines())

        for profile, timeout, target_group, expected in (
            ("release-test", "60", "lib", 0),
            ("release", "60", "integration-b", 0),
            ("release", "60", "integration-c", 0),
            ("release", "60", "integration-d", 0),
            ("release-test", "30", "lib", 1),
            ("release", "30", "integration-b", 1),
            ("debug", "60", "lib", 1),
            ("release", "60", "integration", 1),
            ("release-test", "60", "unknown", 1),
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
        self.assertIn("|| 'Build & Test' }}", self.ci)
        self.assertIn(
            "cargo build --locked --release --bin rhwp --features native-skia "
            "--target ${{ matrix.target }}",
            self.release_binary,
        )
        self.assertIn("wasm-pack build --target web --release", self.ci)

    def test_duration_policy_is_pinned_for_prs_and_refreshed_only_from_devel(self) -> None:
        nextest = (REPO_ROOT / ".config/nextest.toml").read_text()

        self.assertIn('[profile.ci-duration-observation.junit]\npath = "junit.xml"', nextest)
        self.assertNotIn('path = "target/nextest/ci-duration-observation/junit.xml"', nextest)
        self.assertIn("resolve-nextest-duration-policy:", self.ci)
        refresh = (REPO_ROOT / ".github/workflows/refresh-nextest-duration.yml").read_text()
        self.assertNotIn("refresh-nextest-target-duration-data:", self.ci)
        self.assertIn("refresh-nextest-target-duration-data:", refresh)
        # 게시 대상은 workflow가 호출하는 API publisher에서 고정한다.
        publisher = (REPO_ROOT / "scripts/publish-nextest-duration-data.mjs").read_text()
        self.assertIn("scripts/publish-nextest-duration-data.mjs", refresh)
        self.assertIn("heads/ci-metrics/nextest-target-durations", publisher)
        self.assertIn("force: false", publisher)
        self.assertIn("github.ref == 'refs/heads/devel'", refresh)
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

    def test_worker_has_finite_sixty_minute_limit(self):
        self.assertIn("    timeout-minutes: 60", self.runner)
        self.assertNotIn("    timeout-minutes: 30", self.runner)

    def test_e_count_is_actually_checked(self):
        script = step_body(self.ci, "Verify archive shard totals").split("        run: |\n", 1)[1]
        script = "\n".join(line.removeprefix("          ") for line in script.splitlines())
        for value in ("5", "4", "invalid", None):
            with self.subTest(e_count=value), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for label in "abcde":
                    expected = root / f"archive-expected-{label}"
                    expected.mkdir()
                    (expected / "archive-expected-total.txt").write_text("5\n")
                    actual = root / f"shard-counts/shard-count-{label}-1"
                    actual.mkdir(parents=True)
                    content = value if label == "e" else "5"
                    if content is not None:
                        (actual / f"shard-count-{label}-1.txt").write_text(content + "\n")
                result = run_script(f'cd "{directory}"\n' + script, {})
                self.assertEqual(result.returncode == 0, value == "5", result.stdout + result.stderr)

    def test_e_builder_and_worker_cannot_escape_aggregate(self):
        step = step_body(self.ci, "Check Build & Test worker results")
        script = step.split("        run: |\n", 1)[1]
        script = "\n".join(line.removeprefix("          ") for line in script.splitlines())
        env = {"CANONICAL_PROMOTION": "false", "PROMOTION_RESULT": "skipped",
               "PREFLIGHT_RESULT": "success", "FAST_PASS": "false", "RUST_REQUIRED": "true",
               "NATIVE_SKIA_REQUIRED": "false", "NATIVE_SKIA_RESULT": "skipped",
               "FRONTEND_MODE": "none", "FRONTEND_UNIT_RESULT": "skipped",
               "FRONTEND_PACKAGE_RESULT": "skipped", "LINT_RESULT": "success"}
        for label in "ABCDE":
            env[f"BUILD_ARCHIVE_{label}_RESULT"] = "success"
            env[f"TEST_ARCHIVE_{label}_1_RESULT"] = "success"
        self.assertEqual(run_script(script, env).returncode, 0)
        for variable in ("BUILD_ARCHIVE_E_RESULT", "TEST_ARCHIVE_E_1_RESULT"):
            for conclusion in ("failure", "cancelled", "skipped", ""):
                with self.subTest(variable=variable, conclusion=conclusion):
                    self.assertNotEqual(run_script(script, {**env, variable: conclusion}).returncode, 0)


if __name__ == "__main__":
    unittest.main()
