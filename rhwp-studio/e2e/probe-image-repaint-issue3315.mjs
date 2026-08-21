/**
 * Issue #3315 Track 4 — 종결 측정.
 *
 * #2520 이 확정한 결함("그림 1장이 있으면 모든 편집이 그림 크기에 비례해 느려진다")의
 * 잔여 비용이 Track 1~3 이후 어디까지 내려왔는지 잰다. 진단용 벤치마크이며 CI 게이트가 아니다.
 *
 * 종결 기준(#3315 Track 4): **JPEG 1장 문서의 타이핑·드래그가 그림 없음 대비 ×2 이내.**
 * #2520 원측정: 그림 없음 1.76 ms/키, JPEG 2.0MB ×177(311 ms/키), 드래그 ~305 ms/이동(~3fps).
 * 기준과 비교값이 모두 브라우저 타이머 해상도 이하(0ms)이면 비율을 만들지 않고 통과로
 * 기록한다. 기준만 0ms인데 비교값이 검출되면 회귀로 실패한다.
 *
 * 측정 경로는 #2520 본문의 프로브를 그대로 쓴다.
 *   - 타이핑: textarea `input` 이벤트 실제 경로
 *   - 드래그: mousemove 당 `document-changed` 동기 emit (+ 개체 이동은 setProps 를 더한다)
 *
 * 실행:
 *   wasm-pack build --target web --release
 *   cd rhwp-studio && node e2e/probe-image-repaint-issue3315.mjs --mode=headless
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  closeBrowser, closePage, createPage, launchBrowser, loadApp, createNewDocument,
} from './helpers.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '../..');
const JPEG_PATH = path.join(REPO_ROOT, 'samples/images/tiger01.jpg');
const num = (name, def) => {
  const hit = process.argv.find((a) => a.startsWith(`--${name}=`));
  const v = hit ? Number(hit.split('=')[1]) : NaN;
  return Number.isFinite(v) && v > 0 ? Math.floor(v) : def;
};
// sub-ms 구간에서는 표본이 적으면 타이머 해상도가 배율을 지배한다. 기본값으로도 판정이
// 서지만, 경계에 걸리면 `--keys=200` 처럼 올려 다시 잰다.
const KEYS = num('keys', 20);
const REFRESHES = num('refreshes', 20);
const WARMUP = num('warmup', 5);

function arg(name, def) {
  const hit = process.argv.find((a) => a.startsWith(`--${name}=`));
  return hit ? hit.split('=')[1] : def;
}

/**
 * 그림을 실제 편집 라우터로 삽입한다(스냅샷 경로 — 실사용과 같다).
 *
 * 바이트는 페이지 안에서 dev 서버로 fetch 한다 — 4.5MB 를 `page.evaluate` 인자로 넘기면
 * 원소마다 JSON 직렬화가 일어나 측정 전에 수십 초를 태운다.
 */
async function insertJpeg(page, url, treatAsChar = false) {
  return page.evaluate(async ({ src, treatAsChar }) => {
    const resp = await fetch(src);
    if (!resp.ok) throw new Error(`픽스처 fetch 실패: HTTP ${resp.status}`);
    const bytes = new Uint8Array(await resp.arrayBuffer());
    const ih = window.__inputHandler;
    let info = null;
    ih.executeOperation({
      kind: 'snapshot', operationType: 'insertPicture',
      operation: () => {
        const ret = window.__wasm.insertPicture(
          0, 0, 0, '[]', bytes,
          // 2400x1800 원본을 페이지에 맞춘 크기(HWPUNIT)
          12000, 9000, 2400, 1800, 'jpg', '', null, null,
        );
        info = typeof ret === 'string' ? JSON.parse(ret) : ret;
        window.__wasm.setPictureProperties(0, info.paraIdx, info.controlIdx, { treatAsChar });
        return ih.getCursorPosition();
      },
    });
    return info;
  }, { src: url, treatAsChar });
}

/**
 * 브리지 메서드별 누적 시간을 잰다 (#2520 과 같은 계측 형식).
 * 어떤 WASM 호출이 비용을 먹는지 갈라 보여 준다.
 */
async function withBridgeProfile(page, fn) {
  await page.evaluate(() => {
    const w = window.__wasm;
    if (w.__profRestore) w.__profRestore();
    const acc = {}; const restores = [];
    const names = new Set([
      ...Object.getOwnPropertyNames(Object.getPrototypeOf(w) || {}),
      ...Object.getOwnPropertyNames(w),
    ]);
    for (const name of names) {
      if (name === 'constructor') continue;
      const orig = w[name];
      if (typeof orig !== 'function' || name.startsWith('__')) continue;
      restores.push(() => { w[name] = orig; });
      w[name] = function (...args) {
        const t0 = performance.now();
        try { return orig.apply(this, args); }
        finally { const d = performance.now() - t0; const e = acc[name] || (acc[name] = { ms: 0, n: 0 }); e.ms += d; e.n++; }
      };
    }
    window.__prof = acc;
    w.__profRestore = () => { restores.forEach(r => r()); delete w.__profRestore; };
  });
  const result = await fn();
  const prof = await page.evaluate(() => {
    const acc = window.__prof || {};
    window.__wasm.__profRestore?.();
    return Object.entries(acc)
      .map(([name, v]) => ({ name, ms: v.ms, n: v.n }))
      .sort((a, b) => b.ms - a.ms).slice(0, 6);
  });
  return { result, prof };
}

