// [#3080] 글자 서식 행위 러너 — command.ts 의 실제 커맨드 클래스를 로드해 **실제 wasm 문서**에
// 대고 검증한다. cursor.ts/command.ts 의 TS 파라미터 프로퍼티 때문에 기본 test 러너(strip-only)는
// 엔진 클래스를 import 하지 못하므로, 부모 테스트가 --experimental-transform-types 로 spawn 한다
// (tests/undo-drag-command-behaviour.test.ts 와 동일한 방식).
//
// 검증 대상:
//  1) 선택 범위 굵게/색이 문서에 닿는가 + undo 로 되돌아가는가  (신고 ①②)
//  2) 캐럿 대기 서식(pending char shape)이 **다음 삽입 런에만** 적용되는가 (신고 ③④, "굵게 켜고 입력")
//  3) 예약 서식이 다른 삽입은 병합되지 않는가 (앞 글자에 뒤 서식이 덮이는 것 방지)
//  5,6) [PR #4271 리뷰] InputHandler.applyCharFormat/stagePendingCharShape/getPendingCharShape
//       수명주기 회귀 — @wasm/rhwp.js 를 pkg-node 빌드로 재배선해 InputHandler 를 생성자 없이
//       (Object.create) 얹고 실제 프로토타입 메서드로 검증한다(텍스트 매칭이 아니라 실제 실행).
import { registerHooks } from 'node:module';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const studioDir = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const srcDir = join(studioDir, 'src');
const repoDir = join(studioDir, '..');

// @/ 별칭 + 확장자 없는 상대 import 를 .ts 로 해석(tsconfig paths 재현). @wasm/rhwp.js 는
// 브라우저 target(pkg/)이라 Node 에서 그대로 못 띄우므로, 이미 로드해 쓰는 Node target
// (pkg-node/)으로 리다이렉트한다 — InputHandler 가 참조 그래프에서 요구하는 유일한 wasm 진입점.
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === '@wasm/rhwp.js') {
      return { url: pathToFileURL(join(repoDir, 'pkg-node', 'rhwp.js')).href, shortCircuit: true };
    }
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

const { ApplyCharFormatCommand, InsertTextCommand, applyCharShapeModsToRange } =
  await import(pathToFileURL(join(srcDir, 'engine', 'command.ts')).href);
const { CursorState } = await import(pathToFileURL(join(srcDir, 'engine', 'cursor.ts')).href);
const { InputHandler } = await import(pathToFileURL(join(srcDir, 'engine', 'input-handler.ts')).href);

const { HwpDocument } = await import(pathToFileURL(join(repoDir, 'pkg-node', 'rhwp.js')).href);

const doc = new HwpDocument(new Uint8Array(readFileSync(join(repoDir, 'samples', '2010-01-06.hwp'))));

/** 커맨드가 실제로 쓰는 WasmBridge 표면만 구현한 얇은 어댑터. */
const wasm = {
  insertText: (sec, para, off, text) => doc.insertText(sec, para, off, text),
  deleteText: (sec, para, off, count) => doc.deleteText(sec, para, off, count),
  replaceBodyTextLocal: (sec, para, off, deleteCount, text) => {
    if (deleteCount > 0) doc.deleteText(sec, para, off, deleteCount);
    if (text) doc.insertText(sec, para, off, text);
    return { documentPaginationPending: false, flowChanged: false, paginationCompleted: true };
  },
  applyCharFormat: (sec, para, from, to, json) => doc.applyCharFormat(sec, para, from, to, json),
  setCharShapeId: (sec, para, from, to, id) => doc.setCharShapeId(sec, para, from, to, id),
  getCharPropertiesAt: (sec, para, off) => JSON.parse(doc.getCharPropertiesAt(sec, para, off)),
  getParagraphLength: (sec, para) => doc.getParagraphLength(sec, para),
  getTextRange: (sec, para, off, count) => doc.getTextRange(sec, para, off, count),
  // CursorState.updateRect() 가 조회하는 것 — geometry 는 이 회귀 시나리오와 무관하므로 null.
  getCursorRect: () => null,
  getCursorRectInHeaderFooter: () => null,
  getCursorRectInFootnote: () => null,
  getCursorRectInNote: () => null,
};

const pos = (para, charOffset) => ({ sectionIndex: 0, paragraphIndex: para, charOffset });
const props = (para, off) => JSON.parse(doc.getCharPropertiesAt(0, para, off));

