import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  truncateSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { EventEmitter } from 'node:events';
import { Readable } from 'node:stream';
import test from 'node:test';
import {
  buildHwpdocsPdfTwinIndex,
  admitRasterRevision,
  BoundedAsyncWorkQueue,
  CoalescedAsyncMaintenance,
  dedupeRasterRequest,
  evictSourceCacheEntries,
  findPdfTwinAcrossIndexes as findPdfTwinAcrossIndexesWithSession,
  failStreamResponse,
  ghostscriptRasterArgs,
  inspectRasterCache,
  inspectSourceCache,
  isSourceAdmissionError,
  isWorkQueueSaturatedError,
  isSourcePairCurrent,
  isSourceGenerationCurrent,
  isPdfTokenLeased,
  parsePdfPageRasterRequest,
  parsePdfMediaBox,
  PDF_TWIN_SOURCE_BUDGET,
  pdfCacheProcessRoot,
  pdfPageRasterKey,
  pdfRasterSize,
  pdfRasterToolchainRevision,
  publishPdfSnapshotWithLease,
  reclaimDeadPdfCacheProcesses,
  registerPdfCacheServerLifetime,
  removeUnownedPdfCacheRoots,
  resolvePdfSnapshot,
  runCommand,
  runSourceIo,
  serveDocumentErrorLog,
  selectReferenceCacheEvictions,
  refreshPdfTwinIndexes,
  sourceGenerationKey,
  SourceAdmissionError,
  withPdfTokenLease,
  withPdfTokenEviction,
  withStagingArtifact,
  withPdfPageCountLease,
} from '../vite/hwpdocs-pdf-twin-plugin.ts';
import { isDocumentErrorLine } from '../src/dev/document-error-log.ts';
import { DOCUMENT_ERROR_CAPABILITY_HEADER } from '../src/dev/pdf-twin-contract.ts';

const TEST_TWIN_SESSION = {
  rasterRevision: 'ghostscript-media-rgb-v5-0000000000000000',
  errorLogCapability: 'a'.repeat(43),
};
const findPdfTwin = async (
  index: Parameters<typeof findPdfTwinAcrossIndexesWithSession>[0][number],
  request: Parameters<typeof findPdfTwinAcrossIndexesWithSession>[1],
) => {
  return await findPdfTwinAcrossIndexesWithSession([index], request, TEST_TWIN_SESSION);
};
const findPdfTwinAcrossIndexes = (
  indexes: Parameters<typeof findPdfTwinAcrossIndexesWithSession>[0],
  request: Parameters<typeof findPdfTwinAcrossIndexesWithSession>[1],
) => findPdfTwinAcrossIndexesWithSession(indexes, request, TEST_TWIN_SESSION);

test('document-error grammar rejects malformed or multiline evidence', () => {
  assert.equal(isDocumentErrorLine(
    'line-break: [page=3 target=s0/p4/c0.0.0/g2 at=1 expected=0,37 actual=0,39]',
  ), true);
  assert.equal(isDocumentErrorLine('page-count: [page=384 expected=383 actual=390]'), true);
  assert.equal(isDocumentErrorLine('[pdf-diff] {"pageIndex":0}'), false);
  assert.equal(isDocumentErrorLine('line-break: [page=3]\npaint: [page=4]'), false);
});

test('only the current Studio session can print its document error', async () => {
  const capability = 'a'.repeat(43);
  const line = 'paint: [page=3 ratio=0.1 pdfOnly=1 rhwpOnly=2 colorOnly=0 bounds=0,0,1,1]';
  const printed: Array<{ line: string; options: unknown }> = [];
  const logger = {
    error(value: string, options?: unknown) { printed.push({ line: value, options }); },
  };
  const send = async (provided: string, body = line) => {
    const req = Object.assign(Readable.from([body]), {
      method: 'POST',
      headers: { [DOCUMENT_ERROR_CAPABILITY_HEADER]: provided },
    });
    const res = {
      statusCode: 200,
      headersSent: false,
      destroyed: false,
      body: '',
      setHeader() { return this; },
      end(chunk: string) { this.body = String(chunk); this.headersSent = true; return this; },
    };
    await serveDocumentErrorLog(req as never, res as never, capability, logger);
    return res;
  };
  assert.equal((await send('b'.repeat(43))).statusCode, 403);
  assert.deepEqual(printed, []);
  assert.equal((await send(capability, '[pdf-diff] {"pageIndex":0}')).statusCode, 400);
  assert.deepEqual(printed, []);
  const accepted = await send(capability);
  assert.equal(accepted.statusCode, 202);
  assert.equal(JSON.parse(accepted.body).status, 'accepted');
  assert.deepEqual(printed, [{ line, options: { timestamp: true, error: null } }]);
});

