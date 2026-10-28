/**
 * [#6053] 차트 행·열·라벨 구조 편집 UI — 본선 e2e.
 *
 * 수용 기준 실측: 그리드 셀 우클릭 → 행 추가 → [확인] → 재조회에 늘어난 행 →
 * Ctrl+Z 원복 → 무편집 [확인] 무흔적 → ESC 는 메뉴만 닫는다 → 종류별 사전 판정
 * (원형은 안내, 주식형 양끝은 비활성).
 *
 * `gymContract` 를 내보내지 않는다 — 이 파일이 재는 것은 전부 UI 계약(우클릭 메뉴·
 * 사전 비활성·모달 위 ESC)이라 CLI 로 표현할 수 없다. #4694 가 같은 이유로 UI 계약을
 * 계약에 넣지 않은 것과 같은 판단이다.
 */
import { runTest, loadHwpFile, screenshot } from './helpers.mjs';

const BAR = 'chart/세로막대형/묶은세로막대형.hwp';
const PIE = 'chart/원형/2차원원형.hwp';
const STOCK = 'chart/기타/시가고가저가종가.hwp';

async function pause(page, ms = 300) {
  await page.evaluate((d) => new Promise((r) => setTimeout(r, d)), ms);
}

/**
 * 첫 실행 스킨 선택 대화상자를 닫는다. 새 브라우저 프로필(headless)에서는 이 모달이
 * 편집 영역을 덮어 캔버스 클릭이 카드에 먹힌다 — undo-depth-issue5769 와 같은 처리다.
 */
async function dismissSkinOnboarding(page) {
  await page.evaluate(() => {
    const anyCard = document.querySelector('.skin-onboarding-card');
    if (anyCard) anyCard.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    const ok = [...document.querySelectorAll('button.dialog-btn-primary')]
      .find((x) => x.offsetParent !== null);
    if (ok) {
      ok.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      ok.click();
    }
  });
  await pause(page, 600);
}

/** #4694 실측: 이 CDP 환경에서 click({clickCount:2}) 는 dblclick 을 합성하지 못한다. */
async function doubleClick(page, pt) {
  await page.mouse.move(pt.x, pt.y);
  await page.mouse.down(); await page.mouse.up();
  await page.mouse.down({ clickCount: 2 }); await page.mouse.up({ clickCount: 2 });
}

/** 차트 OLE 레이아웃 좌표를 화면 클릭 좌표로 (#3682 프로브와 동일 계약) */
async function oleClickPoint(page) {
  return page.evaluate(() => {
    const layout = window.__wasm.getPageControlLayout(0);
    const ole = (layout?.controls || []).find((c) => c.type === 'ole');
    if (!ole) return null;
    const rect = document.querySelector('#scroll-content').getBoundingClientRect();
    const scale = window.__canvasView?.scale ?? 1;
    return {
      x: rect.left + (ole.x + ole.w / 2) * scale,
      y: rect.top + (ole.y + ole.h / 2) * scale,
    };
  });
}

const dialogOpen = (page) => page.evaluate(() => !!document.querySelector('.chart-data-grid'));

const chartShape = (page) => page.evaluate(() => {
  const d = window.__wasm.getChartDataByIndex(0);
  if (!d?.ok) return null;
  return {
    series: d.series.length,
    rows: d.series[0].values.length,
    first: d.series[0].values[0],
    labels: (d.labels || []).length,
    plot: d.plot,
    hasUpDownBars: d.hasUpDownBars,
  };
});

async function openDialog(page) {
  const pt = await oleClickPoint(page);
  if (!pt) throw new Error('ole 레이아웃 부재 — 샘플/렌더 전제가 깨졌다');
  await page.mouse.click(pt.x, pt.y);
  await pause(page, 300);
  await doubleClick(page, pt);
  await pause(page, 700);
  if (!(await dialogOpen(page))) throw new Error('다이얼로그가 열리지 않았다');
  return pt;
}

/** 그리드 셀에 우클릭을 실제 이벤트로 보낸다. */
async function rightClickCell(page, row, series) {
  return page.evaluate((r, s) => {
    const td = document.querySelector(`.chart-data-grid td[data-row="${r}"][data-series="${s}"]`);
    if (!td) return false;
    const box = td.getBoundingClientRect();
    td.dispatchEvent(new MouseEvent('contextmenu', {
      bubbles: true, cancelable: true,
      clientX: Math.round(box.left + box.width / 2),
      clientY: Math.round(box.top + box.height / 2),
    }));
    return true;
  }, row, series);
}

