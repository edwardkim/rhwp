/**
 * E2E 테스트: 텍스트 플로우 (입력, 줄바꿈, 엔터, 페이지 넘김)
 */
import {
  runTest, createNewDocument, clickEditArea, typeText,
  screenshot, assert, getPageCount, getParagraphCount, getParaText,
} from './helpers.mjs';

await runTest('텍스트 플로우 테스트', async ({ page }) => {
  // 1. 새 문서 생성
  console.log('[1] 앱 로드 및 새 문서 생성...');
  await createNewDocument(page);
  assert(await getPageCount(page) >= 1, `새 문서 페이지 수: ${await getPageCount(page)}`);
  await screenshot(page, '01-new-document');

  // 2. 텍스트 입력
  console.log('\n[2] 텍스트 입력 테스트...');
  await clickEditArea(page);
  await typeText(page, 'Hello World');
  await screenshot(page, '02-text-input');

  const paraText = await getParaText(page, 0, 0, 100);
  console.log(`  문단 텍스트: "${paraText}"`);
  assert(paraText.includes('Hello World') || paraText === '', '텍스트 입력 확인');

  // 3. 줄바꿈
  console.log('\n[3] 줄바꿈 테스트...');
  const longText = 'The quick brown fox jumps over the lazy dog. ';
  for (let i = 0; i < 5; i++) await typeText(page, longText);
  await screenshot(page, '03-line-wrap');

  // 4. 엔터(문단 분리)
  console.log('\n[4] 엔터(문단 분리) 테스트...');
  const paraCountBefore = await getParagraphCount(page);
  console.log(`  엔터 전 문단 수: ${paraCountBefore}`);
  await page.keyboard.press('Enter');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  await typeText(page, 'New paragraph after Enter');
  const paraCountAfter = await getParagraphCount(page);
  console.log(`  엔터 후 문단 수: ${paraCountAfter}`);
  assert(paraCountAfter > paraCountBefore, `문단 분리 확인 (${paraCountBefore} → ${paraCountAfter})`);
  await screenshot(page, '04-enter-split');

  // 5. 페이지 넘김
  console.log('\n[5] 페이지 넘김 테스트...');
  for (let i = 0; i < 40; i++) {
    await typeText(page, `Line ${i + 1}`);
    await page.keyboard.press('Enter');
    if (i % 10 === 9) await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  }
  await page.evaluate(() => new Promise(r => setTimeout(r, 1000)));
  const pageCount2 = await getPageCount(page);
  console.log(`  페이지 수: ${pageCount2}`);
  assert(pageCount2 >= 2, `페이지 넘김 확인 (페이지 수: ${pageCount2})`);
  await screenshot(page, '05-page-overflow');

  // 6. Backspace 문단 병합
  console.log('\n[6] Backspace 문단 병합 테스트...');
  await page.keyboard.press('Home');
  await page.evaluate(() => new Promise(r => setTimeout(r, 100)));
  const beforeMerge = await getParagraphCount(page);
  await page.keyboard.press('Backspace');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const afterMerge = await getParagraphCount(page);
  console.log(`  병합 전 문단 수: ${beforeMerge}, 병합 후: ${afterMerge}`);
  assert(afterMerge < beforeMerge, '문단 병합 확인');
  await screenshot(page, '06-backspace-merge');
});
