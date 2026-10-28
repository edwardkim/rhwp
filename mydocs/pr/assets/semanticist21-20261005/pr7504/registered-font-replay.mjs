#!/usr/bin/env node
// Visual Sweep의 WASM 입력을 실제 Chrome에서 생성한다. 래스터/폰트 정책은 Sweep이 소유한다.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { findChrome } from '../../../../../scripts/rasterize-svg-webfonts.mjs';

const root = process.cwd();
const option = name => {
  const index = process.argv.indexOf(name);
  if (index < 0 || !process.argv[index + 1]) throw Error(`필수 인자 누락: ${name}`);
  return resolve(process.argv[index + 1]);
};

async function main() {
  const pkg = option('--pkg'), input = option('--input'), output = option('--out');
  const environmentJson = process.argv.includes('--font-environment')
    ? readFileSync(option('--font-environment'), 'utf8') : null;
  // 임의 파일 경로를 HTTP 요청으로 받아 열지 않는다.
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/source', ['application/octet-stream', readFileSync(input)]],
    ['/font', ['font/ttf', readFileSync(option('--font-file'))]],
  ]);
  const server = http.createServer((req, res) => {
    if (req.url === '/') { res.setHeader('Content-Type', 'text/html'); res.end('<!doctype html><meta charset="utf-8">'); return; }
    const file = files.get(req.url);
    if (!file) { res.writeHead(404); res.end(); return; }
    res.setHeader('Content-Type', file[0]); res.end(file[1]);
  });
  await new Promise((accept, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', accept);
  });
  let browser;
  try {
    const require = createRequire(join(root, 'rhwp-studio/package.json'));
    browser = await require('puppeteer-core').launch({
      executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true,
      protocolTimeout: 180000,
    });
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    const info = await page.evaluate(async environmentJson => {
      const module = await import('/rhwp.js');
      await module.default({ module_or_path: '/rhwp_bg.wasm' });
      globalThis.sweepDocument = new module.HwpDocument(new Uint8Array(await (await fetch('/source')).arrayBuffer()));
      if (environmentJson !== null) globalThis.sweepDocument.setFontEnvironment(environmentJson);
      const d = globalThis.sweepDocument;
      const font = new Uint8Array(await (await fetch('/font')).arrayBuffer());
      const beforeLayout = JSON.parse(d.getPageTextLayout(0));
      for (const family of new Set(beforeLayout.runs.map(r => r.fontFamily.split(',')[0].replace(/['"]/g, '').trim()))) {
        const face = new FontFace(family, font); await face.load(); document.fonts.add(face);
      }
      const canvas = document.createElement('canvas'); canvas.width=1600; canvas.height=2300; document.body.append(canvas);
      const original = CanvasRenderingContext2D.prototype.fillText; let calls=[];
      CanvasRenderingContext2D.prototype.fillText = function(text,x,y,...rest) {
        const t=this.getTransform(); calls.push({text,x,y,a:t.a,b:t.b,c:t.c,d:t.d,e:t.e,f:t.f,font:this.font});
        return original.call(this,text,x,y,...rest);
      };
      d.renderPageToCanvas(0,canvas,2); const before=calls; calls=[];
      for (const id of new Set(beforeLayout.runs.map(r=>r.charShapeId))) d.registerExactFontSource(id,1,font,0);
      d.renderPageToCanvas(0,canvas,2); const after=calls;
      CanvasRenderingContext2D.prototype.fillText=original;
      globalThis.registeredFontObservations={before,after,beforeLayout,afterLayout:JSON.parse(d.getPageTextLayout(0))};

      return { pageCount: globalThis.sweepDocument.pageCount(), version: module.version() };
    }, environmentJson);
    if (!Number.isInteger(info.pageCount) || info.pageCount < 1) throw Error('WASM이 빈 문서를 반환했습니다.');
    for (const folder of ['raw_svg', 'render_tree']) mkdirSync(join(output, folder), { recursive: true });
    const pages = [];
    for (let index = 0; index < info.pageCount; index++) {
      const result = await page.evaluate(index => ({
        svg: globalThis.sweepDocument.renderPageSvgWithProfile(index, 'print'),
        tree: globalThis.sweepDocument.getPageRenderTree(index),
      }), index);
      // 별도 SVG 래스터 브라우저에도 Canvas에 등록한 동일 source bytes를 공급한다.
      // 위치·폭은 실제 Print renderer 결과를 유지한다.
      const family = await page.evaluate(() => globalThis.registeredFontObservations.beforeLayout.runs[0].fontFamily.split(',')[0].replace(/['"]/g, '').trim());
      const encodedFont = files.get('/font')[1].toString('base64');
      result.svg = result.svg.replace(/(<svg\b[^>]*>)/, `$1<style>@font-face { font-family: "${family}"; src: url(data:font/ttf;base64,${encodedFont}) format("truetype"); }</style>`);
      const tree = JSON.parse(result.tree);
      if (!tree.type || !tree.bbox || !result.svg.includes('<svg')) throw Error(`WASM page ${index + 1} 산출물 손상`);
      const suffix = String(index + 1).padStart(3, '0');
      writeFileSync(join(output, 'raw_svg', `wasm_${suffix}.svg`), result.svg);
      writeFileSync(join(output, 'render_tree', `render_tree_${suffix}.json`), result.tree);
      pages.push({ page: index + 1, svgSha256: createHash('sha256').update(result.svg).digest('hex') });
    }
    await page.evaluate(() => { globalThis.sweepDocument.free(); delete globalThis.sweepDocument; });
    const observations = await page.evaluate(() => globalThis.registeredFontObservations);
    writeFileSync(join(output, 'registered-font-observations.json'), JSON.stringify(observations, null, 2));
    await page.screenshot({path:join(output, 'registered-canvas.png')});
    const hash = file => createHash('sha256').update(readFileSync(file)).digest('hex');
    const manifest = { ...info, comparisonProfile: 'print', browser: await browser.version(), pages,
      producer: 'HwpDocument.registerExactFontSource + renderPageSvgWithProfile(print)',
      inputSha256: hash(input), fontSha256: hash(option('--font-file')),
      wasmSha256: hash(join(pkg,'rhwp_bg.wasm')), jsSha256: hash(join(pkg,'rhwp.js')),
      exporterSha256: hash(fileURLToPath(import.meta.url)), languageIndex: 1, faceIndex: 0 };
    writeFileSync(join(output, 'manifest.json'), JSON.stringify(manifest, null, 2));
    console.log(JSON.stringify({ ...info, browser: manifest.browser }));
  } finally {
    try {
      if (browser) await browser.close();
    } finally {
      await new Promise(resolve => server.close(resolve));
    }
  }
}

main().catch(error => { console.error(error.message); process.exitCode = 1; });
