#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { mkdir, readFile, readdir, lstat, realpath, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { suites } from './run.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const hex = (value, length) => typeof value === 'string' && new RegExp(`^[a-f0-9]{${length}}$`).test(value);
const uuid = value => typeof value === 'string' && /^[a-f0-9]{8}(-[a-f0-9]{4}){3}-[a-f0-9]{12}$/.test(value);
const date = value => typeof value === 'string' && /^\d{4}-\d\d-\d\dT[\d:.]+Z$/.test(value) && Number.isFinite(Date.parse(value));
const escape = value => String(value).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);
// Only version/platform tokens cross the sharing boundary; no free-form text.
const token = value => typeof value === 'string' && /^[a-zA-Z0-9][a-zA-Z0-9 ._()+-]{0,79}$/.test(value) ? value : null;
const inside = (parent, child) => { const rel = path.relative(parent, child); return !rel || (!rel.startsWith('..') && !path.isAbsolute(rel)); };
const limitations = ['Unpacked Chrome for Testing only', '스토어 설치·업데이트, Edge/Firefox 미검증',
  '실제 사용자 환경·문서 표시·인쇄 품질 미검증', '자동 검사 결과는 배포 승인이 아님'];

export function parseReportArgs(args) {
  const options = {};
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--help') return { help: true };
    if (!['--input', '--output', '--expect-candidate-sha256', '--expect-tool-sha', '--manual-review'].includes(args[i]) || !args[i + 1] || args[i + 1].startsWith('--')) throw new Error('Expected --input RUN_DIRECTORY --output NEW_DIRECTORY [--expect-candidate-sha256 HASH] [--expect-tool-sha SHA] [--manual-review JSON]');
    const key = args[i].slice(2);
    if (options[key]) throw new Error('Duplicate option');
    const value = args[++i];
    options[key] = ['input', 'output', 'manual-review'].includes(key) ? path.resolve(value) : value;
  }
  if (!options.input || !options.output) throw new Error('--input and --output are required');
  return options;
}

