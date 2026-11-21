'use strict';

const { WORKFLOW_PATHS, selectLatestWorkflowRun } = require('./ci-impact-policy.cjs');
const MARKER = '<!-- rhwp-pr-ci-summary:v1 -->';
const FAILED = new Set(['failure', 'timed_out', 'action_required', 'startup_failure']);
const VISUAL_STEPS = new Set(['Run canvas visual diff and PDF report', 'Run selected CanvasKit readiness gate', 'Run frontend Canvas gates']);

function selectRuns(pull, runs) {
  return Object.entries(WORKFLOW_PATHS).map(([name, path]) => ({
    name,
    run: selectLatestWorkflowRun(runs, {
      name, path, headSha: pull.head.sha, headBranch: pull.head.ref,
      headRepository: pull.head.repo.full_name, pullNumber: pull.number, baseRef: pull.base.ref,
    }),
  }));
}

function state(run) {
  if (!run) return '실행 기록 없음';
  if (run.status !== 'completed') return '진행 중';
  return ({ success: '통과', skipped: '건너뜀', cancelled: '취소', neutral: '중립' })[run.conclusion]
    || (FAILED.has(run.conclusion) ? '실패' : '결과 확인 필요');
}

function summary(pull, reports) {
  const states = reports.map(({run}) => state(run));
  const outcome = ['실패', '취소', '진행 중', '결과 확인 필요'].find(s => states.includes(s))
    || (states.every(s => s === '통과') ? '통과' : '일부 실행만 확인');
  const base = pull.base.repo.html_url;
  const lines = [MARKER, '### CI 요약 · ' + outcome, '', '커밋 `' + pull.head.sha + '`', ''];
  for (const {name, run, jobs = []} of reports) {
    if (!run) { lines.push('- ' + name + ' · 실행 기록 없음'); continue; }
    const url = base + '/actions/runs/' + run.id;
    lines.push('- [' + name + ' · 실행 ' + run.id + ', 시도 ' + run.run_attempt + '](' +
      url + '/attempts/' + run.run_attempt + ') · ' + state(run));
    for (const job of jobs) {
      const link = url + '/job/' + job.id;
      if (FAILED.has(job.conclusion) || job.conclusion === 'cancelled') {
        const label = job.name.replace(/[^\p{L}\p{N} ._()/&:-]/gu, ' ').slice(0, 120);
        lines.push('  - [' + label + '](' + link + ') · ' + state(job));
      }
      for (const step of job.steps || []) {
        if (VISUAL_STEPS.has(step.name) && step.conclusion === 'success')
          lines.push('  - [' + step.name + '](' + link + ') · 실행 완료');
      }
    }
  }
  return lines.join('\n');
}

module.exports = { MARKER, selectRuns, summary };
