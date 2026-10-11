import assert from 'node:assert/strict';
import { registerHooks } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const srcRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'src');
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier.startsWith('@/')) {
      return nextResolve(pathToFileURL(join(srcRoot, specifier.slice(2) + '.ts')).href, context);
    }
    if (/^\.{1,2}\//.test(specifier) && !/\.[a-z]+$/.test(specifier)) {
      return nextResolve(specifier + '.ts', context);
    }
    return nextResolve(specifier, context);
  },
});

const { onCompositionEnd } = await import(pathToFileURL(join(srcRoot, 'engine/input-handler-text.ts')).href);
const { onKeyDown, handleCtrlKey } = await import(pathToFileURL(join(srcRoot, 'engine/input-handler-keyboard.ts')).href);
const { CursorState } = await import(pathToFileURL(join(srcRoot, 'engine/cursor.ts')).href);
const { ImeNavigationGuard } = await import(pathToFileURL(join(srcRoot, 'engine/ime-navigation-guard.ts')).href);

const BODY_POSITION = { sectionIndex: 0, paragraphIndex: 7, charOffset: 42 };
const text = 'alpha beta';
const bodyText = 'x'.repeat(36) + ' alpha beta' + 'x'.repeat(33);
const rect = { pageIndex: 0, x: 100, y: 100, height: 12 };

function createHandler(mode = 'body') {
  const calls = { bodyNavigation: [], commands: [], barriers: [], selectionUpdates: 0, scrollTop: 100 };
  const wasm = {
    navigateNextEditable(sec, para, offset, delta) {
      calls.bodyNavigation.push({ sec, para, offset, delta });
      return { type: 'text', sec, para, charOffset: offset + delta, context: [] };
    },
    getHeaderFooterParaInfo: () => JSON.stringify({ text, charCount: text.length, paraCount: 1 }),
    getFootnoteInfo: () => ({ texts: [text], paraCount: 1 }),
    getParagraphCount: () => 8,
    getParagraphLength: () => 80,
    getTextRange: (_sec, _para, offset, count) => bodyText.slice(offset, offset + count),
    getLineInfo: () => ({ lineIndex: 0, lineCount: 1, charStart: 0, charEnd: 80 }),
    getCursorRectOnLine: () => rect,
    getCursorRect: () => rect,
    getPageInfo: () => ({ marginLeft: 20, marginTop: 20, marginHeader: 10 }),
    hitTest: () => ({ ...BODY_POSITION }),
  };
  const cursor = new CursorState(wasm);
  cursor.moveTo({ ...BODY_POSITION });
  // 조판과 화면 기하를 바꾸는 검사가 아니다. 실제 CursorState의 위치·선택 경로를 실행한다.
  cursor.updateRect = () => {};
  if (mode === 'header' || mode === 'footer') {
    cursor.enterHeaderFooterMode(mode === 'header', 0, 0, 0);
    cursor.setHfCursorPosition(0, 5);
  } else if (mode === 'footnote') {
    cursor.enterFootnoteMode(0, 3, 0, 0, 0);
    cursor.setFnCursorPosition(0, 5);
  }
  const handler = {
    active: true,
    cursor,
    wasm,
    textarea: { value: '가', focus() {} },
    caret: { hideComposition() {} },
    isComposing: true,
    compositionAnchor: { ...BODY_POSITION, charOffset: mode === 'body' ? 41 : 4 },
    compositionLength: 1,
    _compositionFragment: null,
    _lastCompositionText: '가',
    _pendingNavAfterIME: null,
    _imeNavigationGuard: new ImeNavigationGuard(),
    getTextAt: () => '가',
    executeOperation: (operation) => calls.commands.push(operation),
    flushDeferredPaginationIfNeeded: (reason) => calls.barriers.push(reason),
    resetRawTextMutationEffects() {},
    updateCaret() {},
    updateSelection() { calls.selectionUpdates++; },
    afterEdit() {},
    eventBus: { emit() {} },
    viewportManager: {
      getScrollY: () => calls.scrollTop,
      getViewportSize: () => ({ height: 100, width: 100 }),
      setScrollTop: (value) => { calls.scrollTop = value; },
    },
    virtualScroll: {
      pageCount: 3,
      gap: 0,
      isHorizontalMode: () => false,
      getTotalHeight: () => 300,
      getRowStartPages: () => [0, 1, 2],
      getPageOffset: (page) => page * 100,
    },
  };
  handler.onKeyDown = (event) => onKeyDown.call(handler, event);
  handler.handleCtrlKey = (event) => handleCtrlKey.call(handler, event);
  return { handler, cursor, calls };
}

