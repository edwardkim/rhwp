const { test } = require('node:test');
const assert = require('node:assert/strict');
const { selectRuns, summary } = require('../pr-ci-summary.cjs');
const pull = { number: 1, base: {ref: 'devel', sha: 'b'.repeat(40), repo: {html_url: 'https://github.com/example/rhwp'}},
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
  assert.match(text,/^### CI 요약$/m); assert.match(text,/\| CI \| 실패 \|/); assert.match(text,/\/job\/3/);
  assert.match(text,/Run frontend Canvas gates/); assert.doesNotMatch(text,/CanvasKit/);
});

test('GitHub가 PR 연결을 제공하면 번호, base SHA와 head 브랜치를 대조한다', () => {
  const association = {number: pull.number, base: {ref: pull.base.ref, sha: pull.base.sha},
    head: {ref: pull.head.ref}};
  const associated = {...run, pull_requests: [association]};
  assert.equal(selectRuns(pull, [associated])[0].run, associated);
  const stale = {...association, base: {...association.base, sha: 'c'.repeat(40)}};
  assert.equal(selectRuns(pull, [{...run, pull_requests: [stale]}])[0].run, null);
});

test('일부 실행과 재시도에서도 표와 제목을 유지하고 최신 실행 시각을 쓴다', () => {
  const reports = selectRuns(pull, [{...run, updated_at: '2026-11-21T18:30:47Z'}]);
  const text = summary(pull, reports);
  assert.match(text, /^### CI 요약$/m);
  assert.match(text, /\| CI \| 통과 \| \[시도 2\]/);
  assert.match(text, /\| CodeQL \| 실행 기록 없음 \| — \|/);
  assert.match(text, /\| Render Diff \| 실행 기록 없음 \| — \|/);
  assert.match(text, /최근 실행 갱신 2026-11-21 18:30:47 UTC$/);
  assert.doesNotMatch(text, /실패하거나 취소된 작업|완료된 시각 검사/);
  assert.equal(summary(pull, reports), text);
});

test('동시 실행, 취소, 건너뜀의 행을 유지하고 오래된 실행 시각으로 돌아가지 않는다', () => {
  const reports = [
    {name: 'CI', run: {...run, status: 'in_progress', conclusion: null, updated_at: '2026-11-22T01:02:03Z'}},
    {name: 'CodeQL', run: {...run, conclusion: 'cancelled', updated_at: '2026-11-21T23:59:59Z'}},
    {name: 'Render Diff', run: {...run, conclusion: 'skipped'}},
  ];
  const text = summary(pull, reports);
  assert.match(text, /\| CI \| 진행 중 \|/);
  assert.match(text, /\| CodeQL \| 취소 \|/);
  assert.match(text, /\| Render Diff \| 건너뜀 \|/);
  assert.match(text, /최근 실행 갱신 2026-11-22 01:02:03 UTC$/);
});

const publish = require('../publish-pr-ci-summary.cjs');
test('fork PR의 실제 head 재확인, 봇 코멘트 한 건 유지, 이전 이벤트 재계산', async () => {
  const live = {...pull,state:'open',base:{ref:'devel',repo:{full_name:'example/rhwp',html_url:'https://github.com/example/rhwp'}}};
  const comments = [{id:7,user:{login:'someone'},body:'<!-- rhwp-pr-ci-summary:v1 -->\n'}];
  const runs = [run];
  let edits = 0, headMoved = false;
  const rest = {
    pulls:{list:()=>[live], get:()=>({data:headMoved ? {...live,head:{...live.head,sha:'b'.repeat(40)}} : live})},
    actions:{listWorkflowRunsForRepo:()=>runs,listJobsForWorkflowRunAttempt:()=>[]},
    issues:{listComments:()=>comments,createComment:({body})=>comments.push({id:8,user:{login:'github-actions[bot]'},body}),
      updateComment:({comment_id,body})=>{assert.equal(comment_id,8);comments[1].body=body;edits++;}},
  };
  const args={github:{rest,paginate:(method,params)=>method(params)},context:{repo:{owner:'example',repo:'rhwp'},
    payload:{workflow_run:{...run,head_repository:{...run.head_repository,owner:{login:'contributor'}}}}}};
  await publish(args); await publish(args); assert.equal(comments.length,2); assert.equal(edits,0);
  runs[0]={...run,run_attempt:3,conclusion:'failure'}; await publish(args);
  assert.match(comments[1].body,/시도 3/); assert.equal(edits,1);
  headMoved=true; runs[0]={...run,run_attempt:4}; await publish(args); assert.equal(edits,1);
});
