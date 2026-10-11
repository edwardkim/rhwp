// [#7489] 수정(덮어쓰기) 모드 행위 러너. 실제 onInput/onCompositionStart/onCompositionEnd,
// InsertTextCommand, CommandHistory 를 mock this 와 문단 모델 wasm 에 대고 실행한다.
// command.ts 의 파라미터 프로퍼티 때문에 부모 테스트가 --experimental-transform-types 로 spawn 한다
// (composition-hf-fn-reanchor.runner.mjs 와 같은 패턴).
import { registerHooks } from 'node:module';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import assert from 'node:assert/strict';

const studioDir = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const srcDir = join(studioDir, 'src');

// InputHandler(deactivate·dispose)가 import 하는 wasm 진입점. 이 러너는 문단 모델 wasm 을 직접
// 넘기므로 실제 wasm 바이너리 없이 이름만 채운다.
const WASM_ENTRY_STUB = 'data:text/javascript,export default async function init() {}'
  + ' export class HwpDocument {} export function version() { return "0"; }';

registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === '@wasm/rhwp.js') return { url: WASM_ENTRY_STUB, shortCircuit: true };
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

const text = await import(pathToFileURL(join(srcDir, 'engine', 'input-handler-text.ts')).href);
const { CommandHistory } = await import(pathToFileURL(join(srcDir, 'engine', 'history.ts')).href);
const { InputHandler } = await import(pathToFileURL(join(srcDir, 'engine', 'input-handler.ts')).href);

/**
 * 본문 문단 하나(sec 0, para 0)와 셀 문단 하나를 담은 모델. 글자처럼 취급한 개체는 텍스트에
 * 없고 `objects`(텍스트 위치)로만 있다 — rhwp 의 text/char_offsets 분리와 같다.
 *
 * 본문에는 링크·누름틀 범위(`ranges`)와 글자별 굵게(`bold`)가 있다. 범위와 글자 모양은 실제
 * rhwp 처럼 움직인다(실제 WASM 으로 확인): 지우면 범위가 줄고, 범위 시작에 넣은 글자는 범위
 * 밖(앞)으로, 빈 누름틀에 넣은 글자는 누름틀 안으로 간다. 빈 링크는 그대로 남는다. 새 글자는
 * 앞 글자의 모양을 따른다. 그래서 지운 글자를 다시 넣기만 하면 범위·모양이 원래대로 안 돌아온다.
 * 문단 조각(captureDeleteRange)은 문단을 통째로 복사해 두었다가 되돌린다.
 *
 * 캐럿이 들어간 누름틀은 활성(setActiveField)이 된다. 실제 rhwp 처럼 활성 누름틀은 시작·끝에
 * 넣은 글자도 안으로 받는다. 활성 상태는 문서 데이터가 아니어서 조각 복원과 무관하다.
 * 셀 문단도 같은 규칙의 누름틀 범위(`cellRanges`)를 가진다.
 */