/** #2520 프로브 — textarea input 이벤트로 키스트로크당 비용을 잰다. */
async function measureTyping(page, n, warmup) {
  return page.evaluate(({ n, warmup }) => {
    const ih = window.__inputHandler;
    const fire = () => {
      ih.textarea.value = 'a';
      ih.textarea.dispatchEvent(new InputEvent('input', { data: 'a', inputType: 'insertText' }));
    };
    for (let i = 0; i < warmup; i++) fire();
    const samples = [];
    for (let i = 0; i < n; i++) {
      const t0 = performance.now();
      fire();
      samples.push(performance.now() - t0);
    }
    samples.sort((a, b) => a - b);
    const sum = samples.reduce((a, b) => a + b, 0);
    return { mean: sum / samples.length, median: samples[Math.floor(samples.length / 2)], max: samples[samples.length - 1] };
  }, { n, warmup });
}

/** 드래그 1틱의 공통 비용 — mousemove 마다 도는 `document-changed` 동기 emit. */
async function measureRefresh(page, n, warmup) {
  return page.evaluate(({ n, warmup }) => {
    const bus = window.__eventBus;
    for (let i = 0; i < warmup; i++) bus.emit('document-changed');
    const samples = [];
    for (let i = 0; i < n; i++) {
      const t0 = performance.now();
      bus.emit('document-changed');
      samples.push(performance.now() - t0);
    }
    samples.sort((a, b) => a - b);
    const sum = samples.reduce((a, b) => a + b, 0);
    return { mean: sum / samples.length, median: samples[Math.floor(samples.length / 2)], max: samples[samples.length - 1] };
  }, { n, warmup });
}

/** 개체 이동 1틱 전체 — getProps + setProps + document-changed (input-handler-picture 의 드래그 경로). */
async function measurePictureMove(page, info, n, warmup) {
  return page.evaluate(({ info, n, warmup }) => {
    const wasm = window.__wasm; const bus = window.__eventBus;
    const { paraIdx: ppi, controlIdx: ci } = info;
    const tick = (d) => {
      const props = wasm.getPictureProperties(0, ppi, ci);
      wasm.setPictureProperties(0, ppi, ci, {
        horzOffset: props.horzOffset + d, vertOffset: props.vertOffset,
      });
      bus.emit('document-changed');
    };
    for (let i = 0; i < warmup; i++) tick(75);
    const samples = [];
    for (let i = 0; i < n; i++) {
      const t0 = performance.now();
      tick(i % 2 === 0 ? 75 : -75);
      samples.push(performance.now() - t0);
    }
    samples.sort((a, b) => a - b);
    const sum = samples.reduce((a, b) => a + b, 0);
    return { mean: sum / samples.length, median: samples[Math.floor(samples.length / 2)], max: samples[samples.length - 1] };
  }, { info, n, warmup });
}

const fmt = (m) => `${m.mean.toFixed(2)} ms (median ${m.median.toFixed(2)}, max ${m.max.toFixed(2)})`;

/**
 * 무엇을 쟀는지 산출물에 남긴다.
 *
 * 이 프로브는 dev 서버가 서빙하는 번들을 잰다. 그 번들이 측정 대상 트리가 아니면 수치는
 * 조용히 무의미해진다 — 실제로 그렇게 잘못 잰 적이 있다(#3315 Track 4 초회 측정: 서버가 다른
 * 워크트리를 서빙해 Track 1~3 이전 상태를 쟀다). 트랙별 API 의 실재를 함께 적어 나중에
 * 산출물만 보고도 판별할 수 있게 한다.
 */
async function captureProvenance(page) {
  return page.evaluate(() => {
    const w = window.__wasm;
    const present = (n) => typeof w[n] === 'function';
    return {
      origin: location.origin,
      trackApis: {
        getPageFlowImageOps: present('getPageFlowImageOps'),
        getPageSourceImageKeys: present('getPageSourceImageKeys'),
        getSourceImageBytes: present('getSourceImageBytes'),
      },
    };
  });
}

