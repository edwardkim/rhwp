import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 글자 모양 다이얼로그(char-shape-dialog.ts)의 장평/자간 입력은 <input type=number>의
// min/max 속성만으로는 범위를 강제하지 못한다 — 직접 타이핑 후 blur 없이 값을 읽으면
// 범위 밖 숫자가 그대로 저장된다. 단축키 커맨드(input-handler.ts의 adjustCharRatio/
// adjustCharSpacing)는 Math.max/Math.min으로 장평 50~200, 자간 -50~50을 코드로 clamp
// 하므로, 다이얼로그도 saveLangFields()에서 동일하게 clamp 해야 둘의 상한/하한이 어긋나지
// 않는다. (선례: #2925/#2930 — 툴바 줄 간격 clamp 누락)

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const dialogSrc = readFileSync(join(rootDir, 'src/ui/char-shape-dialog.ts'), 'utf8');
const inputHandlerSrc = readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8');

function fnBlock(src: string, name: string): string {
  const start = src.indexOf(name);
  assert.notEqual(start, -1, `${name} not found`);
  const rel = src.slice(start).indexOf('\n  }');
  assert.notEqual(rel, -1, `${name} 종료 지점 not found`);
  return src.slice(start, start + rel);
}

test('char-shape-dialog saveLangFields가 단축키 커맨드와 동일한 장평/자간 범위로 clamp한다', () => {
  const dialogBlock = fnBlock(dialogSrc, 'private saveLangFields');
  assert.match(
    dialogBlock,
    /Math\.max\(50,\s*Math\.min\(200,\s*parseInt\(this\.langInputs\['cs-ratio'\]\.value\)/,
    '장평 입력이 50~200으로 clamp되어야 한다 (adjustCharRatio와 동일)'
  );
  assert.match(
    dialogBlock,
    /Math\.max\(-50,\s*Math\.min\(50,\s*parseInt\(this\.langInputs\['cs-spacing'\]\.value\)/,
    '자간 입력이 -50~50으로 clamp되어야 한다 (adjustCharSpacing과 동일)'
  );

  // 단축키 커맨드 쪽 clamp 범위가 바뀌면 이 테스트도 같이 깨지도록 소스에서 직접 확인.
  const ratioFn = fnBlock(inputHandlerSrc, 'adjustCharRatio(delta: number)');
  assert.match(ratioFn, /Math\.max\(50,\s*Math\.min\(200,/, 'adjustCharRatio 기준 범위가 50~200이어야 한다');
  const spacingFn = fnBlock(inputHandlerSrc, 'adjustCharSpacing(delta: number)');
  assert.match(spacingFn, /Math\.max\(-50,\s*Math\.min\(50,/, 'adjustCharSpacing 기준 범위가 -50~50이어야 한다');
});
