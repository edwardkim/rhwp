/**
 * E2E 테스트 — 문서를 막 연 직후의 뷰포트 변화가 첫 쪽을 옮기지 않는다 (#6902)
 *
 * 쪽 맞춤에서 문서를 열면 화면이 한 프레임 위아래로 튄다. 사슬은 이렇다.
 *
 *   ① `loadBytes` 가 파싱 전에 **빈 쪽 자리표시자**를 한 쪽 크기로 놓는다.
 *      쪽 맞춤이면 그 한 쪽이 스크롤바 없이 뷰포트에 들어간다.
 *   ② 문서가 실려 여러 쪽이 되면 세로 스크롤바가 생기고 컨테이너 폭이 줄어든다
 *      (실측 1380 → 1365).
 *   ③ 그 폭 전이가 `ResizeObserver` → `viewport-resize` 를 깨우고,
 *      `CanvasView.onViewportResize()` 가 **중심 앵커**로 스크롤을 다시 잡는다.
 *   ④ 그런데 복원할 "이전 위치" 는 새 문서에 존재한 적이 없다 — `loadDocument` 가
 *      방금 `scrollTop = 0` 으로 맞춰 놓은 참이다. 앵커는 자리표시자의 기하를 새
 *      문서에 옮겨 적어 첫 쪽을 7px 밀어 올린다.
 *
 * ## 이 시험이 ②를 흉내 내지 않는 이유
 *
 * ②의 폭 전이는 **클래식 스크롤바**가 자리를 차지해야 생긴다. headless Chrome 은
 * 오버레이 스크롤바만 쓰므로(`--disable-features=OverlayScrollbar` 도, `::-webkit-
 * scrollbar` 크기 지정도 바꾸지 못한다 — 실측) 브라우저 안에서는 재현할 수 없다.
 *
 * 대신 **같은 결함 입력**을 결정적으로 만든다. ③~④가 요구하는 것은 "문서를 막 열어
 * 맨 위에 있는데 뷰포트가 바뀐다" 이지 스크롤바 자체가 아니다. 뷰포트를 줄이면
 * 종전 코드는 똑같이 중심 앵커를 적용해 **아무도 스크롤하지 않았는데 첫 쪽을
 * 끌어올린다**(실측 `scrollTop` 0 → 30, 첫 쪽 top 145 → 115).
 *
 * 검증 항목:
 * 1. 문서를 연 직후 뷰포트가 바뀌어도 `scrollTop` 은 0, 첫 쪽 위치는 그대로다
 * 2. **음성 대조** — 사용자가 한 번 스크롤한 뒤에는 중심 앵커가 그대로 살아 있다
 */

import { assert, runTest, setTestCase } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

/** 3쪽 문서 — 자리표시자(한 쪽)보다 길어야 ②의 형상이 성립한다. */
const DOC = 'biz_plan.hwp';

const WIDE = { width: 1400, height: 1000 };
/** 스크롤바 15px 폭 전이 + 쪽 맞춤이 다시 계산될 만큼의 높이 변화. */
const NARROW = { width: 1385, height: 940 };

/** 사용자 경로(open-document-bytes)로 문서를 연다. */
async function openDocument(page, filename) {
  const result = await page.evaluate(async (fname) => {
    const resp = await fetch(`/samples/${encodeURIComponent(fname)}`);
    if (!resp.ok) return { error: `HTTP ${resp.status}` };
    const bytes = new Uint8Array(await resp.arrayBuffer());
    const requestId = `issue6902-${Math.random().toString(36).slice(2)}`;
    const done = new Promise((resolve) => {
      const off = window.__eventBus.on('open-document-bytes:done', (payload) => {
        if (payload.requestId !== requestId) return;
        off();
        resolve(payload);
      });
    });
    window.__eventBus.emit('open-document-bytes', {
      bytes, fileName: fname, fileHandle: null, skipUnsavedGuard: true, requestId,
    });
    const outcome = await done;
    await new Promise((r) => setTimeout(r, 1200));
    return outcome;
  }, filename);
  if (result.error || result.ok === false) {
    throw new Error(`문서 열기 실패 (${filename}): ${result.error ?? '알 수 없음'}`);
  }
}

const readViewState = () => {
  const container = document.getElementById('scroll-container');
  const first = document.querySelector('.document-page-canvas, .page-placeholder');
  const rect = first ? first.getBoundingClientRect() : null;
  return {
    scrollTop: container.scrollTop,
    clientWidth: container.clientWidth,
    clientHeight: container.clientHeight,
    firstPageTop: rect ? Math.round(rect.top) : null,
  };
};

async function resizeTo(page, size) {
  await page.setViewport(size);
  await new Promise((r) => setTimeout(r, 700));
}

runTest('#6902 문서 로드 직후 뷰포트 변화가 첫 쪽을 옮기지 않는다', async ({ page }) => {
  await page.setViewport(WIDE);
  await page.waitForFunction(() => !!window.__eventBus, { timeout: 60000 });
  await page.click('#sb-zoom-fit');           // 쪽 맞춤
  await new Promise((r) => setTimeout(r, 600));

  // ── TC1: 로드 직후의 뷰포트 변화는 맨 위를 지킨다 ─────────────
  setTestCase('TC1 로드 직후 resize 가 스크롤을 옮기지 않는다');
  await openDocument(page, DOC);
  const loaded = await page.evaluate(readViewState);
  assert(loaded.scrollTop === 0,
    `TC1: 문서를 열면 맨 위에서 시작한다 (scrollTop=${loaded.scrollTop})`);

  await resizeTo(page, NARROW);
  const resized = await page.evaluate(readViewState);
  assert(resized.clientWidth < loaded.clientWidth || resized.clientHeight < loaded.clientHeight,
    `TC1: 뷰포트가 실제로 줄었다 (${loaded.clientWidth}×${loaded.clientHeight}`
    + ` → ${resized.clientWidth}×${resized.clientHeight})`);
  assert(resized.scrollTop === 0,
    `TC1: 아무도 스크롤하지 않았으므로 맨 위 그대로 (scrollTop=${resized.scrollTop})`);
  assert(resized.firstPageTop === loaded.firstPageTop,
    `TC1: 첫 쪽이 제자리다 (${loaded.firstPageTop} → ${resized.firstPageTop})`);

  // ── TC2: 음성 대조 — 스크롤한 뒤에는 중심 앵커가 살아 있다 ────
  setTestCase('TC2 스크롤한 뒤 resize 는 중심 앵커를 유지한다');
  await resizeTo(page, WIDE);
  await openDocument(page, DOC);
  await page.evaluate(() => {
    const container = document.getElementById('scroll-container');
    container.scrollTop = 600;
    container.dispatchEvent(new Event('scroll'));
  });
  await new Promise((r) => setTimeout(r, 500));
  const scrolled = await page.evaluate(readViewState);
  assert(scrolled.scrollTop === 600,
    `TC2: 사용자가 600 으로 스크롤했다 (scrollTop=${scrolled.scrollTop})`);

  await resizeTo(page, NARROW);
  const anchored = await page.evaluate(readViewState);
  const shrink = scrolled.clientHeight - anchored.clientHeight;
  assert(anchored.scrollTop === scrolled.scrollTop + shrink / 2,
    `TC2: 뷰포트 중심이 보존된다 (${scrolled.scrollTop} + ${shrink}/2`
    + ` = ${anchored.scrollTop})`);
});
