"""Protect the Chrome CI trust boundary and artifact producer/consumer contract."""
import pathlib
import json
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]


class ChromeExtensionWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        cls.seed = (ROOT / '.github/workflows/chrome-browser-cache.yml').read_text()

    def job(self, name):
        return re.search(rf'(?ms)^  {name}:\n.*?(?=^  [\w-]+:\n|\Z)', self.workflow).group(0)

    def test_browser_consumes_producer_id_without_rebuilding(self):
        producer = self.job('frontend-package-gates')
        consumer = self.job('chrome-extension-e2e')
        self.assertIn('chrome_dist_artifact_id: ${{ steps.upload-chrome-dist.outputs.artifact-id }}', producer)
        self.assertIn('id: upload-chrome-dist', producer)
        self.assertIn('chrome-extension-dist-${{ github.run_id }}-${{ github.run_attempt }}', producer)
        self.assertIn('artifact-ids: ${{ needs.frontend-package-gates.outputs.chrome_dist_artifact_id }}', consumer)
        self.assertIn('needs: [preflight, frontend-package-gates]', consumer)
        self.assertIn("needs.frontend-package-gates.result == 'success'", consumer)
        self.assertNotIn('wasm-pack', consumer)
        self.assertNotIn('npm run build', consumer)
        self.assertIn('timeout-minutes: 5', consumer)
        self.assertIn('node rhwp-chrome/e2e/run-ci.mjs', consumer)
        self.assertIn('CHROME_DEVEL_SANDBOX: /opt/google/chrome/chrome-sandbox', consumer)
        self.assertIn('test -u "$CHROME_DEVEL_SANDBOX"', consumer)
        self.assertIn("= '0:4755'", consumer)
        self.assertLess(consumer.index('Verify installed Linux sandbox helper'),
                        consumer.index('Run packaged Chrome journeys'))
        # This job receives a prebuilt dist and needs only the harness and its
        # three committed inputs; fetching the PDF corpus consumes its budget.
        sparse = re.search(r'sparse-checkout: \|\n(.*?)          sparse-checkout-cone-mode:', consumer, re.S)
        self.assertIsNotNone(sparse)
        self.assertEqual([line.strip() for line in sparse.group(1).splitlines() if line.strip()], [
            '/rhwp-chrome/', '/samples/hwp3-pagedef-1915.hwp',
            '/samples/hwpx_sample2.hwpx', '/samples/re-font-dotum-empty-hancom.hwp',
        ])
        for fixture in re.findall(r'/samples/[^\s]+', sparse.group(1)):
            self.assertTrue((ROOT / fixture.lstrip('/')).is_file())

    def test_pr_restores_exact_browser_cache_and_only_default_branch_seeds(self):
        consumer = self.job('chrome-extension-e2e')
        self.assertIn("PUPPETEER_SKIP_DOWNLOAD: 'true'", consumer)
        self.assertIn('puppeteer browsers install chrome', consumer)
        self.assertIn('actions/cache/restore@', consumer)
        self.assertNotIn('actions/cache/save@', consumer)
        self.assertNotIn('restore-keys:', consumer)
        self.assertIn('actions/cache/save@', self.seed)
        self.assertIn("github.ref == format('refs/heads/{0}', github.event.repository.default_branch)", self.seed)
        self.assertIn('ref: ${{ github.event.repository.default_branch }}', self.seed)
        self.assertIn('persist-credentials: false', self.seed)
        self.assertIn('workflow_dispatch:', self.seed)
        self.assertNotIn('pull_request:', self.seed)
        self.assertNotIn('push:', self.seed)
        key = "key: chrome-extension-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('rhwp-chrome/package-lock.json') }}"
        self.assertIn(key, consumer)
        self.assertEqual(self.seed.count(key), 2)

    def test_cache_promotion_verifies_installation_without_saving_shared_cache(self):
        verify, prepare = self.seed.split('  prepare:\n', maxsplit=1)
        self.assertIn('default: true', verify)
        self.assertIn('if: ${{ inputs.verify_only }}', verify)
        self.assertIn('puppeteer browsers install chrome', verify)
        self.assertNotIn('actions/cache/save@', verify)
        self.assertIn('!inputs.verify_only && github.ref', prepare)
        policy = json.loads((ROOT / 'scripts/workflow_promotion_policy.json').read_text())
        adapter = policy['workflows']['.github/workflows/chrome-browser-cache.yml']
        self.assertEqual(adapter['executionMode'], 'contracts-only')
        self.assertEqual(adapter['evidencePath'], '.github/workflows/ci.yml')
        self.assertEqual(adapter['requiredJobs'], ['Frontend package gates', 'Chrome extension E2E'])
        self.assertEqual(adapter['requiredSkippedJobs'], [])
        self.assertIn('scripts/tests/test_chrome_extension_workflow.py', self.job('frontend-package-gates'))

    def test_failure_artifact_is_allowlisted_and_success_only_has_summary(self):
        consumer = self.job('chrome-extension-e2e')
        diagnostic = consumer.split('- name: Upload minimal failure diagnostics')[1]
        self.assertIn('if: ${{ failure() }}', diagnostic)
        paths = re.search(r'path: \|\n(.*?)          retention-days:', diagnostic, re.S).group(1)
        self.assertEqual([line.strip() for line in paths.splitlines() if line.strip()], [
            'output/chrome-extension-e2e/*.json', 'output/chrome-extension-e2e/*.log',
            'output/chrome-extension-e2e/*.png',
        ])
        self.assertIn('retention-days: 7', diagnostic)
        self.assertNotIn('pull_request_target:', self.workflow)

    def test_classifier_fallback_forces_browser_and_package_lane(self):
        preflight = self.job('preflight')
        self.assertIn("chrome_extension_e2e_required: ${{ steps.chrome-impact.outputs.chrome_extension_e2e_required || 'true' }}", preflight)
        self.assertIn("== 'true' && 'package' || steps.impact.outputs.frontend_mode || 'package'", preflight)
        self.assertIn('expectedFileCount: context.payload.pull_request.changed_files', preflight)
        self.assertIn('scripts/chrome-extension-impact.cjs .ci-impact-input.json', preflight)
        self.assertIn("steps.finalize.outputs.fast_pass != 'true'", preflight)
        aggregate = self.job('build-and-test')
        self.assertIn('- chrome-extension-e2e', aggregate)
        self.assertIn("CHROME_EXTENSION_RESULT: ${{ needs['chrome-extension-e2e'].result }}", aggregate)


if __name__ == '__main__':
    unittest.main()
