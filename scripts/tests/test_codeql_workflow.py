"""CodeQL 보안 판정 재사용과 Rust no-prebuild의 장기 workflow 계약."""

from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CODEQL_WORKFLOW = REPO_ROOT / ".github/workflows/codeql.yml"


def job_body(workflow: str, job_name: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(job_name)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)",
        workflow,
    )
    if match is None:
        raise AssertionError(f"codeql.yml에 {job_name} job이 없다")
    return match.group(0)


class CodeQLWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = CODEQL_WORKFLOW.read_text(encoding="utf-8")
        script = cls.workflow.split("script: |\n", maxsplit=1)[1].split(
            "\n      # 기준선 병합을 fast-pass bridge로", maxsplit=1
        )[0]
        cls.preflight_script = "\n".join(
            line.removeprefix("            ") for line in script.splitlines()
        )
        language_step = cls.workflow.split(
            "      - name: Finalize CodeQL language selection\n", maxsplit=1
        )[1].split("\n      - name: Summarize CodeQL impact classification", maxsplit=1)[0]
        language_script = language_step.split("        run: |\n", maxsplit=1)[1]
        cls.language_script = "\n".join(
            line.removeprefix("          ") for line in language_script.splitlines()
        )

    def test_reused_result_requires_candidate_bound_security_check(self) -> None:
        workflow = self.workflow
        self.assertIn("github.rest.checks.listForRef", workflow)
        self.assertIn("ref: candidateSha", workflow)
        self.assertIn("check.app?.slug === 'github-advanced-security'", workflow)
        self.assertIn("check.name === 'CodeQL'", workflow)
        self.assertIn("check.head_sha === candidateSha", workflow)
        self.assertIn("workflowRun.run_started_at || workflowRun.created_at", workflow)
        self.assertIn("Date.parse(securityCheck.started_at)", workflow)
        self.assertIn("securityCheckStartedAt < runAttemptStartedAt", workflow)
        self.assertNotIn("check.created_at", workflow)
        self.assertNotIn("securityCheck.created_at", workflow)
        self.assertNotIn("check.started_at || check.created_at || 0", workflow)
        self.assertNotIn(
            "securityCheck.started_at || securityCheck.created_at || 0", workflow
        )
        self.assertNotIn("security-check-workflow-identity-mismatch", workflow)
        self.assertIn("missing-security-check:CodeQL:${candidateSha}", workflow)
        self.assertIn("security-check-not-completed:CodeQL:${securityCheck.status}", workflow)
        self.assertIn("security-check-not-green:CodeQL:${securityCheck.conclusion}", workflow)
        self.assertIn(
            "const allowedSecurityConclusions = new Set(['success', 'neutral']);",
            workflow,
        )
        self.assertIn(
            "!allowedSecurityConclusions.has(securityCheck.conclusion)", workflow
        )
        self.assertNotIn("securityCheck.conclusion !== 'success'", workflow)
        self.assertLess(
            workflow.index("!allowedSecurityConclusions.has(securityCheck.conclusion)"),
            workflow.index("return { state: 'green' };"),
        )

    def test_green_analyze_jobs_cannot_reuse_a_failed_security_check(self) -> None:
        outputs = self._run_preflight("failure")
        self.assertEqual(outputs["fast_pass"], "false")
        self.assertEqual(outputs["candidate_sha"], "code-candidate")
        self.assertEqual(
            outputs["reason"],
            "security-check-not-green:CodeQL:failure",
        )

    def test_green_analyze_jobs_and_early_security_check_remain_reusable(self) -> None:
        outputs = self._run_preflight("success")
        self.assertEqual(outputs["fast_pass"], "true")
        self.assertEqual(outputs["candidate_sha"], "code-candidate")
        self.assertEqual(outputs["reason"], "codeql-checks-green")

    def test_green_analyze_jobs_and_neutral_security_summary_remain_reusable(self) -> None:
        outputs = self._run_preflight("neutral")
        self.assertEqual(outputs["fast_pass"], "true")
        self.assertEqual(outputs["candidate_sha"], "code-candidate")
        self.assertEqual(outputs["reason"], "codeql-checks-green")

    def test_green_analyze_jobs_cannot_reuse_a_skipped_security_summary(self) -> None:
        outputs = self._run_preflight("skipped")
        self.assertEqual(outputs["fast_pass"], "false")
        self.assertEqual(outputs["candidate_sha"], "code-candidate")
        self.assertEqual(
            outputs["reason"],
            "security-check-not-green:CodeQL:skipped",
        )

    def test_security_check_from_an_earlier_run_attempt_is_not_reused(self) -> None:
        outputs = self._run_preflight(
            "success",
            run_started_at="2026-08-09T00:18:00Z",
            security_started_at="2026-08-09T00:15:00Z",
            security_completed_at="2026-08-09T00:15:02Z",
        )
        self.assertEqual(outputs["fast_pass"], "false")
        self.assertEqual(outputs["reason"], "no-green-codeql-candidate")

    def test_security_check_without_started_at_is_not_reused(self) -> None:
        outputs = self._run_preflight("success", security_started_at=None)
        self.assertEqual(outputs["fast_pass"], "false")
        self.assertEqual(outputs["reason"], "no-green-codeql-candidate")

    def test_preflight_uses_trusted_classifier_and_fails_closed_to_all_languages(
        self,
    ) -> None:
        workflow = self.workflow
        preflight = job_body(workflow, "preflight")
        self.assertIn("permissions:\n      actions: read", preflight)
        self.assertIn(
            "codeql_languages: ${{ steps.languages.outputs.codeql_languages "
            "|| 'javascript-typescript,python,rust' }}",
            workflow,
        )
        self.assertIn(
            "ref: ${{ github.event_name == 'pull_request' "
            "&& github.event.pull_request.base.sha || github.sha }}",
            workflow,
        )
        self.assertIn("sparse-checkout: scripts/ci-impact-classifier.cjs", workflow)
        self.assertIn("persist-credentials: false", workflow)
        self.assertIn("forceFullReason: 'non-pull-request'", workflow)
        self.assertIn("forceFullReason: 'collection-error'", workflow)
        self.assertIn("id: languages", workflow)
        self.assertIn("IMPACT_OUTCOME: ${{ steps.impact.outcome }}", workflow)
        self.assertIn("fail-closed:impact-unavailable", workflow)
        self.assertIn("fail-closed:invalid-codeql-languages", workflow)
        for selection in (
            "none",
            "javascript-typescript",
            "python",
            "rust",
            "javascript-typescript,python",
            "javascript-typescript,rust",
            "python,rust",
            "javascript-typescript,python,rust",
        ):
            self.assertIn(f"'{selection}')", workflow)

    def test_analysis_steps_follow_selection_while_check_names_stay_stable(self) -> None:
        analyze = job_body(self.workflow, "analyze")
        selected = (
            "contains(format(',{0},', env.SELECTED_LANGUAGES), "
            "format(',{0},', matrix.language))"
        )
        self.assertIn(
            "SELECTED_LANGUAGES: ${{ needs.preflight.outputs.codeql_languages "
            "|| 'javascript-typescript,python,rust' }}",
            analyze,
        )
        self.assertIn("language: [javascript-typescript, python, rust]", analyze)
        self.assertIn("name: Analyze (${{ matrix.language }})", analyze)
        self.assertIn("name: Skip unselected language", analyze)
        self.assertIn(f"if: ${{{{ !{selected} }}}}", analyze)
        self.assertGreaterEqual(analyze.count(f"if: ${{{{ {selected} }}}}"), 3)
        self.assertIn(
            "if: ${{ matrix.language == 'rust' && " + selected + " }}",
            analyze,
        )
        job_if = next(
            line.strip() for line in analyze.splitlines() if line.strip().startswith("if:")
        )
        self.assertNotIn("codeql_languages", job_if)

    def test_fast_pass_summary_marks_language_classification_not_applicable(
        self,
    ) -> None:
        summary = self.workflow.split(
            "      - name: Summarize CodeQL impact classification\n", maxsplit=1
        )[1].split("\n  analyze:", maxsplit=1)[0]
        self.assertIn("if [[ \"${FAST_PASS}\" == 'true' ]]; then", summary)
        self.assertIn("IMPACT_AUTHORITY='n/a (fast-pass)'", summary)
        self.assertIn("CODEQL_LANGUAGES='n/a (fast-pass)'", summary)
        self.assertIn("CLASSIFICATION_STATUS='n/a (fast-pass)'", summary)
        self.assertIn('IMPACT_REASON="fast-pass:${FAST_PASS_REASON}"', summary)

    def test_language_finalizer_preserves_valid_selection(self) -> None:
        outputs = self._run_language_finalizer(
            outcome="success",
            languages="javascript-typescript",
            status="classified",
            reason="classified:studio-unit",
        )
        self.assertEqual(outputs["codeql_languages"], "javascript-typescript")
        self.assertEqual(outputs["classification_status"], "classified")
        self.assertEqual(outputs["reason"], "classified:studio-unit")

    def test_language_finalizer_rejects_invalid_or_failed_classification(self) -> None:
        invalid = self._run_language_finalizer(
            outcome="success",
            languages="javascript-typescript,ruby",
            status="classified",
            reason="classified:unexpected",
        )
        unavailable = self._run_language_finalizer(
            outcome="failure",
            languages="javascript-typescript",
            status="classified",
            reason="classified:studio-unit",
        )
        inconsistent_full = self._run_language_finalizer(
            outcome="success",
            languages="rust",
            status="full",
            reason="fail-closed:unexpected",
        )
        for outputs, reason in (
            (invalid, "fail-closed:invalid-codeql-languages"),
            (unavailable, "fail-closed:impact-unavailable"),
            (inconsistent_full, "fail-closed:invalid-codeql-languages"),
        ):
            self.assertEqual(
                outputs["codeql_languages"],
                "javascript-typescript,python,rust",
            )
            self.assertEqual(outputs["classification_status"], "full")
            self.assertEqual(outputs["reason"], reason)

    def test_blocking_lane_uses_default_build_mode_without_manual_prebuild(self) -> None:
        analyze = job_body(self.workflow, "analyze")
        self.assertIn("language: [javascript-typescript, python, rust]", analyze)
        self.assertIn("languages: ${{ matrix.language }}", analyze)
        self.assertIn("security-events: write", analyze)
        self.assertIn("contents: read", analyze)
        self.assertIn("Perform CodeQL Analysis", analyze)
        self.assertNotIn("build-mode:", analyze)
        self.assertNotIn("actions/cache/", analyze)
        self.assertNotIn("cargo build", analyze)
        self.assertNotIn("rust-blocking-results", analyze)
        self.assertNotIn("actions/upload-artifact", analyze)

    def test_temporary_measurement_jobs_and_artifacts_are_absent(self) -> None:
        workflow = self.workflow
        self.assertNotIn("rust-no-prebuild-shadow:", workflow)
        self.assertNotIn("Rust no-prebuild shadow", workflow)
        self.assertNotIn("rust-no-prebuild-results", workflow)
        self.assertNotIn("rust-blocking-results", workflow)
        self.assertNotIn("rust-blocking-sarif-", workflow)
        self.assertNotIn("rust-no-prebuild-sarif-", workflow)

    def _run_preflight(
        self,
        security_conclusion: str,
        *,
        run_started_at: str = "2026-08-09T00:10:00Z",
        security_started_at: str | None = "2026-08-09T00:11:00Z",
        security_completed_at: str = "2026-08-09T00:11:02Z",
    ) -> dict[str, str]:
        harness = """
const outputs = {};
const endpoints = {
  listWorkflowRuns: Symbol('listWorkflowRuns'),
  listJobsForWorkflowRun: Symbol('listJobsForWorkflowRun'),
  listFiles: Symbol('listFiles'),
  listCommits: Symbol('listCommits'),
  listForRef: Symbol('listForRef'),
};
const commits = {
  'review-record': {
    parents: [{ sha: 'code-candidate' }],
    files: [{ filename: 'mydocs/working/review.md', status: 'modified' }],
  },
  'code-candidate': {
    parents: [{ sha: 'base-sha' }],
    files: [{ filename: 'src/lib.rs', status: 'modified' }],
  },
};
const github = {
  rest: {
    actions: {
      listWorkflowRuns: endpoints.listWorkflowRuns,
      listJobsForWorkflowRun: endpoints.listJobsForWorkflowRun,
    },
    pulls: {
      listFiles: endpoints.listFiles,
      listCommits: endpoints.listCommits,
    },
    checks: { listForRef: endpoints.listForRef },
    repos: {
      getCommit: async ({ ref }) => ({ data: commits[ref] }),
    },
  },
  paginate: async (endpoint, params) => {
    if (endpoint === endpoints.listFiles) {
      return [
        { filename: 'src/lib.rs', status: 'modified' },
        { filename: 'mydocs/working/review.md', status: 'modified' },
      ];
    }
    if (endpoint === endpoints.listCommits) {
      return [{ sha: 'code-candidate' }, { sha: 'review-record' }];
    }
    if (endpoint === endpoints.listWorkflowRuns) {
      return [{
        id: 3790,
        path: '.github/workflows/codeql.yml',
        event: 'pull_request',
        head_sha: 'code-candidate',
        head_branch: 'feature-3790',
        head_repository: { id: 7 },
        status: 'completed',
        conclusion: 'success',
        created_at: '2026-08-09T00:10:00Z',
        run_started_at: RUN_STARTED_AT,
        completed_at: '2026-08-09T00:20:00Z',
      }];
    }
    if (endpoint === endpoints.listJobsForWorkflowRun) {
      return [
        { name: 'Analyze (python)', completed_at: '2026-08-09T00:12:00Z' },
        {
          name: 'Analyze (javascript-typescript)',
          completed_at: '2026-08-09T00:14:00Z',
        },
        { name: 'Analyze (rust)', completed_at: '2026-08-09T00:19:00Z' },
      ].map((job) => ({
        ...job,
        status: 'completed',
        conclusion: 'success',
      }));
    }
    if (endpoint === endpoints.listForRef) {
      return [{
        name: 'CodeQL',
        app: { slug: 'github-advanced-security' },
        head_sha: params.ref,
        status: 'completed',
        conclusion: SECURITY_CONCLUSION,
        started_at: SECURITY_STARTED_AT,
        completed_at: SECURITY_COMPLETED_AT,
      }];
    }
    throw new Error('unexpected paginate endpoint');
  },
};
const context = {
  eventName: 'pull_request',
  repo: { owner: 'edwardkim', repo: 'rhwp' },
  payload: {
    pull_request: {
      number: 4310,
      created_at: '2026-08-09T00:00:00Z',
      base: { sha: 'base-sha' },
      head: { ref: 'feature-3790', repo: { id: 7 } },
    },
  },
};
const core = {
  setOutput: (name, value) => { outputs[name] = String(value); },
  info: () => {},
  warning: () => {},
};
(async () => {
PREFLIGHT_SCRIPT
})().then(() => {
  process.stdout.write(JSON.stringify(outputs));
}).catch((error) => {
  process.stderr.write(String(error.stack || error));
  process.exitCode = 1;
});
""".replace("SECURITY_CONCLUSION", json.dumps(security_conclusion)).replace(
            "RUN_STARTED_AT", json.dumps(run_started_at)
        ).replace("SECURITY_STARTED_AT", json.dumps(security_started_at)).replace(
            "SECURITY_COMPLETED_AT", json.dumps(security_completed_at)
        ).replace(
            "PREFLIGHT_SCRIPT", self.preflight_script
        )
        completed = subprocess.run(
            ["node"],
            input=harness,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        return json.loads(completed.stdout)

    def _run_language_finalizer(
        self,
        *,
        outcome: str,
        languages: str,
        status: str,
        reason: str,
    ) -> dict[str, str]:
        with tempfile.NamedTemporaryFile(mode="w+", encoding="utf-8") as output:
            env = os.environ.copy()
            env.update(
                {
                    "GITHUB_OUTPUT": output.name,
                    "IMPACT_OUTCOME": outcome,
                    "RAW_LANGUAGES": languages,
                    "RAW_STATUS": status,
                    "RAW_REASON": reason,
                }
            )
            completed = subprocess.run(
                ["bash"],
                input=self.language_script,
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            output.seek(0)
            return dict(
                line.rstrip("\n").split("=", maxsplit=1)
                for line in output
                if "=" in line
            )


if __name__ == "__main__":
    unittest.main()
