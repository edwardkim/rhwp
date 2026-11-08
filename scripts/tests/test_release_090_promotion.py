"""Release 0.9.0 must not accept skipped execution or uncovered workflow changes."""
import copy
import json
from pathlib import Path
import unittest

from scripts.layout_advisory_verdict import verdict
from scripts import workflow_promotion_preflight as promotion

ROOT = Path(__file__).resolve().parents[2]


class Release090PromotionTests(unittest.TestCase):
    def test_new_release_surfaces_are_covered_without_waivers(self):
        policy = json.loads((ROOT / 'scripts/workflow_promotion_policy.json').read_text())
        paths = ['.github/actions/verify-release-wasm/action.yml'] + [
            f'.github/workflows/{name}.yml' for name in (
                'ci-impact-policy', 'full-renderer-sweep', 'fuzz-smoke',
                'layout-anomaly-advisory', 'skill-router-gate')]
        inventory = {'schemaVersion': 1, 'entries': [
            {'path': path, 'classification': 'executable', 'after': {'sha256': 'a' * 64}}
            for path in paths], 'policyViolations': []}
        covered = promotion.apply_execution_policy(inventory, policy)
        self.assertEqual(covered['policyViolations'], [])
        for item in covered['entries']:
            self.assertTrue(item['requiredJobs'])
            self.assertEqual(item['allowedEvents'], ['workflow_dispatch'])
            self.assertEqual(item['allowedActors'], ['edwardkim'])

    def test_release_composite_requires_real_caller_artifacts_and_no_publication(self):
        policy = json.loads((ROOT / 'scripts/workflow_promotion_policy.json').read_text())['workflows']
        entry = policy['.github/actions/verify-release-wasm/action.yml']
        self.assertEqual(entry['evidencePath'], '.github/workflows/npm-publish.yml')
        self.assertEqual(entry['executionMode'], 'verify-only')
        self.assertIn('Build WASM', entry['requiredJobs'])
        self.assertIn('release-wasm-validation', entry['requiredArtifacts'])
        self.assertIn('Publish @rhwp/core', entry['requiredSkippedJobs'])
        self.assertIn('uses: ./.github/actions/verify-release-wasm',
                      (ROOT / entry['evidencePath']).read_text())

    def test_privileged_controller_uses_read_only_executable_contracts(self):
        policy = json.loads((ROOT / 'scripts/workflow_promotion_policy.json').read_text())['workflows']
        entry = policy['.github/workflows/ci-impact-policy.yml']
        self.assertEqual(entry['executionMode'], 'contracts-only')
        self.assertEqual(entry['evidencePath'], '.github/workflows/ci.yml')
        self.assertIn('release-operations-contracts / Release operations contracts', entry['requiredJobs'])
        adapter = (ROOT / '.github/workflows/release-operations-contracts.yml').read_text()
        self.assertIn('node --test scripts/tests/ci-impact-*.test.cjs', adapter)
        self.assertIn('scripts.tests.test_ci_impact_policy_workflow', adapter)
        self.assertNotRegex(adapter, r'(?m)^\s+[a-z-]+: write$')

    def test_advisory_success_alone_is_not_execution_evidence(self):
        policy = json.loads((ROOT / 'scripts/workflow_promotion_policy.json').read_text())['workflows']
        entry = policy['.github/workflows/layout-anomaly-advisory.yml']
        self.assertEqual(entry['requiredVerdictArtifact'], {
            'name': 'layout-anomaly-advisory-verdict',
            'requiredPath': 'verdict.json', 'acceptedVerdicts': ['completed']})
        workflow = (ROOT / '.github/workflows/layout-anomaly-advisory.yml').read_text()
        self.assertIn('scripts/layout_advisory_verdict.py', workflow)
        self.assertIn('name: layout-anomaly-advisory-verdict', workflow)
        self.assertIn('continue-on-error: true', workflow)  # findings stay advisory
        for path in ('vendor', 'saved/blank2010.hwp', 'mydocs/manual/recipes',
                     'mydocs/manual/agent_knowledge_map.md', 'mydocs/tech/agent_roadmap/atlas_r1_r200.md'):
            self.assertIn(f'            {path}\n', workflow)


class AdvisoryExecutionTests(unittest.TestCase):
    def setUp(self):
        self.report = {
            'schema': 'layout_anomaly.advisory_report/v1',
            'summary': {'docs': 2, 'clean': 1, 'anomaly': 1, 'error': 0, 'skipped': 0},
            'rows': [{'verdict': value, 'exit': 0, 'parseError': '', 'pageCount': 1}
                     for value in ('CLEAN', 'ANOMALY')], 'skipped': []}

    def check(self, report, **kwargs):
        return verdict(report, **{'build_outcome': 'success', 'scan_exit': '0', **kwargs})['verdict']

    def test_completed_scan_accepts_anomalies_without_quality_waiver(self):
        self.assertEqual(self.check(self.report), 'completed')

    def test_missing_empty_and_malformed_reports_fail_closed(self):
        for report in (None, [], {}, {'schema': 'wrong'},
                       {**self.report, 'rows': []}, {**self.report, 'rows': [None]},
                       {**self.report, 'summary': {'docs': True}}):
            with self.subTest(report=report):
                self.assertEqual(self.check(report), 'incomplete')

    def test_skipped_samples_parser_errors_and_stale_counts_fail_closed(self):
        for key, value in [('skipped', ['missing.hwp']),
                           ('rows', [{'verdict': 'ERROR'}, {'verdict': 'ANOMALY'}]),
                           ('rows', [{'verdict': 'CLEAN'}])]:
            report = copy.deepcopy(self.report)
            report[key] = value
            self.assertEqual(self.check(report), 'incomplete')

    def test_build_failure_and_timeout_cannot_reuse_old_report(self):
        for args in ({'build_outcome': 'failure'}, {'build_outcome': 'skipped'},
                     {'scan_exit': '124'}, {'scan_exit': ''}, {'scan_exit': '1'}):
            self.assertEqual(self.check(self.report, **args), 'incomplete')

    def test_invalid_parser_output_is_not_a_clean_scan(self):
        for change in ({'pageCount': None}, {'pageCount': 0}, {'pageCount': True},
                       {'parseError': 'invalid JSON'}, {'exit': 1}):
            report = copy.deepcopy(self.report)
            report['rows'][0].update(change)
            self.assertEqual(self.check(report), 'incomplete')


if __name__ == '__main__':
    unittest.main()
