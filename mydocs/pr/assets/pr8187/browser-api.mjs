import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync, writeFileSync, rmSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {pathToFileURL} from 'node:url';

const root = process.cwd();
const work = join(root, 'output/pr-review/pr8187');
const pkg = resolve(process.argv[2] || join(work, 'head-wasm'));
const {createSweepServer, createSweepBrowserProfile} = await import(pathToFileURL(join(root, 'scripts/export-wasm-png-for-sweep.mjs')));
const {findChrome} = await import(pathToFileURL(join(root, 'scripts/rasterize-svg-webfonts.mjs')));
const require = createRequire(join(root, 'rhwp-studio/package.json'));
const server = createSweepServer(new Map([
  ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
  ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
  ['/kps.hwp', ['application/octet-stream', readFileSync(join(root, 'rhwp-studio/public/samples/kps-ai.hwp'))]],
]), new Map());
await new Promise((ok, fail) => { server.once('error', fail); server.listen(0, '127.0.0.1', ok); });
const profile = createSweepBrowserProfile(work, 'logical-edit-api');
let browser;
try {
  browser = await require('puppeteer-core').launch({executablePath: findChrome(), headless: true, userDataDir: profile});
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const result = await page.evaluate(async () => {
    const module = await import('/rhwp.js'); await module.default();
    const checks = [];
    const check = (name, actual, expected) => {
      if (JSON.stringify(actual) !== JSON.stringify(expected)) throw Error(`${name}: ${JSON.stringify(actual)} != ${JSON.stringify(expected)}`);
      checks.push(name);
    };
    const ok = text => { const value = JSON.parse(text); if (!value.ok) throw Error(text); return value; };
    const blank = () => {
      const doc = module.HwpDocument.createEmpty();
      check('blank document section count', JSON.parse(doc.createBlankDocument()).sectionCount, 1);
      return doc;
    };
    const text = (doc, para = 0) => doc.getTextRange(0, para, 0, doc.getParagraphLength(0, para));
    const fields = doc => JSON.parse(doc.getFieldList());
    const roundtrip = (doc, verify) => {
      for (const [kind, bytes] of [['hwp', doc.exportHwp()], ['hwpx', doc.exportHwpx()]]) {
        const saved = new module.HwpDocument(bytes);
        try { verify(saved, kind); } finally { saved.free(); }
      }
    };
    const first = blank();
    try {
      ok(first.insertText(0, 0, 0, 'A🦦BC'));
      ok(first.insertFootnote(0, 0, 1));
      check('note and supplementary scalar share logical axis', first.getLogicalLength(0, 0), 5);
      ok(first.copySelectionLogical(0, 0, 2, 0, 3));
      check('logical copy supplementary scalar', first.getClipboardText(), '🦦');
      check('pure text clipboard classification', first.clipboardIsSingleText(), true);
      const snapshot = first.saveSnapshot();
      ok(first.deleteRangeLogical(0, 0, 2, 0, 3));
      check('logical delete supplementary scalar preserves note', text(first), 'ABC');
      check('note survived delete', first.getLogicalLength(0, 0), 4);
      ok(first.restoreSnapshot(snapshot));
      check('snapshot restores logical content', text(first), 'A🦦BC');
      ok(first.pasteInternalTextLogical(0, 0, 3));
      check('logical internal paste', text(first), 'A🦦🦦BC');
      roundtrip(first, (saved, kind) => {
        check(`${kind} logical text preservation`, text(saved), 'A🦦🦦BC');
        check(`${kind} note ownership`, saved.getLogicalLength(0, 0), 6);
      });
      const before = text(first);
      let rejected = false;
      try { first.deleteRangeLogical(0, 0, 4, 0, 2); } catch { rejected = true; }
      check('inverted selection rejected', rejected, true);
      check('rejected delete keeps content', text(first), before);
    } finally { first.free(); }

    const second = blank();
    try {
      ok(second.insertText(0, 0, 0, '가🦦'));
      ok(second.applyCharFormat(0, 0, 0, 2, '{"bold":true,"textColor":"#cc0000"}'));
      ok(second.copySelectionLogical(0, 0, 0, 0, 2));
      ok(second.insertParagraph(0, 1));
      ok(second.insertText(0, 1, 0, '앞뒤'));
      ok(second.insertClickHereField(0, 1, 1, '안내', '메모', 'target', true));
      ok(second.setFieldValueByName('target', 'ABCD'));
      second.clearActiveField();
      ok(second.pasteInternalTextLogical(0, 1, 2));
      check('internal paste inside field', text(second, 1), '앞A가🦦BCD뒤');
      check('source paragraph unchanged', text(second), '가🦦');
      const expectedFields = fields(second);
      roundtrip(second, (saved, kind) => {
        check(`${kind} styled field text`, text(saved, 1), text(second, 1));
        check(`${kind} field value`, JSON.stringify(fields(saved)).includes('A가🦦BCD'), true);
      });
      ok(second.insertParagraph(0, 2));
      ok(second.insertText(0, 2, 0, '이웃끝'));
      ok(second.pasteHtmlTextLogical(0, 2, 2, '<span style="color:#003399;font-style:italic">가🦦</span>'));
      check('foreign HTML logical paste', text(second, 2), '이웃가🦦끝');
      check('HTML paste preserves internal clipboard', second.getClipboardText(), '가🦦');
      check('HTML paste preserves neighboring field', fields(second), expectedFields);
      const color = JSON.parse(second.getCharPropertiesAt(0, 2, 2));
      check('HTML color transported', color.textColor, '#003399');
      check('HTML italic transported', color.italic, true);
      const canvas = document.createElement('canvas');
      second.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print');
      check('actual Print canvas produced', canvas.width > 0 && canvas.height > 0, true);
    } finally { second.free(); }
    const kps = new module.HwpDocument(new Uint8Array(await (await fetch('/kps.hwp')).arrayBuffer()));
    let savedKpsHwpx;
    try {
      kps.setHangul2024Compat(false);
      check('KPS source page count', kps.pageCount(), 77);
      const label = (doc, index) => {
        const find = (node, predicate) => predicate(node) ? node : (node.children || []).map(child => find(child, predicate)).find(Boolean);
        const table = find(JSON.parse(doc.getPageRenderTree(index)), node => node.type === 'Table' && node.rows === 29 && node.cols === 7);
        if (!table) throw Error('KPS inner table missing');
        const cell = (table.children || []).find(node => node.type === 'Cell' && node.row === 18 && node.col === 0);
        if (!cell) throw Error('KPS label missing');
        const text = node => (node.text || '') + (node.children || []).map(text).join('');
        return text(cell).replace(/\s/g, '');
      };
      check('KPS first fragment source ownership', label(kps, 64), '3.민간소프트웨어');
      check('KPS continuation source ownership', label(kps, 65), '시장침해가능성');
      for (const [kind, bytes] of [['hwp', kps.exportHwp()], ['hwpx', kps.exportHwpx()]]) {
        if (kind === 'hwpx') {
          let data = '';
          for (let start = 0; start < bytes.length; start += 32768) data += String.fromCharCode(...bytes.subarray(start, start + 32768));
          savedKpsHwpx = btoa(data);
        }
        const saved = new module.HwpDocument(bytes);
        try {
          saved.setHangul2024Compat(false);
          check(`KPS ${kind} page count`, saved.pageCount(), 77);
          check(`KPS ${kind} first label`, label(saved, 64), '3.민간소프트웨어');
          check(`KPS ${kind} continuation label`, label(saved, 65), '시장침해가능성');
        } finally { saved.free(); }
      }
    } finally { kps.free(); }
    return {wasmVersion: module.version(), checks, count: checks.length, savedKpsHwpx};
  });
  assert(result.count >= 25);
  writeFileSync(join(work, 'kps-wasm-saved.hwpx'), Buffer.from(result.savedKpsHwpx, 'base64'));
  delete result.savedKpsHwpx;
  writeFileSync(join(work, 'browser-api-results.json'), JSON.stringify({status: 'PASS', pkg, ...result}, null, 2));
  console.log(JSON.stringify(result));
} finally {
  if (browser) await browser.close();
  server.closeAllConnections(); await new Promise(done => server.close(done));
  rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
}