/** 생성자(=DOM) 없이 InputHandler 프로토타입 메서드만 얹는다 — 이 시나리오가 실제로 건드리는
 * 필드(cursor/pendingCharShape*)만 채운다. applyCharFormat/stagePendingCharShape/
 * getPendingCharShape 는 전부 실제 input-handler.ts 소스의 메서드다(재구현 아님). */
function newHandler() {
  const handler = Object.create(InputHandler.prototype);
  handler.cursor = new CursorState(wasm);
  handler.wasm = wasm;
  handler.pendingCharShape = null;
  handler.pendingCharShapeAnchor = null;
  return handler;
}

// 10글자 이상인 첫 본문 문단을 대상으로 한다.
let para = -1;
for (let p = 0; p < 60; p++) {
  if (doc.getParagraphLength(0, p) >= 10) { para = p; break; }
}
assert.ok(para >= 0, '10글자 이상인 본문 문단이 있어야 한다');

// ── 1. 선택 범위 굵게/색 → 문서 반영 + undo ────────────────────────────────
{
  assert.equal(props(para, 0).bold, false, '전제: 문단 앞부분은 굵지 않다');

  const cmd = new ApplyCharFormatCommand(pos(para, 0), pos(para, 5), { bold: true, textColor: '#ff0000' });
  cmd.execute(wasm);

  for (const off of [0, 2, 4]) {
    assert.equal(props(para, off).bold, true, `offset ${off} 은 굵어야 한다`);
    assert.equal(props(para, off).textColor, '#ff0000', `offset ${off} 은 빨강이어야 한다`);
  }
  assert.equal(props(para, 6).bold, false, '선택 밖 글자는 그대로여야 한다');

  cmd.undo(wasm);
  assert.equal(props(para, 0).bold, false, 'undo 는 원래 글자 모양으로 되돌린다');
  assert.equal(props(para, 0).textColor, '#000000', 'undo 는 색도 되돌린다');
}

// ── 2. 캐럿 대기 서식은 다음 삽입 런에만 적용된다 ───────────────────────────
{
  const at = 3;
  const baseline = [at, at + 1, at + 2].map((off) => ({
    bold: props(para, off).bold,
    textColor: props(para, off).textColor,
  }));
  const cmd = new InsertTextCommand(pos(para, at), 'ABC', undefined, { bold: true, textColor: '#0000ff' });
  const after = cmd.execute(wasm);

  assert.equal(doc.getTextRange(0, para, at, 3), 'ABC', '텍스트가 삽입돼야 한다');
  assert.equal(after.charOffset, at + 3, '캐럿은 삽입 길이만큼 전진한다');
  for (let i = 0; i < 3; i++) {
    assert.equal(props(para, at + i).bold, true, `삽입된 ${i}번째 글자에 예약 서식(굵게)이 적용돼야 한다`);
    assert.equal(props(para, at + i).textColor, '#0000ff', `삽입된 ${i}번째 글자에 예약 색이 적용돼야 한다`);
  }
  assert.equal(props(para, at + 3).bold, false, '삽입 범위 뒤 글자는 영향을 받지 않는다');
  assert.equal(props(para, 0).bold, false, '삽입 범위 앞 글자는 영향을 받지 않는다');

  cmd.undo(wasm);
  assert.notEqual(doc.getTextRange(0, para, at, 3), 'ABC', 'undo 는 삽입 텍스트를 지운다');
  assert.deepEqual(
    [at, at + 1, at + 2].map((off) => ({
      bold: props(para, off).bold,
      textColor: props(para, off).textColor,
    })),
    baseline,
    'undo 는 삽입 런의 예약 서식을 원문에 남기면 안 된다',
  );
}

// ── 3. 예약 서식이 다른 삽입끼리 병합 금지 ─────────────────────────────────
{
  const plain = new InsertTextCommand(pos(0, 0), 'a', 1000);
  const bold = new InsertTextCommand(pos(0, 1), 'b', 1010, { bold: true });
  const bold2 = new InsertTextCommand(pos(0, 2), 'c', 1020, { bold: true });

  assert.equal(plain.mergeWith(bold), null, '서식 없는 입력과 굵은 입력은 병합 불가');
  const merged = bold.mergeWith(bold2);
  assert.ok(merged, '같은 예약 서식끼리는 기존대로 병합된다');
  assert.equal(merged.getCharFormat().bold, true, '병합 결과도 예약 서식을 유지한다');
}