test('Vite renders an accepted document error with its red error tag', () => {
  const line = 'line-break: [page=3 target=s0/p4 at=1 expected=0,7 actual=0]';
  const env = { ...process.env, FORCE_COLOR: '1' };
  delete env.NO_COLOR;
  const result = spawnSync(process.execPath, [
    '--input-type=module',
    '--eval',
    `import { createLogger } from 'vite'; createLogger('info', { allowClearScreen: false }).error(${JSON.stringify(line)}, { timestamp: true, error: null });`,
  ], { cwd: new URL('../', import.meta.url), env, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stderr, /\u001b\[31m/);
  const plain = result.stderr.replace(/\u001b\[[0-9;]*m/g, '').trimEnd();
  assert.match(plain, /\[vite\] line-break: \[page=3 target=s0\/p4 at=1 expected=0,7 actual=0\]$/);
});

function sha256(filePath: string): string {
  return createHash('sha256').update(readFileSync(filePath)).digest('hex');
}

function resultToken(result: { pdfPageUrl: string }): string {
  return result.pdfPageUrl.split('/').at(-2)!;
}

test('PDF twin lookup finds the matching PDF in the document directory', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-twin-'));
  const directory = join(root, 'agency', '2024');
  mkdirSync(directory, { recursive: true });
  const documentPath = join(directory, '기준문서.hwp');
  const pdfPath = join(directory, '기준문서.pdf');
  writeFileSync(documentPath, 'hwp bytes');
  writeFileSync(pdfPath, '%PDF twin');

  const index = await buildHwpdocsPdfTwinIndex(root);
  const result = await findPdfTwin(index, {
    fileName: '기준문서.hwp',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  });
  assert.equal(result.status, 'found');
  if (result.status !== 'found') return;
  assert.equal(result.pdfName, '기준문서.pdf');
  assert.match(
    result.pdfPageUrl,
    /^\/__rhwp_harness\/pdf-page\/[A-Za-z0-9_-]{24}\/ghostscript-media-rgb-v5-[a-f0-9]{16}$/,
  );
  assert.equal(result.pdfPageWidth, 2048);
  assert.equal(result.pdfPageCount, null);
  assert.equal(result.relativeDirectory, join('agency', '2024'));
  assert.equal(result.errorLogCapability, TEST_TWIN_SESSION.errorLogCapability);
});

test('an unrelated bad folder does not hide a valid PDF twin', async () => {
  const validRoot = mkdtempSync(join(tmpdir(), 'rhwp-pdf-valid-root-'));
  const invalidRoot = join(validRoot, 'not-a-directory');
  const documentPath = join(validRoot, 'valid.hwp');
  writeFileSync(documentPath, 'valid document');
  writeFileSync(join(validRoot, 'valid.pdf'), '%PDF valid');
  writeFileSync(invalidRoot, 'file root');
  const failures: string[] = [];
  const indexes = Array.from((await refreshPdfTwinIndexes(
    [validRoot, invalidRoot],
    new Map(),
    root => failures.push(root),
  )).values());
  assert.equal(indexes.length, 1);
  assert.deepEqual(failures, [invalidRoot]);
  const found = await findPdfTwinAcrossIndexes(indexes, {
    fileName: 'valid.hwp',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  });
  assert.equal(found.status, 'found');
});

test('an oversized discovery tree is skipped at its entry budget', async () => {
  const validRoot = mkdtempSync(join(tmpdir(), 'rhwp-pdf-bounded-root-'));
  const oversizedRoot = mkdtempSync(join(tmpdir(), 'rhwp-pdf-oversized-root-'));
  const documentPath = join(validRoot, 'valid.hwpx');
  writeFileSync(documentPath, 'valid document');
  writeFileSync(join(validRoot, 'valid.pdf'), '%PDF valid');
  for (let index = 0; index < 3; index++) {
    writeFileSync(join(oversizedRoot, `irrelevant-${index}.txt`), 'x');
  }
  const failures: string[] = [];
  const indexes = Array.from((await refreshPdfTwinIndexes(
    [validRoot, oversizedRoot],
    new Map(),
    root => failures.push(root),
    { maxEntries: 2 },
  )).values());
  assert.deepEqual(failures, [oversizedRoot]);
  assert.equal(indexes.length, 1);
  const found = await findPdfTwinAcrossIndexes(indexes, {
    fileName: 'valid.hwpx',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  });
  assert.equal(found.status, 'found');
});

test('refreshed PDF twin lookup discovers a root created after startup', async () => {
  const parent = mkdtempSync(join(tmpdir(), 'rhwp-pdf-late-root-'));
  const root = join(parent, 'later');
  let indexes = await refreshPdfTwinIndexes([root], new Map());
  assert.equal(indexes.size, 0);
  mkdirSync(root);
  writeFileSync(join(root, 'late.hwp'), 'late document');
  writeFileSync(join(root, 'late.pdf'), '%PDF late');
  indexes = await refreshPdfTwinIndexes([root], indexes);
  assert.equal(indexes.size, 1);
});

test('a temporary index scan failure keeps the last-known PDF twin available', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-index-recovery-'));
  writeFileSync(join(root, 'stable.hwp'), 'stable document');
  writeFileSync(join(root, 'stable.pdf'), '%PDF stable');
  const initial = await refreshPdfTwinIndexes([root], new Map());
  const stable = initial.get(root);
  const failures: string[] = [];
  const refreshed = await refreshPdfTwinIndexes(
    [root],
    initial,
    failedRoot => failures.push(failedRoot),
    { maxEntries: 1 },
  );
  assert.deepEqual(failures, [root]);
  assert.equal(refreshed.get(root), stable);
});

test('replacing a PDF twin gives the next lookup its new identity', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-twin-refresh-'));
  const documentPath = join(root, 'current.hwpx');
  const pdfPath = join(root, 'current.pdf');
  writeFileSync(documentPath, 'same document');
  writeFileSync(pdfPath, '%PDF first');
  const index = await buildHwpdocsPdfTwinIndex(root);
  const request = {
    fileName: 'current.hwpx',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  };
  const first = await findPdfTwin(index, request);
  assert.equal(first.status, 'found');
  if (first.status !== 'found') return;
  const firstToken = resultToken(first);
  const firstSnapshot = resolvePdfSnapshot(firstToken)!;

  writeFileSync(pdfPath, '%PDF secon');
  const second = await findPdfTwin(index, request);
  assert.equal(second.status, 'found');
  if (second.status !== 'found') return;
  const secondToken = resultToken(second);
  assert.notEqual(secondToken, firstToken);
  assert.equal(readFileSync(firstSnapshot, 'utf8'), '%PDF first');
  assert.equal(readFileSync(resolvePdfSnapshot(secondToken)!, 'utf8'), '%PDF secon');
});

