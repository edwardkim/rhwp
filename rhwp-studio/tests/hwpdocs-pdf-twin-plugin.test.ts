import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, truncateSync, writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Readable } from 'node:stream';
import test from 'node:test';
import { stripVTControlCharacters } from 'node:util';
import {
  BoundedWorkQueue,
  MAX_SOURCE_BYTES,
  ReferenceCacheOwner,
  SourceAdmissionError,
  WorkQueueBusyError,
  buildHwpdocsPdfTwinIndex,
  cacheOwnerName,
  findPdfTwin,
  ghostscriptRasterArgs,
  hwpdocsPdfTwinPlugin,
  parsePdfPageSize,
  PdfToolRunner,
  reclaimDeadCacheRoots,
  registerCacheRoot,
  rejectOperationalFailure,
  serveDocumentErrorLog,
  snapshotPdf,
} from '../vite/hwpdocs-pdf-twin-plugin.ts';

const digest = (path: string): string => createHash('sha256').update(readFileSync(path)).digest('hex');

test('the Vite harness accepts more than one corpus root', () => {
  assert.equal(hwpdocsPdfTwinPlugin({ root: '/tmp/a', additionalRoots: ['/tmp/b'] }).apply, 'serve');
});

test('the PDF harness caches a healthy native command and replaces a vanished one once', async () => {
  const healthy = new Set(['gswin64c', 'gswin32c']);
  const probes: string[] = [];
  const executed: string[] = [];
  const windows = new PdfToolRunner(
    'win32',
    async (command) => {
      executed.push(command);
      if (command === 'gswin64c') {
        healthy.delete(command);
        throw Object.assign(new Error('vanished'), { code: 'ENOENT' });
      }
      return '10.0';
    },
    command => { probes.push(command); return { status: healthy.has(command) ? 0 : 1 }; },
  );
  assert.equal(await windows.run('ghostscript', []), '10.0');
  assert.equal(await windows.run('ghostscript', []), '10.0');
  assert.deepEqual(executed, ['gswin64c', 'gswin32c', 'gswin32c']);
  assert.deepEqual(probes, ['gswin64c', 'gswin64c', 'gswin32c']);

  const linux: string[] = [];
  await new PdfToolRunner(
    'linux', async command => { linux.push(command); return '10.0'; }, () => ({ status: 0 }),
  ).run('ghostscript', []);
  assert.deepEqual(linux, ['gs']);

  const fatal = new PdfToolRunner(
    'win32', async () => { throw new Error('bad PDF'); }, () => ({ status: 0 }),
  );
  await assert.rejects(fatal.run('ghostscript', []), /bad PDF/);
});

test('distinct PDF jobs stay inside the shared running and waiting limits', async () => {
  const queue = new BoundedWorkQueue(2, 2);
  const releases: Array<() => void> = [];
  let active = 0;
  let maximum = 0;
  const jobs = Array.from({ length: 4 }, () => queue.run(() => new Promise<void>(resolve => {
    active += 1;
    maximum = Math.max(maximum, active);
    releases.push(() => { active -= 1; resolve(); });
  })));
  await Promise.resolve();
  assert.equal(active, 2);
  await assert.rejects(queue.run(async () => {}), WorkQueueBusyError);
  while (active > 0 || releases.length > 0) {
    releases.splice(0).forEach(release => release());
    await Promise.resolve();
    await Promise.resolve();
  }
  await Promise.all(jobs);
  assert.equal(maximum, 2);
  const busy = response();
  assert.equal(rejectOperationalFailure(busy as never, new WorkQueueBusyError()), true);
  assert.equal(busy.statusCode, 503);
  assert.equal(busy.headers['Retry-After'], '1');
  const oversized = response();
  assert.equal(rejectOperationalFailure(oversized as never, new SourceAdmissionError()), true);
  assert.equal(oversized.statusCode, 413);
});