// ── 4. 빈 범위(캐럿)는 문서에 쓰지 않는다 ──────────────────────────────────
{
  const calls = [];
  const spy = {
    applyCharFormat: (...args) => { calls.push(args); return '{}'; },
    applyCharFormatInCellByPath: (...args) => { calls.push(args); return '{}'; },
  };
  applyCharShapeModsToRange(spy, pos(0, 4), 4, 4, { bold: true });
  assert.equal(calls.length, 0, '빈 범위(캐럿)는 적용 대상이 없으므로 호출하지 않는다');
  applyCharShapeModsToRange(spy, pos(0, 4), 4, 7, { bold: true });
  assert.equal(calls.length, 1, '실제 범위는 본문 applyCharFormat 으로 간다');
  assert.match(JSON.stringify(calls[0]), /\\"bold\\":true/);
}

// ── 5. [PR #4271 리뷰] 캐럿이 실제로 이동하면 낡은 예약이 새 앵커로 안 새야 한다 ──────
// 신고: A 에서 굵게 예약 → B 로 이동 → B 에서 색 지정 → 입력하면 A 의 굵게까지 낀다.
{
  const at = 6;
  const handler = newHandler();

  handler.cursor.moveTo(pos(para, 0));
  handler.applyCharFormat({ bold: true }); // A(offset 0)에서 굵게 예약

  handler.cursor.moveTo(pos(para, at)); // 진짜 캐럿 이동: A → B
  handler.applyCharFormat({ textColor: '#ff0000' }); // B(offset 6)에서 색 지정

  const pending = handler.getPendingCharShape();
  assert.equal(pending?.bold, undefined, 'B 로 이동한 뒤에는 A 의 굵게 예약이 남아 있으면 안 된다');
  assert.equal(pending?.textColor, '#ff0000', 'B 에서 지정한 색은 그대로 예약돼야 한다');

  const cmd = new InsertTextCommand(pos(para, at), 'X', undefined, pending);
  cmd.execute(wasm);
  assert.equal(props(para, at).bold, false, '실제 삽입 글자에 A 의 굵게가 새면 안 된다');
  assert.equal(props(para, at).textColor, '#ff0000', '실제 삽입 글자는 B 에서 지정한 색이어야 한다');
  cmd.undo(wasm);
}

// ── 6. [PR #4271 리뷰] 머리말/꼬리말 모드에서 고른 서식이 본문으로 새면 안 된다 ─────────
// 신고: cursor.getPosition() 이 머리말/꼬리말 모드 중 진입 전 본문 위치에 고정돼 있고,
// 전용 삽입 분기는 예약을 소비하지 않아 모드를 나온 뒤 본문 입력에 그 서식이 묻는다.
{
  const at = 4;
  const baseline = props(para, at);
  const handler = newHandler();

  handler.cursor.moveTo(pos(para, at));
  handler.cursor.enterHeaderFooterMode(true, 0, 0);
  assert.deepEqual(
    handler.cursor.getPosition(), pos(para, at),
    '전제: 머리말 모드 중에도 getPosition() 은 진입 전 본문 위치를 유지한다',
  );
  handler.applyCharFormat({ textColor: '#00aa00' }); // 머리말 "안"에서 색 지정
  handler.cursor.exitHeaderFooterMode();

  const pending = handler.getPendingCharShape();
  assert.equal(pending, undefined, '머리말 모드에서 고른 서식이 본문 예약으로 남으면 안 된다');

  const cmd = new InsertTextCommand(pos(para, at), 'Y', undefined, pending);
  cmd.execute(wasm);
  assert.equal(
    props(para, at).textColor, baseline.textColor,
    '본문 글자 색이 머리말 모드에서 고른 색으로 바뀌면 안 된다',
  );
  cmd.undo(wasm);
}

// ── 7. [adversarial] 모드 진입 "직전" 본문에서 예약한 서식이 IME 조합으로 본문에 새면 안 된다
// (머리말 변형) ──────────────────────────────────────────────────────────────────────
// 6번과 다른 경로: 서식을 머리말 "안"에서 고르는 게 아니라 머리말 진입 전 본문에서 고르면
// (커서가 실제로 움직이지 않았으므로) 앵커가 그대로 유효해 예약이 살아남는다. 이 상태로
// 머리말에서 IME 조합을 하면 applyPendingCharShapeToRange 가 모드를 안 가리고 그 예약을
// 그대로 실제 wasm 범위 적용에 써서, hfCharOffset(머리말 내부 오프셋)을 본문 charOffset인
// 것처럼 anchor.paragraphIndex(본문 문단)에 적용해버린다 — 사용자가 손대지 않은 본문 글자가
// 진짜로 바뀐다.
{
  const at = 0;
  const before = [0, 1, 2].map((i) => props(para, at + i).bold);
  const handler = newHandler();

  handler.cursor.moveTo(pos(para, at));
  handler.applyCharFormat({ bold: true }); // 머리말 진입 "전" 본문에서 굵게 예약
  handler.cursor.enterHeaderFooterMode(true, 0, 0); // 커서 위치는 안 움직인다 — 앵커 그대로 유효

  // onCompositionStart 의 실제 anchor 조립 규약(본문 sec/para + hfCharOffset)을 그대로 재현.
  const compositionAnchor = { ...handler.cursor.getPosition(), charOffset: handler.cursor.hfCharOffset };
  handler.applyPendingCharShapeToRange(compositionAnchor, 3); // 머리말에서 3글자 조합했다고 가정

  const after = [0, 1, 2].map((i) => props(para, at + i).bold);
  assert.deepEqual(after, before, '머리말 진입 직전 예약한 서식이 IME 조합으로 본문 글자를 바꾸면 안 된다');
  handler.cursor.exitHeaderFooterMode();
}