function makeWasm({ body = '', cell = '', objects = [], ranges = [], cellRanges = [], bold = [] } = {}) {
  const toRanges = (list) => list.map(([kind, start, end]) => ({ kind, start, end }));
  const doc = {
    body,
    cell,
    ranges: toRanges(ranges),
    cellRanges: toRanges(cellRanges),
    bold: [...body].map((_, i) => i >= bold[0] && i < bold[1]),
  };
  const fragments = new Map();
  let nextFragmentId = 1;
  let active = null; // { inCell, index }
  const rangesAt = (pos) => (pos.parentParaIndex === undefined ? doc.ranges : doc.cellRanges);
  const fieldIndexAt = (pos) =>
    rangesAt(pos).findIndex((r) => r.kind === 'field' && r.start <= pos.charOffset && pos.charOffset <= r.end);
  const moveRanges = (ranges, inCell, off, del, n) => {
    ranges.forEach((r, i) => {
      const shrink = (p) => (p >= off + del ? p - del : Math.min(p, off));
      r.start = shrink(r.start);
      r.end = shrink(r.end);
      if (n === 0) return;
      const inside = r.kind === 'field' && active?.inCell === inCell && active.index === i;
      if (r.start > off || (r.start === off && r.start < r.end && !inside)) {
        r.start += n;
        r.end += n;
      } else if (r.start === off ? r.kind === 'field' : off < r.end || (off === r.end && inside)) {
        r.end += n;
      }
    });
  };
  const editBody = (off, del, ins) => {
    const chars = [...doc.body];
    const added = [...ins];
    chars.splice(off, del, ...added);
    doc.body = chars.join('');
    doc.bold.splice(off, del, ...added.map(() => off > 0 && doc.bold[off - 1]));
    moveRanges(doc.ranges, false, off, del, added.length);
  };
  const editCell = (off, del, ins) => {
    const chars = [...doc.cell];
    chars.splice(off, del, ...ins);
    doc.cell = chars.join('');
    moveRanges(doc.cellRanges, true, off, del, ins.length);
  };
  const cellResult = (off) => ({ ok: true, charOffset: off, paginationDeferred: false, cellFlowChanged: false });
  return {
    doc,
    fragments,
    getTextRange: (_s, _p, off, n) => [...doc.body].slice(off, off + n).join(''),
    getTextInCell: (_s, _pp, _ci, _cei, _cpi, off, n) => [...doc.cell].slice(off, off + n).join(''),
    insertText: (_s, _p, off, t) => editBody(off, 0, [...t]),
    deleteText: (_s, _p, off, n) => editBody(off, n, []),
    replaceBodyTextLocal: (_s, _p, off, del, t) => {
      editBody(off, del, [...t]);
      return { ok: true, charOffset: off + [...t].length, documentPaginationPending: false, flowChanged: false };
    },
    insertTextInCell: (_s, _pp, _ci, _cei, _cpi, off, t) => editCell(off, 0, [...t]),
    deleteTextInCell: (_s, _pp, _ci, _cei, _cpi, off, n) => editCell(off, n, []),
    insertTextInCellDeferredPagination: (_s, _pp, _ci, _cei, _cpi, off, t) => {
      editCell(off, 0, [...t]);
      return cellResult(off + [...t].length);
    },
    replaceTextInCellDeferredPagination: (_s, _pp, _ci, _cei, _cpi, off, del, t) => {
      editCell(off, del, [...t]);
      return cellResult(off + [...t].length);
    },
    textToLogicalOffset: (_s, _p, off) => off + objects.filter((p) => p < off).length,
    getFieldInfoAt: (pos) => {
      const fieldId = fieldIndexAt(pos);
      if (fieldId < 0) return { inField: false };
      const { start, end } = rangesAt(pos)[fieldId];
      return { inField: true, fieldType: 'clickhere', fieldId, startCharIdx: start, endCharIdx: end, editableInForm: true };
    },
    // 실제 rhwp 처럼 누름틀이 없는 자리면 활성 상태를 그대로 둔다.
    setActiveField: (pos) => {
      const index = fieldIndexAt(pos);
      const inCell = pos.parentParaIndex !== undefined;
      if (index < 0 || (active?.inCell === inCell && active.index === index)) return false;
      active = { inCell, index };
      return true;
    },
    clearActiveField: () => { active = null; },
    captureDeleteRange: () => {
      fragments.set(nextFragmentId, structuredClone(doc));
      return nextFragmentId++;
    },
    restoreDeleteFragment: (id) => {
      if (!fragments.has(id)) throw new Error(`삭제 조각 ${id} 없음`);
      Object.assign(doc, structuredClone(fragments.get(id)));
      fragments.delete(id);
      return '{"ok":true}';
    },
    discardDeleteFragment: (id) => { fragments.delete(id); },
    snapshotCapacity: () => 100,
  };
}

const bodyPos = (charOffset) => ({ sectionIndex: 0, paragraphIndex: 0, charOffset });
const cellPos = (charOffset) => ({
  sectionIndex: 0, paragraphIndex: 0, charOffset,
  parentParaIndex: 0, controlIndex: 0, cellIndex: 0, cellParaIndex: 0,
});
const rangesOf = (wasm, key = 'ranges') => wasm.doc[key].map((r) => `${r.kind} ${r.start}-${r.end}`).join(', ');
const boldOf = (wasm) => [...wasm.doc.body].map((ch, i) => (wasm.doc.bold[i] ? ch.toUpperCase() : ch)).join('');

