/** Actual Worker page progress; observer-only change, no document layout golden. */
import assert from 'node:assert/strict';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {basename} from 'node:path';
import {runTest} from './helpers.mjs';
const input = process.env.RHWP_PDF_PROGRESS_INPUT ?? new URL('../../pdf/issue8413/resource-policy/resource-8k.pdf', import.meta.url).pathname;
const expected = Number(process.env.RHWP_PDF_PROGRESS_PAGES ?? 2);
const output = new URL('../../output/pdf-page-progress-issue8413/', import.meta.url).pathname;
mkdirSync(output, {recursive:true});
await runTest('PDF 실제 쪽별 진행률·취소·다시 열기 (#8413)', async ({page}) => {
  await page.evaluate(() => {
    window.__pageProgressResult = null;
    window.__eventBus.on('open-document-bytes:done', result => {
      if (result.requestId === 'page-progress') window.__pageProgressResult = result;
    });
  });
  async function open(cancelAtProgress = false) {
    await page.evaluate(({base64, fileName}) => {
      window.__pageProgressResult = null; window.__pageProgressEvents = [];
      const raw = atob(base64), bytes = Uint8Array.from(raw, c => c.charCodeAt(0));
      window.__eventBus.emit('open-document-bytes', {bytes, fileName, skipUnsavedGuard:true, fileHandle:null, requestId:'page-progress'});
    }, {base64:readFileSync(input).toString('base64'), fileName:basename(input)});
    await page.waitForSelector('.dialog-pdf-import');
    await page.evaluate(cancelAtProgress => {
      window.__cancelledAtProgress = null;
      const dialog = document.querySelector('.dialog-pdf-import'), progress = dialog.querySelector('progress');
      document.querySelector('#pdf-import-timeout').value = '60';
      window.__pageProgressObserver?.disconnect();
      window.__pageProgressObserver = new MutationObserver(() => {
        if (!progress.hidden) window.__pageProgressEvents.push({value:progress.value,
          determinate:progress.hasAttribute('value'), text:dialog.querySelector('[role="status"]').textContent});
        if (cancelAtProgress && progress.value > 0 && progress.value < 100) {
          cancelAtProgress = false; window.__cancelledAtProgress = progress.value;
          dialog.querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)').click();
        }
      });
      window.__pageProgressObserver.observe(dialog, {subtree:true, attributes:true, childList:true, characterData:true});
    }, cancelAtProgress);
    await page.click('.dialog-pdf-import .dialog-btn-primary');
  }
  async function done() {
    await page.waitForFunction(() => {
      const prompt = [...document.querySelectorAll('.modal-overlay')].find(el => el.querySelector('.dialog-title')?.textContent.includes('로컬 글꼴 감지'));
      [...(prompt?.querySelectorAll('button') ?? [])].find(el => el.textContent === '대체 글꼴로 보기')?.click();
      return window.__pageProgressResult !== null;
    }, {timeout:90000});
    return page.evaluate(() => window.__pageProgressResult);
  }
  await open();
  await page.waitForFunction(() => window.__pageProgressEvents.some(e => e.value >= 40 && e.value < 100), {timeout:60000});
  const modal = await page.$('.dialog-pdf-import');
  if (modal) await modal.screenshot({path:output+'page-progress.png'});
  assert.equal((await done()).ok, true);
  assert.equal(await page.evaluate(() => window.__wasm.pageCount), expected);
  const events = await page.evaluate(() => window.__pageProgressEvents);
  assert(events.every(e => e.determinate), '불확정 좌우 이동 바 없음');
  assert(events.some(e => e.value === 100));
  assert(events.some(e => e.text.includes(`/ ${expected}쪽`) && e.text.includes('%')));
  assert(events.every((e, i) => i === 0 || e.value >= events[i-1].value), '완료 비율이 역행하지 않음');
  writeFileSync(output+'events.json', JSON.stringify(events, null, 2));
  const generation = await page.evaluate(() => window.__wasm.documentGeneration);
  await open(true);
  assert.equal((await done()).ok, false);
  const cancelledAt = await page.evaluate(() => window.__cancelledAtProgress);
  assert(cancelledAt > 0 && cancelledAt < 100, '실제 페이지 처리 중 취소');
  assert.equal(await page.evaluate(() => window.__wasm.documentGeneration), generation);
  await page.evaluate(() => window.__pageProgressObserver.disconnect());
  console.log(`PASS actual Worker ${expected} pages, determinate progress, cancellation preserves document`);
});
