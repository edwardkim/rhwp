import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('빈 viewer 클릭은 hitTest 전에 문서 로드 여부를 확인한다', () => {
  const mouse = source('src/engine/input-handler-mouse.ts');
  const onClickStart = mouse.indexOf('export function onClick');
  const onClickEnd = mouse.indexOf('export function onDblClick', onClickStart);
  assert.notEqual(onClickStart, -1, 'onClick function not found');
  assert.notEqual(onClickEnd, -1, 'onClick function end not found');

  const onClick = mouse.slice(onClickStart, onClickEnd);
  const pageCountGuard = onClick.indexOf('this.wasm?.pageCount');
  const firstHitTest = onClick.indexOf('this.wasm.hitTest');

  assert.notEqual(pageCountGuard, -1, 'pageCount guard not found');
  assert.notEqual(firstHitTest, -1, 'hitTest call not found');
  assert.ok(pageCountGuard < firstHitTest, 'empty viewer guard must run before hitTest');
});
