import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #formula-dialog] 계산식 다이얼로그 히스토리 라우팅 소스 가드.
//
// formula-dialog 는 (wasm,eventBus,ctx) 로만 생성돼 편집 라우터에 도달 못 했고, commit
// (evaluateTableFormula write=true) + 쉼표 insertTextInCell 두 뮤테이션이 미기록이라 undo
// 불가 + 셀 문자 수 변화로 후속 undo 오프셋 오염(#2344 계열). services 주입 + 두 뮤테이션의
// 단일 snapshot 원자화 + dry-run(검증)은 스냅샷 밖 유지를 정적으로 핀한다. 행위 증명은
// 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/formula-dialog.ts'), 'utf8');

test('formula-dialog 는 services 주입 + tableFormula snapshot 라우팅 + fallback 을 갖춘다', () => {
  assert.match(src, /services\?:\s*CommandServices/, '생성자에 services 주입');
  assert.match(src, /import type \{ CommandServices \}/, 'CommandServices import');
  assert.match(src, /this\.services\?\.getInputHandler\(\)/, 'getInputHandler 로 라우터 도달');
  assert.match(src, /operationType:\s*'tableFormula'/, 'tableFormula snapshot 라우팅');
  assert.match(src, /this\.eventBus\.emit\('document-changed'\)/, 'fallback emit 유지');
});

test('commit + 쉼표 insert 는 하나의 commit 클로저로 원자화되고 dry-run 은 밖에 있다', () => {
  // 원자화: 두 뮤테이션이 같은 commit 클로저 안에 있어야 함(사이에 스냅샷 경계 금지).
  const commitStart = src.indexOf('const commit = () => {');
  assert.notEqual(commitStart, -1, 'commit 클로저 존재');
  const commitEnd = src.indexOf('};', src.indexOf('catch { /* 쉼표', commitStart));
  const commitBody = src.slice(commitStart, commitEnd);
  assert.match(commitBody, /evaluateTableFormula\([\s\S]*?true,?\s*\)/, 'commit(write=true)은 클로저 안');
  assert.match(commitBody, /insertTextInCell\(/, '쉼표 insert 도 같은 클로저 안(원자화)');
  // dry-run(write=false) 검증은 클로저/스냅샷 밖(실패 시 no-op 스냅샷 방지).
  const dryRunIdx = src.indexOf('formula, false,');
  assert.ok(dryRunIdx !== -1 && dryRunIdx < commitStart, 'dry-run 검증은 commit 클로저보다 앞(밖)');
});
