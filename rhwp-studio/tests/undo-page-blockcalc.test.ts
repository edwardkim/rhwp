import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #page-blockcalc] 명령 계층(page.ts·table.ts 블록계산) 히스토리 라우팅 소스 가드.
//
// 다단 설정·문단 감추기·표 블록계산(합계/평균/곱)이 wasm 뮤테이터를 직접 호출하면 미기록되어
// undo 불가(블록계산은 셀 문자 수까지 바꿔 오프셋 오염). executeOperation snapshot 라우팅을 핀한다.
// 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');
const pageSrc = src('src/command/commands/page.ts');
const tableSrc = src('src/command/commands/table.ts');

/** 함수/블록 소스를 시그니처~다음 최상위 함수 전까지 추출. */
function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('표 블록계산은 evaluateTableFormula commit 을 snapshot 으로 라우팅한다', () => {
  const block = slice(tableSrc, 'function blockCalcCommand', 'function openFormulaDialog');
  assert.match(block, /operationType:\s*'tableBlockCalc'/, 'tableBlockCalc snapshot 라우팅');
  assert.match(block, /\bwasm\.evaluateTableFormula\([^)]*true\s*\)/, 'commit(write=true)은 operation 콜백 안');
  // 기존 직접 commit + emit 패턴 제거(회귀 방지).
  assert.doesNotMatch(block, /services\.eventBus\.emit\('document-changed'\)/,
    '직접 emit 제거 — 라우터가 refresh');
});

test('page.ts 다단 설정·문단 감추기는 snapshot 으로 라우팅한다', () => {
  // 미라우팅 흔적: services.wasm.setColumnDef( 직접 호출 금지.
  assert.doesNotMatch(pageSrc, /services\.wasm\.setColumnDef\s*\(/,
    'setColumnDef 직접 호출 금지 — executeOperation 경유');
  // setPageHide 직접 호출(operation 밖) 금지: services.wasm 경유 setPageHide 는 없어야.
  assert.doesNotMatch(pageSrc, /services\.wasm\s+as\s+any\)\.doc\.setPageHide/,
    'setPageHide 는 operation 콜백(wasm 파라미터) 경유');
  // 라우팅 마커: 다단 3종 + 감추기.
  const colRouted = pageSrc.match(/operationType:\s*'setColumnDef'/g) ?? [];
  assert.equal(colRouted.length, 3, `다단 설정 3종 라우팅(현재 ${colRouted.length})`);
  assert.match(pageSrc, /operationType:\s*'pageHide'/, '문단 감추기 라우팅');
});
