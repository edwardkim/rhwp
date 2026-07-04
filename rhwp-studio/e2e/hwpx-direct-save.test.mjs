/**
 * HWPX 직접 저장 (file:save) E2E — #1532
 *
 * 검증: HWPX 출처 문서에서 file:save(Ctrl+S) 가
 *   1. 베타 비활성 alert 를 더 이상 띄우지 않음
 *   2. application/hwp+zip blob(HWPX PK 매직)을 생성
 *   3. 그 저장본을 다시 loadDocument 했을 때 페이지 수가 일치(정상 재오픈)
 *
 * 실행: node e2e/hwpx-direct-save.test.mjs --mode=headless
 */
import { runTest, assert, loadHwpFile } from './helpers.mjs';

const SAMPLE = 'hwpx/footnote-01.hwpx';

runTest('HWPX 직접 저장: file:save → HWPX 생성 + 재오픈', async ({ page }) => {
  page.on('console', (msg) => {
    const t = msg.text();
    if (t.includes('[file:save') || t.includes('저장') || t.includes('save')) console.log(`  [page] ${t}`);
  });
  // 0) HWPX 로드 → sourceFormat=hwpx
  console.log(`\n[0] HWPX 로드 (${SAMPLE})`);
  const load = await loadHwpFile(page, SAMPLE);
  const before = load.pageCount;
  console.log(`    페이지 수: ${before}`);
  assert(before >= 1, `페이지 수 1 이상 (current: ${before})`);

  const fmt = await page.evaluate(() => window.__wasm?.getSourceFormat?.());
  console.log(`    sourceFormat: ${fmt}`);
  assert(fmt === 'hwpx', `sourceFormat 은 hwpx 여야 함 (current: ${fmt})`);

  // 1) 훅: alert 캡처 / FS Access API 제거(폴백 강제) / blob 캡처 / 실제 다운로드 차단
  console.log('\n[1] 저장 경로 훅 설치 (alert·showSaveFilePicker·createObjectURL·click)');
  await page.evaluate(() => {
    window.__alerts = [];
    window.alert = (m) => { window.__alerts.push(String(m)); };
    try { delete window.showSaveFilePicker; } catch { window.showSaveFilePicker = undefined; }
    window.__savedBlob = null;
    URL.createObjectURL = (blob) => { window.__savedBlob = blob; return 'blob:captured'; };
    HTMLAnchorElement.prototype.click = function () { /* no-op: 실제 다운로드 차단 */ };
  });

  // 2) file:save 트리거 — 메뉴바 파일 열기(mousedown→updateMenuStates) → 저장 클릭
  console.log('\n[2] file:save 트리거 (메뉴: 파일 → 저장)');
  const triggered = await page.evaluate(() => {
    const fileItem = [...document.querySelectorAll('#menu-bar .menu-item')]
      .find((el) => (el.textContent || '').includes('파일'));
    const title = fileItem?.querySelector('.menu-title');
    if (!title) return { ok: false, reason: '파일 메뉴 타이틀 없음' };
    // 메뉴는 .menu-title 의 mousedown 으로 열리며 그때 updateMenuStates 가 실행된다.
    title.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
    const pageCount = window.__wasm?.pageCount;
    const saveItem = document.querySelector('.md-item[data-cmd="file:save"]');
    if (!saveItem) return { ok: false, reason: 'file:save 메뉴 항목 없음', pageCount };
    const disabled = saveItem.classList.contains('disabled');
    saveItem.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
    return { ok: true, disabled, pageCount };
  });
  console.log(`    트리거: ${JSON.stringify(triggered)}`);
  assert(triggered.ok, `저장 메뉴 항목을 찾아야 함 (${triggered.reason || ''})`);
  assert(!triggered.disabled, '저장 메뉴 항목이 비활성(disabled) 이면 안 됨');
  await page.evaluate(() => new Promise((r) => setTimeout(r, 2000)));

  // 3) 결과 수집 + 저장본 재오픈
  console.log('\n[3] 저장 결과 수집 + 저장본 재오픈');
  const result = await page.evaluate(async () => {
    const alerts = window.__alerts || [];
    if (!window.__savedBlob) return { alerts, captured: false };
    const buf = await window.__savedBlob.arrayBuffer();
    const head = Array.from(new Uint8Array(buf.slice(0, 4)));
    const type = window.__savedBlob.type;
    const len = buf.byteLength;
    // 저장본 재오픈
    const info = window.__wasm?.loadDocument(new Uint8Array(buf), 'saved.hwpx');
    return { alerts, captured: true, head, type, len, reopenPages: info?.pageCount };
  });

  console.log(`    alerts=${JSON.stringify(result.alerts)}`);
  console.log(`    captured=${result.captured} head=[${result.head}] type=${result.type} len=${result.len}`);
  console.log(`    재오픈 페이지수=${result.reopenPages} (원본 ${before})`);

  // 단언
  assert(result.alerts.length === 0,
    `베타 비활성 alert 가 없어야 함 (alerts: ${JSON.stringify(result.alerts)})`);
  assert(result.captured, 'file:save 가 저장 blob 을 생성해야 함 (캡처 실패)');
  assert(result.head[0] === 0x50 && result.head[1] === 0x4B && result.head[2] === 0x03 && result.head[3] === 0x04,
    `저장본은 HWPX PK\\x03\\x04 매직 (current head: ${result.head})`);
  assert(result.type === 'application/hwp+zip',
    `blob type 은 application/hwp+zip (current: ${result.type})`);
  assert(result.len > 0, `저장본 길이 > 0 (current: ${result.len})`);
  assert(result.reopenPages === before,
    `저장본 재오픈 페이지수 일치 (${result.reopenPages} vs ${before})`);

  console.log('\n✅ HWPX 직접 저장 + 재오픈 검증 통과');
});
