// Product WASM/Worker contract. Requires Docker-built root pkg with pdf-import.
// This checks connectivity/content/session ownership, not visual fidelity.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import http from 'node:http';
import { findChrome } from '../rasterize-svg-webfonts.mjs';

const root = resolve(import.meta.dirname, '../..');
const evidence = resolve(process.env.PDF_IMPORT_EVIDENCE || join(root, 'output/pr-review/issue7911/wasm-runtime'));
const pkg = resolve(process.env.PDF_IMPORT_PKG || join(root, 'pkg'));
const disabledPkg = process.env.PDF_IMPORT_DISABLED_PKG;
const hash = bytes => createHash('sha256').update(bytes).digest('hex');

function syntheticPdf(pageCount = 3) {
  const objects = ['<< /Type /Catalog /Pages 2 0 R >>', ''];
  const kids = [];
  for (const text of Array.from({length: pageCount}, (_, i) => ['FIRST', 'SECOND', 'THIRD'][i] ?? `PAGE_${i + 1}`)) {
    const page = objects.length + 1;
    kids.push(`${page} 0 R`);
    const content = `BT /F1 12 Tf 30 140 Td (${text}) Tj ET`;
    objects.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 ${page + 2} 0 R >> >> /Contents ${page + 1} 0 R >>`);
    objects.push(`<< /Length ${content.length} >>\nstream\n${content}\nendstream`);
    objects.push('<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>');
  }
  objects[1] = `<< /Type /Pages /Count ${pageCount} /Kids [${kids.join(' ')}] >>`;
  let pdf = '%PDF-1.4\n';
  const offsets = [];
  objects.forEach((object, i) => { offsets.push(pdf.length); pdf += `${i + 1} 0 obj\n${object}\nendobj\n`; });
  const xref = pdf.length;
  pdf += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`;
  pdf += offsets.map(offset => `${String(offset).padStart(10, '0')} 00000 n \n`).join('');
  pdf += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return Buffer.from(pdf);
}

test('Docker product WASM: public API, active document isolation and real Worker recovery', { timeout: 180_000 }, async () => {
  mkdirSync(evidence, { recursive: true });
  const files = new Map();
  const sources = {};
  function add(url, path, mime) {
    const bytes = readFileSync(path);
    files.set(url, [mime, bytes]);
    sources[url] = { path, sha256: hash(bytes), bytes: bytes.length };
  }
  for (const name of ['rhwp.js', 'rhwp_bg.wasm']) {
    add(`/pkg/${name}`, join(pkg, name), name.endsWith('.wasm') ? 'application/wasm' : 'text/javascript');
    if (disabledPkg) add(`/disabled/${name}`, join(disabledPkg, name), name.endsWith('.wasm') ? 'application/wasm' : 'text/javascript');
  }
  for (const name of ['import-worker.js', 'import-session.js']) {
    add(`/rhwp-shared/pdf/${name}`, join(root, 'rhwp-shared/pdf', name), 'text/javascript');
  }
  add('/real.pdf', join(root, 'pdf/issue5875/nested_table_text_caption-2020.pdf'), 'application/pdf');
  add('/placements.pdf', join(root, 'pdf/issue7966/image-placement-controls.pdf'), 'application/pdf');
  add('/slides.pdf', join(root, 'pdf/issue7966/exceeds-32.pdf'), 'application/pdf');
  add('/scan-buffers.pdf', join(root, 'pdf/issue7971/scan-jpeg-buffer-controls.pdf'), 'application/pdf');
  add('/colour-source.pdf', join(root, 'samples/issue7959/bounded-import-path.pdf'), 'application/pdf');
  add('/control.hwp', join(root, 'samples/hwp_table_test.hwp'), 'application/octet-stream');
  files.set('/synthetic.pdf', ['application/pdf', syntheticPdf()]);
  files.set('/synthetic-many-pages.pdf', ['application/pdf', syntheticPdf(1025)]);
  const server = http.createServer((req, res) => {
    if (req.url === '/') { res.setHeader('Content-Type', 'text/html'); res.end('<!doctype html><meta charset="utf-8">'); return; }
    const file = files.get(req.url);
    if (!file) { res.writeHead(404).end(); return; }
    res.setHeader('Content-Type', file[0]); res.end(file[1]);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  let page;
  const cdp = process.env.CHROME_CDP;
  try {
    const require = createRequire(join(root, 'rhwp-studio/package.json'));
    const puppeteer = require('puppeteer-core');
    browser = cdp
      ? await puppeteer.connect({ browserURL: cdp, defaultViewport: null })
      : await puppeteer.launch({ executablePath: findChrome(), headless: true });
    page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    const result = await page.evaluate(async ({ hasDisabled }) => {
      const checked = [];
      function check(value, name) { if (!value) throw Error(name); checked.push(name); }
      function throwsCode(fn, code) {
        try { fn(); } catch (error) { check(String(error).includes(code), `${code}: ${error}`); return; }
        throw Error(`expected ${code}`);
      }
      async function rejectsCode(promise, code) {
        try { await promise; } catch (error) { check(error.code === code, `${code}: ${error}`); return; }
        throw Error(`expected ${code}`);
      }
      const bytes = new Uint8Array(await (await fetch('/synthetic.pdf')).arrayBuffer());
      const real = new Uint8Array(await (await fetch('/real.pdf')).arrayBuffer());
      const placements = new Uint8Array(await (await fetch('/placements.pdf')).arrayBuffer());
      const slides = new Uint8Array(await (await fetch('/slides.pdf')).arrayBuffer());
      const controlBytes = new Uint8Array(await (await fetch('/control.hwp')).arrayBuffer());
      if (hasDisabled) {
        const disabled = await import('/disabled/rhwp.js'); await disabled.default();
        check(disabled.HwpDocument.supportsPdfImport() === false, 'explicit feature-off runtime advertises PDF disabled');
        throwsCode(() => disabled.HwpDocument.importPdf(bytes, {}), 'PDF_IMPORT_DISABLED');
        throwsCode(() => new disabled.HwpDocument(bytes), 'PDF_IMPORT_DISABLED');
        const control = new disabled.HwpDocument(controlBytes);
        check(control.getPdfImportDiagnostics() === 'null' && control.pageCount() > 0, 'feature-off HWP unaffected');
        control.free();
      }
      const wasm = await import('/pkg/rhwp.js'); await wasm.default();
      check(wasm.HwpDocument.supportsPdfImport() === true, 'PDF runtime advertises PDF enabled');
      const policy = JSON.parse(wasm.HwpDocument.getPdfImportPolicy());
      check(policy.schemaVersion === 1 && Number.isSafeInteger(policy.backend.inputBytes), 'runtime supplies the policy used before transfer');
      check(Number.isSafeInteger(policy.maxDocumentImagePlacements) && policy.maxDocumentImagePlacements >= 99,
        'runtime exposes the bounded cumulative image placement policy');
      const repeated = wasm.HwpDocument.importPdf(placements, {});
      try {
        check(repeated.pageCount() === 3, 'XObject, Form and inline placement pages remain distinct');
        for (let p = 0; p < 3; p++) {
          const xml = new DOMParser().parseFromString(repeated.renderPageSvg(p), 'image/svg+xml');
          check(xml.querySelectorAll('image').length === 33, `page ${p + 1} preserves every repeated image paint`);
        }
      } finally { repeated.free(); }
      const scanBytes = new Uint8Array(await (await fetch('/scan-buffers.pdf')).arrayBuffer());
      const scanImages = svg => [...new DOMParser().parseFromString(svg, 'image/svg+xml').querySelectorAll('image')].map(n => n.getAttribute('href') || n.getAttributeNS('http://www.w3.org/1999/xlink', 'href'));
      const scans = wasm.HwpDocument.importPdf(scanBytes, {});
      try {
        check(scans.pageCount() === 5, 'scan-sized JPEG buffers import all five source pages');
        for (let p = 0; p < 5; p++) {
          const xml = new DOMParser().parseFromString(scans.renderPageSvg(p), 'image/svg+xml');
          check(xml.querySelectorAll('image').length === 1, `scan page ${p + 1} retains its source image`);
        }
        for (const [name, saved] of [['hwp', scans.exportHwp()], ['hwpx', scans.exportHwpx()]]) {
          const reopened = new wasm.HwpDocument(saved);
          try {
            check(reopened.pageCount() === 5, `scan JPEG ${name} save/reopen preserves all pages`);
            for (let p = 0; p < 5; p++) {
              const xml = new DOMParser().parseFromString(reopened.renderPageSvg(p), 'image/svg+xml');
              check(xml.querySelectorAll('image').length === 1 && JSON.stringify(scanImages(reopened.renderPageSvg(p))) === JSON.stringify(scanImages(scans.renderPageSvg(p))), `scan JPEG ${name} page ${p + 1} keeps its image bytes`);
            }
          } finally { reopened.free(); }
        }
      } finally { scans.free(); }
      const colourBytes = new Uint8Array(await (await fetch('/colour-source.pdf')).arrayBuffer());
      const colourDoc = wasm.HwpDocument.importPdf(colourBytes, {});
      try {
        check(colourDoc.pageCount() === 4, 'source text-style PDF imports every page');
        const xml = new DOMParser().parseFromString(colourDoc.renderPageSvg(0), 'image/svg+xml');
        const paints = [...xml.querySelectorAll('text')].flatMap(n =>
          [...n.textContent].map(c => [c, n.getAttribute('fill')?.toLowerCase() || '#000000']));
        function hasWordColour(word, colour) {
          const found = paints.map((_, i) => paints.slice(i, i + word.length))
            .filter(run => run.map(v => v[0]).join('') === word);
          return found.length === 1 && found[0].every(v => v[1] === colour);
        }
        check(hasWordColour('Website:', '#000000'), 'black label retains its own source text style');
        check(hasWordColour('https://ostday.kr', '#0563c1'), 'source URL colour survives contiguous text assembly');
      } finally { colourDoc.free(); }
      const all = new wasm.HwpDocument(bytes);
      check(all.pageCount() === 3 && all.getSourceFormat() === 'pdf', 'constructor imports all PDF pages');
      all.free();
      const selected = wasm.HwpDocument.importPdf(bytes, { sourcePages: [0, 2] });
      const diagnostics = JSON.parse(selected.getPdfImportDiagnostics());
      check(selected.pageCount() === 2, 'explicit selection page count');
      check(JSON.stringify(diagnostics.pages.map(p => [p.sourcePage, p.outputSection])) === '[[0,0],[2,1]]', 'source/output section mapping');
      check(diagnostics.parserAllocationBoundsProven === false && diagnostics.paragraphPolicy && diagnostics.warnings.length, 'incomplete bounds and paragraph policy exposed');
      check(diagnostics.pages.every(p => p.tableInference && !p.tableInference.glyphs), 'default PDF diagnostics omit source glyph ledger');
      const observed = wasm.HwpDocument.importPdf(bytes, { sourcePages: [0], includeTableInference: true });
      try {
        const ledger = JSON.parse(observed.getPdfImportDiagnostics()).pages[0].tableInference;
        check(ledger.ruleVersion === 'tsg-observations-v1' && ledger.glyphs.map(g => g.text).join('') === 'FIRST', 'opt-in table observations preserve source glyphs');
        check(new Set(ledger.glyphs.map(g => g.id)).size === ledger.glyphs.length, 'table observation IDs are unique');
      } finally { observed.free(); }
      for (const flag of ['true', 1, null, []]) {
        throwsCode(() => wasm.HwpDocument.importPdf(bytes, { includeTableInference: flag }), 'PDF_IMPORT_OPTIONS');
      }
      const text = doc => Array.from({ length: doc.pageCount() }, (_, p) => {
        const xml = new DOMParser().parseFromString(doc.renderPageSvg(p), 'image/svg+xml');
        return [...xml.querySelectorAll('text')].map(t => t.textContent).join('');
      });
      check(JSON.stringify(text(selected)) === '["FIRST","THIRD"]', 'selected content and order');
      const manyBytes = new Uint8Array(await (await fetch('/synthetic-many-pages.pdf')).arrayBuffer());
      throwsCode(() => wasm.HwpDocument.importPdf(manyBytes, {}), 'PDF_RESOURCE_LIMIT');
      const late = wasm.HwpDocument.importPdf(manyBytes, { sourcePages: [128, 1024] });
      check(late.pageCount() === 2 && JSON.stringify(text(late)) === '["PAGE_129","PAGE_1025"]', 'source indices beyond the former page ceiling stay distinct from selected count');
      late.free();
      for (const save of ['exportHwp', 'exportHwpx']) {
        const reopen = new wasm.HwpDocument(selected[save]());
        check(JSON.stringify(text(reopen)) === '["FIRST","THIRD"]', `${save} reopen content`);
        check(reopen.getPdfImportDiagnostics() === 'null', `${save} reopen has no invented PDF provenance`);
        reopen.free();
      }
      for (const option of [null, 1, '0,2', [], { sourcePage: [0] }]) {
        throwsCode(() => wasm.HwpDocument.importPdf(bytes, option), 'PDF_IMPORT_OPTIONS');
      }
      for (const pages of [[], [2, 1], [1, 1], [3], [-1], [0.5], [NaN], [Infinity], [4294967296], ['1'], [null], [true], new Uint32Array([0]), [1n], Array(1), Array(policy.maxSelectedPages + 1).fill(0)]) {
        throwsCode(() => wasm.HwpDocument.importPdf(bytes, { sourcePages: pages }), 'PDF_PAGE_SELECTION');
      }
      for (const options of [undefined, {}]) {
        const full = wasm.HwpDocument.importPdf(bytes, options);
        check(full.pageCount() === 3, 'omitted selection imports all'); full.free();
      }
      throwsCode(() => wasm.HwpDocument.importPdf(new Uint8Array([0]), {}), 'PDF_INVALID_INPUT');
      throwsCode(() => wasm.HwpDocument.importPdf(new TextEncoder().encode('%PDF-1.4\n'), {}), 'PDF_PARSE_FAILED');
      throwsCode(() => wasm.HwpDocument.importPdf(new Uint8Array(policy.backend.inputBytes + 1), {}), 'PDF_RESOURCE_LIMIT');
      check(JSON.stringify(text(selected)) === '["FIRST","THIRD"]', 'failed direct import preserves existing document');
      selected.free();
      const { PdfImportSession } = await import('/rhwp-shared/pdf/import-session.js');
      let workers = 0, terminated = 0;
      const session = new PdfImportSession({ workerFactory: () => {
        workers++;
        const worker = new Worker('/rhwp-shared/pdf/import-worker.js', { type: 'module' });
        const original = worker.terminate.bind(worker);
        worker.terminate = () => { terminated++; original(); };
        return worker;
      } });
      try {
        const scanned = await session.importPdf(scanBytes, {}, { timeoutMs: 60_000 });
        check(scanned.pageCount === 5, 'Worker imports every scan-sized JPEG page');
        check(scanImages(await session.renderPageSvg(4)).length === 1, 'Worker preserves the final scan image');
        const painted = await session.importPdf(slides, {}, { timeoutMs: 60_000 });
        check(painted.pageCount === 19 && painted.diagnostics.pages.length === 19, 'Worker imports all 19 image-rich PDF pages');
        const sixth = new DOMParser().parseFromString(await session.renderPageSvg(5), 'image/svg+xml');
        check(sixth.querySelectorAll('image').length >= 35, 'Worker preserves the formerly rejected image placements');
        check((await session.renderPageSvg(6)).includes('<svg'), 'Worker renders the affine-clipped image page');
        const loaded = await session.importPdf(real, {}, { timeoutMs: 60_000 });
        check(loaded.pageCount === 12 && loaded.diagnostics.pages.length === 12, 'Worker full real 12-page PDF');
        const svg = [];
        for (let p = 0; p < 12; p++) svg.push(await session.renderPageSvg(p));
        check(svg.every(s => s.includes('<svg')), 'Worker renders every imported page');
        const hwpx = await session.exportDocument('hwpx');
        const hwp = await session.exportDocument('hwp');
        // Page counts alone do not detect broken BinData references after save.
        const imageSources = svg => [...new DOMParser().parseFromString(svg, 'image/svg+xml').querySelectorAll('image')]
          .map(image => image.getAttribute('href') || image.getAttributeNS('http://www.w3.org/1999/xlink', 'href')).sort();
        const expectedImages = svg.map(imageSources);
        check(expectedImages.some(images => images.length > 0), 'real PDF exercises embedded images');
        const reopenedSvg = {};
        for (const [format, saved] of [['hwpx', hwpx], ['hwp', hwp]]) {
          const reopened = new wasm.HwpDocument(saved);
          try {
            check(reopened.pageCount() === 12, `Worker ${format} save reopens 12 pages`);
            if (format === 'hwp') {
              check(JSON.parse(reopened.getDocumentInfo()).version.startsWith('5.'),
                'Worker HWP save declares HWP5 wire version');
            }
            reopenedSvg[format] = svg.map((_, p) => reopened.renderPageSvg(p));
            for (let p = 0; p < svg.length; p++) {
              check(JSON.stringify(imageSources(reopenedSvg[format][p])) === JSON.stringify(expectedImages[p]),
                `Worker ${format} page ${p + 1} preserves embedded image content and multiplicity`);
            }
          } finally { reopened.free(); }
        }
        await rejectsCode(session.importPdf(real, { sourcePages: [99] }), 'PDF_PAGE_SELECTION');
        const afterFailureWorkers = workers;
        check((await session.inspect()).pageCount === 12, 'ordinary failure preserves active Worker');
        await rejectsCode(session.importPdf(new Uint8Array([0])), 'PDF_INVALID_INPUT');
        await rejectsCode(session.importPdf(new TextEncoder().encode('%PDF-1.4\n')), 'PDF_PARSE_FAILED');
        await rejectsCode(session.importPdf(new Uint8Array(policy.backend.inputBytes + 1)), 'PDF_RESOURCE_LIMIT');
        const valid = await session.importPdf(real, { sourcePages: [0, 6, 7], includeTableInference: true }, { timeoutMs: 60_000 });
        check(workers === afterFailureWorkers && valid.pageCount === 3, 'same Worker failure then normal input');
        check(valid.diagnostics.pages.map(p => p.sourcePage).join() === '0,6,7', 'Worker source selection');
        check(valid.diagnostics.pages.every(p => Array.isArray(p.tableInference.glyphs)
          && Array.isArray(p.tableInference.candidates)), 'Worker transports opt-in table observations and hypotheses');
        await rejectsCode(session.importPdf(real, {}, { timeoutMs: 1 }), 'PDF_IMPORT_TIMEOUT');
        check((await session.inspect()).pageCount === 3, 'timeout preserves active document');
        // Warm the candidate, then abort a real import attempt (no in-WASM phase timing claim).
        await rejectsCode(session.importPdf(real, { sourcePages: [99] }), 'PDF_PAGE_SELECTION');
        const abort = new AbortController();
        const aborted = session.importPdf(real, {}, { signal: abort.signal, timeoutMs: 60_000 });
        setTimeout(() => abort.abort(), 1);
        await rejectsCode(aborted, 'PDF_IMPORT_CANCELLED');
        check((await session.inspect()).pageCount === 3, 'abort preserves active document');
        const recovered = await session.importPdf(bytes, { sourcePages: [1] }, { timeoutMs: 60_000 });
        check(recovered.pageCount === 1 && recovered.diagnostics.pages[0].sourcePage === 1, 'normal input after Worker recreation');
        check(real.byteLength > 0, 'caller PDF buffer not detached');
        return { checked, workers, terminatedBeforeDispose: terminated, diagnostics: loaded.diagnostics,
          hwpx: Array.from(hwpx), hwp: Array.from(hwp), svg, reopenedSvg };
      } finally { session.dispose(); }
    }, { hasDisabled: !!disabledPkg });
    assert.deepEqual(errors, []);
    for (const format of ['hwpx', 'hwp']) {
      const bytes = Buffer.from(result[format]);
      writeFileSync(join(evidence, `worker-all.${format}`), bytes);
      result[format] = { bytes: bytes.length, sha256: hash(bytes) };
    }
    mkdirSync(join(evidence, 'worker-svg'), { recursive: true });
    result.svg = result.svg.map((svg, i) => {
      writeFileSync(join(evidence, 'worker-svg', `${String(i + 1).padStart(3, '0')}.svg`), svg);
      return { page: i + 1, sha256: hash(svg) };
    });
    for (const [format, pages] of Object.entries(result.reopenedSvg)) {
      mkdirSync(join(evidence, `reopened-${format}-svg`), { recursive: true });
      result.reopenedSvg[format] = pages.map((svg, i) => {
        writeFileSync(join(evidence, `reopened-${format}-svg`, `${String(i + 1).padStart(3, '0')}.svg`), svg);
        return { page: i + 1, sha256: hash(svg) };
      });
    }
    writeFileSync(join(evidence, 'browser-result.json'), JSON.stringify({ ...result, sources, browser: await browser.version(), visualJudgment: 'not_evaluated_F4_pending' }, null, 2));
    console.log(`${result.checked.length} assertions, ${result.workers} actual Workers; all 12 pages rendered/saved`);
  } finally {
    await page?.close();
    if (cdp) await browser?.disconnect();
    else await browser?.close();
    const closed = new Promise(resolve => server.close(resolve));
    // CDP clients can leave speculative HTTP connections open after the tab closes.
    // This fixture owns the server, so retire its sockets before awaiting shutdown.
    server.closeAllConnections();
    await closed;
  }
});