/** InputHandler 가 이 경로에서 실제로 쓰는 필드·메서드만 채운 mock this. */
function makeHandler(wasm, start, { insertMode = false, editMode = 'normal', exitedFieldEnd = false } = {}) {
  let position = start;
  const history = new CommandHistory();
  // InputHandler 의 양식 모드 판정과 같다: 편집 가능한 누름틀 안에서만 넣고 지운다.
  const fieldAt = (pos) => {
    const fi = wasm.getFieldInfoAt(pos);
    return editMode === 'form' && fi.inField && fi.editableInForm ? fi : null;
  };
  // InputHandler.fieldBoundaryKey 처럼 빠져나온 누름틀 끝을 문단·누름틀·끝 위치로 기억한다.
  const exitKeyOf = (pos, fi) => `${pos.parentParaIndex ?? -1}:${fi?.fieldId}:${fi?.endCharIdx}`;
  let fieldEndExitKey = exitedFieldEnd ? exitKeyOf(start, wasm.getFieldInfoAt(start)) : null;
  // InputHandler.updateFieldMarkers 와 같다: 빠져나온 끝이면 해제한다. 아니면 빠져나온 상태를 잊고
  // 캐럿이 누름틀 안이면 활성화, 밖이면 해제한다.
  const syncActiveField = () => {
    const fi = wasm.getFieldInfoAt(position);
    if (fi.inField && h.isAtExitedFieldEnd(position, fi)) {
      wasm.clearActiveField();
      return;
    }
    fieldEndExitKey = null;
    if (fi.inField) wasm.setActiveField(position);
    else wasm.clearActiveField();
  };
  const h = {
    active: true,
    insertMode,
    wasm,
    history,
    textarea: { value: '' },
    isComposing: false,
    compositionAnchor: null,
    compositionLength: 0,
    _compositionCovered: '',
    _compositionFragment: null,
    _lastCompositionText: '',
    _lastComposedText: '',
    caret: { hideComposition() {} },
    cursor: {
      isInHeaderFooter: () => false,
      isInFootnote: () => false,
      hasSelection: () => false,
      getPosition: () => ({ ...position }),
      getRect: () => ({ pageIndex: 0 }),
      moveTo: (pos) => { position = { ...pos }; },
    },
    canInsertTextInFormMode(pos) {
      if (editMode !== 'form') return true;
      const fi = fieldAt(pos);
      return !!fi && pos.charOffset >= fi.startCharIdx && pos.charOffset <= fi.endCharIdx;
    },
    canDeleteTextInFormMode(pos, count) {
      if (editMode !== 'form') return true;
      const fi = fieldAt(pos);
      return !!fi && pos.charOffset >= fi.startCharIdx && pos.charOffset + count <= fi.endCharIdx;
    },
    // 오른쪽 화살표로 누름틀 끝을 빠져나온 상태
    isAtExitedFieldEnd: (pos, fi) => fieldEndExitKey !== null && fieldEndExitKey === exitKeyOf(pos, fi),
    resetRawTextMutationEffects() {},
    consumeRawTextMutationBeforeCursor: () => false,
    prepareTextMutationBeforeCursor: () => false,
    flushDeferredPaginationIfNeeded() {},
    // 실제 화면 갱신은 모두 updateCaret → updateFieldMarkers 를 거친다.
    afterTextInputEdit: syncActiveField,
    afterEdit: syncActiveField,
    updateCaret: syncActiveField,
    executeOperation(desc) {
      // InputHandler.isOperationAllowedInEditMode 와 같다: 기록은 늘 통과, 입력은 넣을 수 있는 자리만.
      if (desc.kind === 'command') {
        if (!this.canInsertTextInFormMode(desc.command.position)) return;
        position = history.execute(desc.command, wasm);
        syncActiveField();
      } else {
        history.recordWithoutExecute(desc.command, wasm);
      }
    },
    undo() { position = history.undo(wasm); syncActiveField(); },
    redo() { position = history.redo(wasm); syncActiveField(); },
    type(s) {
      for (const ch of s) this.input(ch);
    },
    /** 입력 이벤트 하나로 `s` 전체를 넣는다(이모지 선택기·CDP insertText 처럼). */
    input(s) {
      this.textarea.value = s;
      text.onInput.call(this);
    },
    /** 한 음절 조합: 갱신 문자열들을 차례로 넣고 끝낸다. */
    compose(...updates) {
      text.onCompositionStart.call(this);
      for (const u of updates) {
        this.textarea.value = u;
        text.onInput.call(this);
      }
      text.onCompositionEnd.call(this);
    },
  };
  h.getTextAt = (pos, n) => text.getTextAt.call(h, pos, n);
  h.replaceTextAtRaw = (pos, del, t) => text.replaceTextAtRaw.call(h, pos, del, t);
  h.insertTextAtRaw = (pos, t) => text.insertTextAtRaw.call(h, pos, t);
  syncActiveField();
  return h;
}

// 시나리오마다 실패를 모아 한꺼번에 알린다 — 앞 시나리오가 깨져도 나머지 결과를 볼 수 있다.
const failures = [];
function scenario(name, fn) {
  try {
    fn();
  } catch (err) {
    failures.push(`${name}: ${err.message}`);
  }
}

scenario('1. 이슈 재현: abcd, Home, 수정 모드, XY → XYcd, 한 번에 되돌린다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  const h = makeHandler(wasm, bodyPos(0));
  h.type('XY');
  assert.equal(wasm.doc.body, 'XYcd', '수정 모드는 캐럿 뒤 글자를 입력한 만큼 덮어써야 한다');
  assert.equal(h.cursor.getPosition().charOffset, 2, '캐럿은 입력 글자 뒤에 있어야 한다');
  h.undo();
  assert.equal(wasm.doc.body, 'abcd', '연속 입력은 한 번의 되돌리기로 원문이 돼야 한다');
  assert.equal(h.history.canUndo(), false, '되돌릴 편집이 하나만 기록돼야 한다');
  assert.equal(wasm.fragments.size, 0, '되돌린 뒤 문단 조각이 남으면 안 된다');
  h.redo();
  assert.equal(wasm.doc.body, 'XYcd', '다시 실행은 같은 덮어쓰기를 재현해야 한다');
});

scenario('2. 삽입 모드는 그대로 끼워 넣는다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  makeHandler(wasm, bodyPos(0), { insertMode: true }).type('XY');
  assert.equal(wasm.doc.body, 'XYabcd');
  assert.equal(wasm.fragments.size, 0, '삽입 모드는 문단 조각을 잡지 않는다');
});

scenario('3. 문단 끝에서는 덮을 글자가 없어 삽입한다', () => {
  const wasm = makeWasm({ body: 'abc' });
  const h = makeHandler(wasm, bodyPos(2));
  h.type('XY');
  assert.equal(wasm.doc.body, 'abXY', 'c 를 덮은 뒤 문단 끝에서는 삽입해야 한다');
  h.undo();
  assert.equal(wasm.doc.body, 'abc');
});

