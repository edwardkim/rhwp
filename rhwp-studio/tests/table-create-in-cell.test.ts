import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#8262] 표 칸 안에 표 만들기 — 칸 안에서도 명령이 열리고, 칸 안이면 셀 경로 생성 API 로 간다.
// 엔진 쪽 구조(한/글 2020 표본과 같은 모양)는 Rust 테스트
// `object_ops::table::tests::create_table_*_cell*` 가 본다. 여기서는 Studio 배선만 본다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string): string => readFileSync(join(rootDir, path), 'utf8');

function createCommandBlock(): string {
  const src = source('src/command/commands/table.ts');
  const start = src.indexOf("id: 'table:create'");
  const end = src.indexOf("id: 'table:cell-props'", start);
  assert.notEqual(start, -1, 'table:create 정의를 찾을 수 있어야 한다');
  assert.notEqual(end, -1, '다음 명령 정의를 찾을 수 있어야 한다');
  return src.slice(start, end);
}

test('표 만들기는 표 칸 안에서도 실행할 수 있다', () => {
  const block = createCommandBlock();
  assert.doesNotMatch(block, /!ctx\.inTable/, '칸 안에서 막는 조건이 남아 있으면 안 된다');
  assert.doesNotMatch(
    block,
    /if \(pos\.parentParaIndex !== undefined\) return;/,
    '칸 안이면 그냥 돌아가는 조기 반환이 남아 있으면 안 된다',
  );
});

test('글상자 안에서는 아직 만들지 않는다', () => {
  assert.match(createCommandBlock(), /if \(pos\.isTextBox\) return;/);
});

test('칸 안이면 셀 경로 생성 API 로 보내고 캐럿을 안쪽 표 첫 칸에 둔다', () => {
  const block = createCommandBlock();
  assert.match(block, /if \(inCell\) return createTableInCell\(wasm, pos, rows, cols, options\);/);
  assert.match(block, /dialog\.inlineOnly = inCell;/);

  const src = source('src/command/commands/table.ts');
  const fnStart = src.indexOf('function createTableInCell(');
  assert.notEqual(fnStart, -1);
  const fn = src.slice(fnStart, src.indexOf('\n}\n', fnStart));
  assert.match(fn, /wasm\.createTableInCellByPath\(/);
  // 깊이 1 칸은 cellPath 가 없을 수 있다 — 평면 필드로 경로를 만든다.
  assert.match(fn, /pos\.cellPath\?\.length/);
  assert.match(fn, /controlIndex: pos\.controlIndex!, cellIndex: pos\.cellIndex!, cellParaIndex: pos\.cellParaIndex!/);
  // 캐럿 = 반환 경로(안쪽 표 첫 칸). 평면 필드는 경로 첫 마디(바깥 표)다.
  assert.match(fn, /cellPath: result\.cellPath/);
  assert.match(fn, /const \[outer\] = result\.cellPath;/);
});

test('칸 안 대화에서는 글자처럼 취급을 켠 채 잠근다', () => {
  const dialog = source('src/ui/table-create-dialog.ts');
  const fnStart = dialog.indexOf('private applyInlineOnly(');
  assert.notEqual(fnStart, -1);
  const fn = dialog.slice(fnStart, dialog.indexOf('\n  }\n', fnStart));
  assert.match(fn, /if \(!this\.inlineOnly\) return;/);
  assert.match(fn, /treatChk\.checked = true;/);
  assert.match(fn, /treatChk\.disabled = true;/);
  assert.match(dialog, /this\.applyInlineOnly\(treatChk\);/);
  for (const locale of ['ko', 'en']) {
    assert.match(
      source(`src/i18n/locales/${locale}.ts`),
      /"dialog\.tableCreate\.treatInCell\.title":/,
      `${locale} 문구`,
    );
  }
});

test('칸 안 생성은 문서 변경 메서드로 등록돼 있다', () => {
  assert.match(source('src/core/mutation-method-registry.ts'), /'createTableInCellByPath'/);
  assert.match(source('src/core/wasm-bridge.ts'), /createTableInCellByPath\(\s*sec: number,/);
});
