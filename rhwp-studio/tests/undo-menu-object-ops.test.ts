import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2343] 메뉴/도구상자 개체 조작 히스토리 라우팅 소스 가드.
//
// 개체 삭제·정렬(z순서)·묶기/풀기·회전/대칭을 메뉴로 실행해도 undo 되도록, 해당 뮤테이션이
// executeOperation({kind:'snapshot'}) 로 라우팅되는지 정적으로 핀한다. 뮤테이션 표면 원장
// (mutation-routing-guard)은 '표면 증가'만 잡고 '라우팅 누락'은 못 잡으므로(라우팅해도
// wasm.X( 텍스트는 그대로 남음) 이 가드로 재발을 차단한다. 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const insertSrc = readFileSync(join(rootDir, 'src/command/commands/insert.ts'), 'utf8');

// recordObjectMutation 경유로 라우팅돼야 하는 개체 뮤테이터(속성 setter 는 setProps 공유·별건).
const OBJECT_MUTATORS = [
  'changeShapeZOrder',
  'deleteShapeControl',
  'deleteEquationControl',
  'deleteCellPictureControlByPath',
  'deletePictureControl',
  'groupShapes',
  'ungroupShape',
];

test('recordObjectMutation 은 executeOperation snapshot 으로 위임한다', () => {
  const start = insertSrc.indexOf('function recordObjectMutation');
  assert.notEqual(start, -1, 'recordObjectMutation 헬퍼가 존재해야 함');
  const block = insertSrc.slice(start, start + 500);
  assert.match(block, /ih\.executeOperation\(/, 'executeOperation 에 위임');
  assert.match(block, /kind:\s*'snapshot'/, 'snapshot 커맨드로 기록(undo/redo 보장)');
});

test('개체 조작 뮤테이터는 services.wasm 직접 호출이 아니라 라우팅된다', () => {
  for (const m of OBJECT_MUTATORS) {
    // 미라우팅 흔적: services.wasm.<mutator>( 가 남아있으면 회귀.
    assert.doesNotMatch(
      insertSrc,
      new RegExp(`services\\.wasm\\.${m}\\s*\\(`),
      `${m} 는 recordObjectMutation 경유여야 함(services.wasm 직접 호출 금지 — 히스토리 우회)`,
    );
    // 라우팅된 호출: operation 콜백 안의 wasm.<mutator>( 는 존재해야 함.
    assert.match(
      insertSrc,
      new RegExp(`\\bwasm\\.${m}\\s*\\(`),
      `${m} 뮤테이션 자체는 operation 콜백에 존재해야 함`,
    );
  }
});

test('회전/대칭도 recordObjectMutation 으로 기록한다', () => {
  const rot = insertSrc.slice(insertSrc.indexOf('function applyRotationDelta'), insertSrc.indexOf('function toggleFlip'));
  assert.match(rot, /recordObjectMutation\(ih, 'rotateObject'/, '회전을 snapshot 으로 기록');
  const flip = insertSrc.slice(insertSrc.indexOf('function toggleFlip'));
  assert.match(flip.slice(0, 700), /recordObjectMutation\(ih, 'flipObject'/, '대칭을 snapshot 으로 기록');
});
