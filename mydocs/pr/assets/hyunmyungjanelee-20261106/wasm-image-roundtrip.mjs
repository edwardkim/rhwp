// 실제 WASM 공개 저장/재열기의 그림 데이터·크기·crop·alpha 계약 검사.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');
const pkg = path.resolve(process.argv[2]);
const output = path.resolve(process.argv[3]);
fs.mkdirSync(output, { recursive: true });
const { default: init, HwpDocument } = await import(pathToFileURL(path.join(pkg, 'rhwp.js')));
await init({ module_or_path: fs.readFileSync(path.join(pkg, 'rhwp_bg.wasm')) });
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function pictures(doc) {
  const result = [];
  function walk(node) {
    if (!node || typeof node !== 'object') return;
    if (node.type === 'image') {
      result.push({ mime: node.mime, dataSha256: sha(Buffer.from(node.base64, 'base64')), originalSizeHu: node.originalSizeHu, crop: node.crop, opacity: node.opacity ?? 1 });
      return;
    }
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) value.forEach(walk);
      else if (value && typeof value === 'object') walk(value);
    }
  }
  for (let p = 0; p < doc.pageCount(); p++) walk(doc.getPageLayerTreeValueWithProfile(p, 'print').root);
  return result;
}
const records = [];
for (const [issue, name] of [[8594, 'bindata_link_shift'], [8593, 'picture_soft_edge']]) {
  const input = fs.readFileSync(path.join(repo, `samples/issue${issue}/${name}.hwp`));
  const doc = new HwpDocument(input);
  const expected = pictures(doc);
  assert.equal(expected.length, 2, '원본 내장 그림 2개');
  const original = new HwpDocument(fs.readFileSync(path.join(repo, 'samples/ta-pic-001-r.hwp')));
  assert.deepEqual(expected, pictures(original), '독립 원본 그림의 bytes/원본 크기/crop/alpha 보존');
  original.free();
  for (const [ext, bytes] of [['hwp', doc.exportHwp()], ['hwpx', doc.exportHwpx()]]) {
    const reopened = new HwpDocument(bytes);
    assert.equal(reopened.pageCount(), doc.pageCount());
    assert.deepEqual(pictures(reopened), expected, `${ext} 저장 후 그림 참조·크기·crop·alpha`);
    fs.writeFileSync(path.join(output, `${name}.${ext}`), bytes);
    records.push({ issue, format: ext, inputSha256: sha(input), outputSha256: sha(bytes), pictures: expected, result: 'PASS' });
    reopened.free();
  }
  doc.free();
}
fs.writeFileSync(path.join(output, 'result.json'), JSON.stringify({ wasmSha256: sha(fs.readFileSync(path.join(pkg, 'rhwp_bg.wasm'))), records }, null, 2) + '\n');
console.log(JSON.stringify({ savedAndReopened: records.length, result: 'PASS' }));
