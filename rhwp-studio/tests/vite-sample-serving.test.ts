// 비시각 HTTP 계약: 실제 Vite가 검증에 쓰는 문서 바이트와 404·경로 경계를 보존한다.
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';
import { createServer } from 'vite';

const studio = resolve(import.meta.dirname, '..');
const repositorySamples = resolve(studio, '..', 'samples');
const publicSamples = resolve(studio, 'public', 'samples');

test('Vite serves full renderer fixtures, repository precedence and missing/path boundaries', async t => {
  const server = await createServer({
    root: studio,
    configFile: resolve(studio, 'vite.config.ts'),
    logLevel: 'silent',
    server: { host: '127.0.0.1', port: 0, watch: null },
  });
  t.after(() => server.close());
  await server.listen();
  const origin = server.resolvedUrls!.local[0];
  const source = readFileSync(resolve(studio, 'e2e/canvas-render-diff.test.mjs'), 'utf8');
  const fixtureList = source.match(/const ALL_FIXTURES = \[([\s\S]*?)\];/);
  assert.ok(fixtureList, 'full renderer fixture inventory must be available');
  const fixtures = [...fixtureList[1].matchAll(/'([^']+)'/g)].map(match => match[1]);
  assert.ok(fixtures.length > 0);
  let repositoryCount = 0, publicCount = 0;
  for (const file of fixtures) {
    const repositoryPath = resolve(repositorySamples, file);
    const repositoryOwned = existsSync(repositoryPath);
    const expected = repositoryOwned ? repositoryPath : resolve(publicSamples, file);
    const response = await fetch(new URL(`samples/${file}`, origin));
    assert.equal(response.status, 200, file);
    assert.deepEqual(Buffer.from(await response.arrayBuffer()), readFileSync(expected), file);
    if (repositoryOwned) {
      repositoryCount++;
      assert.equal(decodeURI(response.headers.get('X-File-Path')!), repositoryPath);
    } else {
      publicCount++;
    }
  }
  assert.ok(repositoryCount > 0 && publicCount > 0, 'exercise both serving roots');
  const missing = await fetch(new URL('samples/__missing_renderer_contract__.hwp', origin));
  assert.equal(missing.status, 404, 'a missing document must not become the SPA HTML');
  await missing.arrayBuffer();
  const traversal = await fetch(`${origin}samples/%2e%2e%2fCargo.toml`);
  assert.equal(traversal.status, 403);
  await traversal.arrayBuffer();
});
