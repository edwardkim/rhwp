#!/usr/bin/env node

/**
 * Issue #4741 — Local Font Access가 설치 face를 누락해도 문서 후보 raw probe가 exact
 * Canvas2D face와 폭을 보존한다.
 *
 * KoPub바탕체 Light가 설치된 호스트 Chrome에서 실행한다. queryLocalFonts는 이 테스트 탭에서만
 * 의도적으로 부분 결과를 반환하도록 덮어쓰며 다른 탭과 실제 저장 글꼴 목록은 변경하지 않는다.
 */

import { assert, loadApp, loadHwpFile, runTest, setTestCase } from './helpers.mjs';

const FACE = 'KoPub바탕체 Light';
const TEST_TEXT = '행정업무운영 편람 KoPub바탕체 Light 0123456789 가나다라마';
const WIDTH_TOLERANCE_PX = 0.01;

runTest('Issue #4741 Local Font Access 부분 열거와 exact Canvas2D face', async ({ page }) => {
  await page.evaluateOnNewDocument(() => {
    Object.defineProperty(globalThis, 'queryLocalFonts', {
      configurable: true,
      value: async () => {
        globalThis.__issue4741QueryCount = (globalThis.__issue4741QueryCount ?? 0) + 1;
        return [{
          family: 'Issue4741 Enumerated Control',
          fullName: 'Issue4741 Enumerated Control Regular',
          postscriptName: 'Issue4741EnumeratedControl-Regular',
          style: 'Regular',
        }];
      },
    });
  });
  await loadApp(page);

  setTestCase('#4741 부분 열거 후보 probe와 patched/raw Canvas 폭');
  const result = await page.evaluate(async ({ face, text }) => {
    const localFonts = globalThis.__localFonts;
    if (!localFonts) throw new Error('개발용 local-font 진단 표면을 찾을 수 없습니다');
    const originalStoredSnapshot = localStorage.getItem('rhwp-local-fonts');
    await localFonts.clearStoredLocalFonts();

    const isolatedFrame = document.createElement('iframe');
    isolatedFrame.hidden = true;
    document.body.appendChild(isolatedFrame);
    const rawContext = isolatedFrame.contentDocument?.createElement('canvas').getContext('2d');
    if (!rawContext) throw new Error('격리된 raw Canvas2D context를 만들 수 없습니다');
    rawContext.font = `72px "${face}"`;
    const rawWidth = rawContext.measureText(text).width;
    const rawEffectiveFont = rawContext.font;

    const detected = await localFonts.detectLocalFonts({
      force: true,
      includeRegistered: true,
      candidateFamilies: [face, 'Issue4741 Missing Control'],
    });
    const state = localFonts.getLocalFontState();
    const record = localFonts.resolveLocalFont(face);

    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) throw new Error('patched Canvas2D context를 만들 수 없습니다');
    context.font = `72px "${face}"`;
    const patchedWidth = context.measureText(text).width;
    const patchedEffectiveFont = context.font;

    const queryCountAfterFirstDetection = globalThis.__issue4741QueryCount ?? 0;
    await localFonts.detectLocalFonts({ candidateFamilies: [face] });
    const queryCountAfterCachedDetection = globalThis.__issue4741QueryCount ?? 0;

    const result = {
      rawWidth,
      patchedWidth,
      widthDelta: Math.abs(rawWidth - patchedWidth),
      rawEffectiveFont,
      patchedEffectiveFont,
      detected,
      state,
      record,
      queryCountAfterFirstDetection,
      queryCountAfterCachedDetection,
    };
    if (originalStoredSnapshot === null) {
      localStorage.removeItem('rhwp-local-fonts');
    } else {
      localStorage.setItem('rhwp-local-fonts', originalStoredSnapshot);
    }
    return result;
  }, { face: FACE, text: TEST_TEXT });

  assert(result.queryCountAfterFirstDetection === 1, 'Local Font Access는 최초 감지에서 한 번만 호출');
  assert(result.queryCountAfterCachedDetection === 1, '같은 감지 세대의 후보는 cache에서 재사용');
  assert(result.detected.includes(FACE), '열거에서 빠진 KoPub exact face를 raw probe로 확인');
  assert(result.record?.detectionSource === 'font-presence-probe', 'probe-only provenance 보존');
  assert(result.state.probedFamilies.includes(FACE), 'snapshot에 probe-positive 후보 기록');
  assert(result.state.unresolvedFamilies.includes('Issue4741 Missing Control'), '음성 후보도 확인 완료로 기록');
  assert(result.patchedEffectiveFont.includes(FACE), 'patched Canvas가 exact KoPub face를 보존');
  assert(!/GulimChe|monospace/u.test(result.patchedEffectiveFont), 'KoPub바탕체를 고정폭 fallback으로 분류하지 않음');
  assert(
    result.widthDelta <= WIDTH_TOLERANCE_PX,
    `patched/raw Canvas 폭 일치: delta=${result.widthDelta}px`,
  );

  console.log(JSON.stringify(result, null, 2));

  setTestCase('#4741 편람 HWP 물리 11쪽 exact face와 페이지 경계');
  await page.evaluate(() => {
    const prototype = CanvasRenderingContext2D.prototype;
    const descriptor = Object.getOwnPropertyDescriptor(prototype, 'font');
    if (!descriptor?.get || !descriptor.set) throw new Error('patched Canvas font descriptor가 없습니다');
    globalThis.__issue4741CanvasFonts = [];
    Object.defineProperty(prototype, 'font', {
      configurable: true,
      enumerable: descriptor.enumerable,
      get() {
        return descriptor.get.call(this);
      },
      set(value) {
        descriptor.set.call(this, value);
        if (/KoPub/u.test(String(value))) {
          globalThis.__issue4741CanvasFonts.push({
            requested: String(value),
            effective: descriptor.get.call(this),
          });
        }
      },
    });
  });
  const sample = await loadHwpFile(page, '2025 행정업무운영 편람(최종).hwp');
  await page.evaluate(() => {
    const view = globalThis.__canvasView;
    const container = document.querySelector('#scroll-container');
    const pageOffset = view?.virtualScroll?.getPageOffset?.(10);
    if (!container || !Number.isFinite(pageOffset)) throw new Error('물리 11쪽 offset을 찾을 수 없습니다');
    globalThis.__issue4741CanvasFonts = [];
    container.scrollTop = pageOffset;
    container.dispatchEvent(new Event('scroll'));
  });
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 1500)));
  const sampleResult = await page.evaluate(() => ({
    pageCount: globalThis.__wasm?.pageCount,
    backend: globalThis.__canvasView?.getRenderBackend?.(),
    fonts: globalThis.__issue4741CanvasFonts ?? [],
  }));
  const exactLightFonts = sampleResult.fonts.filter(item => /KoPub바탕체 Light/u.test(item.requested));
  assert(sample.pageCount === 383 && sampleResult.pageCount === 383, '편람 HWP 페이지 경계는 383쪽 유지');
  assert(sampleResult.backend === 'canvas2d', '편람 검증 backend는 Canvas2D');
  assert(exactLightFonts.length > 0, '물리 11쪽 렌더가 KoPub바탕체 Light를 요청');
  assert(
    exactLightFonts.every(item => item.effective.includes(FACE) && !/GulimChe|monospace/u.test(item.effective)),
    '물리 11쪽 KoPub바탕체 Light가 exact proportional serif chain으로 렌더',
  );
  console.log(JSON.stringify({
    pageCount: sampleResult.pageCount,
    backend: sampleResult.backend,
    exactLightAssignments: exactLightFonts.length,
    representativeFont: exactLightFonts[0] ?? null,
  }, null, 2));
}, { skipLoadApp: true });
