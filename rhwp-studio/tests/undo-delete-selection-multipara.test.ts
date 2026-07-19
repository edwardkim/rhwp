import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// DeleteSelectionCommand.undo 의 다중 문단 복원 분할점 가드.
//
// 본문 분기는 savedTexts[i] 를 복원할 때마다 splitParagraph 로 문단을 다시 만든다.
// 이때 다음 분할점은 "방금 복원한 텍스트 뒤" 여야 한다. 0 으로 고정하면 이미 복원된
// 텍스트 앞을 잘라, 빈 문단이 끼어들고 내용이 다음 문단으로 밀린다.
//
//   p5="head5"+A / p6=B / p7=C+"tail7" 를 걸쳐 선택 삭제 후 Ctrl+Z
//   기대: p5="head5"+A, p6=B,  p7=C+"tail7"
//   실제: p5="head5"+A, p6="", p7=C+B+"tail7"   ← 분할점이 0 으로 고정된 경우
//
// 문단 2개 선택(가장 흔한 경우)은 루프가 1 회라 증상이 없어 오래 살아남았다.
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

test('다중 문단 undo 의 분할점이 0 으로 고정되지 않는다', () => {
  assert.doesNotMatch(block, /currentPara\+\+;\s*\n\s*currentOffset = 0;/,
    '분할 직후 currentOffset 을 0 으로 고정하면 복원된 텍스트 앞을 잘라 문단이 어긋난다');
});

test('다음 분할점은 복원한 텍스트 길이에서 나온다', () => {
  assert.match(block, /currentOffset = text \? text\.length : 0;/,
    '다음 splitParagraph 는 방금 삽입한 텍스트 뒤에서 일어나야 함');

  // 순서 보장: insertText 로 복원한 뒤에 다음 분할점을 계산해야 한다.
  const insertAt = block.search(/wasm\.insertText\(sec, currentPara, 0, text\)/);
  const offsetAt = block.search(/currentOffset = text \? text\.length : 0;/);
  assert.ok(insertAt >= 0 && offsetAt >= 0, 'insertText/분할점 계산이 모두 존재해야 함');
  assert.ok(insertAt < offsetAt, '분할점 계산은 insertText 뒤에 와야 함');
});
