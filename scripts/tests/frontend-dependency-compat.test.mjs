import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = process.env.RHWP_DEPENDENCY_TEST_ROOT
  || fileURLToPath(new URL('../../', import.meta.url));
const studio = createRequire(resolve(root, 'rhwp-studio/package.json'));
const vscode = createRequire(resolve(root, 'rhwp-vscode/package.json'));
const metrics = createRequire(resolve(root, 'scripts/frontend-metrics/package.json'));

for (const [name, require] of [['studio', studio], ['metrics', metrics]]) {
  test(`${name}: brace-expansion named API and bounded expansion`, () => {
    assert.equal(require('brace-expansion/package.json').version, '5.0.12');
    const { expand } = require('brace-expansion');
    for (const [pattern, expected] of [
      ['a{b,c}d', ['abd', 'acd']], ['{a,{b,c}}', ['a', 'b', 'c']],
      ['{1..3}', ['1', '2', '3']], ['{c..a}', ['c', 'b', 'a']],
      ['x\\{a,b\\}', ['x{a,b}']], ['', []], ['x{a,b', ['x{a,b']],
    ]) assert.deepEqual(expand(pattern), expected, pattern);
    assert.deepEqual(expand('{1..10}', { max: 3 }), ['1', '2', '3']);
    assert.ok(expand('abcdefgh{a,b}', { maxLength: 5 }).join('').length <= 5);
  });
}

test('Jake consumes the upstream FileList migration without changing file selection', () => {
  const jakeRequire = createRequire(studio.resolve('jake/package.json'));
  assert.equal(jakeRequire('filelist/package.json').version, '2.0.2');
  const { FileList } = jakeRequire('filelist');
  const directory = mkdtempSync(join(tmpdir(), 'rhwp-filelist-'));
  try {
    for (const name of ['first.js', 'second.ts', 'skip.ts', 'notes.md']) {
      writeFileSync(join(directory, name), '');
    }
    const files = new FileList();
    files.include(join(directory, '*.{js,ts}'));
    files.exclude(join(directory, 'skip.*'));
    assert.deepEqual(files.toArray().map(file => basename(file)).sort(), ['first.js', 'second.ts']);
    const jake = studio('jake');
    assert.equal(jake.FileList, FileList);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

for (const [name, require] of [['studio', studio], ['vscode', vscode]]) {
  test(`${name}: Ajv validates relative, fragment and absolute references with fast-uri 4`, () => {
    const Ajv = require('ajv');
    const ajvRequire = createRequire(require.resolve('ajv/package.json'));
    assert.equal(ajvRequire('fast-uri/package.json').version, '4.2.2');
    const ajv = new Ajv();
    ajv.addSchema({ $id: 'https://schemas.example/parts.json', definitions: { value: { type: 'integer' } } });
    const validate = ajv.compile({
      $id: 'https://schemas.example/root.json', type: 'object',
      properties: { value: { $ref: 'parts.json#/definitions/value' } }, required: ['value'],
    });
    assert.equal(validate({ value: 5 }), true);
    assert.equal(validate({ value: 'five' }), false);
    assert.throws(() => ajv.compile({ type: 'not-a-json-schema-type' }), /schema is invalid/);
    assert.throws(() => new Ajv().compile({
      $id: 'https://schemas.example/root.json', $ref: 'http://bad\\host/schema',
    }), /backslash|authority/i);
  });

  test(`${name}: URI adapter preserves URI semantics and owns its codegen marker`, () => {
    const uri = require('ajv/dist/runtime/uri').default;
    const ajvRequire = createRequire(require.resolve('ajv/package.json'));
    const fastUri = ajvRequire('fast-uri');
    assert.equal(uri.code, 'require("ajv/dist/runtime/uri").default');
    assert.equal(Object.hasOwn(fastUri, 'code'), false);
    assert.equal(uri.resolve('https://example.test/a/b', '../c#part'), 'https://example.test/c#part');
    const absolute = 'https://example.test:8443/a%20b?q=hello#part';
    assert.equal(uri.serialize(uri.parse(absolute)), absolute);
    assert.equal(uri.resolve('https://example.test/base', '//other.test/x'), 'https://other.test/x');
    assert.throws(() => uri.resolve('https://example.test/', '//bad\\host/'), /backslash|authority/i);
    assert.throws(() => uri.resolve('https://example.test/', 'http://bad:invalid/'));
  });
}
