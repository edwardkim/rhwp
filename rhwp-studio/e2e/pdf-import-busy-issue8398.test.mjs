/** #8398: UI 상태 계약. 조판 좌표·픽셀 일치율을 판정하는 검사가 아니다. */
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runTest} from './helpers.mjs';
const pdfPath = new URL('../../pdf/pr8224/lseg-tab-insert-2020.pdf', import.meta.url);
const pdf = Array.from(readFileSync(pdfPath));
const hwp = Array.from(readFileSync(new URL('../../samples/para-001.hwp', import.meta.url)));
const dialog = '.dialog-pdf-import';
async function openBytes(page, bytes, fileName) {
  await page.evaluate(({bytes, fileName}) => {
    window.__openResult = null;
    window.__eventBus.emit('open-document-bytes', {bytes: new Uint8Array(bytes), fileName,
      fileHandle: null, skipUnsavedGuard: true, requestId: 'busy-8398'});
  }, {bytes, fileName});
}
async function waitDone(page) {
  await page.waitForFunction(() => {
    // #8397의 실제 글꼴 선택 단계를 거친다. 로컬 글꼴 권한은 요청하지 않는다.
    const prompt = [...document.querySelectorAll('.modal-overlay')].find(el =>
      el.querySelector('.dialog-title')?.textContent.includes('로컬 글꼴 감지'));
    [...(prompt?.querySelectorAll('button') ?? [])].find(el => el.textContent === '대체 글꼴로 보기')?.click();
    return window.__openResult !== null && !document.documentElement.classList.contains('rhwp-busy');
  });
  return page.evaluate(() => window.__openResult);
}
async function checkChoice(page) {
  await page.waitForSelector(dialog);
  const state = await page.$eval(dialog, d => ({busy: document.documentElement.classList.contains('rhwp-busy'),
    cursors: [...d.querySelectorAll('select,input,textarea,button')].map(el => getComputedStyle(el).cursor),
    status: document.querySelector('#sb-message').textContent}));
  assert.equal(state.busy, false, 'PDF 사용자 선택 중 전역 busy 해제');
  assert.ok(state.cursors.every(c => c !== 'wait'), '선택·확인·취소는 일반 커서');
  assert.doesNotMatch(state.status, /파일 로딩|파싱|쪽 계산/, '미실행 작업의 진행률을 표시하지 않음');
  console.log('PASS: PDF 선택 대기 커서·상태 표시');
}
async function snapshot(page) {
  return page.evaluate(async () => {
    const {listRecentDocs} = await import('/src/recent/recent-store.ts');
    const {listAutosaveDrafts} = await import('/src/recovery/autosave-store.ts');
    const w = window.__wasm;
    return {generation: w.documentGeneration, fileName: w.fileName, text: w.getTextRange(0, 0, 0, 200),
      handlePreserved: w.currentFileHandle === window.__originalHandle, recents: await listRecentDocs(),
      drafts: (await listAutosaveDrafts()).map(d => ({...d, data: Array.from(d.data)}))};
  });
}
await runTest('PDF 선택·변환·재시도의 커서 수명 (#8398)', async ({page}) => {
  await page.evaluate(() => {
    window.__eventBus.on('open-document-bytes:done', r => {if (r.requestId === 'busy-8398') window.__openResult = r;});
    window.__pdfWorkers = 0; window.__terminatedWorkers = 0; window.__holdPdf = false;
    const WorkerBase = window.Worker;
    window.Worker = class extends WorkerBase {
      constructor(url, options) {
        super(url, options); this.pdfImportWorker = String(url).includes('import-worker');
        if (this.pdfImportWorker) window.__pdfWorkers++;
      }
      postMessage(...args) {
        // 실제 Worker의 요청 전송만 보류해 진행 중 취소 시점을 결정적으로 검사한다.
        if (this.pdfImportWorker && window.__holdPdf) return;
        return super.postMessage(...args);
      }
      terminate() {if (this.pdfImportWorker) window.__terminatedWorkers++; return super.terminate();}
    };
    window.__busySeen = false;
    new MutationObserver(records => {
      if (records.some(r => (r.oldValue || '').includes('rhwp-busy'))
        || document.documentElement.classList.contains('rhwp-busy')) window.__busySeen = true;
    }).observe(document.documentElement, {attributes: true, attributeFilter: ['class'], attributeOldValue: true});
  });
  await openBytes(page, hwp, '기존 문서.hwp');
  assert.equal((await waitDone(page)).ok, true);
  assert.equal(await page.evaluate(() => window.__busySeen), true, '일반 HWP 열기 중 busy 유지');
  await page.evaluate(async () => {
    window.__originalHandle = {name: '기존 문서.hwp'}; window.__wasm.currentFileHandle = window.__originalHandle;
    await window.__autosaveManager.flushNow('issue8398');
  });
  const original = await snapshot(page);
  assert.ok(original.drafts.length > 0, '실제 복구본을 보존 대조군으로 준비');
  await openBytes(page, pdf, '선택.pdf');
  await checkChoice(page);
  assert.equal(await page.evaluate(() => window.__pdfWorkers), 0);
  await page.select(`${dialog} select`, 'constrained');
  await page.click(`${dialog} summary`);
  await page.$eval('#pdf-import-timeout', el => {el.value = '60';});
  assert.deepEqual(await snapshot(page), original, '선택 대기에서 문서·복구본·최근·핸들 보존');
  await page.keyboard.press('Escape');
  assert.equal((await waitDone(page)).ok, false);
  assert.deepEqual(await snapshot(page), original, '선택 취소에서 기존 상태 보존');
  console.log('PASS: 바이트 입력 선택·취소 및 기존 문서 보존');
  // 확장자가 바뀐 PDF도 파일 헤더 검사 뒤 같은 대화창을 사용한다.
  await page.evaluate(bytes => {
    const transfer = new DataTransfer();
    transfer.items.add(new File([new Uint8Array(bytes)], '이름만바꾼.hwp'));
    document.querySelector('#scroll-container').dispatchEvent(new DragEvent('drop', {
      dataTransfer: transfer, bubbles: true, cancelable: true,
    }));
  }, pdf);
  await checkChoice(page);
  await page.keyboard.press('Escape');
  await page.waitForFunction(() => !document.querySelector('.dialog-pdf-import')
    && !document.documentElement.classList.contains('rhwp-busy'));
  assert.deepEqual(await snapshot(page), original, '확장자 변경 PDF 드롭·취소에서도 기존 상태 보존');
  console.log('PASS: 드롭·PDF 헤더 감지·취소');
  // 파일 입력은 loadFile → loadBytes의 바깥 busy 소유자까지 검사한다.
  await page.$eval('#file-input', el => {el.dataset.skipUnsavedGuard = 'true';});
  await (await page.$('#file-input')).uploadFile(pdfPath.pathname);
  await checkChoice(page);
  await page.evaluate(() => {window.__holdPdf = true;});
  await page.click(`${dialog} .dialog-btn-primary`);
  await page.waitForFunction(() => window.__pdfWorkers === 1);
  const working = await page.$eval(dialog, d => ({disabled: d.querySelector('.dialog-btn-primary').disabled,
    cancelDisabled: d.querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)').disabled,
    status: d.querySelector('[role="status"]').textContent,
    cursor: getComputedStyle(d.querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)')).cursor}));
  assert.equal(working.disabled, true); assert.equal(working.cancelDisabled, false);
  assert.notEqual(working.cursor, 'wait'); assert.ok(working.status.length > 0);
  await page.click(`${dialog} .dialog-footer .dialog-btn:not(.dialog-btn-primary)`);
  await page.waitForFunction(() => !document.querySelector('.dialog-pdf-import')
    && !document.documentElement.classList.contains('rhwp-busy') && document.querySelector('#file-input').value === '');
  assert.equal(await page.evaluate(() => window.__terminatedWorkers), 1);
  assert.deepEqual(await snapshot(page), original, '실제 Worker 진행 중 취소에서 기존 상태 보존');
  await page.evaluate(() => {window.__holdPdf = false;});
  console.log('PASS: 파일 입력·진행 중 취소·Worker 종료');
  await openBytes(page, pdf, '재시도.pdf');
  await checkChoice(page);
  await page.click(`${dialog} summary`);
  await page.$eval(`${dialog} textarea`, el => {
    el.value = JSON.stringify({backend: {inputBytes: 1}}); el.dispatchEvent(new Event('input'));
  });
  await page.click(`${dialog} .dialog-btn-primary`);
  await page.waitForFunction(() => document.querySelector('.dialog-pdf-import .dialog-btn-primary')?.textContent === '다시 시도');
  await checkChoice(page);
  assert.deepEqual(await snapshot(page), original, '입력 한도 실패 후 기존 상태 보존');
  await page.$eval(`${dialog} textarea`, el => {el.value = ''; el.dispatchEvent(new Event('input'));});
  await page.evaluate(() => {window.__busySeen = false;});
  await page.click(`${dialog} .dialog-btn-primary`);
  assert.equal((await waitDone(page)).ok, true, '한도 수정 뒤 실제 PDF 변환 재시도 성공');
  assert.equal(await page.evaluate(() => window.__busySeen), true, '변환 뒤 뷰 초기화에는 busy 유지');
  assert.equal(await page.evaluate(() => window.__wasm.getSourceFormat()), 'pdf');
  assert.ok(await page.evaluate(() => window.__wasm.pageCount > 0));
  console.log('PASS: 실패 후 선택·재시도·실제 PDF 변환 성공');
  await openBytes(page, [1, 2, 3], '실패.hwp');
  assert.equal((await waitDone(page)).ok, false);
  assert.equal(await page.evaluate(() => window.__wasm.getSourceFormat()), 'pdf');
  await openBytes(page, hwp, '후속 문서.hwp');
  assert.equal((await waitDone(page)).ok, true);
  console.log('PASS: 일반 문서 오류 후 busy 해제·다음 정상 열기');
  await page.evaluate(() => {
    window.__busySeen = false;
    window.__eventBus.emit('create-new-document', {skipUnsavedGuard: true});
  });
  await page.waitForFunction(() => window.__busySeen && !document.documentElement.classList.contains('rhwp-busy'));
  console.log('PASS: 새 문서 생성 중 busy 유지·완료 후 해제');
});
