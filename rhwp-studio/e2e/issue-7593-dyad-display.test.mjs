import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { launchBrowser, createPage, loadApp, loadHwpFile, closeBrowser } from './helpers.mjs';
// 정상 제품의 loadDocument·font 준비·paint 경로를 실행한다.
// 절대 화면 좌표나 SVG 해시 대신 수식 소유 영역의 내용과 replay 진단을 검사한다.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const out = process.env.RHWP_ISSUE7593_E2E_OUT ?? path.join(root, 'output/e2e/issue7593');
mkdirSync(out, { recursive: true });
const wasmBytes = readFileSync(path.join(root, 'pkg/rhwp_bg.wasm'));
const expectedWasm = createHash('sha256').update(wasmBytes).digest('hex');
const browser = await launchBrowser();
const hostFontPath = process.env.RHWP_ISSUE7593_FONT;
const hostFontBytes = hostFontPath ? readFileSync(hostFontPath) : null;
const hostFontSha = hostFontBytes ? createHash('sha256').update(hostFontBytes).digest('hex') : null;
const records = [];
try {
  // 이 원본의 한컴 PUA 수식은 설치한 원본 face가 필요하다. 제품의 실제
  // Local Font Access 감지·바이트 준비 경로를 사용하며 임의 face로 치환하지 않는다.
  const origin = new URL(process.env.VITE_URL ?? 'http://localhost:7700').origin;
  const permissions = await browser.target().createCDPSession();
  if (!hostFontBytes) await permissions.send('Browser.grantPermissions', { origin, permissions: ['localFonts'] });
  for (const backend of ['canvas2d', 'canvaskit']) {
    for (const fixture of ['issue7593/repro-equation-issue7593.hwpx', 'issue7593/repro-dyad-min-no-script.hwpx']) {
      const page = await createPage(browser, 1400, 1600);
      await page.setCacheEnabled(false);
      await page.setBypassServiceWorker(true);
      const errors = [];
      page.on('pageerror', err => errors.push(err.message));
      // 실제 fetch 응답의 복제본을 읽는다. inspector 캐시의 크기·퇴거 정책과
      // 무관하게 제품이 받은 같은 WASM 바이트를 검사하며 별도 재요청하지 않는다.
      await page.evaluateOnNewDocument(() => {
        const originalFetch = globalThis.fetch;
        globalThis.__issue7593WasmResponses = [];
        globalThis.fetch = async (...args) => {
          const response = await originalFetch.apply(globalThis, args);
          if ((response.headers.get('content-type') || '').includes('application/wasm')
              && response.url.includes('rhwp_bg.wasm')) {
            const measurement = response.clone().arrayBuffer().then(async bytes => {
              const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
              return { hash: [...digest].map(value => value.toString(16).padStart(2, '0')).join('') };
            }).catch(error => ({ error: error.message }));
            globalThis.__issue7593WasmResponses.push(measurement);
          }
          return response;
        };
      });
      await loadApp(page, `?renderer=${backend}&profile=print&canvaskitSurface=software`);
      if (hostFontBytes) {
        await page.exposeFunction('__issue7593ReadHostFont', () => [...hostFontBytes]);
      }
      const localFonts = hostFontBytes ? await page.evaluate(async sha256 => {
        const { setHostFontProvider } = await import('/src/core/local-fonts.ts');
        const face = { id: sha256, family: 'HyhwpEQ', fullName: 'HyhwpEQ',
          postscriptName: 'HyhwpEQ', style: 'Regular', weight: 400, slant: 'normal' };
        const bytes = new Uint8Array(await globalThis.__issue7593ReadHostFont()).buffer;
        const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
        const actualSha = [...digest].map(value => value.toString(16).padStart(2, '0')).join('');
        if (actualSha !== sha256) throw Error('Host 글꼴 바이트 불일치');
        await setHostFontProvider({
          async getSnapshot() { return { revision: sha256, faces: [face] }; },
          async readFace(id, revision) {
            if (id !== sha256 || revision !== sha256) throw Error('Host face 소유 불일치');
            return { bytes: bytes.slice(0), faceIndex: 0 };
          },
          subscribe() { return () => {}; },
        });
        // 전쪽 Visual Sweep과 같이 Canvas2D에도 같은 실제 프로그램을 공급한다.
        const cssFace = new FontFace('HyhwpEQ', bytes.slice(0), { weight: '400', style: 'normal' });
        await cssFace.load(); document.fonts.add(cssFace); await document.fonts.ready;
        return [{ family: face.family, fullName: face.fullName, postscriptName: face.postscriptName,
          sha256: actualSha, source: 'host-provider' }];
      }, hostFontSha) : await page.evaluate(async () => {
        const { detectLocalFonts, getLocalFontRecords, loadLocalFontBytesFor, localFontFaceKey }
          = await import('/src/core/local-fonts.ts');
        await detectLocalFonts({ force: true, includeRegistered: true });
        const records = getLocalFontRecords({ includeRegistered: true })
          .filter(record => record.aliases.some(alias => /^(?:HyhwpEQ|HancomEQN)$/i.test(alias)));
        const bytesByFace = await loadLocalFontBytesFor(records.map(record => record.fullName));
        return Promise.all(records.map(async record => {
          const bytes = bytesByFace.get(localFontFaceKey(record));
          const digest = bytes ? new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)) : null;
          return { family: record.family, fullName: record.fullName, postscriptName: record.postscriptName,
            sha256: digest ? [...digest].map(value => value.toString(16).padStart(2, '0')).join('') : null };
        }));
      });
      assert.ok(localFonts.some(record => /hyhwpeq/i.test(`${record.family} ${record.fullName}`)),
        '검증 환경에 원본 수식 글꼴 HyhwpEQ가 필요합니다');
      const loaded = await loadHwpFile(page, fixture);
      await page.evaluate(() => window.__eventBus.emit('zoom-changed', 1));
      await new Promise(resolve => setTimeout(resolve, 1500));
      const result = await page.evaluate(() => {
        const tree = JSON.parse(window.__wasm.doc.getPageRenderTree(0));
        const nodes = [];
        const visit = n => { nodes.push(n); for (const c of n.children || []) visit(c); };
        visit(tree);
        const canvases = [...document.querySelectorAll('#scroll-container canvas')];
        const merged = document.createElement('canvas');
        merged.width = canvases[0].width; merged.height = canvases[0].height;
        const context = merged.getContext('2d');
        context.fillStyle = '#ffffff'; context.fillRect(0, 0, merged.width, merged.height);
        for (const canvas of canvases) context.drawImage(canvas, 0, 0);
        const equations = nodes.filter(n => n.type === 'Equation');
        const pageWidth = (tree.bbox.width ?? tree.bbox.w);
        const content = context.getImageData(0, 0, merged.width, merged.height).data;
        const equationInk = equations.map(eq => {
          const factor = merged.width / pageWidth;
          const box = eq.bbox;
          let count = 0;
          for (let y = Math.floor(box.y * factor); y < Math.ceil((box.y + (box.height ?? box.h)) * factor); y++)
            for (let x = Math.floor(box.x * factor); x < Math.ceil((box.x + (box.width ?? box.w)) * factor); x++) {
              const offset = (y * merged.width + x) * 4;
              if (content[offset] < 200 || content[offset + 1] < 200 || content[offset + 2] < 200) count++;
            }
          return count;
        });
        // UI 용지 꺾쇠·선택 표시를 Print 기준에 섞지 않는다. 같은 문서의 모든
        // PaintOp를 같은 96dpi로 출력하며 저점 영역 마스크나 재배율을 적용하지 않는다.
        const printCanvas = document.createElement('canvas');
        if (window.__renderBackend === 'canvas2d') {
          window.__wasm.renderPageToCanvasFiltered(0, printCanvas, 1, 'all', 'print');
        }
        return {
          printComposite: window.__renderBackend === 'canvas2d'
            ? printCanvas.toDataURL('image/png') : merged.toDataURL('image/png'),
          composite: merged.toDataURL('image/png'),
          equationInk,
          rendererSessionDiagnostics: window.__canvasView.getRendererSessionDiagnostics(),
          canvasKitDiagnostics: window.__canvasView.getCanvasKitRenderDiagnostics(0),
          effectiveBackend: window.__renderBackend,
          fallbackReason: window.__renderBackendFallbackReason,
          pageCount: window.__wasm.doc.pageCount(),
          equationCount: nodes.filter(n => n.type === 'Equation').length,
          masterLineCount: nodes.filter(n => n.type === 'MasterPage').flatMap(n => n.children || []).filter(n => n.type === 'Line').length,
          canvases: canvases.map(c => ({ width: c.width, height: c.height, png: c.toDataURL('image/png') })),
        };
      });
      const stem = fixture.includes('equation-issue') ? 'issue' : 'min';
      writeFileSync(`${out}/${backend}-${stem}-composite.png`, Buffer.from(result.composite.split(',')[1], 'base64'));
      delete result.composite;
      writeFileSync(`${out}/${backend}-${stem}-print.png`, Buffer.from(result.printComposite.split(',')[1], 'base64'));
      delete result.printComposite;
      for (const [i, canvas] of result.canvases.entries()) {
        writeFileSync(`${out}/${backend}-${stem}-${i}.png`, Buffer.from(canvas.png.split(',')[1], 'base64'));
        delete canvas.png;
      }
      const wasmMeasurements = await page.evaluate(() => Promise.all(globalThis.__issue7593WasmResponses));
      const wasmHashes = wasmMeasurements.map(result => result.hash ?? null);
      errors.push(...wasmMeasurements.filter(result => result.error).map(result => result.error));
      const record = { backend, fixture, loaded, localFonts, ...result, errors, wasmHashes, expectedWasm };
      records.push(record);
      console.log(JSON.stringify(record));
      assert.equal(loaded.pageCount, 1);
      assert.equal(result.equationCount, stem === 'issue' ? 7 : 4);
      assert.equal(result.effectiveBackend, backend);
      assert.equal(result.fallbackReason, null);
      if (backend === 'canvaskit') {
        assert.equal(result.canvasKitDiagnostics.lastRenderCompleted, true);
        assert.deepEqual(result.canvasKitDiagnostics.lastUnexpectedUnsupportedOps, []);
        assert.equal(result.canvasKitDiagnostics.hiddenCanvas2dOverlayUsed, false);
      }
      if (wasmHashes.length === 0 || wasmHashes.some(hash => hash !== expectedWasm)) throw Error('Studio의 실제 WASM 바이트 불일치');
      if (errors.length) throw Error('브라우저 실행 오류');
      if (result.equationInk.some(count => count === 0)) throw Error('실제 화면의 수식 누락');
      await page.close();
    }
  }
} finally {
  writeFileSync(`${out}/results.json`, JSON.stringify(records, null, 2));
  await closeBrowser(browser);
}
