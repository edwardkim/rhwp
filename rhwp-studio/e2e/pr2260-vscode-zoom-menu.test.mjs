/**
 * [PR #2260 검증] rhwp-vscode 배율 메뉴 — 호스트 Chrome CDP 로 webview 하네스 구동.
 *
 * 매뉴얼: mydocs/manual/e2e-cdp.md (mirrored 모드, CHROME_CDP=http://localhost:19222,
 * runTest + skipLoadApp 패턴). 하네스: 확장 provider 의 webview HTML 을 추출해
 * acquireVsCodeApi 스텁 + 문서 자동 로드로 standalone 화 (http://localhost:7701).
 * 컨트리뷰터 시나리오 7건 중 F5 전용(사이드바 토글)을 제외한 6건 + 메뉴 UX 2건.
 */
import { runTest, assert, screenshot, setTestCase } from "./helpers.mjs";

const HARNESS = process.env.HARNESS_URL || "http://localhost:7701/harness.html";

runTest("PR #2260 vscode 배율 메뉴", async ({ page }) => {
  await page.setViewport({ width: 1280, height: 900 });

  setTestCase("하네스 로드");
  await page.goto(HARNESS, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(
    () => window.__msgs?.some((m) => m.type === "loaded"),
    { timeout: 30000 },
  );
  const pageCount = await page.evaluate(
    () => window.__msgs.find((m) => m.type === "loaded").pageCount,
  );
  assert(pageCount >= 1, `문서 로드 (${pageCount}쪽)`);

  const state = () =>
    page.evaluate(() => {
      const sc = document.getElementById("scroll-container");
      return {
        label: document.getElementById("stb-zoom-label")?.textContent?.trim(),
        hScroll: sc.scrollWidth > sc.clientWidth + 1,
        vScroll: sc.scrollHeight > sc.clientHeight + 1,
        popupHidden: document.getElementById("stb-zoom-popup")?.hidden,
      };
    });

  async function clickMenu(key) {
    await page.click("#stb-zoom-menu");
    await page.waitForFunction(
      () => !document.getElementById("stb-zoom-popup").hidden,
    );
    await page.click(
      `#stb-zoom-popup .stb-popup-item[data-mode="${key}"], #stb-zoom-popup .stb-popup-item[data-zoom="${key}"]`,
    );
    await page.evaluate(() => new Promise((r) => setTimeout(r, 400)));
  }

  setTestCase("시나리오 1 — 두 쪽 맞춤");
  await clickMenu("fitSpread");
  let s = await state();
  await screenshot(page, "pr2260-01-fitspread");
  assert(!s.hScroll, `두 쪽 맞춤 시 가로 스크롤 없음 (label=${s.label})`);

  setTestCase("시나리오 2 — 폭 맞춤");
  await clickMenu("fitWidth");
  s = await state();
  await screenshot(page, "pr2260-02-fitwidth");
  assert(!s.hScroll, `폭 맞춤 시 가로 스크롤 없음 (label=${s.label})`);
  assert(pageCount <= 1 || s.vScroll, "폭 맞춤 시 세로 스크롤 존재(다쪽)");

  setTestCase("시나리오 3 — 쪽 맞춤");
  await clickMenu("fitPage");
  s = await state();
  const fitsOne = await page.evaluate(() => {
    const sc = document.getElementById("scroll-container");
    const p = document.querySelector("#scroll-content > *");
    if (!p) return false;
    const r = p.getBoundingClientRect();
    return r.height <= sc.clientHeight + 2 && r.width <= sc.clientWidth + 2;
  });
  await screenshot(page, "pr2260-03-fitpage");
  assert(fitsOne, `쪽 맞춤 시 첫 쪽 전체가 뷰포트 이내 (label=${s.label})`);

  setTestCase("시나리오 4 — 리사이즈 추종");
  // fitPage 는 세로 문서에서 높이 제약(min(fitW,fitH))이므로 폭 변화만으로는
  // 불변이 정답. 폭 추종은 fitWidth 로 검증한다.
  await clickMenu("fitWidth");
  const before = (await state()).label;
  await page.setViewport({ width: 900, height: 900 });
  await page.evaluate(() => new Promise((r) => setTimeout(r, 700)));
  const after = (await state()).label;
  assert(before !== after, `fitWidth 폭 리사이즈 추종 (${before} → ${after})`);
  // fitPage + 높이 변화 추종도 확인
  await clickMenu("fitPage");
  const beforeH = (await state()).label;
  await page.setViewport({ width: 900, height: 600 });
  await page.evaluate(() => new Promise((r) => setTimeout(r, 700)));
  const afterH = (await state()).label;
  assert(beforeH !== afterH, `fitPage 높이 리사이즈 추종 (${beforeH} → ${afterH})`);
  await page.setViewport({ width: 1280, height: 900 });
  await page.evaluate(() => new Promise((r) => setTimeout(r, 700)));

  setTestCase("시나리오 6 — 수동 전환 시 맞춤 해제");
  await page.click("#stb-zoom-out");
  await page.evaluate(() => new Promise((r) => setTimeout(r, 300)));
  const b2 = (await state()).label;
  await page.setViewport({ width: 1100, height: 900 });
  await page.evaluate(() => new Promise((r) => setTimeout(r, 700)));
  const a2 = (await state()).label;
  assert(b2 === a2, `수동 전환 후 리사이즈에 배율 고정 (${b2} → ${a2})`);
  await page.setViewport({ width: 1280, height: 900 });

  setTestCase("시나리오 7 — 배율 진동 감시");
  await clickMenu("fitPage");
  const labels = [];
  for (let i = 0; i < 10; i++) {
    labels.push((await state()).label);
    await page.evaluate(() => new Promise((r) => setTimeout(r, 200)));
  }
  assert(
    new Set(labels).size === 1,
    `2초간 배율 안정 (${[...new Set(labels)].join(",")})`,
  );

  setTestCase("메뉴 UX — 체크 표시 + Esc");
  await page.click("#stb-zoom-menu");
  await page.waitForFunction(
    () => !document.getElementById("stb-zoom-popup").hidden,
  );
  const checked = await page.evaluate(() =>
    [...document.querySelectorAll("#stb-zoom-popup .stb-popup-item")]
      .filter((i) => i.querySelector(".stb-check")?.textContent?.trim())
      .map((i) => i.dataset.mode ?? i.dataset.zoom),
  );
  await page.keyboard.press("Escape");
  await page.evaluate(() => new Promise((r) => setTimeout(r, 200)));
  s = await state();
  assert(
    checked.length === 1 && checked[0] === "fitPage",
    `현재 모드 체크 표시 (${checked})`,
  );
  assert(s.popupHidden, "Esc 로 팝업 닫힘");
  await screenshot(page, "pr2260-04-final");
}, { skipLoadApp: true });