test('reference cache pruning evicts oldest versions and protects the active token', () => {
  const entries = [
    { id: 'v1', owner: 'token-v1', bytes: 60, lastAccess: 1 },
    { id: 'v2', owner: 'token-v2', bytes: 60, lastAccess: 2 },
    { id: 'v3', owner: 'token-v3', bytes: 60, lastAccess: 3 },
  ];
  assert.deepEqual(selectReferenceCacheEvictions(
    entries,
    new Set(['token-v3']),
    { maxEntries: 2, maxBytes: 120, maxAgeMs: 1_000, now: 3 },
  ), ['v1']);
  assert.deepEqual(selectReferenceCacheEvictions(
    entries,
    new Set(['token-v1']),
    { maxEntries: 1, maxBytes: 60, maxAgeMs: 1_000, now: 3 },
  ), ['v2', 'v3'], 'protected active token survives even when it is oldest');
});

test('an active PDF response lease protects an old fifth token', async () => {
  const token = 'active-fifth-token';
  let release!: () => void;
  const serving = withPdfTokenLease(
    token,
    () => new Promise<void>(resolve => { release = resolve; }),
  );
  await Promise.resolve();
  assert.equal(isPdfTokenLeased(token), true);
  const entries = Array.from({ length: 5 }, (_, index) => ({
    id: `v${index}`,
    owner: index === 0 ? token : `token-${index}`,
    bytes: 1,
    lastAccess: index,
  }));
  const evictions = selectReferenceCacheEvictions(
    entries,
    new Set(entries.filter(entry => isPdfTokenLeased(entry.owner)).map(entry => entry.owner)),
    { maxEntries: 4, maxBytes: 4, maxAgeMs: 1_000, now: 5 },
  );
  assert.equal(evictions.includes('v0'), false);
  release();
  await serving;
  assert.equal(isPdfTokenLeased(token), false);
});

test('cache ownership is process-local and survives module reloads', async () => {
  assert.notEqual(
    pdfCacheProcessRoot('/target', 100, 'a'.repeat(24)),
    pdfCacheProcessRoot('/target', 100, 'b'.repeat(24)),
  );

  const reloaded = await import(
    `../vite/hwpdocs-pdf-twin-plugin.ts?cache-reload=${Date.now()}`
  );
  const token = `module-reload-lease-${Date.now()}`;
  let release!: () => void;
  const serving = withPdfTokenLease(
    token,
    () => new Promise<void>(resolve => { release = resolve; }),
  );
  await Promise.resolve();
  assert.equal(reloaded.isPdfTokenLeased(token), true);
  assert.equal(reloaded.withPdfTokenEviction(token, () => {
    assert.fail('a reloaded module must not evict another instance\'s active token');
  }), false);
  release();
  await serving;

  let releaseShared!: () => void;
  let sharedCalls = 0;
  const sharedKey = `module-reload-dedupe-${Date.now()}`;
  const shared = runSourceIo(sharedKey, () => {
    sharedCalls += 1;
    return new Promise<void>(resolve => { releaseShared = resolve; });
  });
  const duplicate = reloaded.runSourceIo(sharedKey, async () => { sharedCalls += 1; });
  assert.equal(duplicate, shared);
  assert.equal(sharedCalls, 0);
  await Promise.resolve();
  assert.equal(sharedCalls, 1);
  releaseShared();
  await shared;

  let rejectShared!: () => void;
  const rejectedKey = `module-reload-source-error-${Date.now()}`;
  const rejected = runSourceIo(rejectedKey, () => new Promise<void>((_resolve, reject) => {
    rejectShared = () => reject(new SourceAdmissionError('source changed'));
  }));
  await Promise.resolve();
  const reloadedRejection = reloaded.runSourceIo(rejectedKey, async () => {});
  assert.equal(reloadedRejection, rejected);
  rejectShared();
  await assert.rejects(reloadedRejection, error => reloaded.isSourceAdmissionError(error));
  assert.equal(isSourceAdmissionError(await rejected.catch(error => error)), true);

  let releaseWork!: () => void;
  const gate = new Promise<void>(resolve => { releaseWork = resolve; });
  const prefix = `module-reload-work-${Date.now()}`;
  const admitted = Array.from({ length: 10 }, (_, index) => (
    (index % 2 === 0 ? runSourceIo : reloaded.runSourceIo)(
      `${prefix}-${index}`,
      () => gate,
    )
  ));
  await Promise.resolve();
  await assert.rejects(reloaded.runSourceIo(
    `${prefix}-saturated`,
    async () => {},
  ), error => reloaded.isWorkQueueSaturatedError(error));
  releaseWork();
  await Promise.all(admitted);
});

