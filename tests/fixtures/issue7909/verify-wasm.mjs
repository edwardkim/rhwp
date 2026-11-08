// 저장소 루트에서 node tests/fixtures/issue7909/verify-wasm.mjs [pkg-directory]
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
const pkg = path.resolve(process.argv[2] ?? 'pkg');
const { default: init, HwpDocument } = await import(pathToFileURL(path.join(pkg, 'rhwp.js')));
await init({ module_or_path: fs.readFileSync(path.join(pkg, 'rhwp_bg.wasm')) });
const results = [];
for (const file of ['repro_footer_picture.hwp', 'control_footer_nopicture.hwp', 'footer-no-picture-keep-auto.hwpx', 'footer-no-picture-keep-auto.hwp']) {
  const doc = new HwpDocument(fs.readFileSync(new URL(file, import.meta.url)));
  try {
    const beforeSvg = doc.renderPageSvg(0);
    const { runs } = JSON.parse(doc.getPageTextLayout(0));
    const display = runs.map(r => r.displayText ?? r.text).join('');
    assert.ok(display.includes('ABCDEF양식 Version 0.0[0000-00-05]'));
    const generated = runs.filter(r => r.displayText?.includes('1'));
    const hasAutoNumber = file !== 'control_footer_nopicture.hwp';
    assert.equal(generated.length > 0, hasAutoNumber, `${file}: 자동번호 유무 계약`);
    for (const run of generated) {
      assert.equal(run.displayCharX.length, [...run.displayText].length + 1);
      assert.ok(Number.isFinite(run.x) && Number.isFinite(run.y));
    }
    assert.equal(doc.renderPageSvg(0), beforeSvg, '조회 전후 SVG 불변');
    const converted = new HwpDocument(doc.exportHwpx());
    try {
      const convertedRuns = JSON.parse(converted.getPageTextLayout(0)).runs;
      assert.equal(convertedRuns.some(r => r.displayText?.includes('1')), hasAutoNumber, `${file}: rhwp 생성 HWPX 자동번호 유무`);
      assert.ok(convertedRuns.map(r => r.displayText ?? r.text).join('').includes('ABCDEF양식 Version 0.0[0000-00-05]'));
    } finally { converted.free(); }
    results.push({ file, generated: generated.map(({ text, charStart, charX, displayText, displayCharX, x, y }) => ({ text, charStart, charX, displayText, displayCharX, x, y })) });
  } finally { doc.free(); }
}
const doc = HwpDocument.createEmpty();
try {
  doc.createBlankDocument();
  doc.setFileName('보고서"초안\\.hwp');
  doc.createHeaderFooter(0, true, 0);
  doc.insertTextInHeaderFooter(0, true, 0, 0, 0, 'AB');
  doc.insertFieldInHf(0, true, 0, 0, 0, 3);
  const selection = doc.getSelectionRectsInHeaderFooter(0, true, 0, 0, 0, 0, 0, 1);
  const cursor = doc.getCursorRectInHeaderFooter(0, true, 0, 0, 1, 0);
  const hit = doc.hitTestInHeaderFooter(0, true, 5000, 0);
  assert.equal(JSON.parse(hit).charOffset, 3);
  const { runs } = JSON.parse(doc.getPageTextLayout(0));
  const field = runs.find(r => r.text === '\u0017');
  assert.equal(field.displayText, '보고서"초안\\.hwp');
  assert.equal(field.charX.length, 2);
  assert.equal(field.displayCharX.length, [...field.displayText].length + 1);
  assert.equal(doc.getSelectionRectsInHeaderFooter(0, true, 0, 0, 0, 0, 0, 1), selection);
  assert.equal(doc.getCursorRectInHeaderFooter(0, true, 0, 0, 1, 0), cursor);
  assert.equal(doc.hitTestInHeaderFooter(0, true, 5000, 0), hit);
  doc.insertTextInHeaderFooter(0, true, 0, 0, 1, 'X');
  assert.equal(JSON.parse(doc.getHeaderFooter(0, true, 0)).text, '\u0017XAB');
  results.push({ test: '선택·커서·삽입 모델 오프셋 보존', result: 'PASS' });
} finally { doc.free(); }
console.log(JSON.stringify({ result: 'PASS', results }, null, 2));
