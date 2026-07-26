import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2370 클러스터 B] 스냅샷 라우팅과 중복되는 리프레시/emit 방지 가드.
//
// executeOperation 의 snapshot 분기는 기본 'full' refresh 로 afterEdit() 를 부르고,
// afterEdit 이 'document-mutated' 와 'document-changed' 를 emit 한다
// (input-handler.ts). 라우팅 이관 당시 예전 경로의 수동 emit 이 남아 한 번의 편집에
// 두 번씩 발화했다. 구독자(markDirty·autosave)가 idempotent 라 증상은 없지만 순손해다.
// 되살아나면 다음 사람이 "여기선 수동 emit 이 필요한가 보다"로 읽고 퍼뜨린다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string) => readFileSync(join(rootDir, rel), 'utf8');

test('afterEdit 이 document-mutated·document-changed 를 모두 emit 한다(전제)', () => {
  const ih = src('src/engine/input-handler.ts');
  const idx = ih.indexOf("this.eventBus.emit('document-mutated', 'input-handler-edit');");
  assert.notEqual(idx, -1, 'afterEdit 의 document-mutated emit');
  assert.match(
    ih.slice(idx, idx + 200),
    /emit\('document-changed'\)/,
    'afterEdit 은 document-changed 도 이어서 emit 한다',
  );
});

test('누름틀 넣기·고치기는 수동 document-mutated 를 다시 emit 하지 않는다', () => {
  for (const rel of ['src/command/commands/insert.ts', 'src/command/commands/edit.ts']) {
    assert.doesNotMatch(
      src(rel),
      /emit\('document-mutated'/,
      `${rel}: 스냅샷 라우팅의 afterEdit 이 이미 emit 한다 — 수동 emit 은 중복`,
    );
  }
});

test('셀 숫자 서식 3종은 safeTableOp 을 겹쳐 감싸지 않는다', () => {
  const table = src('src/command/commands/table.ts');
  // 세 핸들러의 executeOperation 은 바깥 try 의 마지막 문장이라 safeTableOp 을 덧대면
  // 바깥 catch 가 도달 불가가 된다(관측 차이는 로그 레벨뿐). 한 겹만 남긴다.
  const wrapped = [...table.matchAll(/safeTableOp\(\(\) => ih\.executeOperation\(\{\s*\n\s*kind: 'snapshot',\s*\n\s*operationType: 'cellNumberFormat'/g)];
  assert.deepEqual(wrapped.map((m) => m[0]), [], 'cellNumberFormat 은 safeTableOp 이중 래핑 금지');
  // 라우팅 자체는 유지돼야 한다.
  assert.equal(
    [...table.matchAll(/operationType: 'cellNumberFormat'/g)].length,
    3,
    '쉼표·자릿점 넣기·자릿점 빼기 3종이 모두 snapshot 라우팅을 유지해야 함',
  );
  // 구체적인 실패 메시지를 내는 바깥 catch 는 살아 있어야 한다.
  for (const id of ['thousand-sep', 'decimal-add', 'decimal-remove']) {
    assert.match(table, new RegExp(`\\[table:${id}\\][^\\n]*실패`), `${id} 의 바깥 catch 로그 유지`);
  }
});