test('dead cache owners are recovered and live owners close their exact root', () => {
  const parent = mkdtempSync(join(tmpdir(), 'rhwp-cache-process-'));
  const current = join(parent, `100-${'a'.repeat(24)}`);
  const dead = join(parent, `101-${'b'.repeat(24)}`);
  const live = join(parent, `102-${'c'.repeat(24)}`);
  for (const root of [current, dead, live]) {
    mkdirSync(root);
    writeFileSync(join(root, 'owned'), 'cache');
  }
  assert.equal(reclaimDeadPdfCacheProcesses(
    parent,
    current,
    processId => processId === 100 || processId === 102,
  ), 1);
  assert.equal(existsSync(current), true);
  assert.equal(existsSync(dead), false);
  assert.equal(existsSync(live), true);

  const firstServer = new EventEmitter();
  const secondServer = new EventEmitter();
  registerPdfCacheServerLifetime(firstServer, current);
  registerPdfCacheServerLifetime(secondServer, current);
  firstServer.emit('close');
  assert.equal(existsSync(current), true);
  secondServer.emit('close');
  assert.equal(existsSync(current), false);
});

test('dead-owner recovery isolates one removal failure', () => {
  const parent = mkdtempSync(join(tmpdir(), 'rhwp-cache-recovery-failure-'));
  const failed = join(parent, `201-${'d'.repeat(24)}`);
  const recovered = join(parent, `202-${'e'.repeat(24)}`);
  mkdirSync(failed);
  mkdirSync(recovered);
  const errors: unknown[] = [];
  const removed = reclaimDeadPdfCacheProcesses(
    parent,
    join(parent, `200-${'f'.repeat(24)}`),
    () => false,
    256,
    root => {
      if (root === failed) throw new Error('busy cache owner');
      rmSync(root, { recursive: true, force: true });
    },
    error => errors.push(error),
  );
  assert.equal(removed, 1);
  assert.equal(existsSync(failed), true);
  assert.equal(existsSync(recovered), false);
  assert.equal(errors.length, 1);

  const unusableParent = join(parent, 'not-a-directory');
  writeFileSync(unusableParent, 'occupied');
  assert.equal(reclaimDeadPdfCacheProcesses(
    unusableParent,
    join(unusableParent, 'current'),
    () => false,
    256,
    undefined,
    error => errors.push(error),
  ), 0);
  assert.equal(errors.length, 2);
});

test('shutdown cleanup retains a failed owner for a safe retry', () => {
  const pending = new Set(['/cache/owner']);
  const errors: unknown[] = [];
  assert.equal(removeUnownedPdfCacheRoots(
    pending,
    new Map(),
    () => { throw new Error('busy cache owner'); },
    error => errors.push(error),
  ), 0);
  assert.deepEqual(Array.from(pending), ['/cache/owner']);
  assert.equal(errors.length, 1);
  assert.doesNotThrow(() => removeUnownedPdfCacheRoots(
    pending,
    new Map(),
    () => { throw new Error('still busy'); },
    () => { throw new Error('reporting failed'); },
  ));
  assert.equal(removeUnownedPdfCacheRoots(pending, new Map(), () => {}), 1);
  assert.equal(pending.size, 0);
});

test('server shutdown waits for an active cache lease', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-cache-shutdown-'));
  const server = new EventEmitter();
  let release!: () => void;
  const serving = withPdfTokenLease(
    `shutdown-lease-${Date.now()}`,
    () => new Promise<void>(resolve => { release = resolve; }),
  );
  await Promise.resolve();
  registerPdfCacheServerLifetime(server, root);
  server.emit('close');
  assert.equal(existsSync(root), true);
  release();
  await serving;
  assert.equal(existsSync(root), false);
});

test('a replacement server cancels deferred shutdown cleanup', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-cache-replacement-'));
  const firstServer = new EventEmitter();
  const replacementServer = new EventEmitter();
  let release!: () => void;
  const serving = withPdfTokenLease(
    `replacement-lease-${Date.now()}`,
    () => new Promise<void>(resolve => { release = resolve; }),
  );
  await Promise.resolve();
  registerPdfCacheServerLifetime(firstServer, root);
  firstServer.emit('close');
  registerPdfCacheServerLifetime(replacementServer, root);
  release();
  await serving;
  assert.equal(existsSync(root), true);
  replacementServer.emit('close');
  assert.equal(existsSync(root), false);
});

test('server shutdown waits for source work queued before staging', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-cache-queued-shutdown-'));
  const prefix = `shutdown-source-${Date.now()}`;
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  const active = [
    runSourceIo(`${prefix}-active-0`, () => gate),
    runSourceIo(`${prefix}-active-1`, () => gate),
  ];
  const queued = runSourceIo(`${prefix}-queued`, async () => {
    writeFileSync(join(root, 'published-after-close'), 'cache');
  });
  const server = new EventEmitter();
  registerPdfCacheServerLifetime(server, root);
  server.emit('close');
  assert.equal(existsSync(root), true);
  release();
  await Promise.all([...active, queued]);
  assert.equal(existsSync(root), false);
});

test('source continuations cannot republish after their server closes', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-cache-closed-continuation-'));
  const server = new EventEmitter();
  registerPdfCacheServerLifetime(server, root);
  let releaseHash!: () => void;
  const hash = runSourceIo(
    `closed-hash-${Date.now()}`,
    () => new Promise<void>(resolve => { releaseHash = resolve; }),
    undefined,
    undefined,
    root,
  );
  await Promise.resolve();
  server.emit('close');
  releaseHash();
  await hash;
  assert.equal(existsSync(root), false);
  let published = false;
  await assert.rejects(runSourceIo(
    `closed-snapshot-${Date.now()}`,
    async () => {
      published = true;
      mkdirSync(root);
    },
    undefined,
    undefined,
    root,
  ), /cache owner is closed/);
  assert.equal(published, false);
  assert.equal(existsSync(root), false);
});

