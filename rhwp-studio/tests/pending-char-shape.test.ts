import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import * as nodeModule from 'node:module';

/**
 * [#3080] 글자 서식이 "안 먹는" 사용자 신고의 회귀 게이트.
 *
 * 신고: ① 굵게 버튼이 안 먹는다 ② 색도 안 먹는다 ③ 입력하면 글꼴이 리셋된다
 * ④ "단축키는 가는데 굵어지지 않는다".
 *
 * 원인은 서식 경로 전체가 **선택 전용**이었다는 것이다. 캐럿만 있으면
 *   - applyToggleFormat 이 `if (!hasSelection()) return;` 로 무언 종료
 *   - format-char(글꼴/크기/색)도 `if (hasSelection())` 안에서만 적용
 *   - format:char-shape 는 `if (!savedSel) return;` 로 대화상자조차 안 열림
 * 이라 아무 일도 안 일어나고, 다음 입력 때 툴바가 런의 실제 모양을 다시 읽어
 * "글꼴이 리셋"되는 것처럼 보였다.
 *
 * 고침은 **캐럿 대기 글자 모양(pending char shape)** — 선택 없이 지정한 서식을 예약해
 * 다음 삽입 런에 적용한다. 행위 증명은 실제 wasm 문서 왕복으로 한다(러너 spawn).
 */

const here = dirname(fileURLToPath(import.meta.url));
const studioDir = dirname(here);
const runner = join(here, 'support', 'pending-char-shape.runner.mjs');
const nodeWasm = join(studioDir, '..', 'pkg-node', 'rhwp.js');

function transformTypesSupported(): boolean {
  return process.allowedNodeEnvironmentFlags.has('--experimental-transform-types')
    && typeof (nodeModule as { registerHooks?: unknown }).registerHooks === 'function';
}

test('굵게/색/캐럿 대기 서식이 실제 문서에 반영된다 (자식 프로세스 + wasm 왕복)', (t) => {
  if (!transformTypesSupported()) {
    t.skip('현재 Node 가 --experimental-transform-types / registerHooks 미지원 — 행위 테스트 skip');
    return;
  }
  if (!existsSync(nodeWasm)) {
    t.skip('pkg-node 빌드가 없어 wasm 왕복 테스트 skip (wasm-pack --target nodejs)');
    return;
  }
  const res = spawnSync(
    process.execPath,
    ['--experimental-transform-types', '--no-warnings', runner],
    { encoding: 'utf8' },
  );
  assert.equal(res.status, 0,
    `러너가 비정상 종료했습니다.\n--- stdout ---\n${res.stdout}\n--- stderr ---\n${res.stderr}`);
  assert.match(res.stdout, /PENDING_CHAR_SHAPE_OK/, '행위 검증 성공 마커가 있어야 함');
});

// ─── 라우팅 가드 (소스 계약) ────────────────────────────────────────────────
// 위 행위 테스트는 커맨드 계층까지만 닿는다. InputHandler 는 DOM 이 필요해 로드할 수 없으므로,
// "선택이 없으면 예약한다 / 토글 방향을 선택 첫 글자로 정한다" 는 분기 자체를 소스로 고정한다.

const inputHandlerSrc = readFileSync(join(studioDir, 'src/engine/input-handler.ts'), 'utf8');

/** `NAME(` 시그니처부터 매칭 중괄호가 닫힐 때까지 메서드 본문을 추출. */
function methodBody(name: string): string {
  const sig = new RegExp(`\\b${name}\\s*\\([^)]*\\)\\s*:[^\\{]*\\{`);
  const m = sig.exec(inputHandlerSrc);
  assert.ok(m, `${name} 메서드 not found`);
  const open = inputHandlerSrc.indexOf('{', m.index + m[0].length - 1);
  let depth = 0;
  for (let i = open; i < inputHandlerSrc.length; i++) {
    if (inputHandlerSrc[i] === '{') depth++;
    else if (inputHandlerSrc[i] === '}') {
      depth--;
      if (depth === 0) return inputHandlerSrc.slice(open, i + 1);
    }
  }
  throw new Error(`${name} 본문의 닫는 괄호를 찾지 못함`);
}