function commitWithKey(state, code, modifiers = {}, { earlyKeyup = false, forwarded = true, handled = true } = {}) {
  state.handler.onKeyDown({
    key: 'Process', code, keyCode: 229, isComposing: true,
    shiftKey: false, ctrlKey: false, metaKey: false, altKey: false,
    preventDefault() {}, ...modifiers,
  });
  assert.equal(state.handler._pendingNavAfterIME?.code, code, '실제 keydown이 탐색을 보류한다');
  if (earlyKeyup) state.handler._imeNavigationGuard.release({ code });
  onCompositionEnd.call(state.handler);
  assert.equal(state.handler.isComposing, false);
  assert.equal(state.handler._pendingNavAfterIME, null, '보류한 탐색을 한 번 소비한다');
  assert.ok(state.calls.barriers.includes('before-navigation'));
  assert.equal(state.calls.commands[0]?.kind, 'record', '조합 텍스트를 먼저 히스토리에 기록한다');
  if (!forwarded) return;
  const snapshot = () => ({
    position: state.cursor.getPosition(),
    hfOffset: state.cursor.hfCharOffset,
    fnOffset: state.cursor.fnCharOffset,
    commandCount: state.calls.commands.length,
    selectionUpdates: state.calls.selectionUpdates,
    scrollTop: state.calls.scrollTop,
  });
  const replayed = snapshot();
  let prevented = false;
  // Linux fcitx의 같은 물리 키가 compositionend 뒤 일반 keydown으로 다시 전달된다.
  state.handler.onKeyDown({
    key: code, code, keyCode: 37, isComposing: false, repeat: false,
    shiftKey: false, ctrlKey: false, metaKey: false, altKey: false,
    preventDefault() { prevented = true; }, ...modifiers,
  });
  assert.equal(prevented, handled,
    handled ? '같은 물리 키의 중복 keydown을 소비한다' : '편집기가 처리하지 않은 키의 브라우저 기본 동작을 유지한다');
  assert.deepEqual(snapshot(), replayed, `${code}: 중복 keydown으로 탐색·명령을 반복하지 않는다`);
  state.handler._imeNavigationGuard.release({ code });
}

// 머리말/꼬리말·각주의 커서는 본문 위치와 독립적이다. 조합 종료 때도 그 모드만 움직인다.
for (const mode of ['header', 'footer', 'footnote']) {
  for (const [key, offset] of [['ArrowLeft', 4], ['ArrowRight', 6], ['Home', 0], ['End', text.length]]) {
    const state = createHandler(mode);
    commitWithKey(state, key);
    const actual = mode === 'footnote' ? state.cursor.fnCharOffset : state.cursor.hfCharOffset;
    assert.equal(actual, offset, `${mode}: 조합 종료 후 ${key}의 문맥 내 오프셋`);
    assert.deepEqual(state.cursor.getPosition(), BODY_POSITION, `${mode}: 본문 커서를 바꾸지 않는다`);
    assert.deepEqual(state.calls.bodyNavigation, [], `${mode}: 본문 탐색을 호출하지 않는다`);
  }
}

// 수정자도 보존한다. Shift는 HF anchor를, Ctrl은 HF 단어 경계를 사용해야 한다.
{
  const state = createHandler('header');
  commitWithKey(state, 'ArrowLeft', { shiftKey: true });
  const selection = state.cursor.getHeaderFooterSelectionOrdered();
  assert.equal(selection.start.charOffset, 4);
  assert.equal(selection.end.charOffset, 5);
  assert.equal(state.cursor.getSelection(), null, 'HF 선택을 본문 anchor에 기록하지 않는다');
}
{
  const state = createHandler('header');
  commitWithKey(state, 'ArrowLeft', { ctrlKey: true });
  assert.equal(state.cursor.hfCharOffset, 0);
  assert.deepEqual(state.cursor.getPosition(), BODY_POSITION);
}
{
  const state = createHandler();
  commitWithKey(state, 'ArrowLeft', { ctrlKey: true });
  assert.equal(state.cursor.getPosition().charOffset, 37, 'Ctrl+방향키는 이전 단어 경계로 한 번 이동한다');
}