test('cache pruning evicts old tokens and widths without touching active work', async (t) => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-reference-cache-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const owner = new ReferenceCacheOwner(root, 2, 1_000, 60_000, 1);
  const artifact = (name: string) => {
    const path = join(root, name);
    mkdirSync(join(path, '..'), { recursive: true });
    writeFileSync(path, name);
    return path;
  };
  const oldSource = artifact('source/old.pdf');
  owner.track(oldSource, 'old', true);
  const oldWidth = artifact('pages/old/0-512.png');
  owner.track(oldWidth, 'old', false);
  const activeSource = artifact('source/active.pdf');
  owner.track(activeSource, 'active', true);
  assert.equal(existsSync(oldSource), false);

  const inFlight = join(root, 'pages/active/0-2048.png');
  await owner.withLease('active', inFlight, async () => {
    artifact('pages/active/0-2048.png');
    owner.track(inFlight, 'active', false);
    assert.equal(existsSync(oldWidth), false);
    assert.equal(existsSync(activeSource), true);
    assert.equal(existsSync(inFlight), true);
    assert.equal(readFileSync(inFlight, 'utf8'), 'pages/active/0-2048.png');
  });
  owner.track(artifact('pages/active/1-2048.png'), 'active', false);
  assert.equal(existsSync(inFlight), false, 'the raster becomes evictable after its consumer');

  const leaseOwner = new ReferenceCacheOwner(join(root, 'lease'), 1, 1_000, 60_000, 0);
  const leasedSource = artifact('lease/source/leased.pdf');
  leaseOwner.track(leasedSource, 'leased', true);
  const release = leaseOwner.acquire('leased', leasedSource);
  const competing = artifact('lease/source/competing.pdf');
  leaseOwner.track(competing, 'competing', true);
  assert.equal(existsSync(leasedSource), true);
  assert.equal(existsSync(competing), false);
  release();

  const lifecycle = new ReferenceCacheOwner(join(root, 'lifecycle'));
  const closeFirst = lifecycle.attachServer();
  const closeSecond = lifecycle.attachServer();
  const held = artifact('lifecycle/source/held.pdf');
  lifecycle.track(held, 'held', true);
  const releaseHeld = lifecycle.acquire('held', held);
  closeFirst();
  closeSecond();
  assert.equal(existsSync(held), true, 'last server close defers cleanup for its active lease');
  releaseHeld();
  assert.equal(existsSync(lifecycle.root), false);

  const operationOwner = new ReferenceCacheOwner(join(root, 'operation'));
  let finishOperation!: () => void;
  const operation = operationOwner.withOperation(async () => {
    artifact('operation/source/.staging');
    await new Promise<void>(resolve => { finishOperation = resolve; });
  });
  await Promise.resolve();
  operationOwner.reset();
  assert.equal(existsSync(operationOwner.root), true);
  finishOperation();
  await operation;
  assert.equal(existsSync(operationOwner.root), false);
  owner.reset();
  assert.equal(existsSync(root), false);
});

test('one cache owner cannot reset or reclaim a live peer root', (t) => {
  const parent = mkdtempSync(join(tmpdir(), 'rhwp-cache-owners-'));
  t.after(() => rmSync(parent, { recursive: true, force: true }));
  const deadOwner = cacheOwnerName(111, 'aaaaaaaaaaaa');
  const liveOwner = cacheOwnerName(222, 'bbbbbbbbbbbb');
  const deadRoot = join(parent, deadOwner);
  const liveRoot = join(parent, liveOwner);
  mkdirSync(deadRoot);
  mkdirSync(liveRoot);
  writeFileSync(join(liveRoot, 'active'), 'active');
  new ReferenceCacheOwner(deadRoot).reset();
  assert.equal(existsSync(join(liveRoot, 'active')), true);

  mkdirSync(deadRoot);
  writeFileSync(join(parent, '.owners'), '0000000111-aaaa');
  registerCacheRoot(parent, liveOwner);
  assert.equal(statSync(join(parent, '.owners')).size, 24, 'partial tail is repaired before append');
  for (let index = 0; index < 127; index++) {
    registerCacheRoot(parent, cacheOwnerName(1_000 + index, 'cccccccccccc'));
  }
  registerCacheRoot(parent, deadOwner);
  reclaimDeadCacheRoots(parent, pid => pid !== 111);
  assert.equal(existsSync(deadRoot), true, 'the first bounded batch stops after 128 live owners');
  reclaimDeadCacheRoots(parent, pid => pid !== 111);
  assert.equal(existsSync(deadRoot), false);
  assert.equal(existsSync(liveRoot), true);
});

