#!/usr/bin/env node
// Synthetic examples only: never represent an actual browser execution.
import { mkdir, writeFile, rm } from 'node:fs/promises';
import path from 'node:path';
import { runChecks } from './run.mjs';
import { generateReport } from './report.mjs';

if (process.argv.length !== 4 || process.argv[2] !== '--output') throw new Error('Usage: node rhwp-chrome/e2e/report-fixtures.mjs --output NEW_DIRECTORY');
const output = path.resolve(process.argv[3]);
await mkdir(output); // Existing directory is an error, never overwrite evidence.
const candidate = path.join(output, 'synthetic-candidate');
await mkdir(path.join(candidate, 'wasm'), { recursive: true });
for (const [file, text] of Object.entries({ 'manifest.json': '{"manifest_version":3,"version":"0.0.0"}', 'viewer.html': 'synthetic fixture', 'wasm/rhwp.js': 'synthetic', 'wasm/rhwp_bg.wasm': 'not an executable WASM' })) await writeFile(path.join(candidate, file), text);
const environment = async () => ({ node: process.version, platform: process.platform, arch: process.arch,
  puppeteer: '0.0.0', chrome: '0.0.0', harnessSha: '0'.repeat(40), harnessDirty: true, executable: '/PRIVATE/fixture/chrome' });
const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=', 'base64');
for (const status of ['pass', 'fail', 'error', 'not-run', 'incomplete']) {
  const input = path.join(output, `${status}-run`);
  const { report } = await runChecks({ dist: candidate, output: input }, {
    env: {}, environment: status === 'error' ? async () => { throw new Error('Synthetic preflight error /PRIVATE/path'); } : environment,
    runner: async (_args, { env }) => {
      if (status === 'fail') {
        await writeFile(path.join(input, 'smoke.json'), JSON.stringify({ runId: env.RHWP_EXTENSION_E2E_RUN_ID,
          stage: 'options-save', error: 'Synthetic failure', downloads: [{ id: 98765, filename: '/PRIVATE/document.hwp' }] }));
        await writeFile(path.join(input, 'smoke-0.png'), png);
      }
      return { status: status === 'fail' ? 7 : 0, timedOut: false, signal: null, durationMs: 1, log: 'Synthetic suite /PRIVATE/profile downloadId=98765' };
    },
  });
  if (status === 'not-run') {
    report.status = 'running'; delete report.finishedAt;
    report.results = report.results.map(({ name }) => ({ name, status: 'not-run' }));
    for (const name of ['extension-smoke', 'download-interceptor', 'extension-lifecycle']) await rm(path.join(input, `${name}.log`));
    await writeFile(path.join(input, 'result.json'), JSON.stringify(report));
  }
  if (status === 'incomplete') await rm(path.join(input, 'extension-smoke.log'));
  const summary = await generateReport({ input, output: path.join(output, `${status}-report`) });
  console.log(JSON.stringify({ fixture: status, sourceStatus: summary.sourceStatus, status: summary.status, issues: summary.issues }));
}
await writeFile(path.join(output, 'README.txt'), '합성 fixture입니다. 실제 브라우저 검사·배포 후보·실제 tool SHA의 증거가 아닙니다. 각 *-report/report.html을 비교하세요. private.html은 공유하지 마세요.\n');
