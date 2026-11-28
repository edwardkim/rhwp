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
  assert.match(text,/^### PR 검증 결과$/m); assert.match(text,/\| CI \| 실패 \|/); assert.match(text,/\/job\/3/);
  assert.match(text,/\| CI \| 완료 \| 프런트엔드 Canvas 검사 \| \[검사 로그\]/); assert.doesNotMatch(text,/CanvasKit/);
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
  assert.match(text, /^### PR 검증 결과$/m);
  assert.match(text, /\| 일반 CI \| 통과 \| 전체 실행 \| \[CI 시도 2\]/);
  assert.doesNotMatch(text, /CodeQL|Render Diff/);
  assert.equal(text.split('\n').filter(line => line.startsWith('|')).length, 3);
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
  let edits = 0, headMoved = false, convertedToDraft = false;
  const rest = {
    pulls:{list:()=>[live], get:()=>({data:{...live,draft:convertedToDraft,
      head:headMoved ? {...live.head,sha:'b'.repeat(40)} : live.head}})},
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
  headMoved=false; convertedToDraft=true; await publish(args); assert.equal(edits,1);
  convertedToDraft=false; live.draft=true; await publish(args); assert.equal(edits,1);
});

test('실패한 작업과 집계 작업이 함께 실패하면 원래 실패 단계만 표시한다', () => {
  const failedJob = (name, step) => ({name, id:3, status:'completed', conclusion:'failure',
    steps:[{name:step,conclusion:'failure'}]});
  const jobs = [failedJob('Frontend package gates', 'Run Studio unit tests'),
    failedJob('Build & Test', 'Check Build & Test worker results')];
  const text = summary(pull, [{name:'CI', run:{...run,conclusion:'failure'}, jobs}]);
  assert.match(text, /Run Studio unit tests/);
  assert.doesNotMatch(text, /worker results|전체 실행/);
});

const {renderEvidence} = require('../pr-ci-render-artifact.cjs');
const {spawnSync} = require('node:child_process');
test('현재 시도의 측정 파일만 집계하고 기준 초과·누락을 구분한다', async () => {
  const time = '2026-11-21T12:00:00Z';
  const step = name => ({name, conclusion:'success', started_at:time, completed_at:time});
  const job = {id:44,name:'Canvas visual diff',steps:[
    step('Run canvas visual diff and PDF report'),step('Run selected CanvasKit readiness gate'),
    step('Upload render diff index')]};
  const zip = spawnSync('python3', ['-c',`
import io,json,sys,zipfile
out=io.BytesIO()
with zipfile.ZipFile(out,'w') as z:
    z.writestr('evidence.json',json.dumps({'schema_version':1,'run_id':2,'run_attempt':2,
        'run_url':'https://github.com/example/rhwp/actions/runs/2/attempts/2',
        'pr_head':'a'*40,'tested_sha':'c'*40,'comparisons':{
            'direct':{'result':'1/1쪽 기준 초과','point':'A 1쪽\\n실패 쪽 최대 픽셀 차이율 2.410% (허용 2.000%) / 0.410%p 초과','priority':0},
            'readiness':{'result':'1/2개 통과, 누락 1개','point':'B','priority':1}}}))
sys.stdout.buffer.write(out.getvalue())
`]);
  assert.equal(zip.status, 0);
  const artifact = {id:91,name:'render-diff-index-2-2',expired:false,size_in_bytes:zip.stdout.length,
    created_at:time,workflow_run:{id:run.id,head_sha:run.head_sha}};
  let downloads = 0;
  const api = {listWorkflowRunArtifacts:()=>[artifact], downloadArtifact:()=>{
    downloads++; return {data:zip.stdout};
  }};
  const github = {rest:{actions:api},paginate:(method,params)=>method(params)};
  const collect = () => renderEvidence(github, {}, run, [job], pull.base.repo.html_url);
  const [direct,browser,canvas,kit] = await collect();
  assert.equal(direct.result,'1/1쪽 기준 초과');
  assert.match(direct.point,/2.410%.*2.000%.*0.410%p/);
  const body = summary(pull, [{name:'Render Diff', run, evidence:[direct]}]);
  assert.match(body, /A 1쪽`<br>`실패 쪽 최대 픽셀 차이율/);
  assert.match(direct.link,/artifacts\/91/);
  assert.equal(browser.result,'결과 미확인');
  assert.equal(canvas.result,'결과 미확인');
  assert.match(kit.result,/1\/2개 통과, 누락 1개/);
  assert.match(kit.point,/B/);
  artifact.created_at = '2026-11-20T12:00:00Z';
  assert.ok((await collect()).every(row=>row.result==='결과 미확인'));
  assert.equal(downloads,1);
});

test('선택된 Render Diff의 비교 단계가 모두 생략되면 일반 CI 통과로 요약하지 않는다', async () => {
  const jobs = [{name:'Canvas visual diff', id:44, conclusion:'success',
    steps:['Run canvas visual diff and PDF report','Run selected CanvasKit readiness gate']
      .map(name => ({name,conclusion:'skipped'}))}];
  const evidence = await renderEvidence({}, {}, run, jobs, pull.base.repo.html_url);
  const text = summary(pull, [{name:'Render Diff',run,jobs,evidence}]);
  assert.match(text,/Render Diff \| 측정 생략 \| `시각 비교 0건` \| \[작업 로그\]/);
  assert.doesNotMatch(text,/일반 CI/);
  assert.deepEqual(await renderEvidence({}, {}, run, [], pull.base.repo.html_url), []);
});
