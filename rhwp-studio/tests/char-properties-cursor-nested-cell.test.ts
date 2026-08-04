import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#2756] getCharPropertiesAtCursor() 의 중첩 셀 좌표 축 가드.
//
// flat controlIndex/cellIndex/cellParaIndex 는 hit-test 가 cellPath[0](최외곽)에서 채운다.
// 중첩 표 셀에서 이를 그대로 getCellCharPropertiesAt(flat)에 넘기면 **바깥 셀**의 글자 서식을
// 읽는다. applyToggleFormat 이 이 값에서 !current[prop] 로 토글 **방향**을 정하므로(그리고
// 실제 적용 ApplyCharFormatCommand 는 이미 ...ByPath 로 안쪽 셀에 적용) 방향이 어긋나
// Ctrl+B/I 가 거꾸로 동작하고, emitCursorFormatState 의 툴바 표시도 오답이 된다.
//
// 형제 선례: ApplyCharFormatCommand 는 이미 getCellCharPropertiesAtByPath 로 교정됐고
// tests/apply-charformat-nested-cell.test.ts 가 그 커맨드를 핀한다. 그러나 그 테스트는
// command.ts 의 커맨드 블록만 보므로 input-handler.ts 의 이 조회는 시야 밖이었다.
//
// 정적 가드인 이유: getCharPropertiesAtCursor 는 wasm 응답을 그대로 돌려줄 뿐 분기 로직이
// 없어(경로 유무만 판단), 동작 증명은 "어느 API 를 호출하는가"로 충분하다. 행위 red→green 은
// comparePositions 쪽(selection-ordering-nested-cell.test.ts)에서 실경로로 확보한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8');

/** `NAME(` 시그니처부터 매칭 중괄호가 닫힐 때까지 메서드 본문을 추출. */
function methodBody(name: string): string {
  const sig = new RegExp(`\\b${name}\\s*\\([^)]*\\)\\s*:[^\\{]*\\{`);
  const m = sig.exec(src);
  assert.ok(m, `${name} 메서드 not found`);
  const open = src.indexOf('{', m.index + m[0].length - 1);
  let depth = 0;
  for (let i = open; i < src.length; i++) {
    if (src[i] === '{') depth++;
    else if (src[i] === '}') {
      depth--;
      if (depth === 0) return src.slice(open, i + 1);
    }
  }
  throw new Error(`${name} 본문의 닫는 괄호를 찾지 못함`);
}

test('getCharPropertiesAtCursor 는 cellPath 가 있으면 ...ByPath 로 최내곽 셀을 조회한다', () => {
  const body = methodBody('getCharPropertiesAtCursor');
  assert.match(
    body,
    /getCellCharPropertiesAtByPath\s*\(/,
    '중첩 셀 서식 조회는 getCellCharPropertiesAtByPath(안쪽 축)를 써야 한다',
  );
  assert.match(
    body,
    /pos\.cellPath\?\.length/,
    'cellPath 유무로 경로/flat 을 분기해야 한다',
  );
});

test('getCharPropertiesAtCursor 의 flat 조회는 cellPath 부재 폴백으로만 남는다', () => {
  const body = methodBody('getCharPropertiesAtCursor');
  // flat 호출 자체는 depth 1/레거시 폴백으로 유지되지만, ByPath 분기보다 뒤에 와야 한다.
  const byPathIdx = body.indexOf('getCellCharPropertiesAtByPath');
  const flatIdx = body.indexOf('getCellCharPropertiesAt(');
  assert.ok(byPathIdx !== -1, 'ByPath 분기가 있어야 한다');
  assert.ok(
    flatIdx === -1 || byPathIdx < flatIdx,
    'flat getCellCharPropertiesAt 은 ByPath 분기 뒤(폴백)에 와야 한다 — 중첩 셀이 flat 으로 새면 안 된다',
  );
});