test('source snapshot publication holds a token lease before its first await', async () => {
  const token = 'publishing-source-token';
  let finishPublication!: () => void;
  const publishing = publishPdfSnapshotWithLease(
    token,
    () => new Promise<void>(resolve => { finishPublication = resolve; }),
  );
  await Promise.resolve();
  let deleted = false;
  assert.equal(withPdfTokenEviction(token, () => { deleted = true; }), false);
  assert.equal(deleted, false);
  finishPublication();
  await publishing;
});

test('crash-orphaned staging files participate in source cache retention', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-source-staging-retention-'));
  const staging = join(root, '.staging');
  mkdirSync(staging);
  const stale = join(staging, 'orphan.tmp');
  writeFileSync(stale, 'orphaned bytes');
  const inspection = await inspectSourceCache(root);
  assert.deepEqual(inspection.entries.map(entry => entry.id), [stale]);
  assert.deepEqual(selectReferenceCacheEvictions(
    inspection.entries,
    new Set(),
    { maxEntries: 0, maxBytes: 0, maxAgeMs: 0, now: Date.now() + 1 },
  ), [stale]);

  let release!: () => void;
  const active = withStagingArtifact(
    stale,
    () => new Promise<void>(resolve => { release = resolve; }),
  );
  await Promise.resolve();
  assert.deepEqual(selectReferenceCacheEvictions(
    inspection.entries,
    new Set([`staging:${stale}`]),
    { maxEntries: 0, maxBytes: 0, maxAgeMs: 0, now: Date.now() + 1 },
  ), []);
  release();
  await active;
});

test('source cache inspection makes bounded progress across oversized namespaces', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-source-inspection-budget-'));
  const staging = join(root, '.staging');
  mkdirSync(staging);
  for (let index = 0; index < 3; index++) {
    writeFileSync(join(staging, `orphan-${index}.tmp`), 'x');
    const token = `${String.fromCharCode(97 + index).repeat(23)}${index}`;
    const tokenDirectory = join(root, token);
    mkdirSync(tokenDirectory);
    writeFileSync(join(tokenDirectory, 'source.pdf'), 'pdf');
  }
  let sawTruncation = false;
  for (let pass = 0; pass < 10; pass++) {
    const inspection = await inspectSourceCache(root, 1);
    sawTruncation ||= inspection.truncated;
    const selected = selectReferenceCacheEvictions(
      inspection.entries,
      new Set(),
      { maxEntries: 0, maxBytes: 0, maxAgeMs: 0, now: Date.now() + 1 },
    );
    evictSourceCacheEntries(inspection.entries, selected);
    if (!inspection.truncated) break;
  }
  assert.equal(sawTruncation, true);
  assert.deepEqual((await inspectSourceCache(root, 1)).entries, []);
});

test('page-count inspection holds its source token through deferred pdfinfo', async () => {
  const token = 'page-count-source-token';
  let finishInspection!: () => void;
  const inspection = withPdfPageCountLease(
    token,
    () => new Promise<void>(resolve => { finishInspection = resolve; }),
  );
  await Promise.resolve();
  assert.equal(withPdfTokenEviction(token, () => {}), false);
  finishInspection();
  await inspection;
});

test('token eviction and lease acquisition are atomically excluded', async () => {
  let releaseSecond!: () => void;
  const secondLease = withPdfTokenLease(
    'second-token',
    () => new Promise<void>(resolve => { releaseSecond = resolve; }),
  );
  await Promise.resolve();
  let secondDeleted = false;
  assert.equal(withPdfTokenEviction('second-token', () => {
    secondDeleted = true;
  }), false);
  assert.equal(secondDeleted, false);
  releaseSecond();
  await secondLease;
  assert.equal(withPdfTokenEviction('second-token', () => {
    secondDeleted = true;
  }), false, 'recently served tokens remain protected after the response settles');
});

test('raster inspection removes empty namespaces and progresses past truncation', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-raster-inspection-'));
  for (let index = 0; index < 3; index++) {
    mkdirSync(join(root, `empty-${index}`, 'revision'), { recursive: true });
  }
  const raster = join(root, 'live-token', 'revision', '0-256.png');
  mkdirSync(join(root, 'live-token', 'revision'), { recursive: true });
  writeFileSync(raster, 'png');
  let inspection = await inspectRasterCache(root, 2);
  assert.equal(inspection.truncated, true);
  for (let attempt = 0; attempt < 8 && inspection.truncated; attempt++) {
    inspection = await inspectRasterCache(root, 2);
  }
  assert.equal(inspection.truncated, false);
  assert.deepEqual(inspection.entries.map(entry => entry.id), [raster]);
});

test('raster inspection preserves an empty directory owned by active generation', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-raster-active-directory-'));
  const token = 'active-raster-token';
  const revisionDirectory = join(root, token, 'revision');
  mkdirSync(revisionDirectory, { recursive: true });
  let release!: () => void;
  const active = withPdfTokenLease(
    token,
    () => new Promise<void>(resolve => { release = resolve; }),
  );
  await Promise.resolve();
  const leasedInspection = await inspectRasterCache(root, 10);
  assert.equal(leasedInspection.removedDirectories, 0);
  assert.equal(statSync(revisionDirectory).isDirectory(), true);
  release();
  await active;
  for (let index = 0; index < 4; index++) {
    await withPdfTokenLease(`newer-raster-${index}`, async () => {});
  }
  await inspectRasterCache(root, 10);
  assert.equal(existsSync(revisionDirectory), false);
});

