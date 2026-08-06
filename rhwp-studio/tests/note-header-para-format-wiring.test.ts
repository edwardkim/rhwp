import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 머리말/꼬리말·각주 문단 서식이 "구현은 됐는데 아무도 안 부르는" 상태로 되돌아가는 것을
// 막는다.
//
// 결함 형태: 코어에 applyParaFormatInHf / applyParaFormatInFootnote 가 있고 브리지에도
// 노출돼 있는데 호출부가 0곳이었다. getParaFormatTargetsForRange 가 두 문맥에서 빈 배열을
// 반환해, 정렬·줄 간격을 눌러도 아무 반응 없이 끝났다. 조회 쪽(getParaProperties)은 두
// 문맥을 정확히 분기하고 있어서 툴바 표시만 맞고 적용은 안 되는 비대칭이었다.
//
// 그래서 "브리지에 있는 쓰기 API 가 실제로 호출되는가"를 직접 센다. 호출부가 사라지면
// (이 결함의 재발 형태) 실패한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(rel: string): string {
  return readFileSync(join(rootDir, rel), 'utf8');
}

/** src/ 아래 .ts 전수 (테스트 제외). */
function sourceFiles(): string[] {
  const out: string[] = [];
  const walk = (rel: string) => {
    for (const ent of readdirSync(join(rootDir, rel), { withFileTypes: true })) {
      const child = `${rel}/${ent.name}`;
      if (ent.isDirectory()) walk(child);
      else if (ent.name.endsWith('.ts') && !ent.name.endsWith('.test.ts')) out.push(child);
    }
  };
  walk('src');
  return out;
}

/** 브리지 정의와 뮤테이션 레지스트리 등재는 "호출"이 아니다. */
const NOT_A_CALL_SITE = new Set([
  'src/core/wasm-bridge.ts',
  'src/core/mutation-method-registry.ts',
]);

function callSites(method: string): string[] {
  const re = new RegExp(`\\.\\s*${method}\\s*\\(`);
  return sourceFiles()
    .filter(rel => !NOT_A_CALL_SITE.has(rel))
    .filter(rel => re.test(source(rel)));
}

for (const method of ['applyParaFormatInHf', 'applyParaFormatInFootnote']) {
  test(`${method} 는 호출되는 코드가 있어야 한다`, () => {
    const sites = callSites(method);
    assert.ok(
      sites.length > 0,
      `${method} 가 브리지에만 있고 호출부가 없다 — 머리말/꼬리말·각주에서 문단 서식이 `
        + `아무 반응 없이 끝난다. getParaProperties 는 두 문맥을 분기하므로 툴바 표시만 `
        + `맞고 적용은 안 되는 비대칭이 된다.`,
    );
  });
}

test('문단 서식 진입점이 머리말/꼬리말과 각주를 모두 분기한다', () => {
  const src = source('src/engine/input-handler.ts');
  const start = src.indexOf('private applyParaFormatInNoteOrHeader');
  assert.notEqual(start, -1, 'applyParaFormatInNoteOrHeader 를 찾지 못함');
  const end = src.indexOf('\n  private ', start + 1);
  const block = src.slice(start, end === -1 ? undefined : end);

  // 한쪽만 배선하면(이 결함의 부분 수정 형태) 실패한다.
  assert.match(block, /isInHeaderFooter\(\)/);
  assert.match(block, /applyParaFormatInHf\(/);
  assert.match(block, /isInFootnote\(\)/);
  assert.match(block, /applyParaFormatInFootnote\(/);
});

test('문단 서식 진입점이 그 분기를 실제로 호출한다', () => {
  // 분기 메서드가 있어도 진입점에서 부르지 않으면 결함은 그대로다.
  // (이 단언 없이는 진입점 호출을 지워도 테스트가 통과했다.)
  const src = source('src/engine/input-handler.ts');
  const start = src.indexOf('private applyParaFormat(props: Record<string, unknown>): void {');
  assert.notEqual(start, -1, 'applyParaFormat 진입점을 찾지 못함');
  const end = src.indexOf('\n  private ', start + 1);
  const entry = src.slice(start, end === -1 ? undefined : end);

  assert.match(
    entry,
    /this\.applyParaFormatInNoteOrHeader\(props\)/,
    'applyParaFormat 이 머리말/꼬리말·각주 분기를 호출하지 않는다 — 분기가 도달 불가라 '
      + '정렬·줄 간격이 여전히 무반응이다.',
  );
});

test('되돌리기 라우팅을 거친다 (executeOperation 경유)', () => {
  const src = source('src/engine/input-handler.ts');
  const start = src.indexOf('private applyParaFormatInNoteOrHeader');
  const end = src.indexOf('\n  private ', start + 1);
  const block = src.slice(start, end === -1 ? undefined : end);

  // 직접 wasm 을 부르면 undo 에 안 남고 redo 스택도 무효화되지 않는다 (#2327).
  const executeOperationCount = [...block.matchAll(/this\.executeOperation\(/g)].length;
  assert.equal(executeOperationCount, 2, '머리말/꼬리말과 각주 각각 executeOperation 을 거쳐야 한다');
});