function response(): {
  statusCode: number;
  body: string;
  headers: Record<string, string>;
  setHeader(name: string, value: string): void;
  end(chunk?: string): void;
} {
  return {
    statusCode: 200,
    body: '',
    headers: {},
    setHeader(name, value) { this.headers[name] = String(value); },
    end(chunk = '') { this.body = String(chunk); },
  };
}

test('the document-error endpoint prints one typed Vite error', async () => {
  const line = 'paint: [page=3 ratio=0.1 pdfOnly=1 rhwpOnly=2 colorOnly=0 bounds=0,0,1,1] ' +
    'trace=[{"id":1,"parentId":null,"function":"layout_body_picture","args":{"para_index":4,"y_offset":32,' +
    '"result_frame_height":200,"result_y":232},"durationMs":2,"depth":0}]';
  const capability = 'a'.repeat(43);
  const printed: unknown[] = [];
  const send = async (value: string, provided = capability) => {
    const req = Object.assign(Readable.from([value]), {
      method: 'POST',
      headers: { 'x-rhwp-harness-capability': provided },
    });
    const res = response();
    await serveDocumentErrorLog(req as never, res as never, {
      error(message, options) {
        printed.push({ message: stripVTControlCharacters(message), options });
      },
    }, capability);
    return res;
  };

  assert.equal((await send(line, 'b'.repeat(43))).statusCode, 403);
  assert.deepEqual(printed, []);
  assert.equal((await send('[pdf-diff] {"page":3}')).statusCode, 400);
  assert.deepEqual(printed, []);
  const accepted = await send(line);
  assert.equal(accepted.statusCode, 202);
  assert.deepEqual(printed, [{
    message: 'paint: [page=3 ratio=0.1 pdfOnly=1 rhwpOnly=2 colorOnly=0 bounds=0,0,1,1]\n' +
      'trace:\n  #1 layout_body_picture(para_index=4, y_offset=32) ' +
      '=> frame_height=200, y=232 2ms',
    options: { timestamp: true, error: null },
  }]);
});

