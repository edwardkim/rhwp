/**
 * E2E 테스트: 플랫폼별 navigation shortcut
 */
import {
  runTest, createNewDocument, clickEditArea, typeText,
  assert, getCursorPosition, getParaText,
} from './helpers.mjs';

const TEXT = 'Alpha Beta 123';
const END = TEXT.length;
const WORD_123_START = 'Alpha Beta '.length;

async function pressCombo(page, modifiers, key) {
  for (const mod of modifiers) await page.keyboard.down(mod);
  await page.keyboard.press(key);
  for (const mod of [...modifiers].reverse()) await page.keyboard.up(mod);
  await page.evaluate(() => new Promise(r => setTimeout(r, 150)));
}

async function setPlatform(page, platform) {
  await page.evaluate((value) => {
    window.__rhwpTestPlatformKind = value;
  }, platform);
}

async function resetFixture(page, platform) {
  await setPlatform(page, platform);
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, TEXT);

  const paraText = await getParaText(page, 0, 0, 100);
  assert(paraText.includes(TEXT), `fixture text 입력 확인: "${paraText}"`);

  const moved = await page.evaluate((offset) => {
    return window.__inputHandler?.moveCursorTo?.({
      sectionIndex: 0,
      paragraphIndex: 0,
      charOffset: offset,
    }) ?? false;
  }, END);
  assert(moved, `커서를 문단 끝 offset=${END}로 이동`);
}

async function selectionState(page) {
  return await page.evaluate(() => {
    const ih = window.__inputHandler;
    return {
      hasSelection: ih?.hasSelection?.() ?? false,
      position: ih?.getCursorPosition?.() ?? null,
      selection: ih?.getSelection?.() ?? null,
    };
  });
}

runTest('플랫폼별 navigation shortcut 테스트', async ({ page }) => {
  console.log('[1] Windows/Linux Ctrl+Arrow 단어 이동...');
  await resetFixture(page, 'other');
  await pressCombo(page, ['Control'], 'ArrowLeft');
  let pos = await getCursorPosition(page);
  assert(pos?.charOffset === WORD_123_START, `Ctrl+← 단어 이동 (${pos?.charOffset} === ${WORD_123_START})`);

  console.log('[2] Windows/Linux Alt+Arrow는 단어 이동으로 처리하지 않음...');
  await resetFixture(page, 'other');
  await pressCombo(page, ['Alt'], 'ArrowLeft');
  pos = await getCursorPosition(page);
  assert(pos?.charOffset === END, `Alt+← 미처리로 커서 유지 (${pos?.charOffset} === ${END})`);

  console.log('[3] Windows/Linux Ctrl+Shift+Arrow 단어 선택...');
  await resetFixture(page, 'other');
  await pressCombo(page, ['Control', 'Shift'], 'ArrowLeft');
  let sel = await selectionState(page);
  assert(sel.hasSelection, 'Ctrl+Shift+← 후 선택 상태');
  assert(sel.position?.charOffset === WORD_123_START, `Ctrl+Shift+← focus offset=${WORD_123_START}`);
  assert(sel.selection?.start?.charOffset === WORD_123_START && sel.selection?.end?.charOffset === END,
    `Ctrl+Shift+← 선택 범위 ${WORD_123_START}..${END}`);

  console.log('[4] macOS Option+Arrow 단어 이동...');
  await resetFixture(page, 'mac');
  await pressCombo(page, ['Alt'], 'ArrowLeft');
  pos = await getCursorPosition(page);
  assert(pos?.charOffset === WORD_123_START, `Option+← 단어 이동 (${pos?.charOffset} === ${WORD_123_START})`);

  console.log('[5] macOS Command+Arrow 줄 처음/끝 이동...');
  await resetFixture(page, 'mac');
  await pressCombo(page, ['Meta'], 'ArrowLeft');
  pos = await getCursorPosition(page);
  assert(pos?.charOffset === 0, `Command+← 줄 처음 (${pos?.charOffset} === 0)`);

  await pressCombo(page, ['Meta'], 'ArrowRight');
  pos = await getCursorPosition(page);
  assert(pos?.charOffset === END, `Command+→ 줄 끝 (${pos?.charOffset} === ${END})`);

  console.log('[6] macOS Command+Shift+Arrow 줄 선택...');
  await resetFixture(page, 'mac');
  await page.evaluate(() => window.__inputHandler?.moveCursorTo?.({
    sectionIndex: 0,
    paragraphIndex: 0,
    charOffset: 0,
  }));
  await pressCombo(page, ['Meta', 'Shift'], 'ArrowRight');
  sel = await selectionState(page);
  assert(sel.hasSelection, 'Command+Shift+→ 후 선택 상태');
  assert(sel.selection?.start?.charOffset === 0 && sel.selection?.end?.charOffset === END,
    `Command+Shift+→ 선택 범위 0..${END}`);
});