scenario('4. 탭·개체·누름틀 끝은 덮어쓰지 않는다', () => {
  let wasm = makeWasm({ body: 'a\tb' });
  makeHandler(wasm, bodyPos(1)).type('X');
  assert.equal(wasm.doc.body, 'aX\tb', '탭은 지우지 않고 그 앞에 삽입해야 한다');
  // a [개체] b — 개체는 텍스트 위치 1, 캐럿은 개체 앞
  wasm = makeWasm({ body: 'ab', objects: [1] });
  makeHandler(wasm, bodyPos(1)).type('X');
  assert.equal(wasm.doc.body, 'aXb', '개체 너머의 b 를 덮어쓰면 안 된다');
  // 빈 누름틀(1..1) 안에서 입력 — 필드 밖 b 를 지우면 안 된다
  wasm = makeWasm({ body: 'ab', ranges: [['field', 1, 1]] });
  makeHandler(wasm, bodyPos(1)).type('X');
  assert.equal(wasm.doc.body, 'aXb', '누름틀 끝 너머 글자를 덮어쓰면 안 된다');
});

scenario('5. 표 셀도 같은 입력 경로로 덮어쓴다', () => {
  const wasm = makeWasm({ cell: 'abcd' });
  const h = makeHandler(wasm, cellPos(1));
  h.type('XY');
  assert.equal(wasm.doc.cell, 'aXYd');
  h.undo();
  assert.equal(wasm.doc.cell, 'abcd');
});

scenario('6. IME 조합: 음절마다 한 글자를 한 번만 덮고, 한 번의 되돌리기로 돌아간다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  const h = makeHandler(wasm, bodyPos(0));
  h.compose('ㅎ', '하', '한');
  assert.equal(wasm.doc.body, '한bcd', '조합 갱신이 거듭돼도 덮는 글자는 a 하나뿐이어야 한다');
  h.compose('ㄱ', '그', '글');
  assert.equal(wasm.doc.body, '한글cd');
  h.undo();
  assert.equal(wasm.doc.body, 'abcd', '조합 입력도 일반 입력처럼 한 번에 되돌아가야 한다');
  assert.equal(wasm.fragments.size, 0, '병합된 조합의 문단 조각도 모두 정리돼야 한다');
});

scenario('7. 조합 취소: 덮었던 글자를 되살리고 아무것도 기록하지 않는다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  const h = makeHandler(wasm, bodyPos(0));
  h.compose('ㅎ', '');
  assert.equal(wasm.doc.body, 'abcd', '조합을 지우면 덮었던 a 가 돌아와야 한다');
  assert.equal(h.cursor.getPosition().charOffset, 0);
  assert.equal(h.history.canUndo(), false, '취소된 조합은 기록하지 않는다');
  assert.equal(wasm.fragments.size, 0, '취소한 조합의 문단 조각이 남으면 안 된다');
});

// ── 범위·글자 모양: abcdef 의 cd(2..4)에 링크 ─────────────────────────────────
const linkDoc = () => makeWasm({ body: 'abcdef', ranges: [['link', 2, 4]] });

scenario('8. 링크 글자를 덮어쓴 뒤 되돌리면 링크 범위도 2-4 로 돌아온다', () => {
  const wasm = linkDoc();
  const h = makeHandler(wasm, bodyPos(2));
  h.type('X');
  assert.equal(wasm.doc.body, 'abXdef');
  const typed = rangesOf(wasm);
  h.undo();
  assert.equal(wasm.doc.body, 'abcdef');
  assert.equal(rangesOf(wasm), 'link 2-4', '되돌리면 c 가 링크 안으로 돌아와야 한다');
  h.redo();
  assert.equal(rangesOf(wasm), typed, '다시 실행은 처음 입력과 같은 범위를 만들어야 한다');
});

scenario('9. IME 로 링크 글자를 덮어 확정한 뒤 되돌리면 링크 범위가 돌아온다', () => {
  const wasm = linkDoc();
  const h = makeHandler(wasm, bodyPos(2));
  h.compose('ㅎ', '하');
  assert.equal(wasm.doc.body, 'ab하def');
  h.undo();
  assert.equal(wasm.doc.body, 'abcdef');
  assert.equal(rangesOf(wasm), 'link 2-4');
});

scenario('10. 링크 글자 위 IME 조합을 취소하면 링크 범위까지 그대로다', () => {
  const wasm = linkDoc();
  const h = makeHandler(wasm, bodyPos(2));
  h.compose('ㅎ', '');
  assert.equal(wasm.doc.body, 'abcdef');
  assert.equal(rangesOf(wasm), 'link 2-4', '취소하면 c 가 링크 안에 있어야 한다');
  assert.equal(h.history.canUndo(), false);
});

scenario('11. 링크 글자 전체를 빠르게 덮어쓴 뒤 한 번 되돌리면 링크가 돌아온다', () => {
  const wasm = linkDoc();
  const h = makeHandler(wasm, bodyPos(2));
  h.type('XY');
  assert.equal(wasm.doc.body, 'abXYef');
  const typed = rangesOf(wasm);
  h.undo();
  assert.equal(h.history.canUndo(), false, '연속 입력은 한 번에 되돌아가야 한다');
  assert.equal(rangesOf(wasm), 'link 2-4', '빈 링크로 남으면 안 된다');
  assert.equal(wasm.fragments.size, 0);
  h.redo();
  assert.equal(rangesOf(wasm), typed, '병합된 입력도 다시 실행하면 처음 입력과 같아야 한다');
});

