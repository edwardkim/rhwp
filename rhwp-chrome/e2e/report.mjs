#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { mkdir, readFile, readdir, lstat, realpath, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isCompletePng } from './png-integrity.mjs';
import { requiredCandidateFiles, suites } from './run.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const hex = (value, length) => typeof value === 'string' && new RegExp(`^[a-f0-9]{${length}}$`).test(value);
const uuid = value => typeof value === 'string' && /^[a-f0-9]{8}(-[a-f0-9]{4}){3}-[a-f0-9]{12}$/.test(value);
const date = value => typeof value === 'string' && /^\d{4}-\d\d-\d\dT[\d:.]+Z$/.test(value) && Number.isFinite(Date.parse(value));
const escape = value => String(value).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);
// Only version/platform tokens cross the sharing boundary; no free-form text.
const token = value => typeof value === 'string' && /^[a-zA-Z0-9][a-zA-Z0-9 ._()+-]{0,79}$/.test(value) ? value : null;
const inside = (parent, child) => { const rel = path.relative(parent, child); return !rel || (!rel.startsWith('..') && !path.isAbsolute(rel)); };
const limitations = ['Unpacked Chrome for Testing only', '스토어 설치·업데이트, Edge/Firefox 미검증',
  '실제 사용자 환경·문서 표시·인쇄 품질 미검증', '자동 검사 결과는 배포 승인이 아님',
  'Chrome 버전은 실행기의 설치 대상 기록이며 브라우저 실행을 증명하지 않음'];

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
    if (!requiredCandidateFiles.every(file => paths.has(file))) problem('candidate-required-files-missing');
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
        if (item.kind === 'png' && !isCompletePng(bytes)) { problem('invalid-png'); continue; }
        item.status = 'verified';
        links.push({ id: item.id, full, kind: item.kind });
      } catch { problem('artifact-missing-or-not-regular'); }
    }
  }
  for (const name of await readdir(input)) if (!['result.json', '.result.json.tmp'].includes(name) && !owned.has(name)) problem('unlisted-artifact-possible-mixed-run');
  const hasOverallError = data.error !== undefined && data.error !== null && data.error !== '';
  if (data.status === 'pass' && (hasOverallError || summary.results.some(x => x.status !== 'pass') || data.candidateUnchanged !== true)) problem('inconsistent-overall-pass');
  if (data.status === 'fail' && !summary.results.some(x => ['fail', 'error'].includes(x.status))) problem('inconsistent-overall-fail');
  if (options['expect-candidate-sha256'] && options['expect-candidate-sha256'] !== summary.candidate?.sha256) problem('expected-candidate-hash-mismatch');
  if (options['expect-tool-sha'] && options['expect-tool-sha'] !== summary.tool?.sha) problem('expected-tool-sha-mismatch');
  if (options['manual-review']) {
    try {
      const manual = JSON.parse(await readFile(options['manual-review'], 'utf8'));
      if (manual.sourceSha256 !== summary.sourceSha256 || !date(manual.checkedAt) || !['pass', 'fail'].includes(manual.readability) || !['pass', 'fail'].includes(manual.sharingExclusions) || (manual.reviewMode !== undefined && !['direct', 'delegated'].includes(manual.reviewMode))) throw new Error('invalid manual record');
      summary.manualReview = { status: '입력된 수동 검토 기록 (자동 검증 아님)', reviewMode: manual.reviewMode ?? 'unspecified', checkedAt: manual.checkedAt, readability: manual.readability, sharingExclusions: manual.sharingExclusions };
    } catch { problem('manual-review-invalid-or-wrong-source'); }
  }
  if (!issues.length) summary.status = summary.sourceStatus;
  // Fail/error remain visible even if evidence is incomplete; never promote to pass.
  return { summary, links };
}