test('Vite displays the typed document error in red', () => {
  const line = 'paint: [page=3 ratio=0.1 pdfOnly=1 rhwpOnly=2 colorOnly=0 bounds=0,0,1,1] ' +
    'trace=[{"id":1,"parentId":null,"function":"layout_body_picture","args":{"para_index":4,"result_y":232},' +
    '"durationMs":2,"depth":0}]';
  const script = `
    import { Readable } from 'node:stream';
    import { createLogger } from 'vite';
    import { serveDocumentErrorLog } from './vite/hwpdocs-pdf-twin-plugin.ts';
    const capability = 'a'.repeat(43);
    const req = Object.assign(Readable.from([${JSON.stringify(line)}]), {
      method: 'POST', headers: { 'x-rhwp-harness-capability': capability },
    });
    const res = { statusCode: 0, setHeader() {}, end() {} };
    await serveDocumentErrorLog(req, res, createLogger('info', { allowClearScreen: false }), capability);
  `;
  const env = { ...process.env, FORCE_COLOR: '1' };
  delete env.NO_COLOR;
  const result = spawnSync(process.execPath, [
    '--input-type=module',
    '--eval',
    script,
  ], { cwd: new URL('../', import.meta.url), env, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stderr, /\u001b\[31mpaint:[\s\S]*layout_body_picture[\s\S]*\u001b\[39m/);
  assert.match(
    result.stderr.replace(/\u001b\[[0-9;]*m/g, '').trimEnd(),
    /\[vite\] paint: \[page=3[\s\S]*trace:\n  #1 layout_body_picture\(para_index=4\) => y=232 2ms$/,
  );

  const noColorEnv = { ...process.env, NO_COLOR: '1' };
  delete noColorEnv.FORCE_COLOR;
  const noColor = spawnSync(process.execPath, [
    '--input-type=module',
    '--eval',
    script,
  ], { cwd: new URL('../', import.meta.url), env: noColorEnv, encoding: 'utf8' });
  assert.equal(noColor.status, 0, noColor.stderr);
  assert.doesNotMatch(noColor.stderr, /\u001b\[/);
});

test('PDF twin lookup selects the same-directory PDF by document bytes', async (t) => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-twin-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const directory = join(root, 'agency', '2024');
  mkdirSync(directory, { recursive: true });
  const document = join(directory, '기준문서.hwp');
  const pdf = join(directory, '기준문서.pdf');
  writeFileSync(document, 'hwp bytes');
  writeFileSync(pdf, '%PDF twin');

  const result = await findPdfTwin([await buildHwpdocsPdfTwinIndex(root)], {
    fileName: '기준문서.hwp',
    size: statSync(document).size,
    sha256: digest(document),
  });
  assert.equal('status' in result, false);
  if ('status' in result) return;
  assert.equal(result.pdfPath, pdf);
  assert.equal(result.index.root, root);
});

test('PDF twin lookup refuses ambiguous copies', async (t) => {
  const roots = [0, 1].map(() => mkdtempSync(join(tmpdir(), 'rhwp-pdf-twin-')));
  t.after(() => roots.forEach(root => rmSync(root, { recursive: true, force: true })));
  for (const root of roots) {
    writeFileSync(join(root, 'same.hwpx'), 'same bytes');
    writeFileSync(join(root, 'same.pdf'), '%PDF twin');
  }
  const document = join(roots[0], 'same.hwpx');
  assert.deepEqual(await findPdfTwin(await Promise.all(roots.map(buildHwpdocsPdfTwinIndex)), {
    fileName: 'same.hwpx',
    size: statSync(document).size,
    sha256: digest(document),
  }), { status: 'ambiguous' });
});

test('oversized sparse sources are rejected before hashing or publication', async (t) => {
  const root = mkdtempSync(join(tmpdir(), 'rhwp-pdf-oversized-'));
  const cache = mkdtempSync(join(tmpdir(), 'rhwp-pdf-cache-'));
  t.after(() => {
    rmSync(root, { recursive: true, force: true });
    rmSync(cache, { recursive: true, force: true });
  });
  const document = join(root, 'large.hwp');
  const pdf = join(root, 'large.pdf');
  writeFileSync(document, '');
  writeFileSync(pdf, '');
  truncateSync(document, MAX_SOURCE_BYTES + 1);
  truncateSync(pdf, MAX_SOURCE_BYTES + 1);
  const index = await buildHwpdocsPdfTwinIndex(root);
  await assert.rejects(findPdfTwin([index], {
    fileName: 'large.hwp',
    size: MAX_SOURCE_BYTES + 1,
    sha256: '0'.repeat(64),
  }), SourceAdmissionError);
  await assert.rejects(snapshotPdf(pdf, cache), SourceAdmissionError);
  assert.equal(existsSync(join(cache, 'source')), false);
});

test('rotated PDF pages keep visible geometry and bounded Ghostscript arguments', () => {
  assert.deepEqual(parsePdfPageSize(
    'Page 2 size: 500 x 700 pts\nPage 2 MediaBox: 10 20 622 812\nPage 2 rot: -90\n',
    2,
  ), { width: 792, height: 612 });
  assert.deepEqual(parsePdfPageSize('Page 1 MediaBox: 10 20 622 812\n', 1), {
    width: 612, height: 792,
  });
  assert.throws(() => parsePdfPageSize('Page 1 size: 612 x 792 pts\nPage 1 rot: 45\n', 1));
  assert.deepEqual(ghostscriptRasterArgs('/tmp/reference.pdf', 2, {
    width: 1024,
    height: 792,
  }, '/tmp/page.png').slice(-6), [
    '-dFirstPage=2', '-dLastPage=2', '-g1024x792', '-r72',
    '-sOutputFile=/tmp/page.png', '/tmp/reference.pdf',
  ]);
});