scenario('12. 누름틀 앞에서 들어가며 빠르게 덮어쓴 뒤 한 번 되돌리면 누름틀이 2-4 다', () => {
  // abcde 의 cd(2..4)가 누름틀. 캐럿 1 에서 XYZ — b 를 덮고 누름틀 안의 c, d 로 이어진다.
  const wasm = makeWasm({ body: 'abcde', ranges: [['field', 2, 4]] });
  const h = makeHandler(wasm, bodyPos(1));
  h.type('XYZ');
  assert.equal(wasm.doc.body, 'aXYZe');
  h.undo();
  assert.equal(wasm.doc.body, 'abcde');
  assert.equal(h.history.canUndo(), false, '연속 입력은 한 번에 되돌아가야 한다');
  assert.equal(rangesOf(wasm), 'field 2-4', '누름틀이 b 까지 넓어지면 안 된다');
});

scenario('13. 굵은 글자를 덮어쓴 뒤 되돌리면 굵게가 돌아온다', () => {
  const wasm = makeWasm({ body: 'abcdef', bold: [2, 4] });
  const h = makeHandler(wasm, bodyPos(2));
  h.type('XY');
  h.undo();
  assert.equal(boldOf(wasm), 'abCDef', 'c, d 가 굵게 돌아와야 한다');
});

// ── 양식 모드: abcde 의 cd(2..4)가 편집 가능한 누름틀, 캐럿은 끝을 빠져나온 4 ──────────
// 누름틀 밖 e 는 지울 수 없다. 수정 모드도 삽입 모드와 똑같이 끝나야 한다.
function formRun(insertMode, act) {
  const wasm = makeWasm({ body: 'abcde', ranges: [['field', 2, 4]] });
  const h = makeHandler(wasm, bodyPos(4), { insertMode, editMode: 'form', exitedFieldEnd: true });
  act(h);
  return { body: wasm.doc.body, ranges: rangesOf(wasm), canUndo: h.history.canUndo() };
}

scenario('14. 양식 모드: 누름틀 끝을 나와 입력해도 보호된 e 를 지우지 않는다', () => {
  const overwrite = formRun(false, (h) => h.type('X'));
  assert.equal(overwrite.body, 'abcdXe', '보호된 e 가 남아야 한다');
  assert.deepEqual(overwrite, formRun(true, (h) => h.type('X')), '삽입 모드와 같아야 한다');
});

scenario('15. 양식 모드: 같은 자리의 IME 조합 취소는 e 를 겹치지 않는다', () => {
  const act = (h) => h.compose('ㅎ', '');
  const overwrite = formRun(false, act);
  assert.ok(!overwrite.body.includes('ee'), `e 가 겹쳤다: ${overwrite.body}`);
  assert.deepEqual(overwrite, formRun(true, act), '삽입 모드와 같아야 한다');
});

scenario('16. 양식 모드: 같은 자리의 IME 확정은 삽입 모드와 같이 기록된다', () => {
  const act = (h) => {
    h.compose('ㅎ', '하');
    h.undo();
  };
  const overwrite = formRun(false, (h) => h.compose('ㅎ', '하'));
  assert.equal(overwrite.body.at(-1), 'e', '보호된 e 가 남아야 한다');
  assert.deepEqual(overwrite, formRun(true, (h) => h.compose('ㅎ', '하')), '삽입 모드와 같아야 한다');
  assert.equal(formRun(false, act).body, 'abcde', '기록된 편집을 되돌리면 원문이어야 한다');
});

// ── 누름틀 바로 앞의 IME: abcde 의 cd(2..4)가 누름틀 ───────────────────────────────
// b 를 덮은 첫 조합 글자 뒤 캐럿이 누름틀 시작에 서면 누름틀이 활성화된다. 다음 조합 갱신이
// 그 글자를 지웠다 다시 넣어도 누름틀 안으로 끌려가면 안 된다.
const fieldDoc = () => makeWasm({ body: 'abcde', ranges: [['field', 2, 4]] });

scenario('17. 누름틀 바로 앞 글자를 덮은 IME 조합은 누름틀 밖에 남는다', () => {
  // Home, → 로 캐럿 1
  let wasm = fieldDoc();
  const h = makeHandler(wasm, bodyPos(1));
  h.compose('ㅎ', '하');
  assert.equal(wasm.doc.body, 'a하cde');
  assert.equal(rangesOf(wasm), 'field 2-4', '하가 누름틀 안으로 들어가면 안 된다');
  // 이어 치면 일반 입력처럼 누름틀 안 글자를 덮는다.
  h.compose('ㄷ', '다');
  assert.equal(wasm.doc.body, 'a하다de');
  assert.equal(rangesOf(wasm), 'field 2-4');
  h.undo();
  assert.equal(wasm.doc.body, 'abcde');
  assert.equal(rangesOf(wasm), 'field 2-4');
  // 처음부터 '한글' — 두 번째 음절이 b 를 덮는다.
  wasm = fieldDoc();
  const h2 = makeHandler(wasm, bodyPos(0));
  h2.compose('ㅎ', '하', '한');
  h2.compose('ㄱ', '그', '글');
  assert.equal(wasm.doc.body, '한글cde');
  assert.equal(rangesOf(wasm), 'field 2-4', '글이 누름틀 안으로 들어가면 안 된다');
});