// 본문 Shift+방향키는 선택 렌더링까지 일반 keydown과 같다.
{
  const state = createHandler();
  commitWithKey(state, 'ArrowLeft', { shiftKey: true });
  assert.equal(state.cursor.getPosition().charOffset, 41);
  assert.equal(state.calls.bodyNavigation.length, 1);
  assert.equal(state.calls.selectionUpdates, 1);
  assert.equal(state.cursor.getSelectionOrdered().start.charOffset, 41);
  assert.equal(state.cursor.getSelectionOrdered().end.charOffset, 42);
  state.handler.onKeyDown({
    key: 'ArrowLeft', code: 'ArrowLeft', keyCode: 37, isComposing: false,
    shiftKey: false, ctrlKey: false, metaKey: false, altKey: false,
    preventDefault() {},
  });
  assert.equal(state.cursor.getPosition().charOffset, 40, '다음 실제 방향키는 정상 이동한다');
}
for (const earlyKeyup of [false, true]) {
  const state = createHandler();
  commitWithKey(state, 'ArrowLeft', {}, { earlyKeyup, forwarded: false });
  state.handler.onKeyDown({
    key: 'ArrowLeft', code: 'ArrowLeft', keyCode: 37, isComposing: false,
    repeat: !earlyKeyup,
    shiftKey: false, ctrlKey: false, metaKey: false, altKey: false,
    preventDefault() {},
  });
  assert.equal(state.cursor.getPosition().charOffset, 40,
    earlyKeyup ? '조합 종료 전 keyup이 있으면 다음 물리 키를 소비하지 않는다' : '누른 키의 repeat는 계속 이동한다');
}

// IME keydown에서 이미 예약하는 PageUp/Down·Tab·Escape도 각 일반 소유자가 처리한다.
for (const [key, top] of [['PageUp', 0], ['PageDown', 200]]) {
  const state = createHandler('header');
  commitWithKey(state, key);
  assert.equal(state.calls.scrollTop, top, `${key}: 쪽 화면을 이동한다`);
  assert.equal(state.cursor.hfCharOffset, 5, `${key}: HF 커서는 유지한다`);
  assert.deepEqual(state.cursor.getPosition(), BODY_POSITION);
}
{
  const state = createHandler();
  commitWithKey(state, 'Tab');
  const commands = state.calls.commands.filter(operation => operation.kind === 'command');
  assert.equal(commands.length, 1);
  assert.equal(commands[0].command.constructor.name, 'InsertTabCommand');
}
for (const mode of ['header', 'footnote']) {
  const state = createHandler(mode);
  commitWithKey(state, 'Escape');
  assert.equal(state.cursor.isInHeaderFooter(), false);
  assert.equal(state.cursor.isInFootnote(), false);
}

// 일반 키보드 경로가 소비하지 않는 키는 재생 여부만으로 native 기본 동작을 막지 않는다.
for (const mode of ['header', 'footnote']) {
  const state = createHandler(mode);
  commitWithKey(state, 'Tab', {}, { handled: false });
  assert.deepEqual(state.cursor.getPosition(), BODY_POSITION);
  assert.equal(state.calls.commands.filter(operation => operation.kind === 'command').length, 0);
}
for (const mode of ['body', 'header', 'footnote']) {
  for (const key of ['PageUp', 'PageDown']) {
    const state = createHandler(mode);
    commitWithKey(state, key, { ctrlKey: true }, { handled: false });
    assert.equal(state.calls.scrollTop, 100);
    assert.deepEqual(state.cursor.getPosition(), BODY_POSITION);
    assert.equal(state.calls.commands.filter(operation => operation.kind === 'command').length, 0);
  }
}

// 조합을 확정하는 Enter는 추가 문단을 만들지 않고 pagination barrier만 유지한다.
{
  const state = createHandler();
  commitWithKey(state, 'Enter');
  assert.deepEqual(state.cursor.getPosition(), BODY_POSITION);
  assert.equal(state.calls.commands.filter(operation => operation.kind === 'command').length, 0);
  assert.equal(state.calls.barriers.length, 1);
  state.handler.onKeyDown({
    key: 'Enter', code: 'Enter', keyCode: 13, isComposing: false,
    shiftKey: false, ctrlKey: false, metaKey: false, altKey: false,
    preventDefault() {},
  });
  const commands = state.calls.commands.filter(operation => operation.kind === 'command');
  assert.equal(commands.length, 1, 'keyup 뒤 다음 Enter는 일반 줄바꿈을 실행한다');
  assert.equal(commands[0].command.constructor.name, 'SplitParagraphCommand');
}

console.log('COMPOSITION_PENDING_NAVIGATION_OK');