test('선택 없는 서식 지정은 무언 종료하지 않고 pending 으로 예약된다', () => {
  const applyCharFormat = methodBody('applyCharFormat');
  assert.match(applyCharFormat, /stagePendingCharShape\(props\)/, '선택이 없으면 예약해야 한다');

  const toggle = methodBody('applyToggleFormat');
  assert.doesNotMatch(toggle, /if \(!this\.cursor\.hasSelection\(\)\) return;/,
    '캐럿만 있을 때 무언 종료하던 가드가 남아 있으면 Ctrl+B 가 다시 죽는다');

  for (const name of ['adjustFontSize', 'adjustCharRatio', 'adjustCharSpacing']) {
    assert.doesNotMatch(methodBody(name), /if \(!this\.cursor\.hasSelection\(\)\) return;/,
      `${name} 도 캐럿 상태에서 예약으로 이어져야 한다`);
  }

  // 툴바(글꼴/크기/색) 경로도 선택 유무와 무관하게 applyCharFormat 으로 들어가야 한다.
  const formatCharListener = inputHandlerSrc.slice(
    inputHandlerSrc.indexOf("eventBus.on('format-char'"),
    inputHandlerSrc.indexOf("eventBus.on('format-char'") + 600,
  );
  assert.match(formatCharListener, /this\.applyCharFormat\(props as Partial<CharProperties>\);/);
  assert.doesNotMatch(formatCharListener, /if \(this\.cursor\.hasSelection\(\)\) \{/,
    '선택이 있을 때만 적용하던 게이트가 남으면 글꼴/색이 다시 무언 no-op 이 된다');
});

test('토글 방향은 선택 첫 글자에서 읽는다 (역방향 드래그에서 거꾸로 동작 방지)', () => {
  const body = methodBody('getCharPropertiesAtCursor');
  assert.match(body, /const sel = this\.getNonEmptySelection\(\);/,
    '선택이 있으면 focus 가 아니라 선택 범위를 기준으로 삼아야 한다');
  assert.match(body, /const pos = sel \? sel\.start : this\.cursor\.getPosition\(\);/,
    '선택 시작(=문서 순서상 앞)이 기준이어야 한다');
  assert.match(body, /const queryOffset = sel \? pos\.charOffset :/,
    '선택 시작 offset 은 그 자리 글자가 곧 선택 첫 글자다 (offset-1 이면 선택 밖을 읽는다)');
});

test('빈 선택(anchor 만 있는 클릭)은 선택으로 치지 않는다', () => {
  // (반환 타입이 객체 리터럴이라 methodBody 의 중괄호 스캔이 타입을 먼저 문다 — 직접 슬라이스)
  const start = inputHandlerSrc.indexOf('private getNonEmptySelection(');
  assert.ok(start > 0, 'getNonEmptySelection 이 있어야 한다');
  const body = inputHandlerSrc.slice(start, start + 500);
  assert.match(body, /CursorState\.comparePositions\(sel\.start, sel\.end\) === 0/,
    'start==end 인 빈 범위는 null 로 접어야 한다 (applyCharFormat 이 조용히 no-op 되는 경로)');

  const hasSelection = inputHandlerSrc.match(/hasSelection\(\): boolean \{[\s\S]*?\n  \}/);
  assert.ok(hasSelection);
  assert.match(hasSelection[0], /this\.getNonEmptySelection\(\) !== null/,
    'EditorContext.hasSelection 도 빈 선택을 "선택 있음"으로 보고하면 안 된다');
});

test('삽입 경로가 예약 서식을 실어 나른다', () => {
  const textSrc = readFileSync(join(studioDir, 'src/engine/input-handler-text.ts'), 'utf8');
  assert.match(
    textSrc,
    /new InsertTextCommand\(insertPos, text, undefined, this\.getPendingCharShape\?\.\(\)\)/,
    '일반 입력은 예약 서식을 커맨드에 실어야 한다',
  );
  assert.match(
    textSrc,
    /this\.applyPendingCharShapeToRange\?\.\(anchor, charCount\(text\)\)/,
    'IME 조합 입력도 예약 서식을 적용해야 한다',
  );
  assert.match(
    inputHandlerSrc,
    /if \(desc\.command\.type === 'insertText'\) \{\s*\n\s*this\.advancePendingCharShapeAnchor/,
    '삽입으로 캐럿이 전진한 것은 "캐럿 이동"이 아니므로 예약을 이어가야 한다',
  );
});