scenario('18. 표 셀에서도 누름틀 바로 앞 IME 조합은 누름틀 밖에 남는다', () => {
  const wasm = makeWasm({ cell: 'abcde', cellRanges: [['field', 2, 4]] });
  makeHandler(wasm, cellPos(1)).compose('ㅎ', '하');
  assert.equal(wasm.doc.cell, 'a하cde');
  assert.equal(rangesOf(wasm, 'cellRanges'), 'field 2-4', '하가 누름틀 안으로 들어가면 안 된다');
});

// ── 빠져나온 누름틀 끝의 IME: abcde 의 ab(0..2)·de(3..5)가 누름틀 A·B, 캐럿은 → 로 A 끝을 나온 2 ──
// 누름틀 조회는 끝 위치도 A 안으로 친다. c 를 덮은 첫 조합 글자 뒤 캐럿이 B 시작에 서면 B 가
// 활성화되고, 다음 조합 갱신이 그 글자를 지웠다 다시 넣어도 B 안으로 끌려가면 안 된다.
function exitedEndRun(act, { body = 'abcde', ranges = [['field', 0, 2], ['field', 3, 5]], at = 2, cell = false, insertMode = false } = {}) {
  const wasm = cell ? makeWasm({ cell: body, cellRanges: ranges }) : makeWasm({ body, ranges });
  const h = makeHandler(wasm, (cell ? cellPos : bodyPos)(at), { insertMode, exitedFieldEnd: true });
  act(h);
  return { text: cell ? wasm.doc.cell : wasm.doc.body, ranges: rangesOf(wasm, cell ? 'cellRanges' : 'ranges') };
}
const twoFields = 'field 0-2, field 3-5';

scenario('19. 빠져나온 누름틀 끝의 IME 조합은 다음 누름틀로 끌려가지 않는다', () => {
  assert.deepEqual(exitedEndRun((h) => h.compose('ㅎ', '하')), { text: 'ab하de', ranges: twoFields },
    '하가 B 안으로 들어가면 안 된다');
  assert.equal(exitedEndRun((h) => h.type('X')).ranges, twoFields, '일반 입력도 범위를 그대로 둔다');
  assert.deepEqual(exitedEndRun((h) => { h.compose('ㅎ', '하'); h.undo(); }), { text: 'abcde', ranges: twoFields });
  assert.deepEqual(exitedEndRun((h) => h.compose('ㅎ', '하'), { insertMode: true }),
    { text: 'ab하cde', ranges: 'field 0-2, field 4-6' }, '삽입 모드는 그대로다');
});

scenario('20. 표 셀에서도 빠져나온 누름틀 끝의 IME 조합은 다음 누름틀 밖에 남는다', () => {
  assert.deepEqual(exitedEndRun((h) => h.compose('ㅎ', '하'), { cell: true }), { text: 'ab하de', ranges: twoFields });
});

scenario('21. 빠져나온 누름틀 끝에서 두 음절을 치면 일반 입력과 같은 범위가 된다', () => {
  assert.deepEqual(exitedEndRun((h) => { h.compose('ㅎ', '하'); h.compose('ㄷ', '다'); }),
    { text: 'ab하다e', ranges: twoFields });
  assert.equal(exitedEndRun((h) => h.type('XY')).ranges, twoFields);
});

scenario('22. 양식 같은 "홍 길동": 이름 누름틀 끝에서 공백을 덮은 글자는 다음 누름틀 밖에 남는다', () => {
  const form = { body: '홍 길동', ranges: [['field', 0, 1], ['field', 2, 4]], at: 1 };
  assert.deepEqual(exitedEndRun((h) => h.compose('ㅇ', '이'), form), { text: '홍이길동', ranges: 'field 0-1, field 2-4' },
    '이가 길동 누름틀 안으로 들어가면 안 된다');
  assert.deepEqual(exitedEndRun((h) => { h.compose('ㅇ', '이'); h.undo(); }, form),
    { text: '홍 길동', ranges: 'field 0-1, field 2-4' });
});

scenario('23. 빠져나온 누름틀 끝 바로 뒤가 다음 누름틀이면 덮지 않고 삽입 모드와 같다', () => {
  // abcd 의 ab(0..2)·cd(2..4)가 누름틀 A·B. 캐럿은 → 로 A 끝을 나온 2 — B 의 c 를 덮으면 안 된다.
  const adjacent = { body: 'abcd', ranges: [['field', 0, 2], ['field', 2, 4]] };
  const cases = [
    ['X', (h) => h.type('X')],
    ['IME 하', (h) => h.compose('ㅎ', '하')],
    ['표 셀 X', (h) => h.type('X'), { cell: true, body: adjacent.body, ranges: adjacent.ranges }],
  ];
  for (const [name, act, opts = adjacent] of cases) {
    const overwrite = exitedEndRun(act, opts);
    assert.deepEqual(overwrite, exitedEndRun(act, { ...opts, insertMode: true }), `${name}: 삽입 모드와 같아야 한다`);
    assert.match(overwrite.text, /cd$/, `${name}: B 의 c 가 남아야 한다`);
  }
  assert.deepEqual(exitedEndRun((h) => h.type('X'), adjacent), { text: 'abXcd', ranges: 'field 0-2, field 3-5' });
});

