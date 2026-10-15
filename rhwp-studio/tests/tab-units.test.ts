import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { codeOnly, functionBodyFrom } from './support/source-guard.ts';

// 탭 위치와 구역 기본 탭 간격은 HWPUNIT 의 2배로 저장된다. 조판(style_resolver.rs)은 탭 위치를
// 저장값의 절반으로 놓고, 한컴 HWPX 는 기본 탭 40pt 를 tabStop="8000" tabStopVal="4000" 으로 적는다.
// 대화상자가 pt×100 으로 보내면 150pt 탭이 75pt 에 놓이고, 저장값÷100 으로 보여 주면 두 배로 보인다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(relativePath: string): string {
  return codeOnly(readFileSync(join(rootDir, relativePath), 'utf8'));
}

test('탭 저장값은 1pt 에 200 이다', async () => {
  const { ptToTabUnits, tabUnitsToPt } = await import('../src/core/tab-units.ts');
  assert.equal(tabUnitsToPt(8000), 40);
  assert.equal(ptToTabUnits(40), 8000);
  assert.equal(ptToTabUnits(150), 30000);
  assert.equal(tabUnitsToPt(92552), 462.76);
  assert.equal(ptToTabUnits(12.34), 2468);
});

test('탭 설정은 탭 위치를 저장 배율로 넣고 보여 준다', () => {
  const builders = source('src/ui/para-shape-tab-builders.ts');
  assert.match(functionBodyFrom(builders, 'function addTabStop'), /ptToTabUnits\(positionPt\)/);
  for (const name of ['function renderTabList', 'function renderDeletedTabList']) {
    assert.match(functionBodyFrom(builders, name), /tabUnitsToPt\(t\.position\)/, name);
  }
  const dialog = source('src/ui/para-shape-dialog.ts');
  assert.match(functionBodyFrom(dialog, 'private populateFromProps'), /tabUnitsToPt\(defSpacing\)/);
});

test('구역 설정은 기본 탭 간격을 저장 배율로 보여 준다', () => {
  const dialog = source('src/ui/section-settings-dialog.ts');
  assert.match(
    functionBodyFrom(dialog, 'private populateFields'),
    /this\.defaultTabSpacingInput\.value\s*=\s*tabUnitsToPt\(sd\.defaultTabSpacing\)/,
  );
});
