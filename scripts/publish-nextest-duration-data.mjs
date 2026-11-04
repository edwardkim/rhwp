import { createHash } from 'node:crypto';
import { parseDurationPolicy } from './select-nextest-archive-targets.mjs';

// 실행 시간 자료만 게시한다. 제품 branch 전환·강제 push·검증 CI 재실행은 하지 않는다.
const REF = 'heads/ci-metrics/nextest-target-durations';
const FILE = 'nextest-target-duration-policy.json';
const SHA = /^[0-9a-f]{40}$/;
export async function publishNextestDurationData({ github, owner, repo, mergeSha, content }) {
  if (!SHA.test(mergeSha) || typeof content !== 'string'
      || Buffer.byteLength(content) > 8 * 1024 * 1024) throw new Error('duration-publish: invalid publication input');
  parseDurationPolicy(JSON.parse(content));
  const base = { owner, repo };
  const ref = async (name, optional = false) => {
    try { return (await github.rest.git.getRef({ ...base, ref: name })).data.object.sha; }
    catch (error) { if (optional && error.status === 404) return null; throw error; }
  };
  const held = reason => ({ published: false, reason, sourceMergeSha: mergeSha });
  if (await ref('heads/devel') !== mergeSha) return held('devel-advanced');
  const previousSha = await ref(REF, true);
  let baseTree;
  // 같은 자료가 이미 게시됐으면 쓰기 API를 호출하지 않는다.
  const blobSha = createHash('sha1').update(`blob ${Buffer.byteLength(content)}\0`).update(content).digest('hex');
  if (previousSha) {
    baseTree = (await github.rest.git.getCommit({ ...base, commit_sha: previousSha })).data.tree.sha;
    const entries = (await github.rest.git.getTree({ ...base, tree_sha: baseTree })).data.tree;
    if (entries.some(entry => entry.path === FILE && entry.type === 'blob' && entry.sha === blobSha)) {
      return { published: false, reason: 'unchanged', commitSha: previousSha, sourceMergeSha: mergeSha };
    }
  }
  const blob = (await github.rest.git.createBlob({ ...base, content, encoding: 'utf-8' })).data;
  if (blob.sha !== blobSha) throw new Error('duration-publish: blob identity mismatch');
  const tree = (await github.rest.git.createTree({ ...base, ...(baseTree ? { base_tree: baseTree } : {}),
    tree: [{ path: FILE, mode: '100644', type: 'blob', sha: blob.sha }] })).data;
  const commit = (await github.rest.git.createCommit({ ...base,
    message: `ci: nextest target duration data ${mergeSha}`, tree: tree.sha,
    parents: previousSha ? [previousSha] : [] })).data;
  // 자료 준비 중 다른 merge나 metrics writer가 전진했으면 현재 자료를 덮지 않는다.
  if (await ref('heads/devel') !== mergeSha) return held('devel-advanced');
  if (await ref(REF, true) !== previousSha) return held('metrics-advanced');
  try {
    if (previousSha) await github.rest.git.updateRef({ ...base, ref: REF, sha: commit.sha, force: false });
    else await github.rest.git.createRef({ ...base, ref: `refs/${REF}`, sha: commit.sha });
  } catch (error) {
    // 응답만 유실된 성공과 게시 직전 다른 writer가 선점한 경우를 실제 ref로 구분한다.
    const current = await ref(REF, true);
    if (current === commit.sha) return { published: true, reason: 'verified-after-response-error',
      previousSha, commitSha: commit.sha, blobSha, sourceMergeSha: mergeSha };
    if ([409, 422].includes(error.status) && current !== previousSha) return held('metrics-advanced');
    // 서버/권한 오류를 정상 보류로 숨기거나 무조건 재시도하지 않는다.
    throw new Error(`duration-publish: ref update failed (HTTP ${error.status ?? 'unknown'}; `
      + `previous=${previousSha ?? 'absent'}; candidate=${commit.sha}; `
      + `request=${error.response?.headers?.['x-github-request-id'] ?? 'unknown'})`, { cause: error });
  }
  if (await ref(REF) !== commit.sha) throw new Error('duration-publish: published ref identity mismatch');
  return { published: true, reason: 'published', previousSha, commitSha: commit.sha, blobSha, sourceMergeSha: mergeSha };
}