scenario('24. 입력 하나는 문자소 단위로 덮고, 밖에서는 누름틀 글자 앞에서 멈춘다', () => {
  // abcde 의 cd(2..4)가 누름틀, 캐럿 1. 이모지 선택기처럼 입력 하나에 여러 글자가 온다.
  const run = (body, at, s, ranges = [['field', 2, 4]]) => {
    const wasm = makeWasm({ body, ranges });
    const h = makeHandler(wasm, bodyPos(at));
    h.input(s);
    const typed = { text: wasm.doc.body, ranges: rangesOf(wasm) };
    h.undo();
    assert.deepEqual({ text: wasm.doc.body, ranges: rangesOf(wasm) }, { text: body, ranges: rangesOf(makeWasm({ body, ranges })) },
      `${s}: 되돌리면 원래대로여야 한다`);
    return typed;
  };
  assert.deepEqual(run('abcde', 1, 'XYZ'), { text: 'aXYZcde', ranges: 'field 4-6' }, 'b 만 덮고 누름틀 cd 는 남아야 한다');
  assert.deepEqual(run('abcde', 1, '❤️'), { text: 'a❤️cde', ranges: 'field 3-5' }, '❤️ 는 b 하나만 덮어야 한다');
  assert.equal(run('abcd', 0, '👍🏽', []).text, '👍🏽bcd', '👍🏽 는 a 하나만 덮어야 한다');
  assert.equal(run('abcd', 0, 'é', []).text, 'ébcd', '결합 문자열도 한 글자만 덮어야 한다');
  assert.equal(run('👍🏽b', 0, 'X', []).text, 'Xb', '덮는 글자도 문자소 하나 전체여야 한다');
});

scenario('25. 빈 누름틀 바로 앞 글자를 덮지 않아 친 글자가 누름틀 값이 되지 않는다', () => {
  // ab·빈 누름틀 E(2..2)·cd. b 를 지우면 E 가 삽입 자리로 당겨져 입력을 값으로 받는다.
  const run = (act, { at = 1, cell = false, insertMode = false } = {}) => {
    const ranges = [['field', 2, 2]];
    const wasm = cell ? makeWasm({ cell: 'abcd', cellRanges: ranges }) : makeWasm({ body: 'abcd', ranges });
    const key = cell ? 'cellRanges' : 'ranges';
    const h = makeHandler(wasm, (cell ? cellPos : bodyPos)(at), { insertMode });
    act(h);
    const typed = `${cell ? wasm.doc.cell : wasm.doc.body} ${rangesOf(wasm, key)}`;
    while (h.history.canUndo()) h.undo();
    assert.equal(`${cell ? wasm.doc.cell : wasm.doc.body} ${rangesOf(wasm, key)}`, 'abcd field 2-2', '되돌리면 원래대로여야 한다');
    return typed;
  };
  const cases = {
    'X': [(h) => h.type('X')],
    '표 셀 X': [(h) => h.type('X'), { cell: true }],
    'IME 하': [(h) => h.compose('ㅎ', '하')],
    '입력 하나 XYZ': [(h) => h.input('XYZ')],
    '캐럿 0 입력 하나 XYZ': [(h) => h.input('XYZ'), { at: 0 }],
    '캐럿 0 키 XY': [(h) => h.type('XY'), { at: 0 }],
  };
  const actual = Object.fromEntries(Object.entries(cases).map(([name, [act, opts]]) => [name, run(act, opts)]));
  assert.deepEqual(actual, {
    'X': 'aXbcd field 3-3',
    '표 셀 X': 'aXbcd field 3-3',
    'IME 하': 'a하bcd field 3-3',
    '입력 하나 XYZ': 'aXYZbcd field 5-5',
    '캐럿 0 입력 하나 XYZ': 'XYZbcd field 4-4',
    '캐럿 0 키 XY': 'XYbcd field 3-3',
  });
  // 캐럿 1 에서는 덮을 글자가 없으니 삽입 모드와 같아야 한다.
  for (const name of ['X', '표 셀 X', 'IME 하', '입력 하나 XYZ']) {
    const [act, opts] = cases[name];
    assert.equal(actual[name], run(act, { ...opts, insertMode: true }), `${name}: 삽입 모드와 같아야 한다`);
  }
});