export async function inspectRun(input, options = {}) {
  const issues = [];
  const problem = code => { if (!issues.includes(code)) issues.push(code); };
  const source = await readFile(path.join(input, 'result.json'));
  const summary = { reportVersion: 1, sourceSha256: hash(source), sourceStatus: 'unknown', status: 'incomplete',
    sourceSchemaVersion: null, evidenceVersion: null, runId: null, candidate: null, tool: null, environment: {}, startedAt: null, finishedAt: null,
    results: suites.map(name => ({ name, status: 'not-run', evidence: [] })), issues,
    limitations, manualReview: { status: '미확인', readability: '미확인', sharingExclusions: '미확인' },
    excluded: ['후보·브라우저 절대 경로', '다운로드 식별자·URL', '원본 로그·진단 JSON·PNG·자유 텍스트', '사용자 프로필·원문 문서(수집하지 않음)'] };
  const links = [];
  let evidenceNumber = 0;
  let data;
  try { data = JSON.parse(source); } catch { problem('result-json-parse-error'); return { summary, links }; }
  if (!data || typeof data !== 'object' || Array.isArray(data)) { problem('invalid-result'); return { summary, links }; }
  if (['pass', 'fail', 'error', 'running'].includes(data.status)) summary.sourceStatus = data.status;
  else problem('invalid-status');
  summary.sourceSchemaVersion = Number.isSafeInteger(data.schemaVersion) ? data.schemaVersion : null;
  summary.evidenceVersion = data.evidenceVersion === 1 ? 1 : null;
  if (data.schemaVersion !== 1) { problem('unsupported-schemaVersion'); return { summary, links }; }
  if (uuid(data.runId)) summary.runId = data.runId;
  if (data.evidenceVersion !== 1 || !summary.runId) problem('unbound-legacy-or-invalid-evidence');
  if (date(data.startedAt)) summary.startedAt = data.startedAt; else problem('invalid-startedAt');
  if (date(data.finishedAt)) summary.finishedAt = data.finishedAt;
  if (data.status === 'running' || !summary.finishedAt) problem('unfinished-run');
  if (summary.finishedAt && summary.startedAt && summary.finishedAt < summary.startedAt) problem('invalid-time-order');
  const c = data.candidate;
  if (c) {
    const files = c.files;
    const paths = new Set();
    const valid = Array.isArray(files) && files.length > 0 && files.every(file => {
      if (!file || typeof file.path !== 'string' || !file.path || file.path.startsWith('/') || file.path.includes('\\') || file.path.split('/').some(x => !x || x === '.' || x === '..') || paths.has(file.path) || !hex(file.sha256, 64)) return false;
      paths.add(file.path); return true;
    });
    if (!valid || !hex(c.sha256, 64) || hash(JSON.stringify(files)) !== c.sha256) problem('candidate-inventory-hash-invalid');
    if (!token(c.version) || c.manifestVersion !== 3) problem('candidate-version-invalid');
    summary.candidate = { sha256: hex(c.sha256, 64) ? c.sha256 : null, version: token(c.version), manifestVersion: c.manifestVersion === 3 ? 3 : null,
      unchanged: typeof data.candidateUnchanged === 'boolean' ? data.candidateUnchanged : null,
      buildSha: '미관측 (검사 도구 SHA와 별개)' };
  } else if (data.status !== 'error') problem('candidate-missing');
  if (data.environment) {
    const e = data.environment;
    summary.tool = { sha: hex(e.harnessSha, 40) ? e.harnessSha : null, dirty: typeof e.harnessDirty === 'boolean' ? e.harnessDirty : null };
    if (!summary.tool.sha || summary.tool.dirty === null) problem('tool-provenance-unverified');
    for (const key of ['node', 'platform', 'arch', 'puppeteer', 'chrome']) {
      summary.environment[key] = token(e[key]);
      if (!summary.environment[key]) problem('environment-incomplete');
    }
  } else if (data.status !== 'error') problem('environment-missing');
  const rows = Array.isArray(data.results) ? data.results : [];
  if (rows.length !== suites.length || rows.some(row => !row || !suites.includes(row.name))) problem('invalid-suite-list');
  const owned = new Set();
  for (const row of summary.results) {
    const matches = rows.filter(x => x?.name === row.name);
    if (matches.length !== 1) { row.status = 'incomplete'; problem('missing-or-duplicate-suite'); continue; }
    const raw = matches[0];
    row.status = ['pass', 'fail', 'error', 'not-run'].includes(raw.status) ? raw.status : 'incomplete';
    if (row.status === 'incomplete') problem('invalid-suite-status');
    row.exitCode = Number.isInteger(raw.exitCode) ? raw.exitCode : null;
    row.timedOut = typeof raw.timedOut === 'boolean' ? raw.timedOut : null;
    if (row.status === 'pass' && (row.exitCode !== 0 || row.timedOut !== false || raw.error || raw.signal)) problem('inconsistent-suite-pass');
    const artifacts = Array.isArray(raw.artifacts) ? raw.artifacts : [];
    if (row.status !== 'not-run' && !artifacts.some(a => a?.path === `${row.name}.log`)) problem('suite-log-missing');
    if (row.status === 'not-run' && artifacts.length) problem('not-run-with-evidence');
    if (['fail', 'error'].includes(row.status)) {
      if (!artifacts.some(a => typeof a?.path === 'string' && a.path.endsWith('.json'))) problem('failure-json-missing');
      if (!artifacts.some(a => typeof a?.path === 'string' && a.path.endsWith('.png'))) problem('failure-png-missing');
    }
    for (const a of artifacts) {
      // Flat regular files only. Never follow links or read source documents.
      if (!a || typeof a.path !== 'string' || !/^[a-zA-Z0-9][a-zA-Z0-9_.-]*\.(log|json|png)$/.test(a.path) || a.path === 'result.json' || owned.has(a.path)) { problem('invalid-or-duplicate-artifact'); continue; }
      owned.add(a.path);
      const item = { id: `evidence-${++evidenceNumber}`, kind: path.extname(a.path).slice(1), sha256: hex(a.sha256, 64) ? a.sha256 : null, status: 'unverified' };
      row.evidence.push(item);
      if (a.runId !== data.runId || !uuid(a.runId) || !hex(a.sha256, 64)) { problem('artifact-binding-invalid'); continue; }
      const full = path.join(input, a.path);
      try {
        if (!(await lstat(full)).isFile()) throw new Error('not regular');
        const bytes = await readFile(full);
        if (hash(bytes) !== a.sha256) { problem('artifact-hash-mismatch'); continue; }
        if (item.kind === 'log' && (a.path !== `${row.name}.log` || !bytes.toString('utf8').startsWith(`RHWP E2E RUN ${data.runId} SUITE ${row.name}\n`))) { problem('log-run-or-suite-mismatch'); continue; }
        if (item.kind === 'json') {
          let diagnostic;
          try { diagnostic = JSON.parse(bytes); } catch { problem('diagnostic-json-parse-error'); continue; }
          if (diagnostic?.runId !== data.runId) { problem('diagnostic-run-mismatch'); continue; }
          const stage = diagnostic.stage ?? diagnostic.surface;
          item.stage = typeof stage === 'string' && /^(launch|service-worker|viewer-dark|print|options|content-script|normal-xlsx|misleading-hwp-url|confirmed-hwp|extensionless-hwp|options-save|worker-stop|download-hwpx?|prepare-history-without-extension|verify-persisted-history|transition-(reenter|worker|restart)|own-blob-save)$/.test(stage) ? stage : null;
        }
        if (item.kind === 'png' && !bytes.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10]))) { problem('invalid-png'); continue; }
        item.status = 'verified';
        links.push({ id: item.id, full, kind: item.kind });
      } catch { problem('artifact-missing-or-not-regular'); }
    }
  }
  for (const name of await readdir(input)) if (!['result.json', '.result.json.tmp'].includes(name) && !owned.has(name)) problem('unlisted-artifact-possible-mixed-run');
  if (data.status === 'pass' && (summary.results.some(x => x.status !== 'pass') || data.candidateUnchanged !== true)) problem('inconsistent-overall-pass');
  if (data.status === 'fail' && !summary.results.some(x => ['fail', 'error'].includes(x.status))) problem('inconsistent-overall-fail');
  if (options['expect-candidate-sha256'] && options['expect-candidate-sha256'] !== summary.candidate?.sha256) problem('expected-candidate-hash-mismatch');
  if (options['expect-tool-sha'] && options['expect-tool-sha'] !== summary.tool?.sha) problem('expected-tool-sha-mismatch');
  if (options['manual-review']) {
    try {
      const manual = JSON.parse(await readFile(options['manual-review'], 'utf8'));
      if (manual.sourceSha256 !== summary.sourceSha256 || !date(manual.checkedAt) || !['pass', 'fail'].includes(manual.readability) || !['pass', 'fail'].includes(manual.sharingExclusions)) throw new Error('invalid manual record');
      summary.manualReview = { status: '사용자 제공 기록 (자동 검증 아님)', checkedAt: manual.checkedAt, readability: manual.readability, sharingExclusions: manual.sharingExclusions };
    } catch { problem('manual-review-invalid-or-wrong-source'); }
  }
  if (!issues.length) summary.status = summary.sourceStatus;
  // Fail/error remain visible even if evidence is incomplete; never promote to pass.
  return { summary, links };
}

