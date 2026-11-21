const { test } = require('node:test');
const assert = require('node:assert/strict');
const { selectRuns, summary } = require('../pr-ci-summary.cjs');
const pull = { number: 1, base: {ref: 'devel', repo: {html_url: 'https://github.com/example/rhwp'}},
  head: {sha: 'a'.repeat(40), ref: 'branch', repo: {full_name: 'contributor/rhwp'}} };
const run = { id: 2, run_attempt: 2, name: 'CI', path: '.github/workflows/ci.yml', event: 'pull_request',
  head_sha: pull.head.sha, head_branch: 'branch', head_repository: pull.head.repo,
  status: 'completed', conclusion: 'success' };
test('현재 PR과 같은 저장소·브랜치·head의 최신 시도를 선택한다', () => {
  const reports = selectRuns(pull, [run, {...run, run_attempt: 1}, {...run, id: 3, head_sha: 'old'},
    {...run, id: 4, pull_requests: [{number: 2, base: {ref: 'devel'}}]}]);
  assert.equal(reports[0].run, run);
  assert.equal(reports[1].run, null);
});
test('실패 job과 완료한 시각 단계만 연결한다', () => {
  const text = summary(pull, [{name:'CI',run:{...run,conclusion:'failure'},jobs:[
    {name:'test',id:3,status:'completed',conclusion:'failure',steps:[
      {name:'Run frontend Canvas gates',conclusion:'success'},
      {name:'Run selected CanvasKit readiness gate',conclusion:'skipped'}]}]}]);
  assert.match(text,/CI 요약 · 실패/); assert.match(text,/\/job\/3/);
  assert.match(text,/Run frontend Canvas gates/); assert.doesNotMatch(text,/CanvasKit/);
});