// ── 코어 오프셋은 Unicode scalar 다: 😀 는 UTF-16 두 칸이지만 한 글자다 ────────────────────
scenario('26. 수정 모드 abcd 에서 😀, X → 😀Xcd 이고 캐럿은 scalar 오프셋에 선다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  const h = makeHandler(wasm, bodyPos(0));
  h.type('😀X');
  assert.equal(wasm.doc.body, '😀Xcd', 'X 는 😀 바로 뒤의 b 를 덮어야 한다');
  assert.equal(h.cursor.getPosition().charOffset, 2, '캐럿은 X 뒤(scalar 2)여야 한다');
  h.undo();
  assert.equal(wasm.doc.body, 'abcd');
  assert.equal(wasm.fragments.size, 0);
  h.redo();
  assert.equal(wasm.doc.body, '😀Xcd', '다시 실행은 입력을 scalar 오프셋으로 되풀이해야 한다');
});

scenario('27. 😀 뒤 scalar 1 에서 친 X 는 앞 입력과 한 묶음으로 되돌리고 다시 실행한다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  const h = makeHandler(wasm, bodyPos(0));
  h.type('😀');
  h.cursor.moveTo(bodyPos(1));
  h.type('X');
  assert.equal(wasm.doc.body, '😀Xcd');
  h.undo();
  assert.equal(wasm.doc.body, 'abcd', '연속 명령은 한 번에 되돌아가야 한다');
  assert.equal(h.history.canUndo(), false, '되돌릴 편집이 하나만 기록돼야 한다');
  h.redo();
  assert.equal(wasm.doc.body, '😀Xcd');
  assert.equal(h.cursor.getPosition().charOffset, 2);
});

scenario('28. 삽입 모드도 😀 뒤 캐럿·병합을 scalar 오프셋으로 한다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  const h = makeHandler(wasm, bodyPos(0), { insertMode: true });
  h.type('😀X');
  assert.equal(wasm.doc.body, '😀Xabcd');
  h.undo();
  assert.equal(wasm.doc.body, 'abcd', '연속 입력은 한 번에 되돌아가야 한다');
  h.redo();
  assert.equal(wasm.doc.body, '😀Xabcd');
});

scenario('29. IME 가 😀 를 확정한 뒤(일본어 변환 등) 다음 조합은 b 를 덮는다', () => {
  const wasm = makeWasm({ body: 'abcd' });
  const h = makeHandler(wasm, bodyPos(0));
  h.compose('😀');
  h.compose('ㅎ', '하');
  assert.equal(wasm.doc.body, '😀하cd');
  assert.equal(h.cursor.getPosition().charOffset, 2);
  h.undo();
  assert.equal(wasm.doc.body, 'abcd');
});

// ── 기록되기 전 조합의 문단 조각 수명 ────────────────────────────────────────────
// 문서를 열거나 새로 만들면 문서를 바꾼 뒤 deactivate 가 불린다. 새 문서는 같은 코어를 다시 쓰므로
// (createBlankDocument) 조합이 잡아 둔 조각을 해제하지 않으면 그대로 남는다.
const inert = new Proxy(function () {}, { get: () => inert, apply: () => undefined });
/** 실제 InputHandler 의 deactivate·dispose 를 실행한다. 화면·DOM 객체는 없으면 아무 일도 안 하는 대역이다. */
function endSession(h, method) {
  const orInert = (obj) => new Proxy(obj, { get: (t, k) => (k in t ? t[k] : inert) });
  Object.assign(h, {
    textarea: orInert(h.textarea), caret: orInert(h.caret), cursor: orInert(h.cursor),
    isResizeDragging: false, dragRafId: 0, resizeHoverRafId: 0,
  });
  const hadDocument = 'document' in globalThis;
  const hadWindow = 'window' in globalThis;
  if (!hadDocument) globalThis.document = inert; // dispose 가 문서 전역 리스너를 뗀다
  if (!hadWindow) globalThis.window = inert; // 창 focus 이탈 리스너도 같은 DOM 수명에 속한다
  try {
    InputHandler.prototype[method].call(orInert(h));
  } finally {
    if (!hadDocument) delete globalThis.document;
    if (!hadWindow) delete globalThis.window;
  }
}

for (const method of ['deactivate', 'dispose']) {
  scenario(`30. 덮은 IME 조합 중 ${method} 하면 기록되지 않은 문단 조각을 해제한다`, () => {
    const wasm = makeWasm({ body: 'abcd' });
    const h = makeHandler(wasm, bodyPos(0));
    text.onCompositionStart.call(h);
    h.textarea.value = 'ㅎ';
    text.onInput.call(h);
    assert.equal(wasm.fragments.size, 1, '덮을 때 문단 조각을 잡는다');
    endSession(h, method);
    assert.equal(wasm.fragments.size, 0, `${method}: 기록되지 않은 조각이 남으면 안 된다`);
    // 브라우저가 늦게 보낸 compositionend 는 아무것도 기록하거나 되살리지 않는다.
    text.onCompositionEnd.call(h);
    assert.equal(h.history.canUndo(), false, `${method}: 끝난 조합을 기록하면 안 된다`);
    assert.equal(wasm.doc.body, 'ㅎbcd', `${method}: 조합이 남긴 문서를 다시 건드리면 안 된다`);
  });
}

if (failures.length > 0) {
  console.error(failures.join('\n'));
  process.exit(1);
}
console.log('OVERWRITE_MODE_OK');
