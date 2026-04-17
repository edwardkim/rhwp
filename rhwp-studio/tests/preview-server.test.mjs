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
