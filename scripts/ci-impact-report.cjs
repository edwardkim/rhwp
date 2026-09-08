'use strict';

// Diagnostic only: never writes a policy output, status, PR comment, or artifact.
const fs = require('node:fs');
const { selectLatestWorkflowRun } = require('./ci-impact-policy.cjs');

const LIMITS = Object.freeze({ requests: 24, durationMs: 45000, requestMs: 5000,
  metadataBytes: 2 * 1024 * 1024, logBytes: 1024 * 1024, jobs: 6, summaryBytes: 16384 });
const PATHS = Object.freeze({ CI: '.github/workflows/ci.yml', CodeQL: '.github/workflows/codeql.yml',
  'Render Diff': '.github/workflows/render-diff.yml' });
const BAD = new Set(['failure', 'timed_out', 'cancelled', 'action_required', 'startup_failure']);
const OUTCOMES = new Set(['success', 'failure', 'cancelled', 'skipped']);
const STAGES = ['resolve', 'checkout', 'collect', 'classify', 'input', 'policy', 'publish'];
const SHA = /^[a-f0-9]{40}$/;
const REPO = /^[A-Za-z0-9][A-Za-z0-9_.-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/;
const numericId = (value) => Number.isSafeInteger(Number(value)) && Number(value) > 0;

function field(value) {
  return String(value ?? '').slice(0, 4096)
    .replace(/\x1b\[[0-?]*[ -/]*[@-~]/g, '')
    .replace(/[\x00-\x1f\x7f-\x9f\u202a-\u202e\u2066-\u2069]/g, '')
    .replace(/(?:github_pat_|gh[pousr]_)[A-Za-z0-9_]+/gi, '[redacted]')
    .replace(/(?:bearer\s+\S+|(?:token|password|secret|authorization)\s*[:=]\s*\S+)/gi, '[redacted]')
    .replace(/https?:\/\/\S+/gi, '[URL omitted]')
    .slice(0, 240)
    .replace(/[&<>"'`\[\]()*_\\|#!]/g, (c) => `&#${c.charCodeAt(0)};`)
    .replace(/::/g, '&#58;&#58;');
}

function shortError(error) {
  // Never expose API response bodies, signed URLs, or raw exception messages.
  return /^http-(403|404|429|5\d\d)$/.test(error?.code || '') ? error.code
    : new Set(['deadline', 'request-limit', 'request-timeout', 'oversize', 'invalid-json',
      'unsafe-redirect', 'redirect-missing', 'network-error', 'identity-mismatch',
      'attempt-changed', 'job-identity-mismatch', 'metadata-unavailable']).has(error?.code)
      ? error.code : 'evidence-unavailable';
}
const fault = (code) => Object.assign(new Error(code), { code });

function allowedLogUrl(value) {
  try {
    const url = new URL(value);
    // Observed GitHub Actions log storage. Unknown hosts fail closed, never guessed/fetched.
    return url.protocol === 'https:' && !url.username && !url.password && !url.port
      && /^[a-z0-9]+\.blob\.core\.windows\.net$/.test(url.hostname) ? url : null;
  } catch { return null; }
}

async function boundedBody(response, cap, partial) {
  if (!response.body) return { text: '', truncated: false };
  const reader = response.body.getReader();
  const chunks = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const remaining = cap - size;
      chunks.push(value.subarray(0, remaining));
      size += Math.min(value.byteLength, remaining);
      if (value.byteLength >= remaining) {
        // An exact-size response is conservatively marked partial too.
        void reader.cancel().catch(() => {});
        if (!partial) throw fault('oversize');
        return { text: Buffer.concat(chunks).toString('utf8'), truncated: true };
      }
    }
    return { text: Buffer.concat(chunks).toString('utf8'), truncated: false };
  } finally { reader.releaseLock(); }
}

function transport({ token, fetchImpl = fetch, now = Date.now }) {
  const started = now();
  let count = 0;
  const check = () => { if (now() - started >= LIMITS.durationMs) throw fault('deadline'); };
  async function request(url, { log = false, storage = false, redirect = false } = {}) {
    check();
    if (count >= LIMITS.requests) throw fault('request-limit');
    count += 1;
    const controller = new AbortController();
    const ms = Math.min(LIMITS.requestMs, LIMITS.durationMs - (now() - started));
    let timer;
    const timeout = new Promise((_, reject) => {
      timer = setTimeout(() => { controller.abort(); reject(fault('request-timeout')); }, ms);
    });
    try {
      return await Promise.race([timeout, (async () => {
        const response = await fetchImpl(url, { method: 'GET', redirect: 'manual',
          signal: controller.signal,
          headers: storage ? {} : { Authorization: `Bearer ${token}`,
            Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' } });
        if (redirect && response.status === 302) {
          void response.body?.cancel().catch(() => {});
          return { location: response.headers.get('location') };
        }
        if (response.status !== 200) {
          void response.body?.cancel().catch(() => {});
          throw fault(`http-${response.status}`);
        }
        if (redirect) throw fault('redirect-missing');
        const result = await boundedBody(response, log ? LIMITS.logBytes : LIMITS.metadataBytes, log);
        check();
        return result;
      })()]);
    } catch (error) {
      controller.abort();
      if (error?.code) throw error;
      throw fault('network-error');
    } finally { clearTimeout(timer); }
  }
  return {
    check,
    stats: () => ({ requests: count, elapsedMs: Math.max(0, now() - started) }),
    async json(route) {
      if (!/^\/repos\/[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+\//.test(route)) throw fault('identity-mismatch');
      const body = await request(`https://api.github.com${route}`);
      try { return JSON.parse(body.text); } catch { throw fault('invalid-json'); }
    },
    async log(repository, jobId) {
      const { location } = await request(`https://api.github.com/repos/${repository}/actions/jobs/${jobId}/logs`,
        { redirect: true });
      const url = allowedLogUrl(location);
      if (!url) throw fault('unsafe-redirect');
      return request(url.href, { log: true, storage: true });
    },
  };
}

function extractErrors(text) {
  const errors = new Set();
  // Only known diagnostic shapes, never arbitrary log lines or document paths/content.
  for (const raw of text.split('\n')) {
    const line = raw.slice(0, 4096).replace(/\x1b\[[0-?]*[ -/]*[@-~]/g, '');
    const panic = line.match(/thread '([A-Za-z0-9_:.-]{1,220})'[^\n]* panicked at /);
    if (panic) errors.add(`테스트 panic: ${field(panic[1])}`);
    const test = line.match(/\bFAIL\s+\[[^\]]{0,100}\]\s+([A-Za-z0-9_:.-]{1,220})\s+([A-Za-z0-9_:.-]{1,220})(?:\s|$)/);
    if (test) errors.add(`실패 테스트: ${field(test[2])}`);
    const rust = line.match(/\berror\[(E\d{4})\]/);
    if (rust) errors.add(`Rust 컴파일 오류: ${rust[1]}`);
    const exit = line.match(/Process completed with exit code (\d{1,3})\./);
    if (exit) errors.add(`프로세스 exit code ${exit[1]}`);
    if (line.includes('layout-anomaly text-overlap')) errors.add('글자 겹침(text-overlap) 보호 검사 실패');
    const baseline = line.match(/신규 발생: .{0,1000} — (\d{1,8})건 \(baseline 없음\)/);
    if (baseline) errors.add(`신규 검출 ${baseline[1]}건 (baseline 없음; 회귀 여부 미확정)`);
    if (/assertion .{0,120}failed/.test(line)) errors.add('assertion 검증 실패');
    if (/##\[error\].{0,200}(?:timed out|timeout exceeded)/i.test(line)) errors.add('실행 시간 제한 초과');
    if (errors.size >= 6) break;
  }
  return [...errors];
}

function identityFor(input) {
  return { pullNumber: input.pullNumber, baseRef: 'devel', baseSha: input.baseSha,
    headSha: input.headSha, headBranch: input.headBranch, headRepository: input.headRepository };
}

async function diagnoseRun(input, name, evidence, client, budget) {
  const ref = evidence?.run;
  const result = { name, jobs: [], notes: [] };
  if (!numericId(ref?.id) || !numericId(ref?.attempt)) {
    result.notes.push('metadata-unavailable'); return result;
  }
  const prefix = `/repos/${input.repository}/actions/runs/${ref.id}`;
  try {
    const run = await client.json(prefix);
    if (!selectLatestWorkflowRun([run], { ...identityFor(input), name, path: PATHS[name] })
      || run.repository?.full_name !== input.repository || Number(run.id) !== Number(ref.id)) {
      throw fault('identity-mismatch');
    }
    if (Number(run.run_attempt) !== Number(ref.attempt) || run.status !== 'completed'
      || run.conclusion !== ref.conclusion) throw fault('attempt-changed');
    result.url = `https://github.com/${input.repository}/actions/runs/${run.id}/attempts/${run.run_attempt}`;
    result.attempt = run.run_attempt;
    result.conclusion = run.conclusion;
    const jobs = [];
    for (let page = 1; page <= 2; page += 1) {
      const data = await client.json(`${prefix}/attempts/${run.run_attempt}/jobs?per_page=100&page=${page}`);
      if (!Array.isArray(data.jobs) || data.jobs.length > 100) throw fault('metadata-unavailable');
      jobs.push(...data.jobs);
      if (jobs.length >= data.total_count || data.jobs.length < 100) break;
      if (page === 2) result.notes.push('job-list-truncated');
    }
    const failures = jobs.filter((job) => BAD.has(job.conclusion));
    // Workers before the aggregate, but never infer chronology == causality.
    failures.sort((a, b) => Number(a.name === 'Build & Test') - Number(b.name === 'Build & Test'));
    for (const job of failures) {
      client.check();
      if (budget.jobs >= LIMITS.jobs) { result.notes.push('failure-list-truncated'); break; }
      if (!numericId(job.id) || Number(job.run_id) !== Number(run.id)
        || Number(job.run_attempt) !== Number(run.run_attempt) || job.head_sha !== input.headSha) {
        result.notes.push('job-identity-mismatch'); continue;
      }
      budget.jobs += 1;
      const item = { name: field(job.name), conclusion: field(job.conclusion),
        aggregate: job.name === 'Build & Test', errors: [],
        url: `https://github.com/${input.repository}/actions/runs/${run.id}/job/${job.id}`,
        steps: (Array.isArray(job.steps) ? job.steps : []).filter((step) => BAD.has(step.conclusion))
          .slice(0, 6).map((step) => `${numericId(step.number) ? step.number : '?'}: ${field(step.name)}`) };
      result.jobs.push(item);
      if (!item.aggregate) {
        try {
          const log = await client.log(input.repository, job.id);
          client.check();
          item.errors = extractErrors(log.text);
          if (log.truncated) item.note = 'log-truncated';
          else if (!item.errors.length) item.note = 'no-recognized-error';
        } catch (error) { item.note = shortError(error); }
      }
    }
    if (!failures.length) result.notes.push('no-failed-job-evidence');
  } catch (error) { result.notes.push(shortError(error)); }
  return result;
}

function render(input, report) {
  const labels = { 'upstream-failure': '선행 검증 실패 — Controller 자체 장애와 구분',
    'controller-error': 'Controller 내부 단계 오류', 'evidence-unavailable': '증적 미확인',
    'policy-blocked': '정책 차단 — 선행 테스트 오류와 구분',
    success: '검증 성공', pending: '검증 대기', 'stale/skipped': '오래된 이벤트 또는 비대상' };
  const lines = ['### CI 실패 진단', '', `- 유형: ${labels[report.kind]}`,
    `- PR: ${numericId(input.pullNumber) ? `#${input.pullNumber}` : '미확인'}`,
    `- head: ${SHA.test(input.headSha || '') ? input.headSha : '미확인'}`,
    `- base: ${SHA.test(input.baseSha || '') ? input.baseSha : '미확인'}`,
    '- 이 보고는 기존 CI 판정을 변경하지 않습니다. 원본 로그의 관찰이며 회귀/근본 원인 확정이 아닙니다.'];
  if (report.internal.length) lines.push(`- Controller 단계: ${report.internal.join(', ')}`);
  if (report.notes.length) lines.push(`- 상세 미확인/수집 제한: ${report.notes.join(', ')}`);
  for (const run of report.runs) {
    lines.push('', `#### ${run.name}`, run.url
      ? `- [원본 실행 · attempt ${run.attempt}](${run.url}) — ${field(run.conclusion)}`
      : '- 원본 실행 identity/attempt 미확인');
    for (const job of run.jobs) {
      lines.push(`- [${job.aggregate ? '집계' : '실패 job'}: ${job.name}](${job.url}) — ${job.conclusion}`);
      if (job.steps.length) lines.push(`  - 실패 step: ${job.steps.join('; ')}`);
      for (const error of job.errors) lines.push(`  - ${error}`);
      if (job.note) lines.push(`  - 오류 상세 미확인/부분 수집: ${job.note}`);
    }
    if (run.notes.length) lines.push(`- 증적 상태: ${run.notes.join(', ')}`);
  }
  lines.push('', `- 진단 추가 요청: ${report.stats.requests}/24; 소요: ${Math.round(report.stats.elapsedMs)}ms`, '');
  let output = '';
  for (const line of lines) {
    if (Buffer.byteLength(output + line + '\n', 'utf8') > LIMITS.summaryBytes - 100) {
      output += '\n상세 수집 제한: summary-truncated\n'; break;
    }
    output += `${line}\n`;
  }
  return output;
}

async function createReport(input, dependencies = {}) {
  const client = transport(dependencies);
  const report = { kind: 'evidence-unavailable', internal: [], notes: [], runs: [] };
  for (const stage of STAGES) {
    if (input.outcomes?.[stage] === 'failure'
      && !(stage === 'publish' && input.published === 'true')) report.internal.push(stage);
  }
  if (report.internal.length) report.kind = 'controller-error';
  else if (input.active !== 'true' || input.stale === 'true' || input.auditPublish === 'false') report.kind = 'stale/skipped';
  else if (input.decision === 'blocked') report.kind = 'policy-blocked';
  else if (input.conclusion === 'success' || input.conclusion === 'pending') report.kind = input.conclusion;
  else if (!input.conclusion && ['success', 'pending'].includes(input.publishedState)) report.kind = input.publishedState;
  else if (input.conclusion === 'failure' && input.decision !== 'blocked') report.kind = 'upstream-failure';
  else report.notes.push('policy-evidence-unavailable');

  if (report.kind === 'upstream-failure') {
    if (!REPO.test(input.repository || '') || !REPO.test(input.headRepository || '')
      || !SHA.test(input.headSha || '') || !SHA.test(input.baseSha || '') || !numericId(input.pullNumber)) {
      report.kind = 'evidence-unavailable'; report.notes.push('identity-mismatch');
    } else {
      const budget = { jobs: 0 };
      try {
        const pull = await client.json(`/repos/${input.repository}/pulls/${input.pullNumber}`);
        if (pull.state !== 'open' || pull.head?.sha !== input.headSha || pull.base?.sha !== input.baseSha
          || pull.head?.repo?.full_name !== input.headRepository || pull.head?.ref !== input.headBranch
          || pull.base?.ref !== 'devel' || Number(pull.number) !== Number(input.pullNumber)) {
          report.kind = 'stale/skipped'; report.notes.push('identity-mismatch');
        } else {
          for (const name of Object.keys(PATHS)) {
            const evidence = input.workflows?.[name];
            if (!BAD.has(evidence?.run?.conclusion)) continue;
            report.runs.push(await diagnoseRun(input, name, evidence, client, budget));
          }
          if (!report.runs.length) { report.kind = 'evidence-unavailable'; report.notes.push('metadata-unavailable'); }
        }
      } catch (error) { report.kind = 'evidence-unavailable'; report.notes.push(shortError(error)); }
    }
  }
  report.stats = client.stats();
  return { ...report, summary: render(input, report) };
}

function loadInput(env, workspace) {
  let workflows = {};
  try {
    const file = `${workspace}/.ci-impact-workflows.json`;
    if (fs.statSync(file).size > LIMITS.metadataBytes) throw fault('oversize');
    workflows = JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch { /* Missing evidence is reported explicitly, never guessed. */ }
  return { repository: env.GITHUB_REPOSITORY, pullNumber: env.PULL_NUMBER,
    headSha: env.HEAD_SHA, baseSha: env.BASE_SHA, headBranch: env.HEAD_BRANCH,
    headRepository: env.HEAD_REPOSITORY, active: env.ACTIVE, stale: env.STALE,
    auditPublish: env.AUDIT_PUBLISH, conclusion: env.AUDIT_CONCLUSION,
    decision: env.DECISION, published: env.STATUS_PUBLISHED, publishedState: env.PUBLISHED_STATE, workflows,
    outcomes: Object.fromEntries(STAGES.map((stage) => [stage,
      OUTCOMES.has(env[`OUTCOME_${stage.toUpperCase()}`]) ? env[`OUTCOME_${stage.toUpperCase()}`] : 'skipped'])) };
}

module.exports = { LIMITS, field, shortError, allowedLogUrl, extractErrors, transport, createReport, loadInput };
