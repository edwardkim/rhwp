// [#4150 리뷰] onInput의 조합 replace 거부 재정박이 머리말/꼬리말·각주 모드에서 잘못된
// charOffset을 wasm에 넘기지 않는지, 실제 onInput 함수를 mock this로 직접 호출해 검증한다.
// input-handler-text.ts는 command.ts(파라미터 프로퍼티 사용)를 import하므로 기본 test
// 러너(strip-only)로는 로드할 수 없어 --experimental-transform-types 로 spawn한다
// (pending-char-shape.runner.mjs 와 동일 패턴).
import { registerHooks } from 'node:module';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import assert from 'node:assert/strict';

const studioDir = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const srcDir = join(studioDir, 'src');

registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier.startsWith('@/')) {
      const abs = join(srcDir, specifier.slice(2));
      const withTs = abs.endsWith('.ts') ? abs : abs + '.ts';
      return { url: pathToFileURL(withTs).href, shortCircuit: true };
    }
    if ((specifier.startsWith('./') || specifier.startsWith('../')) && !/\.[cm]?[tj]s$/.test(specifier)) {
      const parent = context.parentURL ? dirname(fileURLToPath(context.parentURL)) : srcDir;
      return { url: pathToFileURL(join(parent, specifier + '.ts')).href, shortCircuit: true };
    }
    return nextResolve(specifier, context);
  },
});

const { onInput } = await import(pathToFileURL(join(srcDir, 'engine', 'input-handler-text.ts')).href);

/** onInput의 조합 분기를 한 번 실행하고, 재정박 시도(실패 후 재시도) 시 넘긴 anchor를 돌려준다. */
function runOnceAndCaptureRetryAnchor(cursorOverrides) {
  const anchors = [];
  const setHfCalls = [];
  const setFnCalls = [];
  const moveToCalls = [];

  const cursor = {
    getRect: () => ({ pageIndex: 0 }),
    isInHeaderFooter: () => false,
    isInFootnote: () => false,
    getPosition: () => ({ sectionIndex: 0, paragraphIndex: 7, charOffset: 42 }), // 진입 전 stale 본문 위치
    hfCharOffset: 5,
    hfParaIdx: 2,
    setHfCursorPosition: (paraIdx, offset) => setHfCalls.push({ paraIdx, offset }),
    fnCharOffset: 9,
    fnInnerParaIdx: 3,
    setFnCursorPosition: (paraIdx, offset) => setFnCalls.push({ paraIdx, offset }),
    moveTo: (pos) => moveToCalls.push(pos),
    ...cursorOverrides,
  };

  let replaceCallCount = 0;
  const fakeThis = {
    active: true,
    textarea: { value: '가' },
    isComposing: true,
    compositionAnchor: { sectionIndex: 0, paragraphIndex: 0, charOffset: 999 },
    compositionLength: 0,
    cursor,
    canInsertTextInFormMode: () => true,
    resetRawTextMutationEffects: () => {},
    consumeRawTextMutationBeforeCursor: () => false,
    afterTextInputEdit: () => {},
    replaceTextAtRaw: (anchor) => {
      replaceCallCount++;
      if (replaceCallCount === 1) throw new Error('deferred replace 범위 가드 거부(테스트 시뮬레이션)');
      anchors.push({ ...anchor });
    },
  };

  const realWarn = console.warn;
  console.warn = () => {};
  try {
    onInput.call(fakeThis, undefined);
  } finally {
    console.warn = realWarn;
  }

  assert.equal(replaceCallCount, 2, 'replaceTextAtRaw 는 실패 후 정확히 한 번 재시도해야 한다');
  assert.equal(anchors.length, 1, '재시도 anchor 를 캡처하지 못했다');
  return { retryAnchor: anchors[0], setHfCalls, setFnCalls, moveToCalls };
}

// ── 1. 머리말/꼬리말 모드: 재정박 anchor 는 hfCharOffset 을 써야 한다 ──────────
{
  const { retryAnchor, setHfCalls } = runOnceAndCaptureRetryAnchor({
    isInHeaderFooter: () => true,
  });
  assert.equal(retryAnchor.charOffset, 5,
    `HF 모드 재정박은 hfCharOffset(5)을 써야 하는데 ${retryAnchor.charOffset} 을 썼다 ` +
    '(cursor.getPosition()의 stale 본문 charOffset 42 를 그대로 쓰면 이 값이 된다)');
  assert.equal(setHfCalls.length, 1, 'HF 커서 갱신이 한 번 호출돼야 한다');
  assert.equal(setHfCalls[0].offset, 5 + 1, 'HF 커서는 재정박 offset(5) + 삽입 길이(1) 여야 한다');
}

// ── 2. 각주 모드: 재정박 anchor 는 fnCharOffset 을 써야 한다 ──────────────────
{
  const { retryAnchor, setFnCalls } = runOnceAndCaptureRetryAnchor({
    isInFootnote: () => true,
  });
  assert.equal(retryAnchor.charOffset, 9,
    `FN 모드 재정박은 fnCharOffset(9)을 써야 하는데 ${retryAnchor.charOffset} 을 썼다`);
  assert.equal(setFnCalls.length, 1, 'FN 커서 갱신이 한 번 호출돼야 한다');
  assert.equal(setFnCalls[0].offset, 9 + 1, 'FN 커서는 재정박 offset(9) + 삽입 길이(1) 여야 한다');
}

// ── 3. 본문 모드: getPosition() 을 그대로 쓴다(override 없음) ────────────────
{
  const { retryAnchor, moveToCalls } = runOnceAndCaptureRetryAnchor({});
  assert.equal(retryAnchor.charOffset, 42, '본문 모드는 getPosition() 의 charOffset 을 그대로 써야 한다');
  assert.equal(retryAnchor.paragraphIndex, 7);
  assert.equal(moveToCalls.length, 1, '본문 커서 이동이 한 번 호출돼야 한다');
  assert.equal(moveToCalls[0].charOffset, 42 + 1);
}

console.log('COMPOSITION_HF_FN_REANCHOR_OK');