test('cache maintenance reruns when a commit arrives during enumeration', async () => {
  let calls = 0;
  let finishFirst!: () => void;
  const maintenance = new CoalescedAsyncMaintenance(async () => {
    calls += 1;
    if (calls === 1) await new Promise<void>(resolve => { finishFirst = resolve; });
  });
  const first = maintenance.request();
  await Promise.resolve();
  const afterCommit = maintenance.request();
  finishFirst();
  await Promise.all([first, afterCommit]);
  assert.equal(calls, 2);
});

test('dirty failing cache passes stay inside one owned maintenance promise', async () => {
  let calls = 0;
  let finishFirst!: () => void;
  const errors: unknown[] = [];
  const maintenance = new CoalescedAsyncMaintenance(async () => {
    calls += 1;
    if (calls === 1) await new Promise<void>(resolve => { finishFirst = resolve; });
    throw new Error(`failure-${calls}`);
  }, error => errors.push(error));
  const first = maintenance.request();
  await Promise.resolve();
  const dirty = maintenance.request();
  finishFirst();
  await assert.doesNotReject(Promise.all([first, dirty]));
  assert.equal(calls, 2);
  assert.deepEqual(errors.map(error => (error as Error).message), ['failure-1', 'failure-2']);
});

test('cache maintenance keeps a request at the completion microtask edge', async () => {
  let calls = 0;
  let finishFirst!: () => void;
  const maintenance = new CoalescedAsyncMaintenance(async () => {
    calls += 1;
    if (calls === 1) await new Promise<void>(resolve => { finishFirst = resolve; });
  });
  const first = maintenance.request();
  await Promise.resolve();
  finishFirst();
  let edgeRequest!: Promise<void>;
  queueMicrotask(() => { edgeRequest = maintenance.request(); });
  await Promise.resolve();
  await Promise.all([first, edgeRequest]);
  assert.equal(calls, 2);
});

test('an issued PDF token stays valid while another lookup refreshes the index', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-token-lifetime-'));
  const documentPath = join(root, 'stable.hwp');
  writeFileSync(documentPath, 'stable document');
  writeFileSync(join(root, 'stable.pdf'), '%PDF stable');
  const index = await buildHwpdocsPdfTwinIndex(root);
  const result = await findPdfTwin(index, {
    fileName: 'stable.hwp',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  });
  assert.equal(result.status, 'found');
  if (result.status !== 'found') return;
  const token = resultToken(result);
  assert.deepEqual(await findPdfTwin(await buildHwpdocsPdfTwinIndex(root), {
    fileName: 'missing.hwp',
    size: 0,
    sha256: '0'.repeat(64),
  }), { status: 'none' });
  assert.equal(readFileSync(resolvePdfSnapshot(token)!, 'utf8'), '%PDF stable');
  const restarted = await import(
    `../vite/hwpdocs-pdf-twin-plugin.ts?restart=${Date.now()}`
  );
  assert.equal(readFileSync(restarted.resolvePdfSnapshot(token)!, 'utf8'), '%PDF stable');
});

test('regenerating the document pair cannot mix an old HWP with a new PDF', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-document-refresh-'));
  const documentPath = join(root, 'current.hwp');
  const pdfPath = join(root, 'current.pdf');
  writeFileSync(documentPath, 'version one');
  writeFileSync(pdfPath, '%PDF one');
  const index = await buildHwpdocsPdfTwinIndex(root);
  const oldRequest = {
    fileName: 'current.hwp',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  };
  assert.equal((await findPdfTwin(index, oldRequest)).status, 'found');

  writeFileSync(documentPath, 'version two');
  writeFileSync(pdfPath, '%PDF two');
  assert.deepEqual(await findPdfTwin(index, oldRequest), { status: 'none' });
});

test('an oversized twin is rejected before streaming or snapshot allocation', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-source-budget-'));
  const documentPath = join(root, 'oversized.hwpx');
  const pdfPath = join(root, 'oversized.pdf');
  writeFileSync(documentPath, 'small document');
  writeFileSync(pdfPath, '');
  truncateSync(pdfPath, PDF_TWIN_SOURCE_BUDGET.pdfBytes + 1);
  assert.equal(statSync(pdfPath).size, PDF_TWIN_SOURCE_BUDGET.pdfBytes + 1);
  assert.deepEqual(await findPdfTwin(await buildHwpdocsPdfTwinIndex(root), {
    fileName: 'oversized.hwpx',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  }), { status: 'none' });
});

test('PDF page raster route is bounded by token, page, and physical width', () => {
  const revision = 'ghostscript-media-rgb-v5-0000000000000000';
  assert.deepEqual(
    parsePdfPageRasterRequest(new URL(
      `http://localhost/__rhwp_harness/pdf-page/abcdefghijklmnopqrstuvwx/${revision}/3.png?width=1600`,
    )),
    {
      token: 'abcdefghijklmnopqrstuvwx',
      rasterRevision: revision,
      pageIndex: 3,
      pixelWidth: 1600,
    },
  );
  assert.equal(parsePdfPageRasterRequest(new URL(
    `http://localhost/__rhwp_harness/pdf-page/../../secret/${revision}/3.png?width=1600`,
  )), null);
  assert.equal(parsePdfPageRasterRequest(new URL(
    `http://localhost/__rhwp_harness/pdf-page/abcdefghijklmnopqrstuvwx/${revision}/3.png?width=9000`,
  )), null);
  assert.equal(pdfPageRasterKey({
    token: 'abcdefghijklmnopqrstuvwx',
    rasterRevision: revision,
    pageIndex: 3,
    pixelWidth: 1600,
  }), `${revision}:abcdefghijklmnopqrstuvwx:3:1600`);
});

