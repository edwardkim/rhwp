import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';

const root = new URL('../../', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8').replaceAll('\r\n', '\n');
const workflow = read('.github/workflows/issue-form-labels.yml');
const script = workflow.split('          script: |\n')[1]
  .split('\n').map((line) => line.replace(/^            /, '')).join('\n');
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
// Execute the shipped workflow, not a second implementation of its classifier.
const run = new AsyncFunction('github', 'context', 'core', script);
const labels = ['rhwp-studio', 'cli', 'browser-extension', 'api', 'mcp',
  'bug', 'enhancement', 'documentation', 'test', 'font', 'rendering'];

function form(surface = 'Studio', kind = 'document') {
  const responses = {
    document: '### 표시 문제와 기대 결과\n\n표 아래 문단이 겹칩니다.\n\n### 재현 순서·문제 페이지\n\n문서를 열고 3쪽 확인\n\n### 원본 문서·재현 자료\n\n공개 불가: 개인정보 포함',
    operation: '### 동작 문제와 기대 결과\n\n저장 실패\n\n### 재현 순서\n\n파일을 열고 저장',
    feature: '### 하려는 작업과 현재 제약\n\n일괄 변환\n\n### 원하는 동작\n\n여러 파일 선택',
    documentation: '### 문서 URL 또는 위치\n\nrhwp --help\n\n### 잘못되거나 부족한 설명\n\n예제 누락\n\n### 기대하는 설명\n\n명령 예제',
  };
  return `### 사용 경로\n\n${surface}\n\n${responses[kind]}`;
}

async function simulate(options = {}) {
  const current = {
    body: form(), labels: [{ name: 'bug' }, { name: 'table' }],
    state: 'open', updated_at: '2026-09-27T01:00:00Z', ...options.issue,
  };
  const calls = [];
  const info = [];
  const warnings = [];
  let reads = 0;
  const eventsEndpoint = Symbol('events');
  const labelsEndpoint = Symbol('labels');
  const github = {
    rest: { issues: {
      async get(params) {
        assert.deepEqual(params, { owner: 'edwardkim', repo: 'rhwp', issue_number: 123 });
        calls.push('get');
        if (options.getError) throw new Error('API unavailable');
        reads++;
        if (reads > 1) Object.assign(current, options.latest);
        return { data: structuredClone(current) };
      },
      listEvents: eventsEndpoint,
      listLabelsForRepo: labelsEndpoint,
      async addLabels(params) {
        calls.push(params);
        assert.deepEqual(Object.keys(params).sort(), ['issue_number', 'labels', 'owner', 'repo']);
        assert.equal(params.owner, 'edwardkim');
        assert.equal(params.repo, 'rhwp');
        assert.equal(params.issue_number, 123);
        assert.ok(params.labels.length > 0);
        assert.equal(new Set(params.labels).size, params.labels.length);
        assert.ok(params.labels.every((label) => labels.includes(label)));
        if (options.addError) throw new Error('Write denied');
        current.labels.push(...params.labels.map((name) => ({ name })));
      },
      // No remove/update/create methods: calling them fails the test.
    } },
    async paginate(endpoint, params) {
      assert.equal(params.per_page, 100);
      if (options.paginateError) throw new Error('Incomplete API read');
      if (endpoint === eventsEndpoint) {
        calls.push('events');
        return options.events ?? [];
      }
      assert.equal(endpoint, labelsEndpoint);
      calls.push('labels');
      return (options.available ?? labels).map((name) => ({ name }));
    },
  };
  const context = {
    repo: { owner: 'edwardkim', repo: 'rhwp' },
    payload: { action: 'opened', issue: { number: 123, body: form() }, ...options.payload },
  };
  await run(github, context, {
    info: (message) => info.push(message), warning: (message) => warnings.push(message),
  });
  return { current, calls, info, warnings, writes: calls.filter((call) => typeof call === 'object') };
}

test('all four forms and five allowlisted surfaces add only the corresponding label', async () => {
  const mapping = [
    ['Studio', 'rhwp-studio'], ['CLI', 'cli'], ['브라우저 확장', 'browser-extension'],
    ['라이브러리/API', 'api'], ['MCP', 'mcp'],
  ];
  for (const kind of ['document', 'operation', 'feature', 'documentation']) {
    for (const [surface, label] of mapping) {
      const result = await simulate({ issue: { body: form(surface, kind) } });
      assert.deepEqual(result.current.labels.map((entry) => entry.name), ['bug', 'table', label]);
      assert.equal(result.writes.length, 1);
    }
  }
});

test('editing Studio to CLI adds CLI and preserves prior and manually assigned labels', async () => {
  const result = await simulate({
    issue: { body: form('CLI'), labels: ['bug', 'rhwp-studio', 'layout', 'font'] },
    payload: { action: 'edited', changes: { body: { from: form('Studio') } } },
  });
  assert.deepEqual(result.current.labels, ['bug', 'rhwp-studio', 'layout', 'font', { name: 'cli' }]);
});

test('retrying a completed operation does not write again', async () => {
  const first = await simulate();
  const retry = await simulate({ issue: first.current });
  assert.equal(retry.writes.length, 0);
  assert.deepEqual(retry.calls, ['get']);
});

test('a removed label stays removed on opened retry, unrelated edit and reselection', async () => {
  // A removal beyond page one must not disappear from the decision.
  const events = [...Array.from({ length: 100 }, () => ({ event: 'commented' })),
    { event: 'unlabeled', label: { name: 'rhwp-studio' }, actor: { login: 'maintainer' } }];
  for (const payload of [
    { action: 'opened' },
    { action: 'edited', changes: { body: { from: form() + '\nold text' } } },
    { action: 'edited', changes: { body: { from: form('CLI') } } },
  ]) {
    const result = await simulate({ events, payload });
    assert.equal(result.writes.length, 0);
    assert.deepEqual(result.calls, ['get', 'events']);
  }
});

test('removal of a different label does not suppress the selected label', async () => {
  const result = await simulate({ events: [{ event: 'unlabeled', label: { name: 'cli' } }] });
  assert.equal(result.writes.length, 1);
});

test('latest body wins over a stale opened or edited payload', async () => {
  const result = await simulate({ issue: { body: form('MCP') } });
  assert.deepEqual(result.writes[0].labels, ['mcp']);
});

test('live body changes, concurrent label updates and issue closure abort the stale snapshot', async () => {
  for (const latest of [
    { body: form('CLI') },
    { updated_at: '2026-09-27T01:01:00Z' },
    { state: 'closed' },
    { labels: [{ name: 'rhwp-studio' }] },
  ]) {
    assert.equal((await simulate({ latest })).writes.length, 0);
  }
});

test('missing repository label is warned about and never created', async () => {
  const result = await simulate({ available: ['bug', 'cli'] });
  assert.equal(result.writes.length, 0);
  assert.equal(result.warnings.length, 1);
});

test('blank, legacy, null, incomplete and ambiguous forms do not label', async () => {
  for (const body of [
    null, '', '## 현상\nStudio에서 오류', '### 사용 경로\n\nStudio',
    form() + '\n\n### 사용 경로\n\nCLI',
    form() + '\n\n### 동작 문제와 기대 결과\n\n오류\n\n### 재현 순서\n\n실행',
    '```\n' + form() + '\n```',
  ]) {
    const result = await simulate({ issue: { body } });
    assert.equal(result.writes.length, 0);
    assert.deepEqual(result.calls, ['get']);
  }
});

test('unknown, multiple, injected and prototype property values cannot become labels', async () => {
  for (const value of [
    '기타', '모름', '해당 없음', '_No response_', 'Windows', 'ios', 'Studio, CLI',
    'studio', 'constructor', '__proto__', 'Studio\nCLI',
    '$(touch /tmp/should-not-exist)', '${{ secrets.GITHUB_TOKEN }}',
  ]) {
    assert.equal((await simulate({ issue: { body: form(value) } })).writes.length, 0);
  }
});

test('CRLF form responses are accepted', async () => {
  assert.equal((await simulate({ issue: { body: form().replaceAll('\n', '\r\n') } })).writes.length, 1);
});

test('closed issues and PR-shaped objects are not modified', async () => {
  for (const issue of [{ state: 'closed' }, { pull_request: {} }]) {
    assert.equal((await simulate({ issue })).writes.length, 0);
  }
});

test('unrelated edits, label events and PR payloads do not even query APIs', async () => {
  for (const payload of [
    { action: 'edited', changes: { assignees: {} } },
    { action: 'labeled' }, { issue: { number: 123, pull_request: {} } }, { issue: null },
  ]) {
    assert.deepEqual((await simulate({ payload })).calls, []);
  }
});

test('API failures fail the run instead of treating missing data as permission to write', async () => {
  await assert.rejects(simulate({ getError: true }), /API unavailable/);
  await assert.rejects(simulate({ paginateError: true }), /Incomplete API read/);
  await assert.rejects(simulate({ addError: true }), /Write denied/);
});

test('workflow has narrow triggers and permissions, no checkout or issue interpolation', () => {
  assert.match(workflow, /types: \[opened, edited\]/);
  assert.match(workflow, /permissions:\n  issues: write\n\n/);
  assert.match(workflow, /cancel-in-progress: false/);
  assert.match(workflow, /github\.repository == 'edwardkim\/rhwp'/);
  assert.match(workflow, /github\.event\.changes\.body != null/);
  assert.match(workflow, /github\.event\.changes\.title != null/);
  assert.match(workflow, /timeout-minutes: 3/);
  assert.match(workflow, /uses: actions\/github-script@[a-f0-9]{40}/);
  assert.doesNotMatch(workflow, /actions\/checkout|pull_request_target|workflow_dispatch/);
  assert.doesNotMatch(script, /\$\{\{|console\.|core\.(?:info|warning)\([^\n]*\.body/);
  assert.ok(read('.github/workflows/ci.yml').includes('node --test scripts/tests/issue-form-labels.test.mjs'));
});

test('shipped form choices match the supported classifier and contain no OS label defaults', async () => {
  const directory = new URL('.github/ISSUE_TEMPLATE/', root);
  const filenames = readdirSync(directory).filter((name) => name !== 'config.yml').sort();
  assert.deepEqual(filenames, [
    '01-document-bug.yml', '02-operation-bug.yml', '03-feature-request.yml', '04-documentation.yml',
  ]);
  for (const filename of filenames) {
    const yaml = read(`.github/ISSUE_TEMPLATE/${filename}`);
    const surface = yaml.match(/    id: "surface"\n([\s\S]*?)(?=  - type:|$)/)[1];
    const options = [...surface.matchAll(/^        - "(.*)"$/gm)].map((match) => match[1]);
    assert.deepEqual(options, ['Studio', 'CLI', '브라우저 확장', '라이브러리/API', 'MCP', '기타', '모름', '해당 없음']);
    const headings = [...yaml.matchAll(/^      label: "(.*)"$/gm)].map((match) => match[1]);
    assert.equal(new Set(headings).size, headings.length);
    // Build the Markdown shape from the actual form labels to catch renamed headings.
    const body = headings.map((heading) => `### ${heading}\n\n${heading === '사용 경로' ? 'CLI' : '_No response_'}`).join('\n\n');
    assert.deepEqual((await simulate({ issue: { body } })).writes[0].labels, ['cli']);
    const defaults = yaml.match(/labels:\n([\s\S]*?)body:/)[1];
    const expected = filename.startsWith('03') ? 'enhancement' : filename.startsWith('04') ? 'documentation' : 'bug';
    assert.equal(defaults.trim(), `- "${expected}"`);
    if (/^0[12]-/.test(filename)) {
      for (const os of ['Windows', 'macOS', 'Linux', '기타', '모름']) {
        assert.ok(yaml.includes(`- "${os}"`));
      }
      assert.match(yaml, /id: "materials"/);
    }
  }
  const config = read('.github/ISSUE_TEMPLATE/config.yml');
  assert.match(config, /^blank_issues_enabled: true/m);
  assert.match(config, /https:\/\/github.com\/edwardkim\/rhwp\/discussions/);
  assert.doesNotMatch(read('CONTRIBUTING.md'), /template=bug_report\.md/);
});

const freeIssue = (title, body = '## 현상\n\n재현 절차와 기대 동작', extra = {}) =>
  ({ title, body, labels: [], ...extra });
const added = (result) => result.writes.flatMap((write) => write.labels).sort();
const reason = (result, code) => assert.ok(
  result.info.some((message) => message.includes(code)), `Missing skip reason: ${code}`);

test('free-form Studio webfont report #8614 adds only evidence-backed labels', async () => {
  const result = await simulate({ issue: freeIssue(
    '[Bug] Studio 웹폰트 응답 무기한 대기로 문서 초기화가 멈춤',
    '## 현상\n\n웹폰트 로딩 상태에서 멈춥니다.\n\n' +
    '`rhwp-studio/src/main.ts`의 initialize는 `await loadWebFonts()` 이후 초기화합니다.\n' +
    '`rhwp-studio/src/core/font-loader.ts`의 `FontFace.load()`에 대기 제한이 없습니다.',
  ) });
  assert.deepEqual(added(result), ['bug', 'font', 'rhwp-studio']);
  assert.equal(result.writes.length, 1);
});

test('API creation is not the api product surface; clear proposed API #8021 is enhancement', async () => {
  const result = await simulate({ issue: freeIssue(
    '[API] 색인 표시를 넣고 읽는 API가 없다 — IndexMark는 읽고 저장만 한다',
    '저장: HWP와 HWPX를 정상적으로 씁니다.\n\n제안하는 API입니다.\n\n' +
    '- `insertIndexMark(sec, para, offset, firstKey, secondKey)`\n- `getIndexMarks()`',
  ) });
  assert.deepEqual(added(result), ['api', 'enhancement']);
  const apiCreated = await simulate({ issue: freeIssue('문서 처리 확인', 'GitHub API로 등록했습니다.') });
  assert.deepEqual(added(apiCreated), []);
  reason(apiCreated, 'no-surface-evidence');
  reason(apiCreated, 'no-kind-evidence');
  assert.deepEqual(added(await simulate({ issue: freeIssue('GitHub API 이슈 등록', '') })), []);
});

test('described single-surface form values work without changing exact form defaults', async () => {
  assert.deepEqual(added(await simulate({ issue: { body: form('Studio — 브라우저 편집기') } })), ['rhwp-studio']);
  assert.deepEqual(added(await simulate({ issue: { body: form('CLI(native)') } })), ['cli']);
  const result = await simulate({ issue: { body: form('CLI(native), WASM(rhwp-studio)') } });
  assert.deepEqual(added(result), []);
  reason(result, 'ambiguous-surface');
});

test('free-form explicit surfaces, kinds and domains use a fixed allowlist', async () => {
  for (const [title, expected] of [
    ['[Bug] CLI 변환 실패', ['bug', 'cli']],
    ['[Docs] API 예제 보완', ['api', 'documentation']],
    ['[Feature] MCP 도구 추가', ['enhancement', 'mcp']],
    ['[Bug] 브라우저 확장 렌더링 오류', ['browser-extension', 'bug', 'rendering']],
    ['[Test] CLI 회귀 검사 보강', ['cli', 'test']],
  ]) {
    assert.deepEqual(added(await simulate({ issue: freeIssue(title) })), expected.sort());
  }
});

test('only explicit product paths/packages in prose supply body-only surface evidence', async () => {
  for (const [body, expected] of [
    ['`rhwp-studio/src/main.ts`에서 초기화가 멈춥니다.', ['rhwp-studio']],
    ['`@rhwp/core` 사용 중입니다.', ['api']],
    ['`rhwp-firefox/src/viewer.js`에서 발생합니다.', ['browser-extension']],
    ['API로 이슈를 등록했습니다. CLI와 MCP도 참고했습니다.', []],
  ]) {
    assert.deepEqual(added(await simulate({ issue: freeIssue('확인 요청', body) })), expected);
  }
});

test('third-party Studio IDE names are not the rhwp Studio surface', async () => {
  for (const title of [
    '[Bug] Visual Studio에서 빌드 실패',
    '[Bug] Android Studio integration fails',
    '[Bug] Visual Studio Code integration fails',
  ]) {
    const result = await simulate({ issue: freeIssue(title) });
    assert.deepEqual(added(result), ['bug']);
    reason(result, 'no-surface-evidence');
  }
  for (const issue of [
    freeIssue('[Bug][studio] Visual Studio에서 재현한 오류'),
    freeIssue('[Bug] Android Studio integration fails', '`rhwp-studio/src/main.ts`에서 발생합니다.'),
    freeIssue('[Bug] rhwp-studio와 Visual Studio 연동 오류'),
  ]) {
    assert.deepEqual(added(await simulate({ issue })), ['bug', 'rhwp-studio']);
  }
});

test('ambiguous surfaces or kinds are skipped independently with explicit reasons', async () => {
  const surface = await simulate({ issue: freeIssue('[Bug] CLI와 Studio 결과 차이') });
  assert.deepEqual(added(surface), ['bug']);
  reason(surface, 'ambiguous-surface');
  const kind = await simulate({ issue: freeIssue('[Bug][Feature] CLI 동작 변경') });
  assert.deepEqual(added(kind), ['cli']);
  reason(kind, 'ambiguous-kind');
});

test('manual free-form surface and kind decisions are preserved', async () => {
  const result = await simulate({ issue: freeIssue('[Bug] Studio 글꼴 오류', '',
    { labels: ['api', 'enhancement', 'custom'] }) });
  assert.deepEqual(added(result), []);
  assert.deepEqual(result.current.labels, ['api', 'enhancement', 'custom']);
  reason(result, 'existing-surface-decision');
  reason(result, 'existing-kind-decision');
});

test('inferred labels are idempotent and manual removals suppress only that label', async () => {
  const first = await simulate({ issue: freeIssue('[Bug] Studio 글꼴 로딩 실패') });
  const retry = await simulate({ issue: first.current });
  assert.deepEqual(added(retry), []);
  reason(retry, 'already-present');
  const removed = await simulate({ issue: freeIssue('[Bug] Studio 글꼴 로딩 실패'),
    events: [{ event: 'unlabeled', label: { name: 'font' } }] });
  assert.deepEqual(added(removed), ['bug', 'rhwp-studio']);
  reason(removed, 'previously-removed');
});

test('uncertain test/provenance investigations never imply bug or serialization', async () => {
  for (const issue of [
    freeIssue('[7445] 끝 탭 런·charX·hit-test 회귀 재구축',
      '이번 실패는 화면 위치 결함 입증이 아니라 단일 런 문자열 가정이 깨진 것입니다.'),
    freeIssue('[samples] 저장 조판 rhwp 산출 추정, 기준 재검토 필요',
      '추정, 미재현. 현재 rhwp로 재현되지 않는다. 샘플 출처 조사 필요.'),
  ]) {
    const result = await simulate({ issue });
    assert.deepEqual(added(result), []);
    reason(result, 'no-kind-evidence');
  }
});

test('code fences, quoted reports, comments and URLs are not evidence or log content', async () => {
  const body = '```md\n' + form() + '\n```\n' +
    '> [Bug] CLI 오류\n> rhwp-studio/src/main.ts\n' +
    '<!-- rhwp-studio/src/main.ts -->\n' +
    '[다른 보고서](https://example.org/rhwp-studio/src/main.ts)\n' +
    '<img src="data:image/png;base64,PRIVATE_ATTACHMENT">';
  const result = await simulate({ issue: freeIssue('확인 요청', body) });
  assert.deepEqual(added(result), []);
  assert.doesNotMatch(result.info.join('\n'), /PRIVATE_ATTACHMENT|example\.org|확인 요청/);
  for (const code of [
    '    rhwp-studio/src/main.ts', '\trhwp-studio/src/main.ts',
    '```js\n```not-a-closing-fence\nrhwp-studio/src/main.ts',
    '~~~~\n~~~\nrhwp-studio/src/main.ts',
  ]) {
    assert.deepEqual(added(await simulate({ issue: freeIssue('확인 요청', code) })), []);
  }
});

test('free-form classification rechecks title and every label before the batch write', async () => {
  for (const latest of [
    { title: '[Feature] CLI 개선' },
    { labels: ['enhancement'] },
    { labels: [{ name: 'custom' }] },
    { pull_request: {} },
  ]) {
    const result = await simulate({ issue: freeIssue('[Bug] Studio 글꼴 오류'), latest });
    assert.deepEqual(added(result), []);
    reason(result, 'snapshot-changed');
  }
});

test('title-only edits reclassify current title, unrelated edits remain no-ops', async () => {
  const result = await simulate({ issue: freeIssue('[Bug] CLI 변환 실패'),
    payload: { action: 'edited', changes: { title: { from: '확인 요청' } } } });
  assert.deepEqual(added(result), ['bug', 'cli']);
  const unrelated = await simulate({ payload: { action: 'edited', changes: { assignees: {} } } });
  assert.deepEqual(unrelated.calls, []);
  reason(unrelated, 'unsupported-event');
});

test('missing inferred labels permit independent available labels, never creation', async () => {
  const result = await simulate({ issue: freeIssue('[Bug] Studio 글꼴 오류'), available: ['bug', 'rhwp-studio'] });
  assert.deepEqual(added(result), ['bug', 'rhwp-studio']);
  assert.equal(result.warnings.length, 1);
  assert.match(result.warnings[0], /missing-repository-label: font/);
});

test('unsupported or duplicate form responses cannot escape through title/path hints', async () => {
  for (const [body, code] of [
    [form('모름'), 'unknown-form-surface'],
    [form('Studio, CLI'), 'ambiguous-surface'],
    [form() + '\n\n### 사용 경로\n\nCLI', 'ambiguous-form'],
    [form() + '\n\n### 동작 문제와 기대 결과\n\n오류\n\n### 재현 순서\n\n실행', 'ambiguous-form'],
    ['### 사용 경로\n\nStudio', 'incomplete-form'],
    ['### 사용 경로\n\nStudio — 브라우저 편집기', 'incomplete-form'],
  ]) {
    const result = await simulate({ issue: freeIssue('[Bug] Studio 글꼴 오류', body) });
    // Unsupported complete forms block every fallback, including type/domain hints.
    // Incomplete forms suppress surface hints but retain independently explicit title evidence.
    assert.deepEqual(added(result), code === 'incomplete-form' ? ['bug', 'font'] : []);
    reason(result, code);
  }
});

test('all skip paths explain decisions without leaking raw issue content', async () => {
  const closed = await simulate({ issue: { state: 'closed' } });
  reason(closed, 'not-open-issue');
  const empty = await simulate({ issue: freeIssue('', null) });
  reason(empty, 'no-surface-evidence');
  reason(empty, 'no-kind-evidence');
});
