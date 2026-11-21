'use strict';

const { WORKFLOW_PATHS, selectLatestWorkflowRun } = require('./ci-impact-policy.cjs');
const MARKER = '<!-- rhwp-pr-ci-summary:v1 -->';
const FAILED = new Set(['failure', 'timed_out', 'action_required', 'startup_failure']);
const VISUAL_STEPS = new Map([
  ['Run canvas visual diff and PDF report', 'Canvas 및 PDF 비교'],
  ['Run selected CanvasKit readiness gate', 'CanvasKit 전환 검사'],
  ['Run frontend Canvas gates', '프런트엔드 Canvas 검사'],
]);

function selectRuns(pull, runs) {
  return Object.entries(WORKFLOW_PATHS).map(([name, path]) => ({
    name,
    run: selectLatestWorkflowRun(runs, {
      name, path, headSha: pull.head.sha, headBranch: pull.head.ref,
      headRepository: pull.head.repo.full_name, pullNumber: pull.number, baseRef: pull.base.ref, baseSha: pull.base.sha,
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
  const base = pull.base.repo.html_url;
  const sha = pull.head.sha;
  const lines = [
    MARKER, '### PR 검증 결과', '', '대상 커밋 [`' + sha.slice(0, 12) + '`](' + base + '/commit/' + sha + ')',
    '', '| 검사 | 결과 | 항목 | 근거 |', '| --- | --- | --- | --- |',
  ];
  const problems = [];
  const other = [];
  let latestUpdate = 0;
  for (const {name, run, jobs = []} of reports) {
    if (!run) continue;
    const url = base + '/actions/runs/' + run.id;
    const attempt = run.run_attempt;
    const runLink = '[시도 ' + attempt + '](' + url + '/attempts/' + attempt + ')';
    let hasDetails = false;
    const updated = Date.parse(run.updated_at);
    if (Number.isFinite(updated)) latestUpdate = Math.max(latestUpdate, updated);

    for (const job of jobs) {
      const link = url + '/job/' + job.id;
      if (FAILED.has(job.conclusion) || job.conclusion === 'cancelled') {
        const failedStep = job.steps?.find(step => FAILED.has(step.conclusion));
        const label = (failedStep?.name || job.name).replace(/[^\p{L}\p{N} ._()/&:-]/gu, ' ').slice(0, 120);
        problems.push('| ' + name + ' | ' + state(job) + ' | ' + label + ' | [작업 로그](' + link + ') |');
        hasDetails = true;
      }
      for (const step of job.steps || []) {
        if (VISUAL_STEPS.has(step.name) && step.conclusion === 'success') {
          other.push('| ' + name + ' | 완료 | ' + VISUAL_STEPS.get(step.name) + ' | [검사 로그](' + link + ') |');
          hasDetails = true;
        }
      }
    }
    if (!hasDetails || state(run) !== '통과') {
      other.push('| ' + name + ' | ' + state(run) + ' | 전체 실행 | ' + runLink + ' |');
    }
  }
  lines.push(...problems, ...other);
  if (latestUpdate) {
    const timestamp = new Date(latestUpdate).toISOString().slice(0, 19).replace('T', ' ');
    lines.push('', '최근 실행 갱신 ' + timestamp + ' UTC');
  }
  return lines.join('\n');
}

module.exports = { MARKER, selectRuns, summary };
