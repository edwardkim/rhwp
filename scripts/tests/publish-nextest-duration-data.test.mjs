// Git Data API의 실제 계약(단일 부모·비강제 ref 갱신)과 경쟁/실패/동일 자료를 검사한다.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { publishNextestDurationData } from '../publish-nextest-duration-data.mjs';
const mergeSha = 'a'.repeat(40), previous = 'b'.repeat(40), next = 'c'.repeat(40), other = 'd'.repeat(40);
const content = JSON.stringify({ schema_version: 1, fallback_seconds: 1, targets: { example: 2 } }) + '\n';
const hash = value => createHash('sha1').update(`blob ${Buffer.byteLength(value)}\0`).update(value).digest('hex');
function fixture(options = {}) {
  const refs = { 'heads/devel': options.oldDevel ? other : mergeSha,
    'heads/ci-metrics/nextest-target-durations': options.absent ? null : previous };
  const calls = []; let updates = 0;
  const git = {
    getRef: async ({ ref }) => {
      if (!refs[ref]) throw Object.assign(new Error('not found'), { status: 404 });
      return { data: { object: { sha: refs[ref] } } };
    },
    getCommit: async () => ({ data: { tree: { sha: 'old-tree' } } }),
    getTree: async () => ({ data: { tree: [{ path: 'nextest-target-duration-policy.json', type: 'blob',
      sha: options.unchanged ? hash(content) : 'old-blob' }] } }),
    createBlob: async args => { calls.push(['blob', args]); return { data: { sha: options.badBlob ? other : hash(args.content) } }; },
    createTree: async args => { calls.push(['tree', args]); return { data: { sha: 'new-tree' } }; },
    createCommit: async args => {
      calls.push(['commit', args]);
      if (options.develAdvance) refs['heads/devel'] = other;
      if (options.metricsAdvance) refs['heads/ci-metrics/nextest-target-durations'] = other;
      return { data: { sha: next } };
    },
    updateRef: async args => {
      calls.push(['update', args]); updates++;
      assert.equal(args.force, false);
      if (options.race) { refs[args.ref] = other; throw Object.assign(new Error('not a fast-forward'), { status: 422 }); }
      if (options.serverError) throw Object.assign(new Error('Internal Server Error'), { status: 500,
        response: { headers: { 'x-github-request-id': 'request-id' } } });
      if (options.denied) throw Object.assign(new Error('forbidden'), { status: 403 });
      refs[args.ref] = args.sha;
      if (options.lostResponse) throw Object.assign(new Error('response lost'), { status: 500 });
      return { data: {} };
    },
    createRef: async args => { calls.push(['create-ref', args]); refs[args.ref.replace(/^refs\//, '')] = args.sha; return { data: {} }; },
  };
  return { calls, refs, updates: () => updates,
    run: () => publishNextestDurationData({ github: { rest: { git } }, owner: 'edwardkim', repo: 'rhwp', mergeSha, content }) };
}
test('검증 데이터는 기존 metrics 단일 부모와 비강제 API 갱신으로 게시한다', async () => {
  const f = fixture(); const r = await f.run();
  assert.equal(r.published, true); assert.equal(r.commitSha, next); assert.equal(f.updates(), 1);
  assert.deepEqual(f.calls.find(([type]) => type === 'commit')[1].parents, [previous]);
  assert.equal(f.calls.find(([type]) => type === 'tree')[1].base_tree, 'old-tree');
  assert.deepEqual(f.calls.find(([type]) => type === 'tree')[1].tree,
    [{ path: 'nextest-target-duration-policy.json', mode: '100644', type: 'blob', sha: hash(content) }]);
});
test('자료가 같으면 새 커밋이나 ref 쓰기를 하지 않는다', async () => {
  const f = fixture({ unchanged: true }); assert.equal((await f.run()).reason, 'unchanged'); assert.deepEqual(f.calls, []);
});
test('처음 생성하는 metrics branch는 다른 branch 부모 없이 생성한다', async () => {
  const f = fixture({ absent: true }); assert.equal((await f.run()).published, true);
  assert.deepEqual(f.calls.find(([type]) => type === 'commit')[1].parents, []);
  assert.equal(f.calls.at(-1)[0], 'create-ref');
});
for (const [option, reason] of [['oldDevel', 'devel-advanced'], ['develAdvance', 'devel-advanced'],
  ['metricsAdvance', 'metrics-advanced'], ['race', 'metrics-advanced']]) {
  test(`${option}: 전진한 원격 기록을 이전 측정값으로 덮지 않는다`, async () => {
    const f = fixture({ [option]: true }); const r = await f.run();
    assert.equal(r.published, false); assert.equal(r.reason, reason);
    if (option !== 'race') assert.equal(f.updates(), 0);
    if (['metricsAdvance', 'race'].includes(option)) assert.equal(f.refs['heads/ci-metrics/nextest-target-durations'], other);
  });
}
for (const [option, status] of [['serverError', 500], ['denied', 403]]) {
  test(`${status}: 실패를 성공으로 바꾸거나 무조건 재시도하지 않는다`, async () => {
    const f = fixture({ [option]: true }); await assert.rejects(f.run(), new RegExp(`HTTP ${status}`));
    assert.equal(f.updates(), 1); assert.equal(f.refs['heads/ci-metrics/nextest-target-durations'], previous);
  });
}
test('응답이 유실돼도 원격 ref가 정확한 후보면 실제 게시 성공을 확인한다', async () => {
  const f = fixture({ lostResponse: true }); const r = await f.run();
  assert.equal(r.reason, 'verified-after-response-error'); assert.equal(f.updates(), 1);
});
test('원격 blob 내용 식별자가 다르면 commit/ref를 만들지 않는다', async () => {
  const f = fixture({ badBlob: true }); await assert.rejects(f.run(), /blob identity mismatch/);
  assert.equal(f.calls.length, 1); assert.equal(f.updates(), 0);
});
test('잘못된 정책은 API 호출 전에 거부한다', async () => {
  await assert.rejects(publishNextestDurationData({ github: {}, owner: 'x', repo: 'y', mergeSha,
    content: '{"schema_version":2}' }), /fallback_seconds_per_test/);
});
