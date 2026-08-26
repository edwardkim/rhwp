/**
 * E2E 테스트: 실사용 혼합 세션의 실효 undo 깊이 — 무축출 계약 (#5769)
 *
 * 역연산 전환(조각 삭제·deferRecord 붙여넣기)의 회귀 방어다. 되돌릴 것이
 * 스칼라인 조작이 다시 스냅샷으로 기록되는 순간, 예산 98(SNAPSHOT_ID_BUDGET)을
 * 넘는 세션에서 오래된 엔트리가 축출된다 — 이 게이트는 그 축출을 잡는다.
 *
 *   교정 패턴 R라운드(선택 삭제 + 재입력) →
 *   ① 전체 스택 스냅샷 슬롯 합 === 0 (역연산 경로만 썼다는 증거)
 *   ② deleteSelection 엔트리 수 ≥ ⌈R/2⌉ (대다수 라운드가 조각 경로 — 문단이
 *      짧아진 라운드는 refill 분기로 엔트리 1개라, 정확히 R 이 아니다)
 *   ③ undoStack 길이 ≥ 시딩(11) + R (축출 없음 — 회귀 시 예산 초과로 감소)
 *   ④ 연속 Ctrl+Z 가 스택 전량을 소진 (최대 깊이 도달)
 *   ⑤ 전량 되돌림 뒤 문서는 새 문서 상태(빈 단일 문단)와 일치
 *
 * 라운드 기본 110 — 구버전 배선(조작당 1슬롯)이라면 98 예산으로 12개 이상
 * 축출되어 ③④가 깨진다. E2E_DEPTH_ROUNDS 로 줄여 스모크할 수 있다.
 */
import {
  runTest, setTestCase, createNewDocument, clickEditArea, typeText, assert,
} from './helpers.mjs';

const sleep = (page, ms) => page.evaluate(t => new Promise(r => setTimeout(r, t)), ms);

const ROUNDS = Math.max(10, parseInt(process.env.E2E_DEPTH_ROUNDS || '110', 10));
const SEED_ENTRIES = 11; // insertText 6 + splitParagraph 5

async function dismissSkinOnboarding(page) {
  await page.evaluate(() => {
    const anyCard = document.querySelector('.skin-onboarding-card');
    if (anyCard) anyCard.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    const ok = [...document.querySelectorAll('button.dialog-btn-primary')]
      .find(x => x.offsetParent !== null);
    if (ok) {
      ok.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      ok.click();
    }
  });
  await sleep(page, 600);
}

async function stackStats(page) {
  return await page.evaluate(() => {
    const s = window.__inputHandler?.history?.undoStack;
    if (!s) return null;
    let slots = 0;
    let deletes = 0;
    for (const c of s) {
      slots += (typeof c.snapshotResourceCount === 'function' ? c.snapshotResourceCount() : 0);
      if (c.type === 'deleteSelection') deletes += 1;
    }
    return { len: s.length, slots, deletes };
  });
}

async function pressUndo(page) {
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Control');
  await sleep(page, 90);
}

runTest('#5769 혼합 세션 실효 깊이 — 무축출 계약', async ({ page }) => {

  setTestCase(`undo-depth: 교정 ${ROUNDS}라운드 후 전량 복원·슬롯 0`);
  console.log(`\n[1] 6문단 시딩 + 교정 ${ROUNDS}라운드...`);
  await createNewDocument(page);
  await dismissSkinOnboarding(page);
  await clickEditArea(page);

  for (let p = 0; p < 6; p++) {
    await typeText(page, `paragraph ${p} the quick brown fox jumps over the lazy dog`);
    if (p < 5) await page.keyboard.press('Enter');
  }

  for (let i = 0; i < ROUNDS; i++) {
    const para = i % 6;
    // 재입력 패턴 — 같은 자리를 지우고 'fix' 로 대체해 문단 길이를 안정 유지한다.
    const selLen = await page.evaluate(([pi]) => {
      const ih = window.__inputHandler;
      const w = window.__wasm;
      const len = w.getParagraphLength(0, pi);
      if (len < 6) return null;
      const s = 1, e = Math.min(7, len - 1);
      return ih.cursor.selectRange(
        { sectionIndex: 0, paragraphIndex: pi, charOffset: s },
        { sectionIndex: 0, paragraphIndex: pi, charOffset: e },
        null,
      ) ? 1 : null;
    }, [para]);

    if (selLen === null) {
      // 문단이 짧아졌으면 채워 넣고 이번 라운드는 타이핑만 (엔트리 1개)
      await typeText(page, 'refill ');
    } else {
      await page.keyboard.press('Delete');
      await sleep(page, 40);
      await typeText(page, 'fix');
    }
  }
  await sleep(page, 200);

  const stats = await stackStats(page);
  assert(stats && stats.len > 0, `스택 조회 (${JSON.stringify(stats)})`);
  // helpers 의 assert 는 함수 호출 계약이다 — assert.equal·assert.ok 메서드가 없다.
  assert(stats.slots === 0,
    `전체 스택 스냅샷 슬롯 합은 0 이어야 한다(역연산만) — got ${stats.slots}`);
  // 문단이 짧아진 라운드는 refill 분기(deleteSelection 없음)로 빠지므로 deletes 는
  // R 보다 작다 — 정확 일치 대신 "대다수 라운드가 조각 경로"라는 하한으로 핀한다.
  assert(stats.deletes >= Math.ceil(ROUNDS / 2),
    `${ROUNDS}라운드 중 절반 이상이 조각 경로여야 한다 — deleteSelection ${stats.deletes}`);

  // ── 축출 판정: 남은 스택이 수행량을 모두 담고 있는가 ──────────────
  // refill 분기(엔트리 1)와 정상 라운드(엔트리 2)가 섞이므로 "최소치"로 판정한다:
  // 시딩 11 + 라운드당 1 이상은 반드시 기록됐어야 하고, 구버전(라운드당 ≥2슬롯)
  // 이라면 98 예산으로 시딩+라운드 대부분이 축출돼 len 이 이 하한 아래로 떨어진다.
  const minExpected = SEED_ENTRIES + ROUNDS;
  assert(stats.len >= minExpected,
    `무축출 위반 — 스택 ${stats.len} < 최소 기대 ${minExpected} (예산 축출 의심)`);

  console.log('\n[2] Ctrl+Z 전량 소진...');
  const depthStart = stats.len;
  let undos = 0;
  while (undos <= depthStart) {
    const d0 = await page.evaluate(() => window.__inputHandler.history.undoStack.length);
    await pressUndo(page);
    const d1 = await page.evaluate(() => window.__inputHandler.history.undoStack.length);
    if (d1 < d0) undos++;
    else break;
  }
  assert(undos === depthStart,
    `모든 엔트리가 되돌려져야 한다 — ${undos}/${depthStart}`);

  const endLens = await page.evaluate(() => {
    const w = window.__wasm;
    return Array.from({ length: w.getParagraphCount(0) }, (_, i) => w.getParagraphLength(0, i));
  });
  assert(JSON.stringify(endLens) === '[0]',
    `전량 되돌림 뒤 새 문서 상태여야 한다 — got ${JSON.stringify(endLens)}`);

  console.log(`\n✓ 스택 ${depthStart} · 슬롯 0 · 연속 undo ${undos} — 무축출 계약 통과`);
}, { skipLoadApp: false });