test('PDF raster toolchain fingerprints own disjoint cache revisions', () => {
  const first = pdfRasterToolchainRevision({
    path: '/opt/gs-v1:/opt/pdfinfo-v1',
    ghostscriptVersion: '10.0.0',
    pdfinfoVersion: '24.01',
  });
  const second = pdfRasterToolchainRevision({
    path: '/opt/gs-v2:/opt/pdfinfo-v2',
    ghostscriptVersion: '10.1.0',
    pdfinfoVersion: '24.02',
  });
  assert.notEqual(first, second);
  assert.notEqual(
    pdfPageRasterKey({ token: 'abcdefghijklmnopqrstuvwx', rasterRevision: first, pageIndex: 0, pixelWidth: 2048 }),
    pdfPageRasterKey({ token: 'abcdefghijklmnopqrstuvwx', rasterRevision: second, pageIndex: 0, pixelWidth: 2048 }),
  );
  assert.throws(
    () => admitRasterRevision(first, second),
    /toolchain revision is not current/,
  );
  assert.doesNotThrow(() => admitRasterRevision(first, first));
});

test('Ghostscript raster uses opaque RGB and MediaBox-sized fit without CropBox flags', () => {
  const args = ghostscriptRasterArgs('/tmp/input.pdf', 4, 1600, 2200, '/tmp/output.png');
  assert.ok(args.includes('-sDEVICE=png16m'));
  assert.ok(args.includes('-dPDFFitPage'));
  assert.ok(args.includes('-dAutoRotatePages=/PageByPage'));
  assert.ok(args.includes('-g1600x2200'));
  assert.ok(args.includes('-dFirstPage=4'));
  assert.ok(args.includes('-dLastPage=4'));
  assert.ok(!args.some(arg => /CropBox/i.test(arg)));
});

test('PDF page geometry reads MediaBox even when CropBox differs', () => {
  const output = [
    'Page    4 MediaBox:      0.00     0.00   556.00   754.00',
    'Page    4 CropBox:      10.00    10.00   546.00   744.00',
  ].join('\n');
  assert.deepEqual(parsePdfMediaBox(output, 4), { width: 556, height: 754 });
});

test('PDF page geometry follows quarter-turn page rotation', () => {
  const output = [
    'Page    2 rot:        90',
    'Page    2 MediaBox:      0.00     0.00   612.00   792.00',
  ].join('\n');
  assert.deepEqual(parsePdfMediaBox(output, 2), { width: 792, height: 612 });
  assert.deepEqual(pdfRasterSize(parsePdfMediaBox(output, 2), 2048), {
    width: 2048,
    height: 1583,
  });
});

test('PDF-derived raster height and total pixels are bounded before Ghostscript', () => {
  assert.deepEqual(pdfRasterSize({ width: 556, height: 754 }, 2048), {
    width: 2048,
    height: 2777,
  });
  assert.throws(
    () => pdfRasterSize({ width: 1, height: 1_000_000_000 }, 2048),
    /exceed the harness budget/,
  );
  assert.throws(
    () => pdfRasterSize({ width: 2, height: 3 }, 4096),
    /exceed the harness budget/,
  );
});

test('same raster key shares one in-flight render and releases the boundary after settlement', async () => {
  const key = `dedupe-${Date.now()}-${Math.random()}`;
  let calls = 0;
  let finish!: (value: string) => void;
  const first = dedupeRasterRequest(key, () => {
    calls += 1;
    return new Promise(resolve => { finish = resolve; });
  });
  const second = dedupeRasterRequest(key, async () => {
    calls += 1;
    return 'duplicate';
  });
  assert.equal(first, second);
  assert.equal(calls, 1);
  finish('rendered');
  assert.equal(await first, 'rendered');

  assert.equal(await dedupeRasterRequest(key, async () => {
    calls += 1;
    return 'after-settlement';
  }), 'after-settlement');
  assert.equal(calls, 2);
});

test('stream failures produce a bounded response', () => {
  let responseBody = '';
  const response = {
    destroyed: false,
    headersSent: false,
    statusCode: 0,
    setHeader: () => {},
    end: (body: string) => { responseBody = body; },
  } as unknown as import('node:http').ServerResponse;
  failStreamResponse(response, 'test stream', new Error('source missing'));
  assert.equal(response.statusCode, 500);
  assert.equal(responseBody, '{"status":"error"}');
});

test('raster subprocesses time out and terminate', async () => {
  await assert.rejects(runCommand(process.execPath, [
    '-e',
    'process.on("SIGTERM",()=>{});setInterval(()=>{},1000)',
  ], { timeoutMs: 20, killGraceMs: 20 }), /timed out after 20ms/);
});

test('distinct PDF subprocess work has bounded concurrency and admission', async () => {
  const queue = new BoundedAsyncWorkQueue(1, 1);
  let release!: () => void;
  const first = queue.run(() => new Promise<void>(resolve => { release = resolve; }));
  const second = queue.run(async () => 'second');
  await assert.rejects(queue.run(async () => 'third'), /queue is saturated/);
  release();
  await first;
  assert.equal(await second, 'second');
});

test('identical source I/O shares one in-flight operation and releases it', async () => {
  const key = `source-${Date.now()}-${Math.random()}`;
  let calls = 0;
  let release!: (value: string) => void;
  const first = runSourceIo(key, () => {
    calls += 1;
    return new Promise(resolve => { release = resolve; });
  });
  const second = runSourceIo(key, async () => {
    calls += 1;
    return 'duplicate';
  });
  assert.equal(first, second);
  await Promise.resolve();
  assert.equal(calls, 1);
  release('shared');
  assert.equal(await first, 'shared');
  assert.equal(await runSourceIo(key, async () => {
    calls += 1;
    return 'after';
  }), 'after');
  assert.equal(calls, 2);
});

