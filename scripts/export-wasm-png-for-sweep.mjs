#!/usr/bin/env node
// Print Canvas를 PNG로 직접 내보낸다. 각 작업자는 Chrome 하나·탭 하나를 재사용한다.
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import http from 'node:http';
import {createRequire} from 'node:module';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {findChrome} from './rasterize-svg-webfonts.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const option = name => {
  const index = process.argv.indexOf(name);
  if (index < 0 || !process.argv[index + 1]) throw Error(`필수 인자 누락: ${name}`);
  return resolve(process.argv[index + 1]);
};
const sha = bytes => createHash('sha256').update(bytes).digest('hex');

// 실제 Canvas가 요청한 family에 공급 program을 등록한다. 별칭은 글리프를 바꾸지 않는다.
export async function registerCanvasFont(family, bytes, weight, style) {
  // FontFace에는 원본 이름을 전달한다. JSON 문자열의 따옴표나 CSS 식별자의
  // 이스케이프를 붙이면 Canvas가 조회한 이름과 공급 이름이 달라질 수 있다.
  const font = new FontFace(family, bytes, {weight: String(weight), style});
  await font.load();
  document.fonts.add(font);
  return font;
}

// 공급 목록만 공개하며 URL을 파일시스템 경로로 해석하지 않는다.
export function createSweepServer(files, fontByUrl) {
  return http.createServer((req, res) => {
    if (req.url === '/') { res.end('<!doctype html><meta charset="utf-8"><canvas id="page"></canvas>'); return; }
    const entry = files.get(req.url);
    if (entry) { res.setHeader('Content-Type', entry[0]); res.end(entry[1]); return; }
    const font = fontByUrl.get(req.url);
    if (!font) { res.writeHead(404); res.end(); return; }
    try {
      const bytes = readFileSync(font.path);
      if (sha(bytes) !== font.sha256) throw Error('공급 글꼴 바이트가 catalogue 작성 뒤 변경됐습니다.');
      res.setHeader('Content-Type', 'application/octet-stream'); res.end(bytes);
    } catch (error) {
      // 상세 정보는 로컬 로그에만 남기고 HTTP 응답에 경로나 예외를 보내지 않는다.
      console.error('Visual Sweep 글꼴 공급 실패:', error);
      res.writeHead(500, {'Content-Type': 'text/plain; charset=utf-8'});
      res.end('Font resource unavailable.');
    }
  });
}

// Snap Chromium의 /tmp는 host의 /tmp와 다르다. host가 지우는 임시 경로 대신
// 양쪽에서 같은 파일을 가리키는 검토 output 아래에 전용 프로필을 만든다.
export function createSweepBrowserProfile(output, index) {
  mkdirSync(output, {recursive: true});
  return mkdtempSync(join(resolve(output), `chrome-worker-${index}-`));
}

// Print PNG는 투명 Canvas를 종이의 흰 배경에 합성한다.
// ceil로 늘어난 마지막 픽셀의 투명 RGB를 검정 잉크로 해석하지 않게 한다.
export function encodeCanvasOnWhitePaper(canvasId) {
  const source = document.getElementById(canvasId);
  const paper = document.createElement('canvas');
  paper.width = source.width; paper.height = source.height;
  const sourceContext = source.getContext('2d');
  if (sourceContext.isContextLost?.()) throw Error('Canvas context가 유실되어 PNG를 내보낼 수 없습니다.');
  // 실제 RGBA를 동기 readback한 뒤 흰 종이에 합성한다.
  const pixels = sourceContext.getImageData(0, 0, source.width, source.height);
  for (let i = 0; i < pixels.data.length; i += 4) {
    const alpha = pixels.data[i + 3];
    if (alpha === 255) continue;
    for (let channel = 0; channel < 3; channel++) {
      pixels.data[i + channel] = Math.round((pixels.data[i + channel] * alpha + 255 * (255 - alpha)) / 255);
    }
    pixels.data[i + 3] = 255;
  }
  paper.getContext('2d', {alpha: false}).putImageData(pixels, 0, 0);
  return paper.toDataURL('image/png').split(',')[1];
}

