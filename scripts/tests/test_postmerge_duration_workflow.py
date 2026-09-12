"""#7070: no validation workflow may subscribe to devel branch pushes."""
import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / '.github/workflows'

class PostmergeDurationWorkflowTests(unittest.TestCase):
    def test_devel_push_subscribers_are_metadata_only(self):
        # An allowlist makes newly introduced implicit/all-branch push triggers fail too.
        for path in WORKFLOWS.glob('*.yml'):
            text = path.read_text()
            match = re.search(r'^  push:(.*?)(?=^  [a-z_]+:|^\S|\Z)', text, re.M | re.S)
            if not match:
                continue
            block = match[1]
            if re.search(r'branches:\s*\[main\]', block) or ('tags:' in block and 'branches:' not in block):
                continue
            if path.name == 'oracle-public-advisory.yml':
                self.assertNotIn('devel', block)
                self.assertIn("'task_m100_*'", block)
                continue
            self.assertIn(path.name, ['refresh-nextest-duration.yml', 'close-issues-on-devel-push.yml'])

    def test_pr_checks_and_tag_release_are_preserved(self):
        for filename in ['ci.yml', 'codeql.yml', 'adapter-diff.yml', 'proptest-roundtrip.yml']:
            text = (WORKFLOWS / filename).read_text()
            self.assertIn('  pull_request:', text)
            self.assertIn('  workflow_dispatch:', text)
            if filename != 'ci.yml':
                self.assertNotIn('\n  push:', text)
        self.assertIn("tags: ['v*']", (WORKFLOWS / 'ci.yml').read_text())

    def test_refresh_has_only_metadata_job_and_no_ci_fallback(self):
        text = (WORKFLOWS / 'refresh-nextest-duration.yml').read_text()
        jobs = text.split('\njobs:\n', 1)[1]
        self.assertEqual(re.findall(r'^  ([\w-]+):', jobs, re.M), ['refresh-nextest-target-duration-data'])
        for forbidden in ['cargo ', 'npm ', 'workflow_run:', 'actions: write', 'trusted-postmerge-ci-reuse', 'gh run', 'workflow-dispatch', 'pull_request:']:
            self.assertNotIn(forbidden, text)
        self.assertIn('branches: [devel]', text)
        self.assertIn("steps.collect.outputs.ready == 'true'", text)
        self.assertIn('"${current_devel_sha}" != "${GITHUB_SHA}"', text)
        self.assertIn('HEAD:refs/heads/${METRICS_BRANCH}', text)
        self.assertIn('collectPostmergeDurations', text)
        collector = (ROOT / 'scripts/collect-postmerge-duration-data.mjs').read_text()
        for forbidden in ['createWorkflowDispatch', 'reRunWorkflow', 'execFile', 'extractall']:
            self.assertNotIn(forbidden, collector)
