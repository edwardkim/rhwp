/**
 * [#4694] 차트 데이터 편집 UI — 본선 e2e.
 *
 * 수용 기준 실측: 컨텍스트 메뉴 노출 → 더블클릭 진입 → 그리드 값 수정 → [확인] →
 * 재조회에 새 값 → Ctrl+Z 원복(스냅샷 undo 의 bin 바이트 복원, 계획서 R1) →
 * 무편집 [확인] 무흔적. 헬퍼는 #3682 프로브의 oleClickPoint 계약을 그대로 쓴다.
 */
import { runTest, loadHwpFile, screenshot } from './helpers.mjs';

const SAMPLE = 'chart/세로막대형/묶은세로막대형.hwp';
const SENTINEL = '91.7'; // #4055 — 원본 최대값 5 라 반영되면 첫 막대가 솟는다

async function pause(page, ms = 300) {
  await page.evaluate(d => new Promise(r => setTimeout(r, d)), ms);
}

/**
 * 실제 dblclick 이벤트를 만드는 제스처. 이 CDP 환경에서 `click({clickCount:2})` 는
 * (연쇄로도) dblclick 을 합성하지 못함을 실측했다 — down/up 4연타만 발생시킨다.
 */
async function doubleClick(page, pt) {
  await page.mouse.move(pt.x, pt.y);
  await page.mouse.down(); await page.mouse.up();
  await page.mouse.down({ clickCount: 2 }); await page.mouse.up({ clickCount: 2 });
}

/** 차트 OLE 레이아웃 좌표를 화면 클릭 좌표로 변환 (#3682 프로브와 동일 계약) */
async function oleClickPoint(page) {
  return page.evaluate(() => {
    const layout = window.__wasm.getPageControlLayout(0);
    const ole = (layout?.controls || []).find(c => c.type === 'ole');
    if (!ole) return null;
    const el = document.querySelector('#scroll-content');
    const rect = el.getBoundingClientRect();
    const scale = window.__canvasView?.scale ?? 1;
    return {
      x: rect.left + (ole.x + ole.w / 2) * scale,
      y: rect.top + (ole.y + ole.h / 2) * scale,
    };
  });
}

const firstChartValue = (page) => page.evaluate(() => {
  const d = window.__wasm.getChartDataByIndex(0);
  return d?.ok ? d.series[0].values[0] : null;
});

const dialogOpen = (page) => page.evaluate(() =>
  !!document.querySelector('.chart-data-grid'));

runTest('#4694 차트 데이터 편집 — 메뉴·더블클릭·편집·undo·무흔적', async ({ page }) => {
  const info = await loadHwpFile(page, SAMPLE);
  console.log(`문서 로드: ${info.pageCount}쪽`);

  const pt = await oleClickPoint(page);
  if (!pt) throw new Error('ole 레이아웃 부재 — 샘플/렌더 전제가 깨졌다');

  const original = await firstChartValue(page);
  if (original === null) throw new Error('getChartDataByIndex(0) 읽기 실패');
  console.log(`원본 첫 값: ${original}`);

  // ── 1. 컨텍스트 메뉴에 항목이 노출된다 ──
  await page.mouse.click(pt.x, pt.y);
  await pause(page, 400);
  await page.mouse.click(pt.x, pt.y, { button: 'right' });
  await pause(page, 500);
  const menuHasItem = await page.evaluate(() =>
    [...document.querySelectorAll('*')].some(el =>
      el.childElementCount === 0 && (el.textContent || '').includes('차트 데이터 편집')));
  if (!menuHasItem) throw new Error('컨텍스트 메뉴에 "차트 데이터 편집..." 항목이 없다');
  console.log('컨텍스트 메뉴 항목: 노출');
  await screenshot(page, '4694-1-context-menu');
  await page.keyboard.press('Escape');
  await pause(page, 300);

  // ── 2. 더블클릭으로 다이얼로그가 열린다 ──
  await page.mouse.click(pt.x, pt.y);
  await pause(page, 300);
  await doubleClick(page, pt);
  await pause(page, 700);
  if (!(await dialogOpen(page))) throw new Error('더블클릭에 다이얼로그가 열리지 않았다');
  console.log('더블클릭 진입: 열림');
  await screenshot(page, '4694-2-dialog-open');

  // ── 3. 첫 값 수정 → [확인] → 재조회 반영 ──
  await page.evaluate((v) => {
    const input = document.querySelector('.chart-data-grid tbody td input');
    input.focus();
    input.value = v;
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }, SENTINEL);
  await page.click('.dialog-btn-primary');
  await pause(page, 800);
  if (await dialogOpen(page)) throw new Error('[확인] 후에도 다이얼로그가 남아 있다');
  const edited = await firstChartValue(page);
  if (edited !== SENTINEL) throw new Error(`편집 미반영 — 재조회 값 ${edited}`);
  console.log(`편집 반영: ${original} → ${edited}`);
  await screenshot(page, '4694-3-edited');

  // ── 4. Ctrl+Z 원복 — 스냅샷 undo 의 bin 바이트 복원 (R1) ──
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Control');
  await pause(page, 800);
  const undone = await firstChartValue(page);
  if (undone !== original) throw new Error(`undo 미복원 — 값 ${undone} (기대 ${original})`);
  console.log(`undo 원복: ${edited} → ${undone}`);
  await screenshot(page, '4694-4-undo');

  // ── 5. 무편집 [확인] 은 무흔적 ──
  await page.mouse.click(pt.x, pt.y);
  await pause(page, 300);
  await doubleClick(page, pt);
  await pause(page, 700);
  if (!(await dialogOpen(page))) throw new Error('재진입 실패');
  await page.click('.dialog-btn-primary');
  await pause(page, 600);
  if (await dialogOpen(page)) throw new Error('무편집 [확인] 후에도 다이얼로그가 남아 있다');
  const untouched = await firstChartValue(page);
  if (untouched !== original) throw new Error(`무편집인데 값이 바뀌었다 — ${untouched}`);
  console.log('무편집 [확인]: 무흔적');

  console.log('\n=== #4694 e2e 전 단계 통과 ===');
});