function render(summary) {
  return `<!doctype html><html lang="ko"><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"><meta name="viewport" content="width=device-width"><title>확장 E2E 증적 보고서</title>
<style>body{font:16px/1.6 system-ui;max-width:1100px;margin:2em auto;padding:0 1em}table{border-collapse:collapse;width:100%}td,th{border:1px solid #aaa;padding:.5em;text-align:left}pre{white-space:pre-wrap;overflow-wrap:anywhere}code{overflow-wrap:anywhere}</style>
<h1>확장 E2E 증적 보고서</h1><p>보고서 상태: <strong>${escape(summary.status)}</strong> · 원본 실행 상태: <strong>${escape(summary.sourceStatus)}</strong></p>
<p>증적 해시 일치는 기록의 내부 일관성 검사이며 서명·독립 출처 인증이 아닙니다.</p>
<h2>실행 식별·후보·검사 도구</h2><pre>${escape(JSON.stringify({ sourceSha256: summary.sourceSha256, schemaVersion: summary.sourceSchemaVersion, evidenceVersion: summary.evidenceVersion, runId: summary.runId, candidate: summary.candidate, tool: summary.tool, environment: summary.environment, startedAt: summary.startedAt, finishedAt: summary.finishedAt }, null, 2))}</pre>
<h2>Suite 결과</h2><table><tr><th>Suite</th><th>상태</th><th>종료 코드 / timeout</th><th>증적·단계</th></tr>${summary.results.map(row => `<tr><td>${escape(row.name)}</td><td>${escape(row.status)}</td><td>${escape(row.exitCode)} / ${escape(row.timedOut)}</td><td>${row.evidence.map(e => `${escape(e.id)} (${escape(e.kind)}): ${escape(e.status)} ${escape(e.stage ?? '')}`).join('<br>')}</td></tr>`).join('')}</table>
<h2>누락·오류·미완료</h2><pre>${escape(JSON.stringify(summary.issues, null, 2))}</pre>
<h2>검증 범위와 미검증 범위</h2><p>선택 실행의 smoke → download → lifecycle 결과를 표시합니다. 실행하지 않은 suite는 not-run입니다.</p><ul>${summary.limitations.map(x => `<li>${escape(x)}</li>`).join('')}</ul>
<h2>별도 수동 확인</h2><pre>${escape(JSON.stringify(summary.manualReview, null, 2))}</pre>
<h2>공유 전 제외 정보</h2><ul>${summary.excluded.map(x => `<li>${escape(x)}</li>`).join('')}</ul><p>원본 증적은 복사하지 않았습니다. 비공개 검토는 생성 폴더의 private.html에서 원본을 여세요. 공유 대상은 report.html과 report.json 두 파일뿐이며 수동 검토·게시가 필요합니다.</p></html>\n`;
}