async function main() {
  const pkg = option('--pkg'), input = option('--input'), output = option('--out');
  const settings = JSON.parse(readFileSync(option('--options'), 'utf8'));
  const catalogue = JSON.parse(readFileSync(option('--fonts'), 'utf8'));
  if (!['2022', '2024'].includes(settings.compat) || !(settings.scale > 0)
      || !Number.isInteger(settings.workers) || settings.workers < 1) throw Error('PNG 출력 조건이 올바르지 않습니다.');
  // 문서 초기 파싱·조판도 CDP evaluate에 포함된다. 복잡한 실물 HWPX는
  // 180초를 넘으므로 유한한 10분을 기본으로 두고 실행 조건에도 보존한다.
  const protocolTimeout = settings.protocolTimeoutMs ?? 600000;
  if (!Number.isSafeInteger(protocolTimeout) || protocolTimeout < 1 || protocolTimeout > 2147483647) {
    throw Error('브라우저 통신 제한 시간은 유효한 양의 밀리초여야 합니다.');
  }
  const wanted = settings.pages;
  if (wanted !== null && (!Array.isArray(wanted) || !wanted.length || wanted.some(p => !Number.isInteger(p) || p < 1)
      || new Set(wanted).size !== wanted.length)) throw Error('선택 쪽이 올바르지 않습니다.');
  const environmentJson = settings.fontEnvironment ? readFileSync(settings.fontEnvironment, 'utf8') : null;
  const files = new Map([
    ['/font-supply.js', ['application/javascript', 'export ' + registerCanvasFont.toString()]],
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/source', ['application/octet-stream', readFileSync(input)]],
    ['/font-registration.mjs', ['application/javascript', readFileSync(join(root, 'scripts/visual-sweep-font-registration.mjs'))]],
    ['/sweep-font-environment.js', ['application/javascript', readFileSync(join(root, 'scripts/visual-sweep-font-environment.mjs'))]],
  ]);
  // 실제 공급 목록만 HTTP로 읽을 수 있다. 요청 URL을 파일시스템 경로로 변환하지 않는다.
  const fonts = catalogue.faces.map((face, index) => ({...face, url: `/font/${index}`}));
  const fontByUrl = new Map(fonts.map(face => [face.url, face]));
  const server = createSweepServer(files, fontByUrl);
  await new Promise((accept, reject) => {
    server.once('error', reject); server.listen(0, '127.0.0.1', accept);
  });
  const require = createRequire(join(root, 'rhwp-studio/package.json'));
  let pageCount;
  let environmentSignature;
  let fontEnvironment;
  const browsers = new Set();
  const pageRecords = [];
  const workers = [];
  const workerCount = wanted ? Math.min(settings.workers, wanted.length) : settings.workers;
  mkdirSync(join(output, 'rhwp_png'), {recursive: true});
  mkdirSync(join(output, 'render_tree'), {recursive: true});
  try {
    async function worker(index) {
      const userDataDir = createSweepBrowserProfile(output, index);
      let browser;
      try {
        browser = await require('puppeteer-core').launch({
          executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true,
          protocolTimeout, userDataDir,
        });
        browsers.add(browser);
        const page = await browser.newPage();
        await page.goto(`http://127.0.0.1:${server.address().port}`);
        const info = await page.evaluate(async ({compat, environmentJson, fonts}) => {
          const module = await import('/rhwp.js');
          await module.default({module_or_path: '/rhwp_bg.wasm'});
          const doc = new module.HwpDocument(new Uint8Array(await (await fetch('/source')).arrayBuffer()));
          if (typeof doc.renderPageToCanvasFilteredWithProfile !== 'function') throw Error('Print Canvas API가 없는 이전 WASM입니다.');
          doc.setHangul2024Compat(compat === '2024');
          const {applySweepFontEnvironment, loadSweepFontPrograms} = await import('/sweep-font-environment.js');
          // 실제 프로그램을 준비하고 공통 선택을 적용한 다음에 첫 pageCount를 계산한다.
          const fontEnvironment = await applySweepFontEnvironment(doc, environmentJson,
            plan => loadSweepFontPrograms(plan, fonts));
          globalThis.sweepDocument = doc;
          globalThis.sweepRequestedFonts = new Set();
          globalThis.sweepImages = new Set();
          globalThis.sweepImageErrors = [];
          // 실제 Canvas가 요청한 family 사다리를 수집한다. SVG 글꼴 정책용 내보내기는 하지 않는다.
          const fontDescriptor = Object.getOwnPropertyDescriptor(CanvasRenderingContext2D.prototype, 'font');
          Object.defineProperty(CanvasRenderingContext2D.prototype, 'font', {
            ...fontDescriptor, set(value) {
              globalThis.sweepRequestedFonts.add(value);
              return fontDescriptor.set.call(this, value);
            },
          });
          const imageDescriptor = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
          Object.defineProperty(HTMLImageElement.prototype, 'src', {
            ...imageDescriptor, set(value) {
              const image = this;
              const loading = new Promise(resolve => {
                image.addEventListener('load', () => resolve(), {once: true});
                image.addEventListener('error', () => { globalThis.sweepImageErrors.push('Canvas 그림 decode 실패'); resolve(); }, {once: true});
              });
              globalThis.sweepImages.add(loading);
              loading.finally(() => globalThis.sweepImages.delete(loading));
              return imageDescriptor.set.call(this, value);
            },
          });
          const {prepareNativePuaFonts} = await import('/font-registration.mjs');
          const nativePuaFonts = await prepareNativePuaFonts(doc, fonts, async record => {
            const response = await fetch(record.url);
            if (!response.ok) throw Error('PUA 공급 글꼴을 읽을 수 없습니다.');
            return response.arrayBuffer();
          });
          return {pageCount: doc.pageCount(), version: module.version(), nativePuaFonts, fontEnvironment};
        }, {compat: settings.compat, environmentJson, fonts});
        const signature = JSON.stringify(info.fontEnvironment);
        if (environmentSignature !== undefined && environmentSignature !== signature) throw Error('WASM 작업자의 실제 글꼴 환경이 다릅니다.');
        environmentSignature = signature;
        fontEnvironment = info.fontEnvironment;
        if (!Number.isInteger(info.pageCount) || info.pageCount < 1) throw Error('WASM 전체 쪽수가 올바르지 않습니다.');
        if (pageCount !== undefined && pageCount !== info.pageCount) throw Error('WASM 작업자 간 전체 쪽수가 다릅니다.');
        pageCount = info.pageCount;
        const requested = wanted || Array.from({length: pageCount}, (_, i) => i + 1);
        if (requested.some(p => p > pageCount)) throw Error('요청 쪽이 WASM 전체 쪽수를 벗어납니다.');
        const pages = requested.filter((_, i) => i % workerCount === index);
        if (!pages.length) return;
        // Canvas2D는 레이어의 내장 font blob을 자동 FontFace로 등록하지 않는다.
        // 같은 WASM 문서의 metadata/key API로 선택 face를 공급하며 SVG는 만들지 않는다.
        const embedded = await page.evaluate(({pages}) => {
          const doc = globalThis.sweepDocument, faces = new Map();
          for (const page of pages) {
            const tree = JSON.parse(doc.getPageLayerTreeWithProfile(page - 1, 'print', true, true));
            for (const face of tree.fontResources?.faces || []) {
              const blob = tree.fontResources.blobs.find(blob => blob.id === face.blobKey);
              if (!blob || blob.source !== 'embedded') continue;
              if (!blob.dataRef?.id) throw Error('내장 글꼴의 바이트 key가 없습니다.');
              const key = `${blob.dataRef.id}|${face.faceIndex}`;
              if (faces.has(key)) continue;
              const bytes = doc.getSourceFontBytes(blob.dataRef.id);
              let data = '';
              for (let offset = 0; offset < bytes.length; offset += 16384) {
                data += String.fromCharCode(...bytes.subarray(offset, offset + 16384));
              }
              faces.set(key, {bytes: btoa(data), faceIndex: face.faceIndex,
                             families: face.familyNames.map(name => name.value)});
            }
          }
          return [...faces.values()];
        }, {pages});
        const workerFonts = [...fonts];
        if (embedded.length) {
          const folder = join(output, `embedded-fonts-${index}`);
          mkdirSync(join(folder, 'source-fonts'), {recursive: true});
          const requests = embedded.map(face => {
            const bytes = Buffer.from(face.bytes, 'base64'), digest = sha(bytes);
            const path = join(folder, 'source-fonts', `${digest}.${bytes.subarray(0,4).toString() === 'ttcf' ? 'ttc' : 'ttf'}`);
            writeFileSync(path, bytes);
            return {path, faceIndex: face.faceIndex, families: face.families};
          });
          const requestPath = join(folder, 'requests.json');
          writeFileSync(requestPath, JSON.stringify(requests));
          execFileSync(settings.pythonExecutable, [join(root, 'scripts/visual_sweep_png.py'),
                       '--font-requests', requestPath, '--out', folder], {stdio: 'pipe'});
          const supplied = JSON.parse(readFileSync(join(folder, 'font-catalog.json'), 'utf8')).faces;
          for (const [faceIndex, face] of supplied.entries()) {
            face.url = `/embedded-font/${index}/${faceIndex}`;
            fontByUrl.set(face.url, face);
          }
          workerFonts.unshift(...supplied);
        }
        // 같은 문서의 필요한 face를 한 번만 공급한다. 데이터는 SVG/base64로 복제하지 않는다.
        const fontInfo = await page.evaluate(async ({pages, fonts, scale}) => {
          const {installCanvasFont, selectMetafileFontFaces} = await import('/font-registration.mjs');
          const doc = globalThis.sweepDocument, canvas = document.getElementById('page');
          for (const page of pages) doc.renderPageToCanvasFilteredWithProfile(page - 1, canvas, scale, 'all', 'print');
          const metafileRequests = typeof doc.getMetafileFontRequests === 'function' ? JSON.parse(doc.getMetafileFontRequests()) : [];
          const metafileFaces = selectMetafileFontFaces(fonts, metafileRequests);
          const requested = new Set();
          if (metafileRequests.length) {
            for (const request of metafileRequests) {
              for (const family of request.families) {
                if (fonts.some(record => record.families.some(name => name.toLowerCase() === family.toLowerCase()))) {
                  requested.add(family.toLowerCase());
                }
              }
            }
          }
          for (const value of globalThis.sweepRequestedFonts) {
            const style = document.createElement('span').style; style.font = value;
            const chain = (style.fontFamily.match(/"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|[^,]+/g) || [])
              .map(family => family.trim().replace(/^['"]|['"]$/g, '').toLowerCase());
            // 첫 face에 없는 문자의 fallback도 같은 실제 family 사다리로 공급한다.
            for (const family of chain) {
              if (fonts.some(record => record.families.some(name => name.toLowerCase() === family))) requested.add(family);
            }
          }
          const supplied = new Set(), loaded = [];
          for (const record of fonts) {
            const aliases = record.families.filter(family => requested.has(family.toLowerCase()));
            const selected = aliases.filter(family => !supplied.has(`${family.toLowerCase()}|${record.weight}|${record.style}`));
            if (!selected.length) continue;
            const response = await fetch(record.url);
            if (!response.ok) throw Error(`공급 글꼴을 읽을 수 없습니다: ${record.url}`);
            const bytes = await response.arrayBuffer();
            for (const family of selected) {
              // 실제 family 등록과 HFT 원본 윤곽 바인딩을 같은 helper가 소유한다.
              let font;
              try {
                font = await installCanvasFont(family, bytes, {weight: String(record.weight), style: record.style}, doc, metafileFaces.has(`${family.toLowerCase()}|${record.weight}|${record.style}`));
                if (font.family !== family) throw Error(`Canvas 글꼴의 등록 이름이 다릅니다: ${family} → ${font.family}`);
              }
              catch (error) { throw Error(`선택한 Canvas 글꼴을 적재할 수 없습니다: ${family}, ${record.source}, ${error}`); }
              supplied.add(`${family.toLowerCase()}|${record.weight}|${record.style}`);
              loaded.push({family, registeredFamily: font.family, sourceSha256: record.sourceSha256, sha256: record.sha256, faceIndex: record.faceIndex, weight: record.weight, style: record.style, repairs: record.repairs, sourceKind: record.sourceKind || 'external'});
            }
          }
          await document.fonts.ready;
          // 캐시에 적재되기만 하고 아직 paint되지 않은 그림은 준비 뒤 다시 재생한다.
          await Promise.all([...globalThis.sweepImages]);
          if (globalThis.sweepImageErrors.length) throw Error(globalThis.sweepImageErrors.join('; '));
          return {requested: [...requested], loaded};
        }, {pages, fonts: workerFonts, scale: settings.scale});
        for (const number of pages) {
          const result = await page.evaluate(async ({number, scale}) => {
            const doc = globalThis.sweepDocument, canvas = document.getElementById('page');
            for (let attempt = 0; attempt < 3; attempt++) {
              doc.renderPageToCanvasFilteredWithProfile(number - 1, canvas, scale, 'all', 'print');
              if (!globalThis.sweepImages.size) break;
              await Promise.all([...globalThis.sweepImages]);
              if (globalThis.sweepImageErrors.length) throw Error(globalThis.sweepImageErrors.join('; '));
              if (attempt === 2) throw Error('Canvas 그림 캐시가 안정되지 않아 PNG 캡처를 중단합니다.');
            }
            await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
            return {width: canvas.width, height: canvas.height,
                    tree: doc.getPageRenderTree(number - 1)};
          }, {number, scale: settings.scale});
          result.png = await page.evaluate(encodeCanvasOnWhitePaper, 'page');
          const tree = JSON.parse(result.tree);
          if (!tree.type || !tree.bbox || !(result.width > 0 && result.height > 0)) throw Error(`p${number} Canvas 산출물 손상`);
          const bytes = Buffer.from(result.png, 'base64');
          if (!bytes.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10]))) throw Error(`p${number} PNG signature 손상`);
          const suffix = String(number).padStart(3, '0');
          writeFileSync(join(output, 'rhwp_png', `rhwp_${suffix}.png`), bytes);
          writeFileSync(join(output, 'render_tree', `render_tree_${suffix}.json`), result.tree);
          pageRecords.push({page: number, pngSha256: sha(bytes), width: result.width, height: result.height});
          console.log(`Canvas PNG p${suffix}: worker ${index + 1}`);
        }
        workers.push({index, pages, fonts: fontInfo, nativePuaFonts: info.nativePuaFonts, fontEnvironment: info.fontEnvironment, browser: await browser.version()});
        await page.evaluate(() => { globalThis.sweepDocument.free(); delete globalThis.sweepDocument; });
      } finally {
        if (browser) {
          // 종료에 실패하면 활성 프로필을 지우지 않고 바깥 finally가 다시 닫게 둔다.
          await browser.close(); browsers.delete(browser);
        }
        rmSync(userDataDir, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
      }
    }
    await Promise.all(Array.from({length: workerCount}, (_, index) => worker(index)));
    const expected = wanted || Array.from({length: pageCount}, (_, i) => i + 1);
    if (pageRecords.length !== expected.length || new Set(pageRecords.map(p => p.page)).size !== expected.length) throw Error('WASM PNG 쪽 누락·중복');
    writeFileSync(join(output, 'wasm-export-complete.json'), JSON.stringify({
      pageCount, layoutGeneration: settings.compat, comparisonProfile: 'print', rasterBackend: 'wasm-canvas2d', protocolTimeoutMs: protocolTimeout,
      pngBackground: 'opaque-white-print-paper',
      fontEnvironment,
      pages: pageRecords.sort((a,b) => a.page-b.page), workers,
    }, null, 2));
  } finally {
    await Promise.allSettled([...browsers].map(browser => browser.close()));
    await new Promise(resolve => server.close(resolve));
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.stack || error); process.exitCode = 1; });
}
