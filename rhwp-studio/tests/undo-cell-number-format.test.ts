import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2344] 셀 숫자 서식(천단위 쉼표·자릿점) 히스토리 라우팅 소스 가드.
//
// 세 서식 op(thousand-sep / decimal-add / decimal-remove)가 deleteTextInCell+insertTextInCell
// 을 직접 호출하면 미기록되어, 미기록으로 바뀐 셀 문자 수가 후속 undo 의 오프셋을 오염시켜
// 텍스트가 손상된다("1234567"→쉼표→Ctrl+Z="67"). delete+insert 를 하나의 snapshot 으로
// 원자화해 라우팅했는지 정적으로 핀한다. 행위 증명(손상 회귀 게이트)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const tableSrc = readFileSync(join(rootDir, 'src/command/commands/table.ts'), 'utf8');

test('셀 숫자 서식은 deleteTextInCell/insertTextInCell 를 직접 호출하지 않고 라우팅한다', () => {
  // 미라우팅 흔적: services.wasm.<X>InCell( 직접 호출이 남아있으면 손상 재발.
  assert.doesNotMatch(tableSrc, /services\.wasm\.deleteTextInCell\s*\(/,
    'deleteTextInCell 직접 호출 금지 — executeOperation snapshot 경유여야 함');
  assert.doesNotMatch(tableSrc, /services\.wasm\.insertTextInCell\s*\(/,
    'insertTextInCell 직접 호출 금지 — executeOperation snapshot 경유여야 함');
});

test('세 서식 op 는 delete+insert 를 하나의 cellNumberFormat snapshot 으로 원자화한다', () => {
  // 3개 커맨드(thousand-sep/decimal-add/decimal-remove) 각각 라우팅.
  const routed = tableSrc.match(/operationType:\s*'cellNumberFormat'/g) ?? [];
  assert.equal(routed.length, 3, `3개 서식 op 가 모두 라우팅돼야 함(현재 ${routed.length})`);
  // 뮤테이션 자체는 operation 콜백 안의 wasm.<X>InCell( 로 존재.
  assert.match(tableSrc, /\bwasm\.deleteTextInCell\s*\(/, 'delete 는 operation 콜백에 존재');
  assert.match(tableSrc, /\bwasm\.insertTextInCell\s*\(/, 'insert 는 operation 콜백에 존재');
  // 세 커맨드 id 존재(회귀 시 커맨드 자체가 사라지는 것 방지).
  for (const id of ['table:thousand-sep', 'table:decimal-add', 'table:decimal-remove']) {
    assert.match(tableSrc, new RegExp(`id:\\s*'${id}'`), `${id} 커맨드 존재`);
  }
});