export async function generateReport({ input, output, ...options }) {
  input = await realpath(input);
  const parent = await realpath(path.dirname(path.resolve(output)));
  output = path.join(parent, path.basename(output));
  if (inside(input, output)) throw new Error('Output must be outside input');
  if (!(await lstat(path.join(input, 'result.json'))).isFile()) throw new Error('result.json must be a regular file');
  const { summary, links } = await inspectRun(input, options);
  await mkdir(output); // Never overwrite a prior report.
  await writeFile(path.join(output, 'report.json'), JSON.stringify(summary, null, 2) + '\n');
  await writeFile(path.join(output, 'report.html'), render(summary));
  const href = full => path.relative(output, full).split(path.sep).map(encodeURIComponent).join('/');
  await writeFile(path.join(output, 'private.html'), `<!doctype html><html lang="ko"><meta charset="utf-8"><title>비공개 증적 인덱스</title><h1>비공개 — 공유 금지</h1><p>원본의 경로·다운로드 정보·화면이 노출될 수 있습니다. 열기 전후 원본 SHA를 공유 요약과 대조하세요.</p><a href="${escape(href(path.join(input, 'result.json')))}">원본 result.json</a><ul>${links.map(x => `<li><a href="${escape(href(x.full))}">${escape(x.id)} (${escape(x.kind)})</a></li>`).join('')}</ul></html>\n`);
  return summary;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const options = parseReportArgs(process.argv.slice(2));
    if (options.help) console.log('Usage: node rhwp-chrome/e2e/report.mjs --input RUN_DIRECTORY --output NEW_DIRECTORY [--expect-candidate-sha256 HASH] [--expect-tool-sha SHA] [--manual-review JSON]');
    else { const result = await generateReport(options); console.log(JSON.stringify({ status: result.status, sourceStatus: result.sourceStatus, issues: result.issues })); process.exitCode = result.status === 'pass' ? 0 : 1; }
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
