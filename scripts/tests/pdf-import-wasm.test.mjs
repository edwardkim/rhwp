// Product WASM/Worker contract. Requires a fresh optimized root pkg with pdf-import.
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

test('fresh product WASM: public API, active document isolation and real Worker recovery', { timeout: 180_000 }, async () => {
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
  add('/fullwidth-space-source.pdf', join(root, 'samples/issue7957/resource-limit.pdf'), 'application/pdf');
  add('/colour-source.pdf', join(root, 'samples/issue7959/bounded-import-path.pdf'), 'application/pdf');
  add('/icc-budget.pdf', join(root, 'pdf/issue8174/features/icc-path-rgb.pdf'), 'application/pdf');
  add('/xref-budget.pdf', join(root, 'pdf/issue8174/features/recovery-fake-header.pdf'), 'application/pdf');
  files.set('/memory-failure-worker.js', ['text/javascript', `import { PdfImportRequest } from '/pkg/rhwp.js';
const buffered = []; self.onmessage = event => buffered.push(event);
const original = PdfImportRequest.prepare;
PdfImportRequest.prepare = (options) => {
  if (options?.includeTableInference) new WebAssembly.Memory({initial: 1, maximum: 1}).grow(1);
  return original.call(PdfImportRequest, options);
};
await import('/rhwp-shared/pdf/import-worker.js');
const consume = self.onmessage; for (const event of buffered) consume(event);`]);
  // Controlled engine trap in an actual browser Worker, independent of PDF parsing.
  files.set('/trap-failure-worker.js', ['text/javascript', `import { PdfImportRequest } from '/pkg/rhwp.js';
const buffered = []; self.onmessage = event => buffered.push(event);
const original = PdfImportRequest.prepare;
const trap = new WebAssembly.Instance(new WebAssembly.Module(new Uint8Array([
  0,97,115,109,1,0,0,0, 1,4,1,96,0,0, 3,2,1,0, 7,5,1,1,102,0,0, 10,5,1,3,0,0,11
]))).exports.f;
PdfImportRequest.prepare = options => {
  if (options?.includeTableInference) trap();
  return original.call(PdfImportRequest, options);
};
await import('/rhwp-shared/pdf/import-worker.js');
const consume = self.onmessage; for (const event of buffered) consume(event);`]);
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
      const wasm = await import('/pkg/rhwp.js'); const instance = await wasm.default();
      check(wasm.HwpDocument.supportsPdfImport() === true, 'PDF runtime advertises PDF enabled');
      const boundaryCapabilities = JSON.parse(wasm.HwpDocument.getPdfImportRuntimeCapabilities());
      check(boundaryCapabilities.requestTransportReservations === true && boundaryCapabilities.editorReopenRemainingCredit === true
        && boundaryCapabilities.mainThreadPreemption === false && boundaryCapabilities.processMemoryLimitEnforced === false,
        'capabilities distinguish actual copy/reopen boundaries from main-thread preemption and process memory limits');
      const bounded = wasm.PdfImportRequest.importPdf(bytes, { resourceProfile: 'constrained' });
      const limits = JSON.parse(bounded.getTransferLimits());
      check(limits.outputBytes === 32 * 1024 * 1024 && limits.diagnosticsBytes === 8 * 1024 * 1024, 'request uses shared constrained transfer limits');
      const beforeSave = JSON.parse(bounded.inspect());
      const boundedBytes = bounded.exportDocument('hwpx', limits.outputBytes);
      const afterSave = JSON.parse(bounded.inspect());
      check(afterSave.diagnostics.resourceUsage.decodedBytes > beforeSave.diagnostics.resourceUsage.decodedBytes, 'same WASM request accounts first save and diagnostics');
      const unbounded = wasm.HwpDocument.importPdf(bytes, { resourceProfile: 'constrained' });
      const legacyBytes = unbounded.exportHwpx(); unbounded.free();
      check(boundedBytes.length === legacyBytes.length && boundedBytes.every((v, i) => v === legacyBytes[i]), 'bounded first save preserves ordinary HWPX bytes');
      bounded.free();
      const failedRequest = wasm.PdfImportRequest.importPdf(bytes, {});
      throwsCode(() => failedRequest.exportDocument('hwpx', 1), 'SERIALIZE_RESOURCE_LIMIT');
      throwsCode(() => failedRequest.inspect(), 'PDF_IMPORT_SESSION');
      failedRequest.free();
      const recoveredRequest = wasm.PdfImportRequest.importPdf(bytes, {});
      check(recoveredRequest.exportDocument('hwpx', 1024 * 1024).length > 1, 'fresh WASM request recovers after output rejection');
      recoveredRequest.free();

      const tiny = wasm.PdfImportRequest.prepare({ resourceLimits: { backend: { decodedBytes: 8 } } });
      throwsCode(() => tiny.reserveInput(9), 'PDF_RESOURCE_LIMIT');
      throwsCode(() => tiny.open(bytes), 'PDF_IMPORT_SESSION'); tiny.free();
      const svgOwner = wasm.PdfImportRequest.importPdf(bytes, {});
      const ordinarySvgDoc = wasm.HwpDocument.importPdf(bytes, {});
      check(svgOwner.renderPageSvg(0) === ordinarySvgDoc.renderPageSvg(0), 'bounded SVG preserves ordinary product output');
      ordinarySvgDoc.free();
      throwsCode(() => svgOwner.renderPageSvgLimited(0, 16), 'PDF_RESOURCE_LIMIT');
      throwsCode(() => svgOwner.inspect(), 'PDF_IMPORT_SESSION'); svgOwner.free();
      const preparedInput = wasm.PdfImportRequest.prepare({});
      preparedInput.reserveInput(bytes.length); preparedInput.open(bytes);
      const handoff = preparedInput.exportDocument('hwpx', 1024 * 1024);
      const remaining = preparedInput.remainingDecodedBytes();
      check(remaining > 0, 'actual remaining decoded credit follows input, import and export copies');
      throwsCode(() => wasm.PdfEditorReopen.open(handoff, {}, 1024 * 1024, 8, 1000), 'PDF_RESOURCE_LIMIT');
      throwsCode(() => wasm.PdfEditorReopen.open(handoff, {}, 1024 * 1024, remaining, 0), 'PDF_IMPORT_TIMEOUT');
      throwsCode(() => wasm.PdfEditorReopen.open(handoff, {}, 1024 * 1024, remaining, 0.000001), 'PDF_IMPORT_TIMEOUT');
      const editorOwner = wasm.PdfEditorReopen.open(handoff, {}, 1024 * 1024, remaining, 30000);
      const reopenUsage = JSON.parse(editorOwner.resourceUsage());
      check(reopenUsage.decodedBytes > handoff.length * 2 && reopenUsage.decodedBytes <= remaining,
        'reopen reserves WASM input, archive backing, decoded entries and Core from remaining import credit');
      const editor = editorOwner.takeDocument();
      throwsCode(() => editorOwner.takeDocument(), 'PDF_EDITOR_REOPEN'); editorOwner.free(); preparedInput.free();
      check(editor.pageCount() === 3, 'reopen takes one fully editable document');
      editor.convertToEditable(); editor.insertText(0, 0, 0, 'T05');
      for (const saved of [editor.exportHwp(), editor.exportHwpx()]) {
        const reopened = new wasm.HwpDocument(saved);
        const rendered = Array.from({ length: reopened.pageCount() }, (_, page) => {
          const xml = new DOMParser().parseFromString(reopened.renderPageSvg(page), 'image/svg+xml');
          return [...xml.querySelectorAll('text')].map(node => node.textContent).join('');
        });
        check(JSON.stringify(rendered) === JSON.stringify(['T05FIRST', 'SECOND', 'THIRD']),
          'recovered editor preserves the edit, all source text, page ownership and order through both saves');
        reopened.free();
      }
      editor.free();
      const policy = JSON.parse(wasm.HwpDocument.getPdfImportPolicy());
      check(policy.schemaVersion === 1 && Number.isSafeInteger(policy.backend.inputBytes), 'runtime supplies the policy used before transfer');
      check(Number.isSafeInteger(policy.maxDocumentImagePlacements) && policy.maxDocumentImagePlacements >= 99,
        'runtime exposes the bounded cumulative image placement policy');
      check(JSON.stringify(JSON.parse(wasm.HwpDocument.getPdfImportPolicyWithOptions({}))) === JSON.stringify(policy), 'standard options and legacy policy query agree');
      const constrained = JSON.parse(wasm.HwpDocument.getPdfImportPolicyWithOptions({ resourceProfile: 'constrained' }));
      check(constrained.backend.inputBytes === 16 * 1024 * 1024 && constrained.maxSelectedPages === 64, 'constrained profile is resolved by the Rust authority');
      const capabilities = JSON.parse(wasm.HwpDocument.getPdfImportRuntimeCapabilities());
      check(capabilities.preCopyPolicyValidation && capabilities.interruption === 'hostWorkerTermination'
        && capabilities.processMemoryLimitEnforced === false, 'runtime distinguishes pre-copy counters and host Worker interruption from process memory limits');
      for (const resourceProfile of ['unknown', '', null, 1, false, {}]) {
        throwsCode(() => wasm.HwpDocument.importPdf(bytes, { resourceProfile }), 'PDF_RESOURCE_POLICY');
      }
      for (const resourceLimits of [null, [], 1, new Date(), { unknown: 1 }, { maxSelectedPages: 0 },
        { maxSelectedPages: undefined }, { maxSelectedPages: null }, { maxSelectedPages: -1 },
        { maxSelectedPages: 1.5 }, { maxSelectedPages: NaN }, { maxSelectedPages: Infinity },
        { maxSelectedPages: Number.MAX_SAFE_INTEGER + 1 }, { maxSelectedPages: 4294967296 },
        { backend: null }, { backend: [] }, { backend: new Date() }, { backend: { inputBytes: 0 } },
        { backend: { inputBytes: '16' } }, { backend: { inputBytes: true } }, { backend: { inputBytes: 1n } },
        { backend: { inputBytes: constrained.backend.inputBytes + 1 } },
        { backend: { codecWorkUnits: undefined } }, { backend: { unknown: 1 } },
        { [Symbol('hidden')]: 1 }, { backend: { [Symbol('hidden')]: 1 } }]) {
        throwsCode(() => wasm.HwpDocument.importPdf(bytes, { resourceProfile: 'constrained', resourceLimits }), 'PDF_RESOURCE_POLICY');
      }
      const exactOptions = { resourceLimits: { backend: { inputBytes: bytes.length } } };
      const exact = wasm.HwpDocument.importPdf(bytes, exactOptions);
      check(JSON.parse(exact.getPdfImportDiagnostics()).resourcePolicy.backend.inputBytes === bytes.length, 'exact input boundary and effective policy reach the actual backend'); exact.free();
      const visible = new Uint8Array(bytes.length + 8192); visible.set(bytes, 4096);
      const view = wasm.HwpDocument.importPdf(visible.subarray(4096, 4096 + bytes.length), exactOptions);
      check(view.pageCount() === 3, 'direct WASM copies only the visible input view'); view.free();
      const lowerOptions = { resourceLimits: { backend: { inputBytes: bytes.length - 1 } } };
      wasm.HwpDocument.getPdfImportPolicyWithOptions(lowerOptions);
      const oversized = new Uint8Array(8 * 1024 * 1024);
      const memoryBefore = instance.memory.buffer.byteLength;
      throwsCode(() => wasm.HwpDocument.importPdf(oversized, lowerOptions), 'PDF_RESOURCE_LIMIT');
      const memoryAfterRejection = instance.memory.buffer.byteLength;
      check(memoryAfterRejection === memoryBefore, 'rejected input does not grow WASM memory for an automatic argument copy');
      for (const wrong of [null, [], {}, 'PDF', bytes.buffer, new Uint16Array(4)]) {
        throwsCode(() => wasm.HwpDocument.importPdf(wrong, {}), 'PDF_INVALID_INPUT');
      }
      const one = wasm.HwpDocument.importPdf(bytes, { resourceProfile: 'constrained', sourcePages: [1], resourceLimits: { maxSelectedPages: 1 } });
      check(one.pageCount() === 1 && JSON.parse(one.getPdfImportDiagnostics()).resourcePolicy.maxSelectedPages === 1, 'selection uses the resolved constrained policy'); one.free();
      throwsCode(() => wasm.HwpDocument.importPdf(bytes, { sourcePages: [0, 1], resourceLimits: { maxSelectedPages: 1 } }), 'PDF_PAGE_SELECTION');
      const iccBudget = new Uint8Array(await (await fetch('/icc-budget.pdf')).arrayBuffer());
      const xrefBudget = new Uint8Array(await (await fetch('/xref-budget.pdf')).arrayBuffer());
      for (const [name, input] of [['ICC', iccBudget], ['xref', xrefBudget]]) {
        throwsCode(() => wasm.HwpDocument.importPdf(input, { resourceLimits: { backend: { codecWorkUnits: 1 } } }), 'PDF_RESOURCE_LIMIT');
        const normal = wasm.HwpDocument.importPdf(input, { resourceProfile: 'constrained' });
        check(normal.pageCount() === 1, `${name} actual work budget failure does not poison the next request`); normal.free();
      }
      for (const [uri, kind] of [['https://example.com/한글?q=1', 'web'], ['mailto:user@example.com', 'mail'],
        ['#목표', 'documentAnchor'], ['../서류.pdf', 'relativeFile'], ['C:\\docs\\문서.hwp', 'localFile'], ['rhwp-app:open/document', 'registeredApplication']]) {
        const destination = JSON.parse(wasm.HwpDocument.classifyHyperlinkDestination(uri));
        check(destination.uri === uri && destination.kind === kind, `shared WASM destination classification: ${kind}`);
        const doc = wasm.HwpDocument.createEmpty(); doc.createBlankDocument(); doc.insertText(0, 0, 0, 'Link target');
        doc.insertHyperlinkEx(JSON.stringify({ target: { section: 0, para: 0, cellPath: [] }, start: 0, end: 4, uri }));
        for (const [name, saved] of [['hwp', doc.exportHwp()], ['hwpx', doc.exportHwpx()]]) {
          const reopen = new wasm.HwpDocument(saved);
          check(JSON.parse(reopen.getHyperlinkContext('{"section":0,"para":0,"cellPath":[]}')).links[0].uri === uri,
            `${kind}: actual WASM ${name} roundtrip keeps original destination`); reopen.free();
        }
        doc.free();
      }
      for (const uri of ['javascript:alert(1)', 'data:text/html,test', 'https://user:secret@example.com', '../bad\nfile']) {
        check(JSON.parse(wasm.HwpDocument.classifyHyperlinkDestination(uri)).kind === null, 'rejected host destination is not classified as executable');
      }
      const sampleCanvas = document.createElement('canvas'); sampleCanvas.width = sampleCanvas.height = 4;
      sampleCanvas.getContext('2d').fillStyle = '#0000ff'; sampleCanvas.getContext('2d').fillRect(0, 0, 4, 4);
      const rasterCases = [
        ['gradient', '<defs><linearGradient id="g"><stop stop-color="#ff0000"/><stop offset="1" stop-color="#0000ff"/></linearGradient></defs><rect x="10" y="20" width="30" height="30" fill="url(#g)"/>'],
        ['clip-opacity', '<defs><clipPath id="c"><circle cx="25" cy="35" r="14"/></clipPath></defs><rect x="10" y="20" width="30" height="30" fill="#00ff00" opacity="0.5" clip-path="url(#c)"/>'],
        ['internal-use', '<defs><rect id="r" x="10" y="20" width="30" height="30" fill="#0000ff"/></defs><use href="#r"/>'],
        ['restricted-script', '<script>alert(1)</script><rect x="10" y="20" width="30" height="30" fill="#0000ff"/>'],
        ['restricted-before-gradient', '<script>alert(1)</script><defs><linearGradient id="g"><stop stop-color="#ff0000"/><stop offset="1" stop-color="#0000ff"/></linearGradient></defs><rect x="10" y="20" width="30" height="30" fill="url(#g)"/>'],
        ...['png', 'jpeg'].map(format => [`embedded-${format}`, `<image x="10" y="20" width="30" height="30" href="${sampleCanvas.toDataURL(`image/${format}`)}"/>`]),
        ['foreground', '<rect x="10" y="20" width="30" height="30" fill="currentColor"/>'],
      ];
      const svgRaster = [];
      const pixels = async source => {
        const image = new Image(); image.src = source; await image.decode();
        const canvas = document.createElement('canvas'); canvas.width = canvas.height = 30;
        const context = canvas.getContext('2d'); context.drawImage(image, 0, 0);
        return { rgba: context.getImageData(0, 0, 30, 30).data, png: canvas.toDataURL() };
      };
      const rasterOptions = JSON.stringify({ viewBox: [10, 20, 30, 30], viewport: [10, 10, 30, 30], scale: 1, currentColor: '#008000' });
      for (const [name, artwork] of rasterCases) {
        const fragment = `<?xml version="1.0" encoding="UTF-8"?><svg xmlns="http://www.w3.org/2000/svg">${artwork}</svg>`;
        const prepared = wasm.prepareStaticSvgRaster(rasterOptions, fragment);
        check(prepared && prepared.width === 30 && prepared.height === 30 && prepared.byteCost === 3600,
          `${name}: actual WASM consumes the bounded common SVG raster result`);
        const url = URL.createObjectURL(new Blob([prepared.bytes], { type: 'image/png' }));
        const actual = await pixels(url); URL.revokeObjectURL(url);
        // Independent browser SVG renderer; no RHWP output is used to construct the source.
        const safeArtwork = artwork.replace('<script>alert(1)</script>', '');
        const reference = `<svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="10 20 30 30" color="#008000">${safeArtwork}</svg>`;
        const refUrl = URL.createObjectURL(new Blob([reference], { type: 'image/svg+xml' }));
        const expected = await pixels(refUrl); URL.revokeObjectURL(refUrl);
        const center = (15 * 30 + 15) * 4;
        check(actual.rgba[center + 3] > 0 && Math.abs(actual.rgba[center + 3] - expected.rgba[center + 3]) <= 1,
          `${name}: actual SVG center ink/alpha matches independent browser output`);
        if (name === 'foreground') check(actual.rgba[center] === 0 && actual.rgba[center + 1] === 128 && actual.rgba[center + 2] === 0,
          'foreground color reaches the common raster before paint');
        svgRaster.push({ name, source: fragment, referenceSvg: reference, png: actual.png, referencePng: expected.png });
      }
      for (const fragment of ['<svg><use href="https://example.com/a.svg#glyph"/></svg>', '<svg><path d="M1 1broken"/></svg>',
        '<svg><defs><g id="a"><use href="#a"/></g></defs><use href="#a"/></svg>',
        '<svg onload="alert(1)"><rect width="2" height="2"/></svg>']) {
        check(wasm.prepareStaticSvgRaster(rasterOptions, fragment) === undefined, 'unsafe or invalid artwork is rejected before raster replay');
      }
      // Public replay boundary: bounded markup may still describe exponential <use> expansion.
      const expansion = ['<g id="n0"><rect x="10" y="20" width="30" height="30"/></g>'];
      for (let level = 1; level <= 20; level++) expansion.push(`<g id="n${level}"><use href="#n${level - 1}"/><use href="#n${level - 1}"/></g>`);
      check(wasm.prepareStaticSvgRaster(rasterOptions, `<svg><defs>${expansion.join('')}</defs><use href="#n20"/></svg>`) === undefined,
        'exponential internal-use expansion is refused before actual WASM replay');
      check(wasm.prepareStaticSvgRaster(rasterOptions, '<svg><rect x="10" y="20" width="30" height="30" fill="#0000ff"/></svg>')?.byteCost === 3600,
        'a refused expansion does not poison ordinary fresh SVG replay');
      check(wasm.prepareStaticSvgRaster(JSON.stringify({ viewBox: [0, 0, 1, 1], viewport: [0, 0, 100000, 100000], scale: 1, currentColor: '#000000' }),
        '<svg><rect width="1" height="1"/></svg>') === undefined, 'SVG output pixel ceiling rejects before bitmap allocation');
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
      // Independent source p43 and Hancom Print: a literal one-em U+3000
      // stays in the editable cell, with one-em saved advance rather than 197%.
      // This contract uses serialized font metrics, not host font installation.
      const resourceBytes = new Uint8Array(await (await fetch('/fullwidth-space-source.pdf')).arrayBuffer());
      const spaceDoc = wasm.HwpDocument.importPdf(resourceBytes, { sourcePages: [42] });
      async function zipXml(bytes, name) {
        const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        let end = bytes.length - 22;
        while (end >= 0 && view.getUint32(end, true) !== 0x06054b50) end--;
        check(end >= 0, 'saved HWPX has a ZIP directory');
        let cursor = view.getUint32(end + 16, true);
        for (let i = 0; i < view.getUint16(end + 10, true); i++) {
          const n = view.getUint16(cursor + 28, true), extra = view.getUint16(cursor + 30, true), comment = view.getUint16(cursor + 32, true);
          if (new TextDecoder().decode(bytes.subarray(cursor + 46, cursor + 46 + n)) === name) {
            const local = view.getUint32(cursor + 42, true), size = view.getUint32(cursor + 20, true);
            const start = local + 30 + view.getUint16(local + 26, true) + view.getUint16(local + 28, true);
            const raw = bytes.subarray(start, start + size), method = view.getUint16(cursor + 10, true);
            const xml = method === 0 ? raw : new Uint8Array(await new Response(new Blob([raw]).stream().pipeThrough(new DecompressionStream('deflate-raw'))).arrayBuffer());
            return new DOMParser().parseFromString(new TextDecoder().decode(xml), 'application/xml');
          }
          cursor += 46 + n + extra + comment;
        }
        throw Error('missing saved XML ' + name);
      }
      try {
        for (const route of ['source', 'hwp', 'hwpx']) {
          const doc = route === 'source' ? spaceDoc : new wasm.HwpDocument(route === 'hwp' ? spaceDoc.exportHwp() : spaceDoc.exportHwpx());
          try {
            const saved = doc.exportHwpx(), header = await zipXml(saved, 'Contents/header.xml'), section = await zipXml(saved, 'Contents/section0.xml');
            const ownRuns = p => [...p.children].filter(n => n.localName === 'run');
            const runText = r => [...r.children].filter(n => n.localName === 't').map(n => n.textContent).join('');
            const owners = [...section.getElementsByTagNameNS('*', 'p')].filter(p => ownRuns(p).map(runText).join('') === '\u3000행정종합관찰관');
            check(owners.length === 1, `${route}: fullwidth space and label have one editable paragraph owner`);
            const run = ownRuns(owners[0]).find(r => runText(r).startsWith('\u3000'));
            const shape = [...header.getElementsByTagNameNS('*', 'charPr')].find(n => n.getAttribute('id') === run.getAttribute('charPrIDRef'));
            const ratio = Number(shape.getElementsByTagNameNS('*', 'ratio')[0].getAttribute('hangul'));
            check(ratio >= 95 && ratio <= 105, `${route}: one-em source blank is not saved as two em (${ratio}%)`);
            check(doc.pageCount() === 1, `${route}: selected source page count survives`);
          } finally { if (doc !== spaceDoc) doc.free(); }
        }
      } finally { spaceDoc.free(); }
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
        await session.importPdf(bytes);
        await rejectsCode(session.exportDocument('hwpx', { outputBytes: 1 }), 'PDF_RESOURCE_LIMIT');
        await rejectsCode(session.inspect(), 'PDF_NO_DOCUMENT');
        check((await session.importPdf(bytes)).pageCount === 3, 'actual Worker recreated after bounded serialization failure');
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
        await rejectsCode(session.importPdf(iccBudget, { resourceLimits: { backend: { codecWorkUnits: 1 } } }), 'PDF_RESOURCE_LIMIT');
        check((await session.inspect()).pageCount === 3, 'actual ICC budget failure preserves active Worker');
        const progress = [];
        const controlled = await session.importPdf(bytes, { resourceProfile: 'constrained', sourcePages: [1], resourceLimits: { maxSelectedPages: 1 } },
          { timeoutMs: 60_000, onProgress: phase => progress.push(phase) });
        check(controlled.diagnostics.resourcePolicy.maxSelectedPages === 1 && controlled.diagnostics.resourcePolicy.backend.inputBytes === constrained.backend.inputBytes,
          'Worker passes selected profile and explicit lower cap into actual diagnostics');
        check(progress.includes('importing') && progress.includes('ready'), 'Worker progress does not finish the request before result publication');
        let memoryWorkers = 0;
        const memorySession = new PdfImportSession({ workerFactory: () => {
          const url = memoryWorkers++ === 1 ? '/memory-failure-worker.js' : '/rhwp-shared/pdf/import-worker.js';
          return new Worker(url, { type: 'module' });
        } });
        try {
          await memorySession.importPdf(bytes, { sourcePages: [0] }, { timeoutMs: 60_000 });
          await rejectsCode(memorySession.importPdf(bytes, { includeTableInference: true }, { timeoutMs: 60_000 }), 'PDF_WORKER_MEMORY');
          check((await memorySession.inspect()).pageCount === 1, 'actual bounded WebAssembly.Memory.grow failure preserves active document');
          const next = await memorySession.importPdf(bytes, { sourcePages: [2] }, { timeoutMs: 60_000 });
          check(memoryWorkers === 3 && next.diagnostics.pages[0].sourcePage === 2, 'normal request succeeds after memory-failed Worker recreation');
        } finally { memorySession.dispose(); }
        let trapWorkers = 0;
        const trapSession = new PdfImportSession({ workerFactory: () => {
          const url = trapWorkers++ === 1 ? '/trap-failure-worker.js' : '/rhwp-shared/pdf/import-worker.js';
          return new Worker(url, { type: 'module' });
        } });
        try {
          await trapSession.importPdf(bytes, { sourcePages: [0] }, { timeoutMs: 60_000 });
          let trapped;
          try { await trapSession.importPdf(bytes, { includeTableInference: true }, { timeoutMs: 60_000 }); }
          catch (error) { trapped = error; }
          check(trapped?.code === 'PDF_WORKER_FAILED' && trapped.message.includes('unreachable'),
            'an actual WebAssembly unreachable trap retires the failed candidate');
          check((await trapSession.inspect()).pageCount === 1, 'engine trap preserves the prior active Worker');
          const next = await trapSession.importPdf(bytes, { sourcePages: [2] }, { timeoutMs: 60_000 });
          check(trapWorkers === 3 && next.diagnostics.pages[0].sourcePage === 2,
            'a fresh product Worker recovers after an actual engine trap');
        } finally { trapSession.dispose(); }
        const phaseAbort = new AbortController();
        await rejectsCode(session.importPdf(real, {}, { signal: phaseAbort.signal, timeoutMs: 60_000,
          onProgress: phase => { if (phase === 'importing') phaseAbort.abort(); } }), 'PDF_IMPORT_CANCELLED');
        check((await session.inspect()).pageCount === 1, 'cancellation after importing phase preserves active document');
        const recovered = await session.importPdf(bytes, { sourcePages: [1] }, { timeoutMs: 60_000 });
        check(recovered.pageCount === 1 && recovered.diagnostics.pages[0].sourcePage === 1, 'normal input after Worker recreation');
        check(real.byteLength > 0, 'caller PDF buffer not detached');
        return { checked, svgRaster, workers, memoryWorkers, trapWorkers, preCopyMemory: { inputBytes: oversized.byteLength, before: memoryBefore, after: memoryAfterRejection }, terminatedBeforeDispose: terminated, diagnostics: loaded.diagnostics,
          hwpx: Array.from(hwpx), hwp: Array.from(hwp), svg, reopenedSvg };
      } finally { session.dispose(); }
    }, { hasDisabled: !!disabledPkg });
    assert.deepEqual(errors, []);
    mkdirSync(join(evidence, 'svg-raster'), { recursive: true });
    result.svgRaster = result.svgRaster.map(({ name, source, referenceSvg, png, referencePng }) => {
      writeFileSync(join(evidence, 'svg-raster', `${name}.svg`), source);
      writeFileSync(join(evidence, 'svg-raster', `${name}-reference.svg`), referenceSvg);
      const paths = {};
      for (const [suffix, data] of [['wasm', png], ['reference', referencePng]]) {
        const bytes = Buffer.from(data.split(',')[1], 'base64');
        const path = join(evidence, 'svg-raster', `${name}-${suffix}.png`); writeFileSync(path, bytes);
        paths[suffix] = { path, sha256: hash(bytes) };
      }
      return { name, ...paths };
    });
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
    console.log(`${result.checked.length} assertions, ${result.workers + result.memoryWorkers + result.trapWorkers} actual Workers (${result.workers} normal, ${result.memoryWorkers} memory-failure controls, ${result.trapWorkers} engine-trap controls); all 12 pages rendered/saved`);
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
