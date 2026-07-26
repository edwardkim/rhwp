import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2370 클러스터 A] 무변경 연산 기록 스킵 장치의 소스 가드.
//
// 아무것도 바꾸지 않은 연산이 undo 스택에 들어가면 ①Ctrl+Z 한 번이 무효과로 소모되고
// ②새 명령이라 redo 스택이 파기되며 ③스냅샷 명령이면 예산 2슬롯을 먹어 오래된 진짜
// 이력이 축출된다(#2328). 세 피해를 끊는 지점이 아래 3곳이며, 하나라도 빠지면 phantom
// 엔트리가 조용히 되살아난다(행위는 브라우저 왕복으로 증명 — PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string) => readFileSync(join(rootDir, rel), 'utf8');

test('SnapshotCommand 는 operation 이 null 을 반환하면 스냅샷을 버리고 noOp 를 세운다', () => {
  const commandSrc = src('src/engine/command.ts');
  const start = commandSrc.indexOf('export class SnapshotCommand');
  assert.notEqual(start, -1, 'SnapshotCommand 가 존재해야 함');
  const block = commandSrc.slice(start);
  const end = block.indexOf('\n}\n');
  const body = block.slice(0, end === -1 ? block.length : end);

  assert.match(body, /result === null/, 'operation 의 null 반환을 무변경 신호로 읽어야 함');
  assert.match(body, /this\.noOp = true/, '무변경이면 noOp 플래그를 세워야 함');
  assert.match(body, /isNoOp\(\)\s*:\s*boolean/, 'isNoOp() 로 히스토리에 노출해야 함');
  // after 스냅샷을 저장하지 않고 before 를 해제해야 예산을 점유하지 않는다.
  const noOpBranch = body.slice(body.indexOf('result === null'), body.indexOf('this.cursorAfter = result'));
  assert.match(noOpBranch, /this\.discard\(wasm\)/, '무변경 경로는 before 스냅샷을 즉시 해제해야 함');
  assert.doesNotMatch(noOpBranch, /saveSnapshot/, '무변경 경로에서 after 스냅샷을 저장하면 안 됨');
});

test('CommandHistory.execute 는 isNoOp 명령을 스택에 넣지 않고 redo 도 보존한다', () => {
  const historySrc = src('src/engine/history.ts');
  const start = historySrc.indexOf('execute(command: EditCommand, wasm: WasmBridge)');
  assert.notEqual(start, -1, 'execute 가 존재해야 함');
  const body = historySrc.slice(start, historySrc.indexOf('/** Undo', start));

  const guardIdx = body.indexOf('command.isNoOp?.()');
  assert.notEqual(guardIdx, -1, 'execute 가 isNoOp 를 확인해야 함');
  const pushIdx = body.indexOf('this.undoStack.push(command)');
  const discardIdx = body.indexOf('discardAll(this.redoStack');
  assert.ok(guardIdx < pushIdx, 'isNoOp 확인은 undoStack.push 앞이어야 함(기록 자체를 막음)');
  assert.ok(guardIdx < discardIdx, 'isNoOp 확인은 redo 파기 앞이어야 함(무변경이 redo 를 지우면 안 됨)');
  const guardBlock = body.slice(guardIdx, pushIdx);
  assert.match(guardBlock, /command\.discard\?\.\(wasm\)/, '기록하지 않는 명령의 리소스를 해제해야 함');
});

test('executeOperation 의 snapshot 분기는 무변경이면 커서 이동·리프레시를 건너뛴다', () => {
  const ihSrc = src('src/engine/input-handler.ts');
  const start = ihSrc.indexOf("case 'snapshot': {");
  assert.notEqual(start, -1, 'snapshot 분기가 존재해야 함');
  const body = ihSrc.slice(start, ihSrc.indexOf("case 'record': {", start));

  const guardIdx = body.indexOf('cmd.isNoOp()');
  assert.notEqual(guardIdx, -1, 'snapshot 분기가 isNoOp 를 확인해야 함');
  assert.ok(guardIdx < body.indexOf('this.cursor.moveTo'), '무변경이면 커서를 옮기지 않아야 함');
  assert.ok(guardIdx < body.indexOf('this.refreshAfterOperation'), '무변경이면 리프레시하지 않아야 함');
  // 무변경 경로에서도 pending 플래그는 소비돼야 한다(다음 연산으로 새면 안 됨).
  assert.ok(
    body.indexOf('this.pastedFieldEndOutsidePending = false') < guardIdx,
    'pastedFieldEndOutsidePending 소비는 조기 종료 앞이어야 함',
  );
});

