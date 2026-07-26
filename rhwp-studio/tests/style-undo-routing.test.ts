import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

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

test('스타일 다이얼로그는 services 를 받고 history-jumped 로 목록을 무효화한다', () => {
  const dialog = source('src/ui/style-dialog.ts');

  assert.match(dialog, /private services\?: CommandServices/, 'services 주입');
  // undo/redo 는 스타일 목록을 되돌리므로 열려 있는 목록이 stale 이 된다 (#2341).
  assert.match(
    dialog,
    /this\.eventBus\.on\('history-jumped', \(\) => this\.syncAfterHistoryJump\(\)\)/,
    'history-jumped 구독으로 표시 갱신',
  );
  // 목록만 다시 읽으면 `현재 커서 위치 스타일` 라벨이 되돌아가기 전 값으로 남는다 —
  // 히스토리 점프는 캐럿 문단의 style_id 자체가 바뀌는 자리다.
  const sync = slice(dialog, 'private syncAfterHistoryJump', '\n  }');
  assert.match(sync, /getInputHandler\(\)\?\.getCurrentStyleId\(\)/, '캐럿 기준으로 현재 스타일 재조회');
  assert.match(sync, /this\.setCurrentStyleId\(currentId\)/, '라벨·선택·정보 패널까지 갱신');
  // 닫힌 뒤 구독이 남으면 사라진 목록을 갱신하려 든다.
  assert.match(slice(dialog, 'override hide', '\n  }'), /this\.historyJumpOff\?\.\(\)/, '닫을 때 구독 해제');
});

test('스타일 생성·수정은 모양 적용까지 한 스냅샷으로 원자화된다', () => {
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
  assert.match(applyFn, /throw new Error\('\[StyleEditDialog\] 스타일 생성 실패'\)/, '실패 시 무변 엔트리 차단');
  assert.match(save, /this\.eventBus\.emit\('document-changed'\)/, '미주입 fallback 유지');
});

test('스타일 다이얼로그 생성 지점이 services 를 넘긴다', () => {
  const format = source('src/command/commands/format.ts');
  const cmd = slice(format, "id: 'format:style-dialog'", "id: 'format:object-properties'");

  assert.match(cmd, /new StyleDialog\(services\.wasm, services\.eventBus, services\)/, 'StyleDialog');
  assert.match(cmd, /new StyleEditDialog\([\s\S]*?'edit'[\s\S]*?, services\)/, 'StyleEditDialog(edit)');
  assert.match(cmd, /new StyleEditDialog\(services\.wasm, services\.eventBus, 'add', undefined, baseInfo, services\)/,
    'StyleEditDialog(add)');
});
