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
    objects: [...objects],
    ranges: toRanges(ranges),
    cellRanges: toRanges(cellRanges),
    bold: [...body].map((_, i) => i >= bold[0] && i < bold[1]),
    colors: [...body].map(() => '#000000'),
    emptyRunStyle: { bold: false, textColor: '#000000' },
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
  const editBody = (off, del, ins, afterControl) => {
    const chars = [...doc.body];
    const added = [...ins];
    const after = afterControl ?? off > chars.length;
    const at = Math.min(off, chars.length);
    const removed = Math.min(del, chars.length - at);
    doc.objects = doc.objects.map((position) => {
      const before = position >= at + removed ? position - removed : Math.min(position, at);
      return before > at || (before === at && !after) ? before + added.length : before;
    });
    chars.splice(at, removed, ...added);
    doc.body = chars.join('');
    doc.bold.splice(at, removed, ...added.map(() => at > 0 && doc.bold[at - 1]));
    doc.colors.splice(at, removed, ...added.map(() => at > 0 ? doc.colors[at - 1] : '#000000'));
    moveRanges(doc.ranges, false, at, removed, added.length);
  };
  const editCell = (off, del, ins) => {
    const chars = [...doc.cell];
    chars.splice(off, del, ...ins);
    doc.cell = chars.join('');
    moveRanges(doc.cellRanges, true, off, del, ins.length);
  };
  const cellResult = (off) => ({ ok: true, charOffset: off, paginationDeferred: false, cellFlowChanged: false });
  const logicalPosition = (target) => {
    const positions = [...doc.objects].sort((a, b) => a - b);
    let scalar = 0;
    let logical = 0;
    let control = 0;
    while (logical < target) {
      if (positions[control] === scalar) {
        logical++;
        control++;
        if (logical === target) return { scalar, afterControl: true };
      } else if (scalar < [...doc.body].length) {
        scalar++;
        logical++;
      } else break;
    }
    return { scalar, afterControl: false };
  };
  return {
    doc,
    fragments,
    getParagraphLength: () => [...doc.body].length,
    getLogicalLength: () => [...doc.body].length + doc.objects.length,
    logicalToTextOffset: (_s, _p, off) => logicalPosition(off).scalar,
    insertTextLogical: (_s, _p, off, value) => {
      const { scalar, afterControl } = logicalPosition(off);
      editBody(scalar, 0, value, afterControl);
      return JSON.stringify({ ok: true, logicalOffset: off + [...value].length });
    },
    runInBatch: (apply) => apply(),
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
    textToLogicalOffset: (_s, _p, off) => off + doc.objects.filter((p) => p < off).length,
    applyCharFormat: (_s, _p, from, to, props) => {
      const { bold, textColor } = JSON.parse(props);
      if (bold !== undefined) for (let at = from; at < to; at++) doc.bold[at] = bold;
      if (textColor !== undefined) for (let at = from; at < to; at++) doc.colors[at] = textColor;
      // 실제 문단은 마지막 글을 지워도 끝의 빈 run은 직전 글자 모양을 참조한다.
      doc.emptyRunStyle = { ...doc.emptyRunStyle, ...JSON.parse(props) };
      return '{"ok":true}';
    },
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
    compositionBodyLogical: false,
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

// 실제 빈 그림 문단의 본문 anchor는 개체 한 칸을 포함하지만 문자열은 scalar만 센다.
for (const prefix of ['', '앞🦦']) {
  for (const updates of [['한'], ['ㅎ', '하', '한'], ['😀', '😀X'], ['한', ''], ['ㅎ', '하', '한', '']]) {
    scenario(`31. 개체 뒤 ${JSON.stringify(prefix)} ${JSON.stringify(updates)} 조합의 확정·취소·이력`, () => {
      const length = [...prefix].length;
      const wasm = makeWasm({ body: prefix, objects: [length] });
      const h = makeHandler(wasm, bodyPos(length + 1), { insertMode: true });
      const value = updates.at(-1);
      h.compose(...updates);
      assert.equal(wasm.doc.body, prefix + value, '앞 조합 글이 잔상으로 남으면 안 된다');
      assert.deepEqual(wasm.doc.objects, [length], '개체는 원문 뒤·새 글 앞에 남아야 한다');
      assert.equal(h.cursor.getPosition().charOffset, length + 1 + [...value].length);
      assert.equal(h.compositionBodyLogical, false, '확정·취소 뒤 논리 조합 상태가 남으면 안 된다');
      assert.equal(wasm.fragments.size, 0, '미리보기 복원 조각을 보관하지 않는다');
      assert.equal(h.history.canUndo(), !!value, '확정한 글만 이력에 남겨야 한다');
      if (!value) return;
      for (let cycle = 0; cycle < 2; cycle++) {
        h.undo();
        assert.equal(wasm.doc.body, prefix);
        assert.deepEqual(wasm.doc.objects, [length]);
        assert.equal(h.history.canUndo(), false);
        h.redo();
        assert.equal(wasm.doc.body, prefix + value);
        assert.deepEqual(wasm.doc.objects, [length]);
      }
    });
  }
}

scenario('32. 개체 뒤 연속 두 음절은 순서와 하나의 병합 이력을 유지한다', () => {
  const wasm = makeWasm({ objects: [0] });
  const h = makeHandler(wasm, bodyPos(1), { insertMode: true });
  h.compose('ㅎ', '하', '한');
  h.compose('ㄱ', '그', '글');
  assert.equal(wasm.doc.body, '한글');
  assert.deepEqual(wasm.doc.objects, [0]);
  assert.equal(h.cursor.getPosition().charOffset, 3);
  h.undo();
  assert.equal(wasm.doc.body, '');
  assert.equal(h.history.canUndo(), false);
  h.redo();
  assert.equal(wasm.doc.body, '한글');
  assert.deepEqual(wasm.doc.objects, [0]);
});

scenario('33. 개체 뒤 수정 모드의 조합도 원문과 개체를 덮어쓰지 않는다', () => {
  const wasm = makeWasm({ body: '앞🦦', objects: [2] });
  const h = makeHandler(wasm, bodyPos(3));
  h.compose('ㅎ', '하', '한');
  assert.equal(wasm.doc.body, '앞🦦한');
  h.undo();
  assert.equal(wasm.doc.body, '앞🦦');
  assert.deepEqual(wasm.doc.objects, [2]);
});

scenario('34. 개체 뒤 조합 조각의 첫 준비 실패는 원문 손실이나 조각 누수를 만들지 않는다', () => {
  const wasm = makeWasm({ body: '앞🦦', objects: [2] });
  const capture = wasm.captureDeleteRange;
  let attempted = 0;
  wasm.captureDeleteRange = (...args) => {
    if (++attempted === 1) throw new Error('조합 복원 조각 준비 거절');
    return capture(...args);
  };
  const h = makeHandler(wasm, bodyPos(3), { insertMode: true });
  h.compose('ㅎ', '하', '한');
  assert.equal(wasm.doc.body, '앞🦦한');
  assert.ok(attempted > 0, '복원 조각 준비 거절 경로를 실제로 실행해야 한다');
  assert.equal(wasm.fragments.size, 0);
  h.undo();
  assert.equal(wasm.doc.body, '앞🦦');
  assert.deepEqual(wasm.doc.objects, [2]);
});

scenario('35. 조합 삭제 뒤 새 글 삽입이 실패하면 미리보기와 개체를 복구하고 다음 갱신을 받는다', () => {
  const wasm = makeWasm({ body: '앞🦦', objects: [2] });
  const h = makeHandler(wasm, bodyPos(3), { insertMode: true });
  text.onCompositionStart.call(h);
  h.input('ㅎ');
  assert.equal(wasm.doc.body, '앞🦦ㅎ');
  const insert = wasm.insertTextLogical;
  let rejected = false;
  wasm.insertTextLogical = (...args) => {
    const result = insert(...args);
    if (!rejected) { rejected = true; throw new Error('조합 새 글 삽입 뒤 거절'); }
    return result;
  };
  h.input('하');
  assert.equal(wasm.doc.body, '앞🦦ㅎ', '실패한 갱신은 이전 미리보기를 복구해야 한다');
  assert.equal(rejected, true, '삽입 후 거절 경로를 실제로 실행해야 한다');
  assert.equal(h.history.canUndo(), false, '미리보기는 확정 전 이력에 넣지 않는다');
  assert.equal(wasm.fragments.size, 0);
  h.input('한');
  text.onCompositionEnd.call(h);
  assert.equal(wasm.doc.body, '앞🦦한');
  h.undo();
  assert.equal(wasm.doc.body, '앞🦦');
  assert.deepEqual(wasm.doc.objects, [2]);
});

scenario('36. 논리 범위를 넘긴 시작 주소는 개체와 원문을 바꾸지 않는다', () => {
  const wasm = makeWasm({ body: '앞🦦', objects: [2] });
  const before = structuredClone(wasm.doc);
  const h = makeHandler(wasm, bodyPos(100), { insertMode: true });
  try { h.compose('ㅎ', '하', '한'); } catch {}
  assert.deepEqual(wasm.doc, before);
  assert.equal(h.history.canUndo(), false);
  assert.equal(wasm.fragments.size, 0);
});

scenario('37. 조합 중 범위가 낡아도 scalar 원문을 추측해 지우지 않는다', () => {
  const wasm = makeWasm({ body: '앞🦦', objects: [2] });
  const h = makeHandler(wasm, bodyPos(3), { insertMode: true });
  text.onCompositionStart.call(h);
  h.compositionLength = 100;
  h._compositionCovered = '';
  try { h.input('한'); } catch {}
  assert.ok(wasm.doc.body.startsWith('앞🦦'), '음수나 낡은 조합 길이로 원문을 지우면 안 된다');
  assert.equal(h.history.canUndo(), false);
  assert.deepEqual(wasm.doc.objects, [2]);
  assert.equal(wasm.fragments.size, 0);
});

scenario('38. 실제 예약 서식은 개체 뒤 모든 갱신과 다음 음절에 이어진다', () => {
  const wasm = makeWasm({ objects: [0] });
  const originalStyle = structuredClone(wasm.doc.emptyRunStyle);
  const h = makeHandler(wasm, bodyPos(1), { insertMode: true });
  h.pendingCharShape = { bold: true, textColor: '#1122ff' };
  h.pendingCharShapeAnchor = bodyPos(1);
  for (const name of ['getPendingCharShape', 'advancePendingCharShapeAnchor', 'applyPendingCharShapeToRange']) {
    h[name] = InputHandler.prototype[name];
  }
  for (const updates of [['ㅎ', '하', '한'], ['ㄱ', '그', '글']]) {
    text.onCompositionStart.call(h);
    for (const value of updates) {
      h.input(value);
      assert.ok(wasm.doc.bold.every(Boolean), '실제 scalar 조합 글에 예약 서식을 적용해야 한다');
      assert.ok(wasm.doc.colors.every(color => color === '#1122ff'));
      assert.deepEqual(h.pendingCharShapeAnchor, h.cursor.getPosition(), '현재 커서에서 예약을 이어가야 한다');
    }
    text.onCompositionEnd.call(h);
  }
  assert.equal(wasm.doc.body, '한글');
  assert.deepEqual(wasm.doc.bold, [true, true]);
  assert.deepEqual(wasm.doc.colors, ['#1122ff', '#1122ff']);
  assert.deepEqual(wasm.doc.objects, [0]);
  for (let cycle = 0; cycle < 2; cycle++) {
    h.undo();
    assert.equal(wasm.doc.body, '');
    assert.deepEqual(wasm.doc.emptyRunStyle, originalStyle, '빈 run의 글자 모양 참조까지 되돌려야 한다');
    assert.equal(wasm.fragments.size, 0);
    h.redo();
    assert.equal(wasm.doc.body, '한글');
    assert.deepEqual(wasm.doc.bold, [true, true]);
    assert.deepEqual(wasm.doc.colors, ['#1122ff', '#1122ff']);
    assert.deepEqual(wasm.doc.objects, [0]);
    assert.deepEqual(wasm.doc.emptyRunStyle, { bold: true, textColor: '#1122ff' });
    assert.equal(wasm.fragments.size, 1, '다시 실행도 다음 Undo의 원문 조각을 보유해야 한다');
  }
});

scenario('39. 개체 뒤 양식 입력 거절은 조각과 이력을 만들지 않는다', () => {
  const wasm = makeWasm({ objects: [0] });
  const before = structuredClone(wasm.doc);
  const h = makeHandler(wasm, bodyPos(1), { insertMode: true, editMode: 'form' });
  h.compose('ㅎ', '하', '한');
  assert.deepEqual(wasm.doc, before);
  assert.equal(h.history.canUndo(), false);
  assert.equal(wasm.fragments.size, 0);
});

scenario('40. 실제 조합 삭제 권한이 사라지면 새 글만 넣지 않고 다음 허용된 갱신을 받는다', () => {
  const wasm = makeWasm({ body: '앞🦦', objects: [2] });
  const h = makeHandler(wasm, bodyPos(3), { insertMode: true });
  text.onCompositionStart.call(h);
  h.input('ㅎ');
  const before = structuredClone(wasm.doc);
  const allowed = h.canDeleteTextInFormMode;
  h.canDeleteTextInFormMode = () => false;
  h.input('하나');
  assert.deepEqual(wasm.doc, before, '삭제 거절 뒤 새 조합만 끼워 넣으면 안 된다');
  assert.equal(h.compositionLength, 1, '거절된 미리보기 길이를 적용하면 안 된다');
  assert.equal(h._lastCompositionText, 'ㅎ');
  assert.equal(h.history.canUndo(), false);
  assert.equal(wasm.fragments.size, 0);
  h.canDeleteTextInFormMode = allowed;
  h.input('한');
  text.onCompositionEnd.call(h);
  assert.equal(wasm.doc.body, '앞🦦한');
  h.undo();
  assert.equal(wasm.doc.body, '앞🦦');
});

scenario('41. 실제 조합 readback이 다르면 캐시 문자열만 믿고 이력을 만들지 않는다', () => {
  const wasm = makeWasm({ objects: [0] });
  const h = makeHandler(wasm, bodyPos(1), { insertMode: true });
  text.onCompositionStart.call(h);
  h.input('한');
  const read = wasm.getTextRange;
  wasm.getTextRange = () => '틀';
  text.onCompositionEnd.call(h);
  assert.equal(wasm.doc.body, '한');
  assert.equal(h.history.canUndo(), false, '실제 글과 다른 내용을 기록하면 안 된다');
  assert.equal(h._lastComposedText, '', '확정 실패를 유령 입력 캐시에 남기면 안 된다');
  assert.equal(h.compositionBodyLogical, false);
  assert.equal(wasm.fragments.size, 0);
  wasm.getTextRange = read;
});

for (const method of ['deactivate', 'dispose']) {
  scenario(`42. 개체 뒤 조합 중 ${method}는 논리 조합 상태와 임시 조각을 해제한다`, () => {
    const wasm = makeWasm({ objects: [0] });
    const h = makeHandler(wasm, bodyPos(1), { insertMode: true });
    text.onCompositionStart.call(h);
    h.input('ㅎ');
    assert.equal(h.compositionBodyLogical, true, '논리 조합 경로를 실제로 시작해야 한다');
    endSession(h, method);
    assert.equal(h.compositionBodyLogical, false);
    assert.equal(wasm.fragments.size, 0);
    text.onCompositionEnd.call(h);
    assert.equal(h.history.canUndo(), false, '끝난 세션의 늦은 확정은 기록하지 않는다');
    assert.equal(wasm.doc.body, 'ㅎ');
  });
}

scenario('43. 다음 음절의 첫 조각 준비 거절은 지난 확정 글을 조합 버퍼에 되살리지 않는다', () => {
  const wasm = makeWasm({ objects: [0] });
  const h = makeHandler(wasm, bodyPos(1), { insertMode: true });
  h.compose('한');
  const before = structuredClone(wasm.doc);
  const previous = h.history.peekUndoTop();
  text.onCompositionStart.call(h);
  const capture = wasm.captureDeleteRange;
  let failed = false;
  wasm.captureDeleteRange = (...args) => {
    if (!failed) { failed = true; throw new Error('새 음절 첫 조각 준비 거절'); }
    return capture(...args);
  };
  h.input('ㄱ');
  assert.equal(failed, true);
  assert.deepEqual(wasm.doc, before);
  assert.equal(h.textarea.value, '', '빈 새 음절에는 지난 음절을 복원하면 안 된다');
  assert.equal(h.compositionLength, 0);
  assert.equal(h.history.peekUndoTop(), previous);
  assert.equal(wasm.fragments.size, 0);
  h.input('그');
  h.input('글');
  text.onCompositionEnd.call(h);
  assert.equal(wasm.doc.body, '한글');
  assert.deepEqual(wasm.doc.objects, [0]);
});

scenario('44. 본문 중간의 혼합 좌표에는 논리 조합 경로를 일괄 적용하지 않는다', () => {
  const wasm = makeWasm({ body: '앞뒤', objects: [0, 0] });
  const h = makeHandler(wasm, bodyPos(3), { insertMode: true });
  let logicalCalls = 0;
  const insert = wasm.insertTextLogical;
  wasm.insertTextLogical = (...args) => { logicalCalls++; return insert(...args); };
  text.onCompositionStart.call(h);
  assert.equal(h.compositionBodyLogical, false);
  h.input('한');
  assert.equal(logicalCalls, 0);
  text.onCompositionEnd.call(h);
});

scenario('45. 논리 본문 조합과 기존 scalar 입력은 서로 다른 이력 주소를 합치지 않는다', () => {
  const wasm = makeWasm({ objects: [0] });
  const h = makeHandler(wasm, bodyPos(1), { insertMode: true });
  h.compose('한');
  assert.equal(h.history.canUndo(), true);
  h.input('X');
  assert.equal(wasm.doc.body, '한X');
  h.undo();
  assert.equal(wasm.doc.body, '한');
  h.undo();
  assert.equal(wasm.doc.body, '');
  assert.deepEqual(wasm.doc.objects, [0]);
});

for (const prefix of ['', '앞🦦']) {
  scenario(`46. ${JSON.stringify(prefix)} 뒤 예약 서식 조합 취소는 빈 run 참조와 첫 원문 조각을 복원한다`, () => {
    const length = [...prefix].length;
    const wasm = makeWasm({ body: prefix, objects: [length] });
    const before = structuredClone(wasm.doc);
    const h = makeHandler(wasm, bodyPos(length + 1), { insertMode: true });
    h.pendingCharShape = { bold: true, textColor: '#1122ff' };
    h.pendingCharShapeAnchor = bodyPos(length + 1);
    for (const name of ['getPendingCharShape', 'advancePendingCharShapeAnchor', 'applyPendingCharShapeToRange']) h[name] = InputHandler.prototype[name];
    text.onCompositionStart.call(h);
    let firstFragment;
    for (const value of ['ㅎ', '하', '한']) {
      h.input(value);
      firstFragment ??= h._compositionFragment;
      assert.notEqual(firstFragment, null, '첫 원문 조각을 서식 적용 전에 확보해야 한다');
      assert.equal(h._compositionFragment, firstFragment, '갱신마다 첫 원문을 교체하면 안 된다');
      assert.equal(wasm.fragments.size, 1, '중간 미리보기 복원 조각은 해제해야 한다');
    }
    h.input('');
    text.onCompositionEnd.call(h);
    assert.deepEqual(wasm.doc, before);
    assert.equal(h._compositionFragment, null);
    assert.equal(h.history.canUndo(), false);
    assert.equal(wasm.fragments.size, 0);
  });
}

scenario('47. 예약 서식 확정 조각은 이력 초기화 시 해제하고 현재 문서는 유지한다', () => {
  const wasm = makeWasm({ objects: [0] });
  const h = makeHandler(wasm, bodyPos(1), { insertMode: true });
  h.pendingCharShape = { bold: true, textColor: '#1122ff' };
  h.pendingCharShapeAnchor = bodyPos(1);
  for (const name of ['getPendingCharShape', 'advancePendingCharShapeAnchor', 'applyPendingCharShapeToRange']) h[name] = InputHandler.prototype[name];
  h.compose('ㅎ', '하', '한');
  assert.equal(h._compositionFragment, null, '확정 뒤 조각 소유권은 command로 넘어가야 한다');
  assert.equal(wasm.fragments.size, 1);
  const before = structuredClone(wasm.doc);
  h.history.clear(wasm);
  assert.deepEqual(wasm.doc, before);
  assert.equal(wasm.fragments.size, 0);
  assert.equal(h.history.canUndo(), false);
});

scenario('48. 예약 서식 Redo가 서식 적용 뒤 실패하면 원문의 빈 run 참조까지 복원한다', () => {
  const wasm = makeWasm({ body: '앞🦦', objects: [2] });
  const before = structuredClone(wasm.doc);
  const h = makeHandler(wasm, bodyPos(3), { insertMode: true });
  h.pendingCharShape = { bold: true, textColor: '#1122ff' };
  h.pendingCharShapeAnchor = bodyPos(3);
  for (const name of ['getPendingCharShape', 'advancePendingCharShapeAnchor', 'applyPendingCharShapeToRange']) h[name] = InputHandler.prototype[name];
  h.compose('한');
  h.undo();
  assert.deepEqual(wasm.doc, before);
  const apply = wasm.applyCharFormat;
  let failed = false;
  wasm.applyCharFormat = (...args) => {
    const result = apply(...args);
    failed = true;
    throw new Error('Redo 서식 적용 뒤 거절');
  };
  assert.throws(() => h.history.redo(wasm), /Redo 서식 적용 뒤 거절/);
  assert.equal(failed, true);
  assert.deepEqual(wasm.doc, before, '삽입 글과 빈 run의 글자 모양 참조를 함께 복구해야 한다');
  assert.equal(wasm.fragments.size, 0);
  assert.equal(h.history.canUndo(), false);
  assert.equal(h.history.canRedo(), false);
});

if (failures.length > 0) {
  console.error(failures.join('\n'));
  process.exit(1);
}
console.log('OVERWRITE_MODE_OK');