/** 열린 로컬 메뉴의 항목 — 문구·비활성·사유. */
const menuItems = (page) => page.evaluate(() =>
  [...document.querySelectorAll('.context-menu .md-item')].map((el) => ({
    label: (el.textContent || '').trim(),
    disabled: el.classList.contains('disabled'),
    title: el.title || '',
  })));

async function clickMenuItem(page, label) {
  const hit = await page.evaluate((want) => {
    const row = [...document.querySelectorAll('.context-menu .md-item')]
      .find((el) => (el.textContent || '').trim() === want);
    if (!row || row.classList.contains('disabled')) return false;
    row.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
    return true;
  }, label);
  if (!hit) throw new Error(`메뉴 항목을 누르지 못했다: ${label}`);
  await pause(page, 300);
}

runTest('#6053 차트 구조 편집 — 우클릭 행 추가·undo·무흔적·ESC·종류별 사전 판정', async ({ page }) => {
  // ── 1. 본선: 묶은세로막대형에서 행을 넣는다 ──
  const info = await loadHwpFile(page, BAR);
  await dismissSkinOnboarding(page);
  console.log(`문서 로드: ${info.pageCount}쪽`);

  const before = await chartShape(page);
  if (!before) throw new Error('getChartDataByIndex(0) 읽기 실패');
  console.log(`원본: 계열 ${before.series} · 행 ${before.rows} · plot=${before.plot}`);
  // [#6037] 봉투에 새 필드가 실려 있어야 사전 판정이 성립한다. undefined 면 pkg 가 낡은 것이다.
  if (before.plot === undefined || before.hasUpDownBars === undefined) {
    throw new Error('봉투에 plot/hasUpDownBars 가 없다 — pkg/ 를 다시 빌드해야 한다(#6037)');
  }

  await openDialog(page);
  await screenshot(page, '6053-1-dialog-open');

  if (!(await rightClickCell(page, 0, 0))) throw new Error('그리드 셀에 data-row/data-series 가 없다');
  await pause(page, 300);
  const barMenu = await menuItems(page);
  console.log(`메뉴 항목 ${barMenu.length}종: ${barMenu.map((i) => i.label).join(' / ')}`);
  if (barMenu.length !== 6) throw new Error(`메뉴 항목이 6종이 아니다 — ${barMenu.length}`);
  await screenshot(page, '6053-2-context-menu');

  await clickMenuItem(page, '아래에 행 추가');
  const gridRows = await page.evaluate(() =>
    document.querySelectorAll('.chart-data-grid tbody tr').length);
  if (gridRows !== before.rows + 1) throw new Error(`그리드 행이 안 늘었다 — ${gridRows}`);
  console.log(`그리드 행: ${before.rows} → ${gridRows}`);

  await page.click('.dialog-btn-primary');
  await pause(page, 900);
  if (await dialogOpen(page)) throw new Error('[확인] 후에도 다이얼로그가 남아 있다');

  const after = await chartShape(page);
  if (after.rows !== before.rows + 1) throw new Error(`행 추가 미반영 — ${after.rows}`);
  if (after.labels !== before.labels + 1) throw new Error(`라벨이 함께 늘지 않았다 — ${after.labels}`);
  if (after.first !== before.first) throw new Error(`손대지 않은 값이 바뀌었다 — ${after.first}`);
  console.log(`저장본 반영: 행 ${before.rows} → ${after.rows}, 라벨 ${after.labels}`);
  await screenshot(page, '6053-3-row-added');

  // ── 2. Ctrl+Z 원복 — 스냅샷 undo 가 bin_data_content 를 복원한다 ──
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Control');
  await pause(page, 900);
  const undone = await chartShape(page);
  if (undone.rows !== before.rows) throw new Error(`undo 미복원 — 행 ${undone.rows}`);
  if (undone.series !== before.series) throw new Error(`undo 미복원 — 계열 ${undone.series}`);
  console.log(`undo 원복: 행 ${after.rows} → ${undone.rows}`);
  await screenshot(page, '6053-4-undo');

  // ── 3. 무편집 [확인] 은 무흔적 ──
  await openDialog(page);
  await page.click('.dialog-btn-primary');
  await pause(page, 700);
  if (await dialogOpen(page)) throw new Error('무편집 [확인] 후에도 다이얼로그가 남아 있다');
  const untouched = await chartShape(page);
  if (untouched.rows !== before.rows || untouched.first !== before.first) {
    throw new Error(`무편집인데 바뀌었다 — 행 ${untouched.rows}, 첫 값 ${untouched.first}`);
  }
  console.log('무편집 [확인]: 무흔적');

  // ── 4. ESC 는 메뉴만 닫는다 — 다이얼로그는 남는다 ──
  // ModalDialog 는 document capture 에 ESC=닫기를 건다. 메뉴가 window capture 에서
  // 먼저 먹고 전파를 끊어야 이 단언이 선다.
  await openDialog(page);
  await rightClickCell(page, 0, 0);
  await pause(page, 300);
  if ((await menuItems(page)).length === 0) throw new Error('메뉴가 열리지 않았다');
  await page.keyboard.press('Escape');
  await pause(page, 400);
  if ((await menuItems(page)).length !== 0) throw new Error('ESC 에 메뉴가 닫히지 않았다');
  if (!(await dialogOpen(page))) throw new Error('ESC 가 다이얼로그까지 닫았다 — window capture 실패');
  console.log('ESC: 메뉴만 닫힘 (다이얼로그 유지)');
  await page.keyboard.press('Escape');
  await pause(page, 400);
  if (await dialogOpen(page)) throw new Error('두 번째 ESC 에 다이얼로그가 닫히지 않았다');

  // ── 5. 원형 — 계열 추가를 막지 않고 안내한다 (#6037: 파손이 아니라 무효과) ──
  await loadHwpFile(page, PIE);
  await dismissSkinOnboarding(page);
  const pie = await chartShape(page);
  console.log(`원형: plot=${pie.plot} · 계열 ${pie.series}`);
  await openDialog(page);
  await rightClickCell(page, 0, 0);
  await pause(page, 300);
  const pieMenu = await menuItems(page);
  const pieAdd = pieMenu.find((i) => i.label === '오른쪽에 계열 추가');
  if (!pieAdd) throw new Error('원형 메뉴에 계열 추가가 없다');
  if (pieAdd.disabled) throw new Error('원형에서 계열 추가가 막혔다 — #6037 이 가드를 없앴다');
  if (!pieAdd.title.includes('원형')) throw new Error(`원형 안내가 없다 — title="${pieAdd.title}"`);
  console.log(`원형 계열 추가: 활성 + 안내 "${pieAdd.title}"`);
  await screenshot(page, '6053-5-pie-note');
  await page.keyboard.press('Escape');
  await pause(page, 300);
  await page.keyboard.press('Escape');
  await pause(page, 300);

  // ── 6. 주식형 — 캔들 양끝 계열은 사전 비활성 (candleAnchorBroken) ──
  await loadHwpFile(page, STOCK);
  await dismissSkinOnboarding(page);
  const stock = await chartShape(page);
  console.log(`주식형: plot=${stock.plot} · hasUpDownBars=${stock.hasUpDownBars} · 계열 ${stock.series}`);
  if (stock.hasUpDownBars !== true) throw new Error('시가고가저가종가에 캔들 장치가 없다 — 샘플 전제 붕괴');
  await openDialog(page);

  await rightClickCell(page, 0, 0); // 첫 계열
  await pause(page, 300);
  const firstMenu = await menuItems(page);
  for (const label of ['계열 삭제', '왼쪽에 계열 추가']) {
    const item = firstMenu.find((i) => i.label === label);
    if (!item?.disabled) throw new Error(`첫 계열의 "${label}" 가 비활성이 아니다`);
  }
  const midInsert = firstMenu.find((i) => i.label === '오른쪽에 계열 추가');
  if (midInsert?.disabled) throw new Error('중간 삽입까지 막혔다 — 양끝이 유지되면 정상이다');
  console.log('주식형 첫 계열: 삭제·바깥 삽입 비활성, 중간 삽입 활성');
  await screenshot(page, '6053-6-stock-disabled');
  await page.keyboard.press('Escape');
  await pause(page, 300);

  await rightClickCell(page, 0, stock.series - 1); // 끝 계열
  await pause(page, 300);
  const lastMenu = await menuItems(page);
  for (const label of ['계열 삭제', '오른쪽에 계열 추가']) {
    const item = lastMenu.find((i) => i.label === label);
    if (!item?.disabled) throw new Error(`끝 계열의 "${label}" 가 비활성이 아니다`);
  }
  console.log('주식형 끝 계열: 삭제·바깥 삽입 비활성');

  console.log('\n=== #6053 e2e 전 단계 통과 ===');
});
