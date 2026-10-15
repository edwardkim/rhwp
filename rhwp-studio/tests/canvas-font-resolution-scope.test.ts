import test from 'node:test';
import assert from 'node:assert/strict';
import { CanvasFontResolutionScope } from '../src/core/canvas-font-resolution-scope.ts';

test('one paint resolves each exact CSS font once without merging size or style', () => {
  const calls: string[] = [];
  const scope = new CanvasFontResolutionScope(font => { calls.push(font); return `${font}, serif`; });
  const regular = '12px "함초롬바탕"';
  const bold = 'bold 12px "함초롬바탕"';
  scope.run(() => {
    assert.equal(scope.resolve(regular), `${regular}, serif`);
    assert.equal(scope.resolve(bold), `${bold}, serif`);
    assert.equal(scope.resolve(regular), `${regular}, serif`);
  });
  assert.deepEqual(calls, [regular, bold]);
});

test('new paint and unscoped calls observe newly published font policy', () => {
  let family = 'first-document-face';
  const scope = new CanvasFontResolutionScope(() => family);
  assert.equal(scope.run(() => scope.resolve('12px input')), family);
  family = 'replacement-document-face';
  assert.equal(scope.run(() => scope.resolve('12px input')), family);
  family = 'new-local-font-face';
  assert.equal(scope.resolve('12px input'), family);
  family = 'font-fallback';
  assert.equal(scope.resolve('12px input'), family);
});

test('nested host or portable paint owns its values and restores the parent on throw', () => {
  let family = 'outer-host';
  const scope = new CanvasFontResolutionScope(() => family);
  scope.run(() => {
    assert.equal(scope.resolve('font'), 'outer-host');
    family = 'portable-child';
    assert.throws(() => scope.run(() => {
      assert.equal(scope.resolve('font'), 'portable-child');
      throw new Error('paint failed');
    }), /paint failed/);
    family = 'outer-host';
    assert.equal(scope.resolve('font'), 'outer-host');
  });
  family = 'next-document';
  assert.equal(scope.resolve('font'), 'next-document');
});

test('a synchronous scope cannot retain resolved values across an await', async () => {
  let family = 'before-await';
  const scope = new CanvasFontResolutionScope(() => family);
  const result = scope.run(async () => {
    assert.equal(scope.resolve('font'), 'before-await');
    await Promise.resolve();
    family = 'after-await';
    return scope.resolve('font');
  });
  assert.equal(await result, 'after-await');
});