// ── 8. [adversarial] 위와 동일하지만 각주 변형 ──────────────────────────────────────────
// 각주 캐럿은 진입 시 offset 2 에서 시작하는 계약(placeholder 2칸)이라, 새는 위치가 예약한
// 자리(offset 0)와도 다르다 — "엉뚱한 오프셋"이라는 걸 같이 확인한다.
{
  const at = 0;
  const before = [0, 1, 2, 3].map((i) => props(para, at + i).bold);
  const handler = newHandler();

  handler.cursor.moveTo(pos(para, at));
  handler.applyCharFormat({ bold: true }); // 각주 진입 "전" 본문에서 굵게 예약
  handler.cursor.enterFootnoteMode(0, 0, 0, 0, 0); // sectionIdx, paraIdx, controlIdx, footnoteIndex, pageNum

  const compositionAnchor = { ...handler.cursor.getPosition(), charOffset: handler.cursor.fnCharOffset };
  handler.applyPendingCharShapeToRange(compositionAnchor, 2); // 각주에서 2글자 조합했다고 가정

  const after = [0, 1, 2, 3].map((i) => props(para, at + i).bold);
  assert.deepEqual(after, before, '각주 진입 직전 예약한 서식이 IME 조합으로 본문 글자를 바꾸면 안 된다');
  handler.cursor.exitFootnoteMode();
}

// ── 9. [adversarial] 예약과 무관한 삽입(예: 붙여넣기)이 낡은 예약을 되살리면 안 된다 ──────
// 신고 경로와 또 다른 변형: executeOperation 은 desc.command.type === 'insertText' 이기만
// 하면 advancePendingCharShapeAnchor 를 부르는데, pastePlainText 가 만드는 InsertTextCommand
// 는 예약 서식과 무관하다(4번째 인자 없음). 옛 구현은 raw pendingCharShape 필드만 보고 무조건
// 새 위치로 예약 앵커를 옮겨, A 에서 예약한 뒤 실제로 C 로 이동해 낡아버린 예약이 그 이후
// (서식과 무관한) 삽입 위치로 되살아났다. advancePendingCharShapeAnchor 는 이제 이번 삽입이
// 예약 지점(oldPos)에서 실제로 시작했는지 검증하고, 아니면 이어가지 않고 버린다.
{
  const handler = newHandler();

  handler.cursor.moveTo(pos(para, 0));
  handler.applyCharFormat({ bold: true }); // A(offset 0)에서 굵게 예약
  handler.cursor.moveTo(pos(para, 8)); // 진짜 이동: A → C(offset 8), 아직 아무도 예약을 읽지 않았다

  // pastePlainText(input-handler-keyboard.ts)가 만드는 것과 동일한 모양 — 서식 인자 없음.
  // executeOperation 전체 대신 문제의 메서드를 직접 검증한다(같은 실제 프로토타입 메서드).
  handler.advancePendingCharShapeAnchor(pos(para, 8), pos(para, 14)); // C 에서 시작해 14로 끝난 삽입(붙여넣기)

  handler.cursor.moveTo(pos(para, 14)); // 캐럿은 붙여넣기 뒤 위치에 있다
  assert.equal(
    handler.getPendingCharShape(), undefined,
    'A 에서 예약한 뒤 실제로 이동해 낡은 예약이 무관한 삽입(붙여넣기) 위치로 되살아나면 안 된다',
  );

  const cmd = new InsertTextCommand(pos(para, 14), 'Z', undefined, handler.getPendingCharShape());
  cmd.execute(wasm);
  assert.equal(props(para, 14).bold, false, '붙여넣기 뒤 입력한 글자에 A 의 굵게가 새면 안 된다');
  cmd.undo(wasm);
}

console.log('PENDING_CHAR_SHAPE_OK');