test('source I/O admission rejection releases a pre-opened lease', async () => {
  const queue = new BoundedAsyncWorkQueue(1, 0);
  let release!: () => void;
  const active = runSourceIo(
    `active-${Date.now()}-${Math.random()}`,
    () => new Promise<void>(resolve => { release = resolve; }),
    undefined,
    queue,
  );
  await Promise.resolve();
  let releasedUnused = 0;
  await assert.rejects(runSourceIo(
    `rejected-${Date.now()}-${Math.random()}`,
    async () => {},
    () => { releasedUnused += 1; },
    queue,
  ), /queue is saturated/);
  assert.equal(releasedUnused, 1);
  release();
  await active;
});

test('a saturated source queue does not turn a valid twin into none', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-source-busy-'));
  const documentPath = join(root, 'busy.hwp');
  writeFileSync(documentPath, 'busy document');
  writeFileSync(join(root, 'busy.pdf'), '%PDF busy');
  const index = await buildHwpdocsPdfTwinIndex(root);
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  const blockers = Array.from({ length: 10 }, (_, index) => runSourceIo(
    `busy-${Date.now()}-${Math.random()}-${index}`,
    () => gate,
  ));
  await assert.rejects(findPdfTwin(index, {
    fileName: 'busy.hwp',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  }), error => isWorkQueueSaturatedError(error));
  release();
  await Promise.all(blockers);
});

test('replacing a path while old source I/O is pending gets a new generation key', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'rhwp-source-generation-'));
  const filePath = join(directory, 'changing.pdf');
  writeFileSync(filePath, 'old');
  const oldKey = sourceGenerationKey(filePath, statSync(filePath));
  let release!: () => void;
  const oldWork = runSourceIo(oldKey, () => new Promise<void>(resolve => { release = resolve; }));
  await Promise.resolve();
  writeFileSync(filePath, 'new generation');
  const newKey = sourceGenerationKey(filePath, statSync(filePath));
  assert.notEqual(newKey, oldKey);
  assert.equal(await isSourceGenerationCurrent(filePath, oldKey), false);
  assert.equal(await isSourceGenerationCurrent(filePath, newKey), true);
  assert.equal(await runSourceIo(newKey, async () => 'new'), 'new');
  release();
  await oldWork;
});

test('ambiguity cardinality drops a pair whose PDF generation changed', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'rhwp-pair-generation-'));
  const documentPath = join(directory, 'pair.hwp');
  const pdfPath = join(directory, 'pair.pdf');
  writeFileSync(documentPath, 'document');
  writeFileSync(pdfPath, '%PDF old');
  const admitted = {
    documentPath,
    documentGeneration: sourceGenerationKey(documentPath, statSync(documentPath)),
    pdfPath,
    pdfGeneration: sourceGenerationKey(pdfPath, statSync(pdfPath)),
  };
  assert.equal(await isSourcePairCurrent(admitted), true);
  writeFileSync(pdfPath, '%PDF replaced generation');
  assert.equal(await isSourcePairCurrent(admitted), false);
});

test('PDF twin lookup rejects a same-named PDF from another directory', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-twin-'));
  const documentDirectory = join(root, 'documents');
  const pdfDirectory = join(root, 'pdfs');
  mkdirSync(documentDirectory);
  mkdirSync(pdfDirectory);
  const documentPath = join(documentDirectory, 'orphan.hwpx');
  writeFileSync(documentPath, 'hwpx bytes');
  writeFileSync(join(pdfDirectory, 'orphan.pdf'), '%PDF wrong directory');

  const index = await buildHwpdocsPdfTwinIndex(root);
  assert.deepEqual(await findPdfTwin(index, {
    fileName: 'orphan.hwpx',
    size: statSize(documentPath),
    sha256: sha256(documentPath),
  }), { status: 'none' });
});

test('PDF twin lookup reports ambiguity when two roots contain the same twin', async () => {
  const roots = [
    mkdtempSync(join(tmpdir(), 'rhwp-pdf-root-a-')),
    mkdtempSync(join(tmpdir(), 'rhwp-pdf-root-b-')),
  ];
  for (const root of roots) {
    writeFileSync(join(root, 'same.hwp'), 'same document');
    writeFileSync(join(root, 'same.pdf'), '%PDF same-dir twin');
  }
  const documentPath = join(roots[0], 'same.hwp');
  const indexes = await Promise.all(roots.map(root => buildHwpdocsPdfTwinIndex(root)));
  assert.deepEqual(await findPdfTwinAcrossIndexes(
    indexes,
    {
      fileName: 'same.hwp',
      size: statSize(documentPath),
      sha256: sha256(documentPath),
    },
  ), { status: 'ambiguous' });
});

test('a same-sized namesake PDF is rejected when its document bytes differ', async () => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-twin-'));
  const directory = join(root, 'agency');
  mkdirSync(directory);
  const documentPath = join(directory, 'collision.hwp');
  writeFileSync(documentPath, 'dataset');
  writeFileSync(join(directory, 'collision.pdf'), '%PDF twin');

  const index = await buildHwpdocsPdfTwinIndex(root);
  assert.deepEqual(await findPdfTwin(index, {
    fileName: 'collision.hwp',
    size: statSize(documentPath),
    sha256: createHash('sha256').update('outside').digest('hex'),
  }), { status: 'none' });
});

function statSize(filePath: string): number {
  return readFileSync(filePath).byteLength;
}
