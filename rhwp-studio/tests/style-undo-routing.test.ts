// 관계형: 스타일 API의 성공·실패와 모양 적용·snapshot 반환·후속 알림의 관계.
// 주석 갱신: 2026-11-06T01:29:04+09:00. 직전 파일 commit: b3ceeb3303ad033218237a61e4d24dea8a8d7227 (2026-08-13T17:14:22+09:00).
// UI 트랜잭션 계약 검사이며 문서 배치나 px 좌표를 측정하지 않는다. #8526의 실제 API 근거는 해당 PR 리뷰에 연결한다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { stripTypeScriptTypes } from 'node:module';
import { functionBodyFrom } from './support/source-guard.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

// [Task #3387 / #2369 Track 4] 스타일 생성·수정·삭제가 편집 라우터를 통과해야 한다.
// 종전에는 다이얼로그가 wasm 을 직접 호출해 Ctrl+Z 로 되돌아가지 않았고, 삭제는 그 스타일을
// 쓰던 전 문단의 style_id 재배정까지 복구 불가였다.
//
// 라우팅 여부는 뮤테이션 표면 원장이 아니라 파일별 가드가 담당한다(#2369 공통 규약).

test('스타일 삭제는 snapshot 으로 기록된다', () => {
  const dialog = source('src/ui/style-dialog.ts');
  const handleDelete = slice(dialog, 'private handleDelete', '\n  /**');

  assert.match(
    handleDelete,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: 'deleteStyle'/,
    '삭제는 편집 라우터의 snapshot 으로 기록',
  );
  // 의미상 실패를 그대로 기록하면 before==after 무변 엔트리가 스냅샷 예산만 먹는다.
  assert.match(handleDelete, /if \(!wasm\.deleteStyle\(deletedId\)\)/, '실패 시 throw 로 무변 엔트리 차단');
  // services 미주입 경로는 종전 동작을 유지한다(#2077 동형).
  assert.match(handleDelete, /this\.eventBus\.emit\('document-changed'\)/, '미주입 fallback 유지');
});

test('스타일 다이얼로그는 모달 중 도달 불가한 history-jumped 를 구독하지 않는다', () => {
  const dialog = source('src/ui/style-dialog.ts');

  assert.match(dialog, /private services\?: CommandServices/, 'services 주입');
  // 한컴 2022도 F6 스타일 대화상자 중 Ctrl+Z를 문서에 전달하지 않는다. rhwp의 모달
  // capture 계약도 같으므로 이 다이얼로그가 소비할 history-jumped 자체가 없다 (#3438).
  assert.doesNotMatch(dialog, /history-jumped|historyJumpOff|syncAfterHistoryJump/,
    '도달 불가한 구독·동기화·해제 코드를 두지 않음');
});

