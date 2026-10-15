import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { runTest, setTestCase } from './helpers.mjs';

// Independent Print: the circular dot is centered within its advance, with
// adjacent text preserved. C22 Native/fresh WASM all27pages >=94.12523%;
// actual Canvas page3=96.94814%, and all7 dots were read at the same8x scale.
// No page number, absolute placement, SVG hash, or implementation constant is pinned.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const fixture = path.join(root, 'samples/k-water-rfp.hwp');
const packageDir = path.resolve(process.env.RHWP_CANVAS_TEST_WASM_PACKAGE || path.join(root, 'pkg'));
const output = process.env.RHWP_CANVAS_TEST_OUTPUT;
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const fsUrl = filename => `/@fs${pathToFileURL(filename).pathname}`;

await runTest('가운뎃점의 실제 Canvas 원형 경로와 내용 소유', async ({ page }) => {
  setTestCase('독립 Print에서 확인한 원형 점과 앞뒤 본문');
  const result = await page.evaluate(async ({ moduleUrl, fixtureUrl }) => {
    const module = await import(moduleUrl);
    await module.default();
    const response = await fetch(fixtureUrl);
    if (!response.ok) throw Error(`원본 읽기 실패: ${response.status}`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    const digest = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)))
      .map(x => x.toString(16).padStart(2, '0')).join('');
    const doc = new module.HwpDocument(bytes);
    try {
      doc.setHangul2024Compat(false);
      const pageCount = doc.pageCount();
      let pageIndex, runs;
      for (let i = 0; i < pageCount; i++) {
        const current = JSON.parse(doc.getPageTextLayout(i)).runs || [];
        if (current.some(run => run.text?.includes('·'))) {
          pageIndex = i;
          runs = current;
          break;
        }
      }
      if (!runs) throw Error('실제 원본에서 가운뎃점 소유 run을 찾지 못함');
      const dots = [];
      for (const run of runs) {
        for (const [i, character] of Array.from(run.text || '').entries()) {
          if (character !== '·') continue;
          if (!Number.isFinite(run.charX?.[i]) || !Number.isFinite(run.charX?.[i + 1])) {
            throw Error('가운뎃점의 실제 전진 소유 경계가 없음');
          }
          dots.push({ left: run.x + run.charX[i], right: run.x + run.charX[i + 1],
            top: run.y, bottom: run.y + run.h });
        }
      }
      const canvas = document.createElement('canvas');
      const calls = [], glyphs = [];
      const proto = CanvasRenderingContext2D.prototype;
      const original = { arc: proto.arc, fillText: proto.fillText, strokeText: proto.strokeText };
      proto.arc = function (x, y, radius, start, end, ...rest) {
        const t = this.getTransform();
        calls.push({ x: t.a * x + t.c * y + t.e, y: t.b * x + t.d * y + t.f,
          rx: Math.hypot(t.a, t.c) * radius, ry: Math.hypot(t.b, t.d) * radius, start, end });
        return original.arc.call(this, x, y, radius, start, end, ...rest);
      };
      for (const name of ['fillText', 'strokeText']) {
        proto[name] = function (text, ...args) {
          if (String(text).includes('·')) glyphs.push({ kind: name, text: String(text) });
          return original[name].call(this, text, ...args);
        };
      }
      try {
        doc.renderPageToCanvas(pageIndex, canvas, 1);
      } finally {
        Object.assign(proto, original);
      }
      const owners = values => values.map(r => [r.secIdx, r.paraIdx, r.charStart, r.text]);
      const after = JSON.parse(doc.getPageTextLayout(pageIndex)).runs || [];
      return { inputSha256: digest, pageCount, afterPageCount: doc.pageCount(), pageIndex,
        dots, calls, glyphs, ownersBefore: owners(runs), ownersAfter: owners(after) };
    } finally {
      doc.free();
    }
  }, { moduleUrl: fsUrl(path.join(packageDir, 'rhwp.js')), fixtureUrl: fsUrl(fixture) });
  result.packageFiles = Object.fromEntries(['rhwp.js', 'rhwp_bg.wasm']
    .map(name => [name, sha(readFileSync(path.join(packageDir, name)))]));
  if (output) {
    mkdirSync(output, { recursive: true });
    writeFileSync(path.join(output, 'result.json'), `${JSON.stringify(result, null, 2)}\n`);
  }
  assert.equal(result.inputSha256, sha(readFileSync(fixture)), '실제 원본 바이트 소비');
  assert.deepEqual(result.glyphs, [], '원형 점을 대체 글리프의 fillText/strokeText로 그리면 안 됨');
  assert.ok(result.dots.length > 0, '실제 내용의 가운뎃점 소유');
  const used = new Set();
  for (const dot of result.dots) {
    const candidates = result.calls.map((call, i) => ({ call, i })).filter(({ call, i }) => {
      const middle = (dot.right - dot.left) / 3;
      return !used.has(i) && call.rx > 0 && call.ry > 0
        && Math.abs(call.rx - call.ry) <= Number.EPSILON * Math.max(call.rx, call.ry) * 16
        && call.end - call.start >= Math.PI * 2
        && call.x > dot.left + middle && call.x < dot.right - middle
        && call.x - call.rx >= dot.left && call.x + call.rx <= dot.right
        && call.y - call.ry >= dot.top && call.y + call.ry <= dot.bottom;
    });
    assert.equal(candidates.length, 1, '소유 전진의 중앙과 글줄 안에 원형 점 하나만 배치');
    used.add(candidates[0].i);
  }
  assert.deepEqual(result.ownersAfter, result.ownersBefore, '앞뒤 내용·문단 소유·순서 보존');
  assert.equal(result.afterPageCount, result.pageCount, 'paint가 쪽 소유를 변경하지 않음');
  console.log(`PASS: 실제 원형 점 ${used.size}개, 대체 글리프 0개, 내용 소유 보존`);
});
