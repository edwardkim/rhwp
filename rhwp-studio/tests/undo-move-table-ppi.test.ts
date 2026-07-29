import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2903] MoveTableCommand.undo() 는 자신이 호출한 moveTableOffset 의 반환값을
// 버리고 stale 한 this.ppi/this.ci 를 그대로 반환했다. execute() 는 동일한 반환값을
// this.resultPpi/resultCi 에 캡처하는데 undo() 만 이를 무시해 표 이동과 문단 구조 변경이
// 섞인 세션에서 undo 후 커서가 잘못된 문단을 가리킬 수 있었다.
// 소스 가드: undo() 블록이 moveTableOffset 반환값을 변수로 받아 this.ppi/this.ci 를
// 갱신하는지 확인한다(회귀 시 다시 return 문에서 미갱신 stale 값을 쓰게 되면 실패).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const commandSrc = readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8');

function classBlock(src: string, name: string): string {
  const start = src.indexOf(`export class ${name}`);
  assert.notEqual(start, -1, `${name} 클래스 not found`);
  const rel = src.slice(start + 1).indexOf('\nexport class ');
  return rel === -1 ? src.slice(start) : src.slice(start, start + 1 + rel);
}

function undoBlock(block: string): string {
  const uIdx = block.indexOf('undo(wasm: WasmBridge): DocumentPosition {');
  assert.notEqual(uIdx, -1, 'undo 시그니처 not found');
  const rel = block.slice(uIdx + 1).indexOf('\n  mergeWith');
  return rel === -1 ? block.slice(uIdx) : block.slice(uIdx, uIdx + 1 + rel);
}

test('MoveTableCommand.undo 는 moveTableOffset 반환값을 캡처해 this.ppi/this.ci 를 갱신한다', () => {
  const undo = undoBlock(classBlock(commandSrc, 'MoveTableCommand'));

  // 반환값을 그냥 버리는 옛 형태(대입 없이 호출만)가 재발하지 않았는지 확인.
  assert.match(undo, /const result = wasm\.moveTableOffset\(/,
    'undo() 가 moveTableOffset 반환값을 변수로 캡처해야 함(execute() 와 대칭)');
  assert.match(undo, /this\.ppi = result\.ppi/,
    'undo() 가 반환값으로 this.ppi 를 갱신해야 함 — 안 그러면 stale 문단 인덱스 반환');
  assert.match(undo, /this\.ci = result\.ci/,
    'undo() 가 반환값으로 this.ci 를 갱신해야 함 — 안 그러면 stale 컨트롤 인덱스 반환');

  // 반환 DocumentPosition 이 갱신된 this.ppi 를 쓰는지(재대입 후 stale 값을 참조하면 안 됨).
  assert.match(undo, /paragraphIndex:\s*this\.ppi/,
    'undo() 의 반환 DocumentPosition 은 갱신된 this.ppi 를 사용해야 함');
});

test('MoveTableCommand.execute 도 여전히 moveTableOffset 반환값으로 resultPpi/resultCi 를 갱신한다 (회귀 가드)', () => {
  const block = classBlock(commandSrc, 'MoveTableCommand');
  const eIdx = block.indexOf('execute(wasm: WasmBridge): DocumentPosition {');
  assert.notEqual(eIdx, -1, 'execute 시그니처 not found');
  const undoStart = block.indexOf('\n  undo(wasm: WasmBridge)');
  const execBlock = block.slice(eIdx, undoStart === -1 ? undefined : undoStart);

  assert.match(execBlock, /this\.resultPpi = result\.ppi/);
  assert.match(execBlock, /this\.resultCi = result\.ci/);
});
