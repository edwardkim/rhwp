/** #8411: 대화창 UI·키보드·실제 PDF Worker 계약. 문서 조판 좌표 검사가 아니다. */
import assert from 'node:assert/strict';
import {readFileSync, mkdirSync} from 'node:fs';
import {runTest} from './helpers.mjs';
const pdf = Array.from(readFileSync(new URL('../../pdf/issue8403/image-patterns.pdf', import.meta.url)));
const output = new URL('../../output/pdf-import-dialog-20261031/', import.meta.url).pathname;
mkdirSync(output, {recursive: true});
const dialog = '.dialog-pdf-import';
const advanced = `${dialog} .dialog-pdf-import-advanced`;
const expert = `${dialog} .dialog-pdf-import-expert`;
async function fill(page, selector, text) {
  await page.focus(selector); await page.keyboard.down('Control'); await page.keyboard.press('a');
  await page.keyboard.up('Control'); await page.keyboard.type(text);
}
async function footerVisible(page) {
  const bounds = await page.$eval(`${dialog} .dialog-footer`, el => {
    const r = el.getBoundingClientRect();
    return {top:r.top, bottom:r.bottom, right:r.right, left:r.left, w:innerWidth, h:innerHeight};
  });
  assert.ok(bounds.top >= 0 && bounds.bottom <= bounds.h && bounds.left >= 0 && bounds.right <= bounds.w,
    `하단 버튼이 뷰포트 안에 있음: ${JSON.stringify(bounds)}`);
}
await runTest('PDF 가져오기 화면·키보드·한도·재시도 (#8411)', async ({page}) => {
  await page.evaluate(bytes => {
    window.__pdfBytes8411 = bytes; window.__pdfRequests8411 = []; window.__pdfReads8411 = 0;
    window.__pdfHoldRead8411 = false; window.__pdfResult8411 = null;
    const Base = window.Worker;
    window.Worker = class extends Base {
      constructor(url, options) {super(url, options); this.pdf8411 = String(url).includes('import-worker');}
      postMessage(message, ...rest) {
        if (this.pdf8411 && message.method === 'importPdf') {
          window.__pdfRequests8411.push(structuredClone(message.options));
        }
        return super.postMessage(message, ...rest);
      }
    };
  }, pdf);
  const open = async (fileName = '검토용_매우긴파일명_'.repeat(4) + '.pdf') => {
    await page.evaluate(async fileName => {
      const {showPdfImportDialog} = await import('/src/ui/pdf-import-dialog.ts');
      window.__pdfResult8411 = null;
      void showPdfImportDialog({byteLength:window.__pdfBytes8411.length, read:async () => {
        window.__pdfReads8411++;
        if (window.__pdfHoldRead8411) return new Promise(() => {});
        return new Uint8Array(window.__pdfBytes8411);
      }}, fileName, options => window.__wasm.getPdfImportPolicy(options))
        .then(result => {window.__pdfResult8411 = {ok:true, pageCount:result.pageCount};},
          error => {window.__pdfResult8411 = {ok:false, code:error.code};});
    }, fileName);
    await page.waitForSelector(dialog);
  };
  await open();
  assert.equal(await page.$eval(`${advanced}`, el => el.open), false);
  assert.equal(await page.$eval(`${dialog} textarea`, el => el.checkVisibility()), false);
  // 초기 확인 버튼에서도 Tab이 멈추지 않고 대화창 내부를 순환한다.
  await page.keyboard.press('Tab');
  assert.equal(await page.$eval(`${dialog} .dialog-footer .dialog-btn:not(.dialog-btn-primary)`, el => el === document.activeElement), true);
  await page.keyboard.press('Tab');
  assert.equal(await page.$eval(`${dialog} .dialog-close`, el => el === document.activeElement), true);
  await page.keyboard.down('Shift'); await page.keyboard.press('Tab'); await page.keyboard.up('Shift');
  assert.equal(await page.$eval(`${dialog} .dialog-footer .dialog-btn:not(.dialog-btn-primary)`, el => el === document.activeElement), true);
  await page.focus(`${dialog} select`); await page.keyboard.press('Tab');
  assert.equal(await page.$eval(`${advanced} > summary`, el => el === document.activeElement), true);
  await page.keyboard.press('Tab');
  assert.equal(await page.$eval(`${dialog} .dialog-btn-primary`, el => el === document.activeElement), true, '접힌 영역의 입력은 Tab에서 제외');
  await page.focus(`${dialog} select`); await page.keyboard.press('ArrowDown');
  assert.equal(await page.$eval(`${dialog} select`, el => el.value), 'constrained');
  await page.focus(`${advanced} > summary`); await page.keyboard.press('Enter');
  assert.equal(await page.$eval(advanced, el => el.open), true);
  assert.equal(await page.evaluate(() => window.__pdfReads8411), 0, 'summary Enter는 가져오기를 시작하지 않음');
  assert.equal(await page.$eval(`${dialog} textarea`, el => el.checkVisibility()), false, '고급 설정에서도 JSON은 별도 접힘');
  await page.focus(`${expert} > summary`); await page.keyboard.press('Space');
  assert.equal(await page.$eval(expert, el => el.open), true);
  await page.focus(`${dialog} pre`); await page.keyboard.press('ArrowDown');
  await page.waitForFunction(() => document.querySelector('.dialog-pdf-import pre').scrollTop > 0);
  await page.keyboard.press('Enter');
  assert.equal(await page.evaluate(() => window.__pdfReads8411), 0, '읽기 전용 한도에서 Enter는 가져오기를 시작하지 않음');
  await page.click(`${expert} .dialog-pdf-import-actions button:first-child`);
  assert.deepEqual(await page.$eval('#pdf-import-limits', el => JSON.parse(el.value)), {maxSelectedPages:8,backend:{inputBytes:8388608}});
  assert.equal(await page.$eval('#pdf-import-limits', el => el.getAttribute('aria-invalid')), null, '예시가 실제 정책을 통과');
  await page.click(`${expert} .dialog-pdf-import-actions button:last-child`);
  assert.equal(await page.$eval('#pdf-import-limits', el => el.value), '');
  const defaultPolicy = await page.evaluate(() => ({shown:JSON.parse(document.querySelector('.dialog-pdf-import pre').textContent),
    expected:window.__wasm.getPdfImportPolicy({resourceProfile:'constrained'})}));
  assert.deepEqual(defaultPolicy.shown, defaultPolicy.expected, '기본 한도 사용은 선택한 프로필만 유지');
  await fill(page, '#pdf-import-limits', '{');
  assert.equal(await page.$eval('#pdf-import-limits', el => el.getAttribute('aria-invalid')), 'true');
  await page.click(`${dialog} .dialog-btn-primary`);
  assert.equal(await page.evaluate(() => document.activeElement.id), 'pdf-import-limits');
  assert.equal(await page.evaluate(() => window.__pdfReads8411), 0);
  await fill(page, '#pdf-import-limits', JSON.stringify({backend:{inputBytes:1000000000}}));
  assert.equal(await page.$eval('#pdf-import-limits', el => el.getAttribute('aria-invalid')), 'true', '프로필 상한 유지');
  const limits = {backend:{inputBytes:1048576}, maxSelectedPages:8};
  await fill(page, '#pdf-import-limits', JSON.stringify(limits));
  assert.equal(await page.$eval('#pdf-import-limits', el => el.getAttribute('aria-invalid')), null);
  const comparison = await page.evaluate(limits => ({
    shown:JSON.parse(document.querySelector('.dialog-pdf-import pre').textContent),
    expected:window.__wasm.getPdfImportPolicy({resourceProfile:'constrained',resourceLimits:limits}),
  }), limits);
  assert.deepEqual(comparison.shown, comparison.expected, '표시 한도와 실제 정책이 일치');
  await fill(page, '#pdf-import-timeout', '301'); await page.click(`${dialog} .dialog-btn-primary`);
  assert.equal(await page.evaluate(() => document.activeElement.id), 'pdf-import-timeout');
  assert.equal(await page.$eval('#pdf-import-timeout', el => el.getAttribute('aria-invalid')), 'true');
  await fill(page, '#pdf-import-timeout', '30');
  for (const theme of ['light','dark']) {
    await page.evaluate(async mode => {const {setThemeMode} = await import('/src/core/theme.ts');setThemeMode(mode);}, theme);
    await page.$eval(advanced, el => {el.open = false;});
    await (await page.$(dialog)).screenshot({path:output+`after-${theme}-basic.png`});
    await page.$eval(advanced, el => {el.open = true;});
    await page.$eval(expert, el => {el.open = false;});
    await (await page.$(dialog)).screenshot({path:output+`after-${theme}-advanced.png`});
    await page.$eval(expert, el => {el.open = true;});
    await (await page.$(dialog)).screenshot({path:output+`after-${theme}-expert.png`});
    await page.setViewport({width:420,height:320}); await footerVisible(page);
    await page.$eval(`${dialog} .dialog-body`, el => {el.scrollTop = el.scrollHeight;});
    await footerVisible(page);
    await page.screenshot({path:output+`after-${theme}-small.png`});
    await page.setViewport({width:1280,height:900});
  }
  console.log('PASS: 2개 테마·작은 화면·긴 파일명·키보드·유효 한도/입력 오류');
  await page.evaluate(() => {window.__pdfHoldRead8411 = true;});
  await fill(page, '#pdf-import-timeout', '1'); await page.click(`${dialog} .dialog-btn-primary`);
  assert.equal(await page.$eval(`${dialog} select`, el => el.disabled), true);
  assert.equal(await page.$$eval(`${expert} .dialog-pdf-import-actions button`, buttons => buttons.every(b => b.disabled)), true);
  assert.equal(await page.$eval(`${dialog} .dialog-footer .dialog-btn:not(.dialog-btn-primary)`, el => el.disabled), false);
  await page.waitForFunction(() => document.querySelector('.dialog-pdf-import .dialog-btn-primary')?.textContent === '다시 시도');
  assert.match(await page.$eval(`${dialog} [role="alert"]`, el => el.textContent), /PDF_IMPORT_TIMEOUT/);
  assert.equal(await page.$eval(`${dialog} select`, el => el.disabled), false);
  const errorVisible = await page.$eval(`${dialog} [role="alert"]`, el => {
    const r = el.getBoundingClientRect(); const b = el.closest('.dialog-body').getBoundingClientRect();
    return r.top >= b.top && r.bottom <= b.bottom;
  });
  assert.equal(errorVisible, true, '실패 메시지를 내용 스크롤 안에서 실제로 볼 수 있음');
  await (await page.$(dialog)).screenshot({path:output+'after-timeout.png'});
  await page.evaluate(() => {window.__pdfHoldRead8411 = false;});
  await fill(page, '#pdf-import-timeout', '30'); await page.click(`${dialog} .dialog-btn-primary`);
  await page.waitForFunction(() => window.__pdfResult8411 !== null, {timeout:45000});
  assert.equal((await page.evaluate(() => window.__pdfResult8411)).ok, true);
  assert.deepEqual(await page.evaluate(() => window.__pdfRequests8411), [{resourceProfile:'constrained',resourceLimits:limits}]);
  console.log('PASS: 실제 1초 제한·실패 후 재시도·동일 한도의 Worker 전달과 PDF 변환');
  await open('image-patterns.pdf');
  for (const theme of ['light','dark']) {
    await page.evaluate(async mode => {const {setThemeMode} = await import('/src/core/theme.ts');setThemeMode(mode);}, theme);
    await (await page.$(dialog)).screenshot({path:output+`default-${theme}.png`});
  }
  await page.keyboard.press('Escape');
  await page.waitForFunction(() => window.__pdfResult8411 !== null);
  assert.equal((await page.evaluate(() => window.__pdfResult8411)).code, 'PDF_IMPORT_CANCELLED');
  console.log('PASS: Escape 취소');
});
