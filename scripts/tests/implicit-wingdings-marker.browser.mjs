// 관계형: 실물 원본의 기호 소유와 실제 Canvas 윤곽을 독립 Print의 em 계약으로 검사한다.
// 주석 작성: 2026-11-07 13:57 +09:00. 새 파일이므로 직전 변경 커밋은 없다.
// 2026-11-07, ef77d740e1, Print·96 DPI·보정 전 2px raw, 전체 94쪽이 PDF와 일치.
// 전쪽 최저 Native 99.98% / fresh WASM 99.99%, 검사 5쪽 100.00% / 99.99%.
// 원본 samples/basic/sungeo.hwp, 독립 pdf/basic/sungeo-hwp-2020.pdf.
// 측정·SHA·같은 쪽 review/overlay: mydocs/pr/assets/planet6897_20261106/pr8611/r33-source-symbol/.
// PDF의 Wingdings-Regular glyph 3: UPEM 2048, advance 937, bbox (173,444)..(765,1036).
// 실제 원본의 절대 배치 px를 핀하지 않는다. 원본 기호의 em 비율과 실제 잉크의 포함을 검사한다.
// 글꼴 프로그램·FontFace·로컬 글꼴 열거를 공급하지 않으며 설치된 OTF를 전제로 삼지 않는다.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync, rmSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {createSweepBrowserProfile, createSweepServer} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const require = createRequire(join(root, 'rhwp-studio/package.json'));

test('원 PUA 마커의 실제 Canvas 잉크는 대체 face 대신 독립 Print의 기호를 사용한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/source.hwp', ['application/octet-stream', readFileSync(join(root, 'samples/basic/sungeo.hwp'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'implicit-symbol');
  let browser;
  t.after(async () => {
    if (browser) await browser.close();
    rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
  });
  browser = await require('puppeteer-core').launch({
    executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir: profile,
  });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const rows = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const doc = new wasm.HwpDocument(new Uint8Array(await (await fetch('/source.hwp')).arrayBuffer()));
    try {
      doc.setHangul2024Compat(false);
      const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
      const ellipses = [], originalEllipse = ctx.ellipse.bind(ctx);
      ctx.ellipse = (...args) => {
        ellipses.push({args: [...args], fill: ctx.fillStyle});
        return originalEllipse(...args);
      };
      const rows = [];
      for (const output of ['print', 'high-quality']) {
        const layer = JSON.parse(doc.getPageLayerTreeWithProfile(4, output, true, true));
        let marker;
        const walk = value => {
          if (!value || typeof value !== 'object') return;
          if (value.type === 'textRun' && value.source?.stableSourceKey === 'section:1/para:63/char:0'
            && ['•', '\uf09f'].includes(value.text)) marker = value;
          if (Array.isArray(value)) value.forEach(walk); else Object.values(value).forEach(walk);
        };
        walk(layer);
        if (!marker) throw Error('원본 5쪽의 첫 머리 기호가 누락됐습니다.');
        ellipses.length = 0;
        doc.renderPageToCanvasFilteredWithProfile(4, canvas, 1, 'all', output);
        const size = marker.paintStyle.fontSize;
        const expected = {x: marker.bbox.x + size * 469 / 2048,
          y: marker.bbox.y + marker.baseline - size * 740 / 2048,
          rx: size * 296 / 2048, ry: size * 296 / 2048};
        const actual = ellipses.filter(e => e.fill === '#000080'
          && Math.abs(e.args[0] - expected.x) < size / 100
          && Math.abs(e.args[1] - expected.y) < size / 100);
        const rgba = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        const blue = [];
        // 기호 em의 상대 영역 안에서 실제 남색 잉크를 읽는다. 페이지 절대 좌표 핀이 아니다.
        for (let y = Math.floor(expected.y - size / 3); y <= Math.ceil(expected.y + size / 3); y++) {
          for (let x = Math.floor(expected.x - size / 3); x <= Math.ceil(expected.x + size / 3); x++) {
            const i = (y * canvas.width + x) * 4;
            if (rgba[i] < 24 && rgba[i + 1] < 24 && rgba[i + 2] > 80 && rgba[i + 2] < 180 && rgba[i + 3] > 200) blue.push([x, y]);
          }
        }
        rows.push({output, text: marker.text, width: marker.bbox.width, size, expected, actual, blue});
      }
      return rows;
    } finally { doc.free(); }
  });
  for (const row of rows) {
    assert.equal(row.text, '\uf09f', '표시 Unicode로 바꾸기 전에 원 기호 주소를 보존한다');
    // 레이아웃은 정수 HWPUNIT(96 DPI에서 1/75 CSSpx), 공개 JSON은 소수 셋째 자리다.
    // 이 두 단계의 양자화 한 칸만 허용하며 대체 face의 전각 폭은 허용하지 않는다.
    assert.ok(Math.abs(row.width - row.size * 937 / 2048) <= 1 / 75 + 0.001,
      `원 프로그램의 전진을 사용한다: ${JSON.stringify(row)}`);
    assert.equal(row.actual.length, 1, `실제 Canvas의 원 기호 윤곽이 정확히 한 번 그려져야 한다: ${JSON.stringify(row)}`);
    const [x, y, rx, ry] = row.actual[0].args;
    assert.ok(Math.abs(rx - row.size * 296 / 2048) <= 0.001);
    assert.ok(Math.abs(ry - row.size * 296 / 2048) <= 0.001);
    assert.ok(row.blue.length > 0, '그리기 명령만 기록하고 실제 캔버스를 비우면 실패한다');
    assert.ok(row.blue.every(([bx, by]) => Math.abs(bx + 0.5 - x) < rx + 1 && Math.abs(by + 0.5 - y) < ry + 1),
      '실제 기호 잉크는 독립 윤곽 내부와 래스터 경계에 포함된다');
  }
});