test('경계 z순서는 반환 zOrder 비교로 무변경을 보고한다', () => {
  const insertSrc = src('src/command/commands/insert.ts');
  const start = insertSrc.indexOf('function changeZOrder');
  assert.notEqual(start, -1, 'changeZOrder 헬퍼가 존재해야 함');
  const body = insertSrc.slice(start, start + 700);

  assert.match(body, /getProps\(services, ref\)/, '호출 전 zOrder 를 읽어야 함');
  assert.match(body, /r\.zOrder !== zBefore/, '반환 zOrder 와 호출 전 값을 비교해 변경 여부를 판정해야 함');
  assert.match(body, /wasm\.changeShapeZOrder\(/, '뮤테이션은 operation 콜백 안에 있어야 함');
});

test('레이아웃 setter 의 {ok:false} 는 도달 불가 — 없는 신호를 검사하지 않는다', () => {
  // 자체 리뷰에서 확인: setPageDef/setSectionDef/setSectionDefAll/setPageBorderFill/
  // setColumnDef/applyEndnoteShape 의 네이티브는 성공 시 `{"ok":true,…}` **하나만** 반환하고
  // 실패는 전부 `Err(HwpError)` → WASM 경계에서 throw 다. 브리지도 pass-through
  // (`JSON.parse(this.doc.setPageDef(...))`)라 {ok:false} 를 합성하지 않는다.
  // 선언 타입이 `{ ok: boolean }` 이라 검사할 값처럼 보이지만 실제로는 항상 true 다.
  //
  // 그래서 이 두 다이얼로그에는 무변경 신호로 삼을 것이 없다 — `apply().ok ? … : null`
  // 같은 가드를 넣으면 #3438 이 지적한 "존재하지 않는 실패 신호를 가정한 방어"가 된다.
  // 거부는 throw 로 오고, 그건 no-op 장치가 아니라 예외 경로가 다룬다.
  const rendering = readFileSync(join(rootDir, '../src/document_core/queries/rendering.rs'), 'utf8');
  for (const fn of ['set_page_def_native', 'set_section_def_native']) {
    const start = rendering.indexOf(`pub fn ${fn}`);
    assert.notEqual(start, -1, `${fn} 이 존재해야 함`);
    const body = rendering.slice(start, rendering.indexOf('\n    pub fn ', start + 1));
    assert.doesNotMatch(body, /"ok\\":false/, `${fn} 이 {ok:false} 를 반환하게 되면 이 전제가 깨진다`);
  }
  for (const rel of ['src/ui/page-setup-dialog.ts', 'src/ui/section-settings-dialog.ts']) {
    assert.doesNotMatch(
      src(rel),
      /operation: \(ih\) => \(apply\(\)\.ok \?/,
      `${rel}: 도달 불가한 {ok:false} 를 무변경 신호로 쓰지 말 것`,
    );
  }
});

test('미주 모양은 값이 안 바뀌면 뮤테이션도 기록도 하지 않는다', () => {
  const dialogSrc = src('src/ui/endnote-shape-dialog.ts');
  assert.match(dialogSrc, /const unchanged = \(Object\.keys\(next\)/, '변경 여부를 next 의 전 키로 판정해야 함');
  const idx = dialogSrc.indexOf('operation: (ih) => {');
  assert.notEqual(idx, -1, 'snapshot operation 콜백이 존재해야 함');
  const block = dialogSrc.slice(idx, idx + 400);
  assert.match(block, /if \(unchanged\) return null/, '무변경이면 null 로 기록을 취소해야 함');
  assert.ok(
    block.indexOf('if (unchanged) return null') < block.indexOf('apply()'),
    '무변경 판정은 apply() 앞이어야 함(뮤테이션 자체를 생략)',
  );
});

test('선택 해제가 뒤따르는 개체 조작은 스냅샷 리프레시를 넘긴다(중복 repaint 방지)', () => {
  const insertSrc = src('src/command/commands/insert.ts');
  assert.match(
    insertSrc,
    /const DEFER_REFRESH_TO_EXIT = \{ refresh: 'none' \} as const/,
    'exit 경로로 리프레시를 미루는 상수가 있어야 함',
  );
  // exitPictureObjectSelectionAndAfterEdit 가 뒤따르는 recordObjectMutation 은
  // 스냅샷 쪽 'full' refresh 를 꺼야 afterEdit 이 두 번 돌지 않는다.
  for (const op of ['deleteObject', 'groupShapes', 'ungroupShape']) {
    const idx = insertSrc.indexOf(`recordObjectMutation(ih, '${op}'`);
    assert.notEqual(idx, -1, `${op} 라우팅이 존재해야 함`);
    assert.match(
      insertSrc.slice(idx, idx + 600),
      /DEFER_REFRESH_TO_EXIT/,
      `${op} 는 선택 해제 afterEdit 과 중복되지 않도록 refresh 를 미뤄야 함`,
    );
  }
});