async function main() {
  if (!existsSync(JPEG_PATH)) throw new Error(`픽스처 없음: ${JPEG_PATH}`);
  const jpeg = readFileSync(JPEG_PATH);
  console.log(`[fixture] ${path.basename(JPEG_PATH)} ${(jpeg.length / 1048576).toFixed(2)} MB`);

  const browser = await launchBrowser();
  const page = await createPage(browser, 1280, 900);
  const out = { fixtureBytes: jpeg.length, keys: KEYS, refreshes: REFRESHES };
  try {
    await loadApp(page);

    out.provenance = await captureProvenance(page);
    const missingApis = Object.entries(out.provenance.trackApis)
      .filter(([, ok]) => !ok).map(([n]) => n);
    console.log(`[served] ${out.provenance.origin}`);
    if (missingApis.length > 0) {
      out.provenanceWarning = `Track 3 API 부재: ${missingApis.join(', ')}`;
      console.log(`  !! ${out.provenanceWarning}`);
      console.log('     서빙되는 트리가 측정 대상이 맞는지 확인하십시오 — 이 상태의 수치는');
      console.log('     Track 1~3 이전을 잰 것일 수 있습니다.');
    } else {
      console.log('  Track 3 API 확인됨');
    }

    console.log('\n=== A. 그림 없음 (베이스라인) ===');
    await createNewDocument(page);
    out.baselineTyping = await measureTyping(page, KEYS, WARMUP);
    out.baselineRefresh = await measureRefresh(page, REFRESHES, WARMUP);
    console.log(`  타이핑        ${fmt(out.baselineTyping)}`);
    console.log(`  document-changed ${fmt(out.baselineRefresh)}`);

    console.log('\n=== B. JPEG 1장 ===');
    await createNewDocument(page);
    const info = await insertJpeg(page, '/samples/images/tiger01.jpg', false);
    if (!info) throw new Error('그림 삽입 실패');
    console.log(`  삽입됨 para=${info.paraIdx} ci=${info.controlIdx}`);
    const profiled = await withBridgeProfile(page, () => measureTyping(page, KEYS, WARMUP));
    out.jpegTyping = profiled.result;
    out.jpegBridgeProfile = profiled.prof;
    console.log('  [브리지 내역 — 타이핑 20타 누적]');
    for (const e of profiled.prof) {
      console.log(`    ${e.name.padEnd(32)} ${e.ms.toFixed(1)} ms / ${e.n}회 = ${(e.ms / e.n).toFixed(2)} ms`);
    }
    out.jpegRefresh = await measureRefresh(page, REFRESHES, WARMUP);
    out.jpegMove = await measurePictureMove(page, info, REFRESHES, WARMUP);
    console.log(`  타이핑        ${fmt(out.jpegTyping)}`);
    console.log(`  document-changed ${fmt(out.jpegRefresh)}`);
    console.log(`  개체 이동 1틱  ${fmt(out.jpegMove)}`);

    const compareAgainstBaseline = (measured, baseline) => {
      if (baseline > 0) {
        const ratio = measured / baseline;
        return { ratio, pass: ratio <= 2, display: `×${ratio.toFixed(2)}` };
      }
      if (measured <= 0) {
        return { ratio: null, pass: true, display: '해상도 이하 (양쪽 0.00 ms)' };
      }
      return { ratio: null, pass: false, display: '기준 0.00 ms 대비 측정값 검출' };
    };
    const typingComparison = compareAgainstBaseline(
      out.jpegTyping.mean, out.baselineTyping.mean,
    );
    const refreshComparison = compareAgainstBaseline(
      out.jpegRefresh.mean, out.baselineRefresh.mean,
    );
    out.ratioTyping = typingComparison.ratio;
    out.ratioRefresh = refreshComparison.ratio;
    out.typingComparison = typingComparison;
    out.refreshComparison = refreshComparison;
    out.moveFps = 1000 / out.jpegMove.mean;

    console.log('\n=== 종결 판정 (기준: ×2 이내) ===');
    console.log(`  타이핑 배율            ${typingComparison.display}  ${typingComparison.pass ? 'PASS' : 'FAIL'}`);
    console.log(`  document-changed 배율  ${refreshComparison.display}  ${refreshComparison.pass ? 'PASS' : 'FAIL'}`);
    console.log(`  개체 이동              ${out.moveFps.toFixed(1)} fps (#2520 원측정 ~3fps)`);
    out.verdict = typingComparison.pass && refreshComparison.pass ? 'PASS' : 'FAIL';
    console.log(`\n  종결 기준: ${out.verdict}`);

    console.log('');
    console.log('=== C. JPEG 1장 — 본문(flow) 배치 ===');
    await createNewDocument(page);
    const flowInfo = await insertJpeg(page, '/samples/images/tiger01.jpg', true);
    if (!flowInfo) throw new Error('flow 그림 삽입 실패');
    out.flowTyping = await measureTyping(page, KEYS, WARMUP);
    const flowTypingComparison = compareAgainstBaseline(
      out.flowTyping.mean, out.baselineTyping.mean,
    );
    out.ratioFlowTyping = flowTypingComparison.ratio;
    out.flowTypingComparison = flowTypingComparison;
    console.log(`  타이핑        ${fmt(out.flowTyping)}  (${flowTypingComparison.display})`);

    const dir = path.join(REPO_ROOT, 'output/issue-3315');
    mkdirSync(dir, { recursive: true });
    const file = path.join(dir, 'track4-closing-measurement.json');
    writeFileSync(file, JSON.stringify(out, null, 2), 'utf8');
    console.log(`\n[written] ${file}`);
  } finally {
    await closePage(page);
    await closeBrowser(browser);
  }
}

main().catch((e) => { console.error(e); process.exit(1); });
