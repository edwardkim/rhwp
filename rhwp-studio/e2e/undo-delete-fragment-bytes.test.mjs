/**
 * E2E 테스트: 선택 삭제 조각(fragment) undo 의 저장 바이트 왕복 동일성 (#5769)
 *
 * Stage 2 게이트 — "저장 바이트 왕복 동일성 브라우저 검증". Rust 게이트
 * (tests/cases/issue_5769_delete_fragment_byte_identity.rs)가 코어 수준의
 * 동일성을 증명하면, 이 테스트는 스튜디오 실경로를 증명한다:
 *
 *   키보드 Delete → deleteSelection() → DeleteSelectionCommand
 *   → FragmentDeleteCommand(capture → deleteRange) → Ctrl+Z
 *   → restoreDeleteFragment → exportHwp 가 삭제 전과 완전히 같음
 *
 * 검증 포인트:
 *  - undo 스택 top 이 `deleteSelection` 이면서 snapshotResourceCount() === 0 —
 *    스냅샷 폴백이 아니라 조각 경로로 기록됐다는 런타임 증거(소스 가드만으론 불충분).
 *  - 삭제 직후 export 는 베이스라인과 달라야 하고(삭제 실재),
 *    undo 직후 export 는 베이스라인과 **바이트 단위로 같아야** 한다.
 *  - 다문단 부분 삭제(p0 offset2 ~ p2 offset1) — 병합 잔여 문단 + 꼬리 line_segs +
 *    raw_stream + 캐럿이 모두 관여하는 가장 까다로운 형태다.
 */
import {
  runTest, setTestCase, createNewDocument, clickEditArea, typeText, assert,
} from './helpers.mjs';

const sleep = (page, ms) => page.evaluate(t => new Promise(r => setTimeout(r, t)), ms);

/**
 * 첫 실행 프로필에서 뜨는 스킨 온보딩을 닫는다 — 이 카드가 포커스를 가로채면
 * 첫 텍스트 입력 통째가 유실된다(실측). 카드가 없으면 아무것도 하지 않는다.
 */
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

async function undoDepth(page) {
  return await page.evaluate(() => window.__inputHandler?.history?.undoStack?.length ?? -1);
}

async function pressUndo(page) {
  await clickEditArea(page);
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Control');
  await sleep(page, 450);
}

/** exportHwp 바이트를 node 배열로 가져온다 */
async function exportBytes(page) {
  return await page.evaluate(() => Array.from(window.__wasm.exportHwp()));
}

function firstDiff(a, b) {
  const n = Math.min(a.length, b.length);
  for (let i = 0; i < n; i++) if (a[i] !== b[i]) return i;
  return a.length === b.length ? -1 : n;
}

runTest('#5769 선택 삭제 조각 undo 저장 바이트 왕복 동일성', async ({ page }) => {

  // ── 준비: 다문단 문서 ──────────────────────────────────
  setTestCase('delete-fragment-bytes: 다문단 부분 선택 삭제 → Ctrl+Z → exportHwp 동일');
  console.log('\n[1] 다문단 문서 입력...');
  await createNewDocument(page);
  await dismissSkinOnboarding(page);
  await clickEditArea(page);
  await typeText(page, 'first paragraph body');
  await page.keyboard.press('Enter');
  await typeText(page, 'second paragraph body');
  await page.keyboard.press('Enter');
  await typeText(page, 'third paragraph body');

  const baseLens = await page.evaluate(() => {
    const w = window.__wasm;
    return Array.from({ length: w.getParagraphCount(0) }, (_, i) => w.getParagraphLength(0, i));
  });
  assert(JSON.stringify(baseLens) === '[20,21,20]',
    `3문단 준비 완료 (lens=${JSON.stringify(baseLens)})`);

  // [#4180] 캐럿 스탬핑 대비 — baseline export 시점의 JS 커서를 기록해 둔다.
  const basePos = await page.evaluate(() => window.__inputHandler.getCursorPosition());

  const before = await exportBytes(page);
  assert(before.length > 0, `삭제 전 export 바이트 존재 (${before.length}B)`);

  // ── 삭제: 문단 경계를 건너는 부분 선택 + 실제 Delete 키 ──
  console.log('\n[2] 다문단 부분 선택 삭제 (Delete 키 실경로)...');
  const selected = await page.evaluate(() => {
    const ih = window.__inputHandler;
    return ih.cursor.selectRange(
      { sectionIndex: 0, paragraphIndex: 0, charOffset: 6 },
      { sectionIndex: 0, paragraphIndex: 2, charOffset: 5 },
      null,
    );
  });
  assert(selected, '다문단 선택 세워짐(selectRange=true)');

  const depthBefore = await undoDepth(page);
  await page.keyboard.press('Delete');
  await sleep(page, 500);

  assert(await undoDepth(page) === depthBefore + 1,
    `삭제가 undo 스택에 1건 기록 (${depthBefore}→${await undoDepth(page)})`);

  // 조각 경로 증거 — top 엔트리가 스냅샷이 아니라 조각 소비자인지 런타임에 확인
  const entry = await page.evaluate(() => {
    const c = window.__inputHandler.history.undoStack.at(-1);
    return { type: c?.type, slots: c?.snapshotResourceCount?.() ?? null };
  });
  assert(entry.type === 'deleteSelection', `top 엔트리 타입 (${entry.type})`);
  assert(entry.slots === 0,
    `조각 경로는 스냅샷 슬롯 0 이어야 한다 (#5769 예산 무기여) — got ${entry.slots}`);

  const afterDelete = await exportBytes(page);
  assert(firstDiff(before, afterDelete) !== -1, '삭제 직후 export 는 베이스라인과 달라야 함');

  // ── undo 후 바이트 동일성 ────────────────────────────────
  console.log('\n[3] Ctrl+Z → 저장 바이트 왕복 비교...');
  await pressUndo(page);

  // 문단 내용 복원 확인 — 조각이 되돌린 본문 실체.
  const lens = await page.evaluate(() => {
    const w = window.__wasm;
    return Array.from({ length: w.getParagraphCount(0) }, (_, i) => w.getParagraphLength(0, i));
  });
  assert(JSON.stringify(lens) === JSON.stringify(baseLens),
    `문단 길이 복원 (${JSON.stringify(baseLens)} → ${JSON.stringify(lens)})`);

  // [#4180] 저장 시점 캐럿 스탬핑 — exportHwp 는 JS 커서를 WASM 캐럿에 찍고 직렬화한다.
  // undo 는 #3416 계약대로 선택 끝으로 커서를 되돌리므로(#3416) baseline 시점 커서와 다르고,
  // 그 차이는 DocProperties 캐럿 바이트(@888 부근)로 나온다. 이는 조각 결함이 아니라
  // 의도된 저장 경로 동작이라, baseline 과 같은 커서로 되돌린 뒤 순수 문서 바이트를 비교한다.
  await page.evaluate((pos) => window.__inputHandler.cursor.moveTo(pos), basePos);
  await sleep(page, 300);

  const afterUndo = await exportBytes(page);
  assert(afterUndo.length === before.length,
    `바이트 길이 복원 (${before.length} → ${afterUndo.length})`);
  const diffAt = firstDiff(before, afterUndo);
  assert(diffAt === -1, `undo 후 export 가 삭제 전과 동일해야 함(커서 동일화) — 첫 불일치 @${diffAt}`);
  console.log(`  바이트 ${before.length}B 왕복 동일 ✓`);
}, { skipLoadApp: false });
