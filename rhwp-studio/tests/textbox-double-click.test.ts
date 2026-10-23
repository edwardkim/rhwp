import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// The real mouse entry handler and CursorState execute in an isolated Node
// process, with only the browser and WASM boundaries substituted.
const run = spawnSync(process.execPath, [
  '--experimental-transform-types',
  fileURLToPath(new URL('./support/textbox-double-click-driver.mjs', import.meta.url)),
], { encoding: 'utf8' });
assert.equal(run.status, 0, `${run.stdout}\n${run.stderr}`);
const rows = JSON.parse(run.stdout);

for (const name of ['boolean-page15', 'rotated-page4']) {
  test(`#7998 double click preserves the hit owner, offset and page: ${name}`, () => {
    const row = rows.find((r: { name: string }) => r.name === name);
    assert.equal(row.position.parentParaIndex, 0);
    assert.equal(row.position.controlIndex, 275);
    assert.equal(row.position.charOffset, name === 'boolean-page15' ? 4 : 2);
    assert.equal(row.rect.pageIndex, name === 'boolean-page15' ? 14 : 3);
    assert.equal(row.updates, 1);
    if (name === 'rotated-page4') assert.equal(row.rect.rotation, -45);
  });
}
for (const name of ['repeated-master', 'different-owner']) {
  test(`#7998 double click cannot enter another page or object: ${name}`, () => {
    const row = rows.find((r: { name: string }) => r.name === name);
    assert.equal(row.updates, 0);
    assert.equal(row.position.parentParaIndex, undefined);
  });
}

test('#7998 a TextBox handle click preserves geometry and undo history', () => {
  const row = rows.find((r: { name: string }) => r.name === 'handle-click');
  assert.deepEqual(row.writes, []);
  assert.deepEqual(row.operations, []);
  assert.deepEqual(row.events, []);
});
test('#7998 a real handle drag still resizes and records undo', () => {
  const row = rows.find((r: { name: string }) => r.name === 'real-resize');
  assert.ok(row.writes.length > 0);
  assert.ok(row.writes.at(-1)[3].width > 1633);
  assert.deepEqual(row.operations, ['record']);
  assert.ok(row.events.includes('document-changed'));
});
