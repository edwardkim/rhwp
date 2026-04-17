import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { createPreviewServer } from '../scripts/preview-server.mjs';

test('preview server는 런타임에 생성된 staged hwp 파일을 그대로 서빙한다', async (t) => {
  const rootDir = await mkdtemp(join(tmpdir(), 'rhwp-preview-server-'));
  const openedDir = join(rootDir, '__opened');
  const stagedBytes = Buffer.from([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);

  await mkdir(openedDir, { recursive: true });
  await writeFile(join(rootDir, 'index.html'), '<!doctype html><title>rhwp-studio</title>');
  await writeFile(join(openedDir, 'launch.hwp'), stagedBytes);

  const server = createPreviewServer({ rootDir });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));

  t.after(async () => {
    await new Promise((resolve) => server.close(resolve));
    await rm(rootDir, { recursive: true, force: true });
  });

  const { port } = server.address();
  const response = await fetch(`http://127.0.0.1:${port}/__opened/launch.hwp`);
  const body = Buffer.from(await response.arrayBuffer());

  assert.equal(response.status, 200);
  assert.equal(response.headers.get('content-type'), 'application/octet-stream');
  assert.deepEqual(body, stagedBytes);
});

test('preview server는 앱 경로 요청에 index.html fallback을 반환한다', async (t) => {
  const rootDir = await mkdtemp(join(tmpdir(), 'rhwp-preview-server-'));
  await writeFile(join(rootDir, 'index.html'), '<!doctype html><title>rhwp-studio</title>');

  const server = createPreviewServer({ rootDir });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));

  t.after(async () => {
    await new Promise((resolve) => server.close(resolve));
    await rm(rootDir, { recursive: true, force: true });
  });

  const { port } = server.address();
  const response = await fetch(`http://127.0.0.1:${port}/?url=http://127.0.0.1:7701/__opened/launch.hwp`);
  const body = await response.text();

  assert.equal(response.status, 200);
  assert.match(body, /rhwp-studio/);
});

test('preview server는 save endpoint로 받은 bytes를 원본 파일과 staged 파일에 함께 저장한다', async (t) => {
  const rootDir = await mkdtemp(join(tmpdir(), 'rhwp-preview-server-'));
  const openedDir = join(rootDir, '__opened');
  const originalFilePath = join(rootDir, '..', 'origin.hwp');
  const token = 'launch-token';
  const manifestPath = join(openedDir, `${token}.json`);
  const stagePath = join(openedDir, `${token}.hwp`);

  await mkdir(openedDir, { recursive: true });
  await writeFile(join(rootDir, 'index.html'), '<!doctype html><title>rhwp-studio</title>');
  await writeFile(originalFilePath, Buffer.from('old-origin'));
  await writeFile(stagePath, Buffer.from('old-stage'));
  await writeFile(manifestPath, JSON.stringify({
    token,
    originalFilePath,
    originalFileName: 'origin.hwp',
    stageFileName: `${token}.hwp`,
  }));

  const server = createPreviewServer({ rootDir });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));

  t.after(async () => {
    await new Promise((resolve) => server.close(resolve));
    await rm(rootDir, { recursive: true, force: true });
  });

  const { port } = server.address();
  const nextBytes = Buffer.from([1, 2, 3, 4, 5]);
  const response = await fetch(`http://127.0.0.1:${port}/__rhwp_save/${token}`, {
    method: 'PUT',
    body: nextBytes,
    headers: { 'content-type': 'application/octet-stream' },
  });

  assert.equal(response.status, 200);
  assert.deepEqual(await readFile(originalFilePath), nextBytes);
  assert.deepEqual(await readFile(stagePath), nextBytes);
});

test('preview server health는 세션 식별 정보와 마지막 저장 정보를 노출한다', async (t) => {
  const rootDir = await mkdtemp(join(tmpdir(), 'rhwp-preview-server-'));
  const openedDir = join(rootDir, '__opened');
  const originalFilePath = join(rootDir, '..', 'origin.hwp');
  const token = 'health-token';

  await mkdir(openedDir, { recursive: true });
  await writeFile(join(rootDir, 'index.html'), '<!doctype html><title>rhwp-studio</title>');
  await writeFile(originalFilePath, Buffer.from('before'));
  await writeFile(join(openedDir, `${token}.hwp`), Buffer.from('before'));
  await writeFile(join(openedDir, `${token}.json`), JSON.stringify({
    token,
    originalFilePath,
    originalFileName: 'origin.hwp',
    stageFileName: `${token}.hwp`,
  }));

  const server = createPreviewServer({ rootDir });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));

  t.after(async () => {
    await new Promise((resolve) => server.close(resolve));
    await rm(rootDir, { recursive: true, force: true });
  });

  const { port } = server.address();
  const beforeResponse = await fetch(`http://127.0.0.1:${port}/__rhwp_health`);
  const beforeHealth = await beforeResponse.json();

  assert.equal(beforeResponse.status, 200);
  assert.equal(beforeHealth.ok, true);
  assert.equal(beforeHealth.rootDir, rootDir);
  assert.equal(typeof beforeHealth.sessionId, 'string');
  assert.equal(typeof beforeHealth.pid, 'number');
  assert.equal(beforeHealth.lastSave, null);

  await fetch(`http://127.0.0.1:${port}/__rhwp_save/${token}`, {
    method: 'PUT',
    body: Buffer.from('after'),
    headers: { 'content-type': 'application/octet-stream' },
  });

  const afterResponse = await fetch(`http://127.0.0.1:${port}/__rhwp_health`);
  const afterHealth = await afterResponse.json();

  assert.equal(afterHealth.lastSave.token, token);
  assert.equal(afterHealth.lastSave.originalFilePath, originalFilePath);
  assert.equal(afterHealth.lastSave.fileName, 'origin.hwp');
});

test('preview server는 BOM이 포함된 launch manifest도 읽고 저장한다', async (t) => {
  const rootDir = await mkdtemp(join(tmpdir(), 'rhwp-preview-server-'));
  const openedDir = join(rootDir, '__opened');
  const originalFilePath = join(rootDir, '..', 'origin-bom.hwp');
  const token = 'bom-token';
  const stagePath = join(openedDir, `${token}.hwp`);
  const manifestPath = join(openedDir, `${token}.json`);

  await mkdir(openedDir, { recursive: true });
  await writeFile(join(rootDir, 'index.html'), '<!doctype html><title>rhwp-studio</title>');
  await writeFile(originalFilePath, Buffer.from('old-origin'));
  await writeFile(stagePath, Buffer.from('old-stage'));
  await writeFile(
    manifestPath,
    `\uFEFF${JSON.stringify({
      token,
      originalFilePath,
      originalFileName: 'origin-bom.hwp',
      stageFileName: `${token}.hwp`,
    })}`,
    'utf8',
  );

  const server = createPreviewServer({ rootDir });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));

  t.after(async () => {
    await new Promise((resolve) => server.close(resolve));
    await rm(rootDir, { recursive: true, force: true });
  });

  const { port } = server.address();
  const nextBytes = Buffer.from([7, 8, 9]);
  const response = await fetch(`http://127.0.0.1:${port}/__rhwp_save/${token}`, {
    method: 'PUT',
    body: nextBytes,
    headers: { 'content-type': 'application/octet-stream' },
  });

  assert.equal(response.status, 200);
  assert.deepEqual(await readFile(originalFilePath), nextBytes);
  assert.deepEqual(await readFile(stagePath), nextBytes);
});