function render(summary) {
  const statuses = { pass: '통과', fail: '실패', error: '오류', 'not-run': '미실행', running: '진행 중', incomplete: '증적 불완전', unknown: '미확인' };
  const badge = value => `<strong class="badge ${Object.hasOwn(statuses, value) ? value : 'unknown'}">${escape(statuses[value] ?? value)} <small>${escape(value)}</small></strong>`;
  const value = item => item === null || item === undefined ? '미관측' : escape(item);
  const fields = entries => `<dl>${entries.map(([label, item]) => `<dt>${escape(label)}</dt><dd>${value(item)}</dd>`).join('')}</dl>`;
  const issues = {
    'result-json-parse-error': '원본 결과 JSON을 읽을 수 없습니다.',
    'invalid-result': '원본 결과 형식이 올바르지 않습니다.',
    'invalid-status': '원본 실행 상태를 확인할 수 없습니다.',
    'unsupported-schemaVersion': '지원하지 않는 결과 스키마입니다.',
    'unbound-legacy-or-invalid-evidence': '실행 ID 또는 증적 소속 정보가 없어 같은 실행의 자료인지 확인할 수 없습니다.',
    'invalid-startedAt': '실행 시작 시각이 올바르지 않습니다.',
    'unfinished-run': '실행이 끝나지 않았습니다. 미실행 검사가 남아 있습니다.',
    'invalid-time-order': '종료 시각이 시작 시각보다 빠릅니다.',
    'candidate-inventory-hash-invalid': '후보 파일 목록이나 해시가 일치하지 않습니다.',
    'candidate-required-files-missing': '확장 후보의 필수 파일이 빠져 있습니다.',
    'candidate-version-invalid': '후보 버전 정보를 확인할 수 없습니다.',
    'candidate-missing': '검사한 확장 후보 정보가 없습니다.',
    'tool-provenance-unverified': '검사 도구 SHA 또는 변경 여부를 확인할 수 없습니다.',
    'environment-incomplete': '실행 환경 기록이 불완전합니다.',
    'environment-missing': '실행 환경 기록이 없습니다.',
    'invalid-suite-list': '검사 목록이 올바르지 않습니다.',
    'missing-or-duplicate-suite': '검사가 누락됐거나 중복돼 있습니다.',
    'invalid-suite-status': '개별 검사 상태를 확인할 수 없습니다.',
    'inconsistent-suite-pass': '통과로 기록된 검사에 오류나 비정상 종료가 함께 있습니다.',
    'suite-log-missing': '실행한 검사의 로그가 증적 목록에 없습니다.',
    'not-run-with-evidence': '미실행 검사에 증적이 연결돼 있습니다.',
    'failure-json-missing': '실패 진단 JSON이 증적 목록에 없습니다.',
    'failure-png-missing': '실패 화면 PNG가 없습니다. 브라우저 실행 전 실패한 경우에도 화면 증거는 미확인입니다.',
    'invalid-or-duplicate-artifact': '증적 항목이 올바르지 않거나 중복돼 있습니다.',
    'artifact-binding-invalid': '증적의 실행 ID나 해시 형식이 올바르지 않습니다.',
    'artifact-hash-mismatch': '증적 파일의 내용이 기록된 해시와 다릅니다.',
    'log-run-or-suite-mismatch': '로그의 실행 ID 또는 검사 이름이 원본 결과와 다릅니다.',
    'diagnostic-json-parse-error': '실패 진단 JSON을 읽을 수 없습니다.',
    'diagnostic-run-mismatch': '실패 진단이 다른 실행에 속합니다.',
    'invalid-png': 'PNG가 손상됐거나 지원하는 검증 한도를 초과했습니다.',
    'artifact-missing-or-not-regular': '증적 파일이 없거나 일반 파일이 아닙니다.',
    'unlisted-artifact-possible-mixed-run': '목록에 없는 파일이 있어 다른 실행의 자료가 섞였을 수 있습니다.',
    'inconsistent-overall-pass': '전체 통과 기록이 오류·개별 검사·후보 변경 정보와 모순됩니다.',
    'inconsistent-overall-fail': '전체 실패 기록에 대응하는 실패 검사가 없습니다.',
    'expected-candidate-hash-mismatch': '후보 해시가 별도로 지정한 기대값과 다릅니다.',
    'expected-tool-sha-mismatch': '검사 도구 SHA가 별도로 지정한 기대값과 다릅니다.',
    'manual-review-invalid-or-wrong-source': '수동 검토 기록이 올바르지 않거나 다른 원본에 대한 기록입니다.',
  };
  const stages = { launch: '브라우저 실행', 'service-worker': '서비스 워커', 'options-save': '설정 저장', 'viewer-dark': '뷰어·다크 모드', print: '인쇄', options: '설정', 'content-script': '콘텐츠 스크립트' };
  const evidence = row => row.evidence.length ? `<ul class="evidence">${row.evidence.map(e => `<li>${escape(e.id)} · ${escape(e.kind)} · ${e.status === 'verified' ? '파일 검증됨' : '파일 미검증'}${e.stage ? `<br>단계: ${escape(stages[e.stage] ?? e.stage)} <code>${escape(e.stage)}</code>` : ''}</li>`).join('')}</ul>` : row.status === 'not-run' ? '검사를 실행하지 않아 증적이 없습니다.' : '확인할 증적이 없습니다.';
  const count = summary.results.filter(row => row.status === 'not-run').length;
  const explanation = summary.issues.length ? '누락·불일치 또는 미완료 기록이 있습니다. 원본 실행 상태와 아래 확인할 문제를 함께 검토하세요.'
    : summary.status === 'pass' ? '기록된 검사가 모두 통과했고 증적의 내부 일관성을 확인했습니다. 수동 검토와 미검증 범위도 확인하세요.'
      : '원본의 실패·오류 상태를 보존했습니다. 증적에서 누락·불일치는 발견하지 못했으며, 검사 성공을 뜻하지 않습니다.';
  const review = summary.manualReview;
  const reviewMode = { direct: '직접 검토', delegated: '사용자 위임에 따른 검토', unspecified: '검토 방식 미기록' };
  return `<!doctype html><html lang="ko"><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"><meta name="viewport" content="width=device-width"><title>확장 E2E 증적 보고서</title>
<style>*{box-sizing:border-box}body{font:16px/1.6 system-ui,sans-serif;color:#172033;background:#f5f7fa;max-width:1100px;margin:2em auto;padding:0 1em}h1{font-size:1.8em;margin:.2em 0}h2{font-size:1.2em;margin:0 0 .7em}p{margin:.6em 0}.muted,small{color:#526078}section,details{background:#fff;border:1px solid #d5dce6;border-radius:10px;padding:1.1em;margin:1em 0}.overview{display:grid;grid-template-columns:1fr 1fr;gap:1em}.overview p{margin:0 0 .4em}.badge{display:inline-block;border-radius:5px;padding:.2em .5em;background:#edf0f5}.badge small{font-weight:normal;color:inherit}.pass{background:#dff3e7;color:#14532d}.fail,.error{background:#fee2e2;color:#881337}.incomplete,.running{background:#fff0cd;color:#713f12}.problems{padding-left:1.4em}.problems li{margin:.5em 0}code,dd{overflow-wrap:anywhere}code{font-size:.85em}table{border-collapse:collapse;width:100%;table-layout:fixed}td,th{border-bottom:1px solid #d5dce6;padding:.65em;text-align:left;vertical-align:top;overflow-wrap:anywhere}th{background:#f5f7fa}.evidence{padding-left:1.1em;margin:0}.evidence li{margin-bottom:.4em}summary{cursor:pointer;font-weight:700}dl{display:grid;grid-template-columns:12em 1fr;gap:.5em 1em}dt{color:#526078}dd{margin:0;min-width:0}footer{font-size:.9em;color:#526078;margin:1.5em 0}@media(max-width:640px){body{margin:1em auto}.overview{grid-template-columns:1fr}dl{grid-template-columns:1fr;gap:.2em}dd{margin-bottom:.6em}thead{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%)}tbody,tr,td{display:block}tr{border:1px solid #d5dce6;border-radius:6px;margin:.7em 0}td{border:0}td::before{content:attr(data-label);display:block;font-size:.8em;color:#526078}td:first-child{font-weight:700}}</style>
<header><p class="muted">공유 전 검토용 요약</p><h1>확장 E2E 증적 보고서</h1><p class="muted">후보 버전 ${value(summary.candidate?.version)} · 시작 ${value(summary.startedAt)}</p></header>
<section aria-label="결과 요약"><div class="overview"><div><p>원본 실행 상태</p>${badge(summary.sourceStatus)}</div><div><p>보고서 판정</p>${badge(summary.status)}</div></div><p>${explanation}</p><p>전체 ${summary.results.length}개 검사 · 미실행 ${count}개</p></section>
<section><h2>확인할 문제</h2>${summary.issues.length ? `<ul class="problems">${summary.issues.map(code => `<li>${escape(issues[code] ?? '기록을 확인해야 합니다.')}<br><code>${escape(code)}</code></li>`).join('')}</ul>` : '<p>증적의 누락·불일치가 발견되지 않았습니다.</p>'}</section>
<section><h2>검사별 결과</h2><table><thead><tr><th scope="col">검사</th><th scope="col">상태</th><th scope="col">종료</th><th scope="col">증적·실패 단계</th></tr></thead><tbody>${summary.results.map(row => `<tr><td data-label="검사">${escape(row.name)}</td><td data-label="상태">${badge(row.status)}</td><td data-label="종료">코드 ${row.exitCode == null ? '—' : value(row.exitCode)}<br>시간 초과 ${row.timedOut === true ? '있음' : row.timedOut === false ? '없음' : '미관측'}</td><td data-label="증적·실패 단계">${evidence(row)}</td></tr>`).join('')}</tbody></table><p class="muted">파일 검증은 증적의 해시·실행 소속 확인입니다. 검사 통과나 화면 품질 판정과는 별개입니다.</p></section>
<details><summary>실행 식별·후보·검사 도구 상세</summary>${fields([['원본 SHA-256', summary.sourceSha256], ['실행 ID', summary.runId], ['결과 스키마', summary.sourceSchemaVersion], ['증적 버전', summary.evidenceVersion], ['후보 버전', summary.candidate?.version], ['후보 inventory SHA-256', summary.candidate?.sha256], ['후보 검사 전후 동일', summary.candidate?.unchanged === null || summary.candidate?.unchanged === undefined ? null : summary.candidate.unchanged ? '예' : '아니요'], ['후보 빌드 SHA', summary.candidate?.buildSha], ['검사 도구 SHA', summary.tool?.sha], ['도구 변경 사항', summary.tool?.dirty === null || summary.tool?.dirty === undefined ? null : summary.tool.dirty ? '있음' : '없음'], ['Node', summary.environment.node], ['운영체제 / 아키텍처', [summary.environment.platform, summary.environment.arch].filter(Boolean).join(' / ') || null], ['Puppeteer', summary.environment.puppeteer], ['Chrome 설치 대상 버전', summary.environment.chrome], ['시작 시각 (UTC)', summary.startedAt], ['종료 시각 (UTC)', summary.finishedAt]])}<p class="muted">Chrome 값은 실행기가 기록한 설치 대상 버전이며 브라우저 실행을 증명하지 않습니다. 증적 해시 일치는 내부 일관성 검사이며 서명·독립 출처 인증이 아닙니다.</p></details>
<section><h2>별도 수동 확인</h2>${fields([['읽기 편의성', statuses[review.readability] ?? review.readability], ['공유 정보 제외', statuses[review.sharingExclusions] ?? review.sharingExclusions], ['기록 상태', review.status], ...(review.reviewMode ? [['검토 방식', reviewMode[review.reviewMode]]] : []), ...(review.checkedAt ? [['검토 시각 (UTC)', review.checkedAt]] : [])])}<p class="muted">입력된 수동 판정은 자동 검사 결과를 바꾸지 않습니다.</p></section>
<details><summary>검증 범위와 미검증 범위</summary><p>smoke → download → lifecycle 순서의 선택 실행 결과입니다. 미실행 검사는 not-run으로 남습니다.</p><ul>${summary.limitations.map(x => `<li>${escape(x)}</li>`).join('')}</ul></details>
<details><summary>공유 전 제외 정보와 원본 확인 방법</summary><ul>${summary.excluded.map(x => `<li>${escape(x)}</li>`).join('')}</ul><p>공유 대상은 report.html과 report.json 두 파일입니다. SHA·버전·실행 시각의 공개 범위도 검토하세요. 원본 증적은 비공개 private.html에서 확인하며, 이 요약에는 복사하지 않았습니다.</p></details>
<footer>비공개 private.html과 원본 결과 폴더는 공유하지 마세요.</footer></html>\n`;
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