test('스타일 생성·수정은 모양 적용까지 한 스냅샷으로 원자화되고 실제 실패 신호만 처리한다', () => {
  const dialog = source('src/ui/style-edit-dialog.ts');
  const save = slice(dialog, 'const apply = (wasm: WasmBridge)', 'override show()');

  assert.match(
    save,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: this\.addMode \? 'createStyle' : 'updateStyle'/,
    '생성/수정은 snapshot 으로 기록',
  );
  // createStyle/updateStyle + updateStyleShapes 는 2뮤테이션이다. 따로 기록하면 undo 가
  // 두 번 필요하고 그 사이에 모양만 빠진 스타일이 남는다 (#2366 계산식과 동형).
  const applyFn = slice(save, 'const apply = (wasm: WasmBridge)', '\n    };');
  assert.match(applyFn, /wasm\.createStyle\(/, '생성이 같은 apply 안에');
  assert.match(applyFn, /wasm\.updateStyle\(/, '수정이 같은 apply 안에');
  assert.match(applyFn, /wasm\.updateStyleShapes\(/, '모양 적용이 같은 apply 안에');
  assert.equal(
    (save.match(/executeOperation\(/g) ?? []).length,
    1,
    '스냅샷은 하나여야 한다(2뮤테이션 원자화)',
  );
  // #8526은 잘못된 삽입 위치를 -1로 거절한다. 이 실패를 성공한 snapshot으로 남기지 않는다.
  assert.match(applyFn, /if \(newId < 0\) return false;/, '실제 생성 실패는 모양 적용 전에 중단');
  // 반면 기존 style ID를 받는 두 API는 false를 반환하므로, 실패를 무시하면 스냅샷에
  // 무변 엔트리가 남거나 부분 변경을 성공처럼 처리하게 된다.
  assert.match(applyFn, /if \(!wasm\.updateStyle\(this\.styleInfo\.id/, 'updateStyle false를 확인');
  assert.match(applyFn, /if \(!wasm\.updateStyleShapes\(/, 'updateStyleShapes false를 확인');
  assert.match(
    save,
    /operation: \(wasm\) => \{\s*saved = apply\(wasm\);\s*return saved \? ih\.getPosition\(\) : null;/,
    'updateStyle false는 snapshot no-op으로 전달',
  );
  assert.match(save, /let saved = false/, '실제 저장 성공 여부를 확인');
  assert.match(save, /if \(!saved\) return false/, 'false 반환은 모달을 닫지 않음');
  assert.match(save, /catch \(err\) \{[\s\S]*?return false;/, '예외도 모달을 닫지 않음');
  assert.match(save, /this\.onSave\?\.\(\);\s*return true;/, '성공한 저장 때만 후속 새로고침');
  assert.match(save, /this\.eventBus\.emit\('document-changed'\)/, '미주입 fallback 유지');
});

test('기존 스타일 갱신 API는 없는 ID에 false를 반환한다', () => {
  const wasmApi = source('../src/wasm_api.rs');
  const updateStyle = slice(wasmApi, 'pub fn update_style', '\n    /// 스타일의 CharShape/ParaShape를 수정한다.');
  const updateStyleShapes = slice(wasmApi, 'pub fn update_style_shapes', '\n    /// 새 스타일을 생성한다.');

  assert.match(updateStyle, /None => return false/, '없는 기존 스타일은 updateStyle=false');
  assert.match(updateStyleShapes, /None => return false/, '없는 기존 스타일은 updateStyleShapes=false');
  // 생성·삽입·실패 시 문서 보존은 inserted_style_keeps_references.rs의 실제 API 검사로 보호한다.
  // Vec의 push/insert나 반환 지역 변수 이름을 생성 계약의 대용으로 고정하지 않는다.
});

function styleCreation(id: number, routed: boolean, withFormatting: boolean) {
  const method = functionBodyFrom(source('src/ui/style-edit-dialog.ts'), 'protected onConfirm(');
  const executable = stripTypeScriptTypes(`const holder = { ${method.replace('protected ', '')} };`);
  const confirm = new Function('MAX_STYLE_NAME_LEN', 't', `${executable}; return holder.onConfirm;`)(250, () => '입력 오류');
  const effects = { shapes: [] as number[], notifications: 0, saved: 0, operations: [] as unknown[] };
  const cursor = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
  const wasm = {
    createStyle: () => id,
    updateStyleShapes: (actual: number) => { effects.shapes.push(actual); return true; },
  };
  const dialog = {
    nameInput: { value: '새 스타일' }, enNameInput: { value: '' },
    typePara: { checked: true }, nextStyleSelect: { value: '0' },
    styleInfo: { type: 0, nextStyleId: 0 }, addMode: true, baseInfo: {},
    charModsJson: withFormatting ? '{"bold":true}' : '{}', paraModsJson: '{}',
    wasm, eventBus: { emit: () => { effects.notifications++; } },
    onSave: () => { effects.saved++; },
    services: routed ? { getInputHandler: () => ({
      getPosition: () => cursor,
      executeOperation: (operation: { operation: (value: typeof wasm) => unknown }) => {
        effects.operations.push(operation.operation(wasm));
      },
    }) } : undefined,
  };
  return { confirm: () => confirm.call(dialog), effects, cursor };
}

test('스타일 생성 실패는 실제 확인 경로에서 모양 적용·성공 알림을 생략한다', () => {
  for (const routed of [false, true]) {
    for (const withFormatting of [false, true]) {
      const run = styleCreation(-1, routed, withFormatting);
      assert.equal(run.confirm(), false, `routed=${routed}, formatting=${withFormatting}`);
      assert.deepEqual(run.effects.shapes, [], '실패 ID로 다른 스타일을 수정하지 않는다');
      assert.equal(run.effects.saved, 0);
      assert.equal(run.effects.notifications, 0);
      assert.deepEqual(run.effects.operations, routed ? [null] : [], '실패 snapshot은 no-op을 반환');
    }
  }
});

test('0을 포함한 성공 스타일 ID는 모양 적용과 성공 후속 처리를 유지한다', () => {
  for (const id of [0, 3]) {
    for (const routed of [false, true]) {
      const run = styleCreation(id, routed, true);
      assert.equal(run.confirm(), true);
      assert.deepEqual(run.effects.shapes, [id]);
      assert.equal(run.effects.saved, 1);
      assert.equal(run.effects.notifications, routed ? 0 : 1);
      assert.deepEqual(run.effects.operations, routed ? [run.cursor] : []);
    }
  }
});

test('스타일 다이얼로그 생성 지점이 services 를 넘긴다', () => {
  const format = source('src/command/commands/format.ts');
  const cmd = slice(format, "id: 'format:style-dialog'", "id: 'format:object-properties'");

  assert.match(cmd, /new StyleDialog\(services\.wasm, services\.eventBus, services\)/, 'StyleDialog');
  assert.match(cmd, /new StyleEditDialog\([\s\S]*?'edit'[\s\S]*?, services\)/, 'StyleEditDialog(edit)');
  assert.match(cmd, /new StyleEditDialog\(services\.wasm, services\.eventBus, 'add', undefined, baseInfo, services\)/,
    'StyleEditDialog(add)');
});
