import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// DeleteSelectionCommand.undo 의 복원 방식 가드.
//
// [이력] 예전 undo 는 savedTexts 를 splitParagraph + insertText 로 다시 조립했다. 이때
// 다음 분할점을 0 으로 고정하면 이미 복원된 텍스트 앞을 잘라 빈 문단이 끼어들고 내용이
// 다음 문단으로 밀렸다(#2406). 문단 2개 선택은 루프가 1회라 증상이 없어 오래 살아남았다.
//
//   p5="head5"+A / p6=B / p7=C+"tail7" 를 걸쳐 선택 삭제 후 Ctrl+Z
//   기대: p5="head5"+A, p6=B,  p7=C+"tail7"
//   실제: p5="head5"+A, p6="", p7=C+B+"tail7"   ← 분할점이 0 으로 고정된 경우
//
// [현재] #2418 에서 복원을 문서 스냅샷에 맡기면서 이 분할점 산술 자체가 사라졌다 —
// 조립 과정이 없으므로 위 결함은 재현될 수 없다. 그래서 분할점 계산을 고정하는 대신,
// undo 가 텍스트 재조립으로 되돌아가지 않는지를 고정한다. 되돌아가는 순간 분할점 결함과
// 서식·컨트롤 손실이 함께 살아난다.
//
// node --test 는 strip-only TS 라 engine 클래스를 실행할 수 없어(이 저장소 undo 테스트
// 관례) 소스 배선을 정적으로 검증한다. 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const commandSrc = readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8');

/** `export class NAME ...` 부터 다음 `export class` 전까지 클래스 본문을 추출. */
function classBlock(src: string, name: string): string {
  const start = src.indexOf(`export class ${name}`);
  assert.notEqual(start, -1, `${name} 클래스 not found`);
  const rel = src.slice(start + 1).indexOf('\nexport class ');
  return rel === -1 ? src.slice(start) : src.slice(start, start + 1 + rel);
}

const block = classBlock(commandSrc, 'DeleteSelectionCommand');

test('undo 는 스냅샷 복원에 위임한다', () => {
  assert.match(block, /new SnapshotCommand\('deleteSelection'/, '스냅샷 커맨드에 위임');
  assert.match(block, /undo\(wasm: WasmBridge\): DocumentPosition \{\s*\n\s*return this\.snapshot\.undo\(wasm\);/,
    'undo 는 스냅샷 복원만 한다');
});

test('스냅샷 커서 인자는 (cursorBefore=end, cursorAfter=start) 순서다', () => {
  // [Task #2370 클러스터 D] 종전 가드는 `new SnapshotCommand('deleteSelection'` 존재만
  // 봐서 인자를 `start, end` 로 뒤집어도 green 이었다. SnapshotCommand 의 계약은
  // (operationType, cursorBefore, cursorAfter, operation) 이고 undo 는 cursorBefore 를
  // 돌려주므로, 뒤집으면 undo 후 캐럿이 선택 **끝** 대신 **시작**에 놓이는 무언 회귀가 된다.
  assert.match(
    block,
    /new SnapshotCommand\(\s*'deleteSelection',\s*end,\s*start\s*,/,
    "인자 순서는 ('deleteSelection', end, start, …) — undo 후 캐럿이 선택 끝으로 돌아가야 함",
  );
  // 계약 자체도 함께 핀한다(SnapshotCommand 쪽이 바뀌면 위 순서의 의미가 달라진다).
  assert.match(
    commandSrc,
    /private cursorBefore: DocumentPosition,\s*\n\s*private cursorAfter: DocumentPosition,/,
    'SnapshotCommand 생성자는 cursorBefore, cursorAfter 순서',
  );
  assert.match(
    commandSrc,
    /undo\(wasm: WasmBridge\): DocumentPosition \{[\s\S]{0,220}?return \{ \.\.\.this\.cursorBefore \};/,
    'SnapshotCommand.undo 는 cursorBefore 를 반환',
  );
});

test('undo 가 텍스트 재조립으로 되돌아가지 않는다', () => {
  // 아래가 다시 등장하면 #2406 분할점 결함과 #2418 서식·컨트롤 손실이 함께 살아난다.
  assert.doesNotMatch(block, /savedTexts/, '평문 캡처 부활 금지');
  assert.doesNotMatch(block, /wasm\.splitParagraph\(/, 'undo 에서 문단 재분할 금지');
  assert.doesNotMatch(block, /doInsertTextImmediate\(/, 'undo 에서 텍스트 재삽입 금지');
});

test('스냅샷 예산에 참여한다', () => {
  // 위임만 하고 snapshotResourceCount/discard 를 안 넘기면 히스토리가 이 커맨드의 WASM
  // 스냅샷 id 를 세지 못해 예산이 어긋나고, 축출 시 id 가 반환되지 않아 누수된다(#2328).
  assert.match(block, /snapshotResourceCount\(\): number \{\s*\n\s*return this\.snapshot\.snapshotResourceCount\(\);/,
    'id 개수 위임');
  assert.match(block, /discard\(wasm: WasmBridge\): void \{\s*\n\s*this\.snapshot\.discard\(wasm\);/,
    'discard 위임');
});
