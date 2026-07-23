import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2337-review 후속] 삭제 count 의 char 단위 가드 — 본문/셀 경로.
//
// WASM 삭제 count 는 Rust `Paragraph::delete_text_at` 의 char(Unicode scalar) 단위다
// (src/model/paragraph.rs). JS `String.length`(UTF-16 code unit)를 넘기면 astral 문자
// (😀 등)에서 실제보다 많이 삭제해 undo 가 인접 문자를 잃는다.
//
// #2337 리뷰에서 HF/FN 경로에는 charCount() 가 적용됐지만 본문/셀의
// InsertTextCommand.undo 에는 누락돼 있었다. 되돌아가지 않도록 정면으로 핀한다.
// 행위 증명(😀 입력 → Ctrl+Z 왕복)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const commandSrc = readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8');

/** `export class NAME ...` 부터 다음 `export class` 전까지 클래스 본문을 추출. */
function classBlock(src: string, name: string): string {
  const start = src.indexOf(`export class ${name}`);
  assert.notEqual(start, -1, `${name} 클래스 not found`);
  const rel = src.slice(start + 1).indexOf('\nexport class ');
  return rel === -1 ? src.slice(start) : src.slice(start, start + 1 + rel);
}

/** 클래스 본문에서 undo() 블록만 분리. */
function undoBlock(block: string): string {
  const uIdx = block.indexOf('undo(wasm: WasmBridge): DocumentPosition {');
  assert.notEqual(uIdx, -1, 'undo 시그니처 not found');
  return block.slice(uIdx);
}

test('InsertTextCommand.undo 는 삭제 count 를 charCount 로 계산한다', () => {
  const undo = undoBlock(classBlock(commandSrc, 'InsertTextCommand'));

  assert.match(undo, /doDeleteTextImmediate\([^)]*charCount\(this\.text\)/,
    'astral 문자 over-delete 방지 — charCount 로 코드포인트 수를 넘겨야 함');
  assert.doesNotMatch(undo, /doDeleteTextImmediate\([^)]*this\.text\.length/,
    'UTF-16 length 를 삭제 count 로 넘기면 😀 입력 후 undo 가 인접 문자까지 지운다');
});

test('command.ts 의 삭제 count 에 UTF-16 length 를 넘기는 호출이 없다', () => {
  // 커서 오프셋(charOffset + text.length)은 studio 의 UTF-16 관례를 유지하므로 제외하고,
  // 삭제 count 인자만 본다.
  const deleteCalls = /(doDeleteTextImmediate|deleteTextWithMutationEffects|deleteTextInHeaderFooter|deleteTextInFootnote|deleteTextInCell)\(([^;]{0,400}?)\)/g;
  const offenders: string[] = [];
  for (const m of commandSrc.matchAll(deleteCalls)) {
    const args = m[2];
    // 마지막 인자(count)가 `.length` 로 끝나면 UTF-16 단위다.
    if (/\.length\s*$/.test(args.trim())) offenders.push(m[0]);
  }
  assert.deepEqual(offenders, [],
    `삭제 count 는 charCount() 로 계산해야 함:\n${offenders.join('\n')}`);
});
