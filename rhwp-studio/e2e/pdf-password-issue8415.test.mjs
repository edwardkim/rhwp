/** #8415: 실제 파일 열기·Worker 인증·재시도·취소 UI 계약. 픽셀 일치율 검사는 별도로 수행한다. */
import assert from 'node:assert/strict';
import {mkdirSync, readFileSync} from 'node:fs';
import {runTest} from './helpers.mjs';
const source = new URL('../../pdf/issue8415/authentication/password-r6.pdf', import.meta.url);
const encrypted = Array.from(readFileSync(source));
const output = new URL('../../output/pdf-password-issue8415/', import.meta.url).pathname;
mkdirSync(output, {recursive:true});
const dialog = '.dialog-pdf-import';
const confirm = `${dialog} .dialog-btn-primary`;
async function open(page) {
  await page.evaluate(bytes => {
    window.__passwordOpen8415 = null;
    window.__eventBus.emit('open-document-bytes', {bytes: new Uint8Array(bytes),
      fileName:'password-r6.pdf', fileHandle:null, skipUnsavedGuard:true, requestId:'password-8415'});
  }, encrypted);
  await page.waitForSelector(dialog);
}
async function done(page) {
  await page.waitForFunction(() => {
    const prompt = [...document.querySelectorAll('.modal-overlay')].find(el =>
      el.querySelector('.dialog-title')?.textContent.includes('로컬 글꼴 감지'));
    [...(prompt?.querySelectorAll('button') ?? [])].find(el => el.textContent === '대체 글꼴로 보기')?.click();
    return window.__passwordOpen8415 !== null && !document.documentElement.classList.contains('rhwp-busy');
  }, {timeout:120000});
  return page.evaluate(() => window.__passwordOpen8415);
}
await runTest('암호 PDF 열기·오입력·재시도·취소 (#8415)', async ({page}) => {
  await page.evaluate(() => {
    window.__eventBus.on('open-document-bytes:done', result => {
      if (result.requestId === 'password-8415') window.__passwordOpen8415 = result;
    });
  });
  const generation = await page.evaluate(() => window.__wasm.documentGeneration);
  await open(page); await page.click(confirm);
  await page.waitForFunction(() => document.querySelector('#pdf-import-password')?.checkVisibility());
  assert.equal(await page.evaluate(() => document.activeElement.id), 'pdf-import-password');
  assert.equal(await page.evaluate(() => window.__wasm.documentGeneration), generation);
  assert.equal(await page.$eval('#pdf-import-password', el => el.type), 'password');
  assert.notEqual(await page.$eval('#pdf-import-password', el => getComputedStyle(el).cursor), 'wait');
  await (await page.$(dialog)).screenshot({path:output+'password-required.png'});
  await page.type('#pdf-import-password', 'wrong'); await page.click(confirm);
  await page.waitForFunction(() => document.querySelector('#pdf-import-error')?.textContent.includes('올바르지'));
  assert.equal(await page.$eval('#pdf-import-password', el => el.value), '');
  assert.equal(await page.evaluate(() => window.__wasm.documentGeneration), generation);
  await (await page.$(dialog)).screenshot({path:output+'password-invalid.png'});
  await page.type('#pdf-import-password', 'synthetic-open');
  await page.$eval('#pdf-import-password', el => el.dispatchEvent(
    new KeyboardEvent('keydown', {key:'Enter', bubbles:true, isComposing:true})));
  assert.equal(await page.$eval(confirm, el => el.disabled), false, 'IME 조합 확정 Enter는 제출하지 않음');
  await page.keyboard.press('Enter');
  assert.equal((await done(page)).ok, true, '실제 편집 문서 게시까지 성공');
  assert.equal(await page.evaluate(() => window.__wasm.fileName), 'password-r6.pdf');
  assert.equal(await page.evaluate(() => window.__wasm.pageCount), 1);
  assert.equal(await page.$(dialog), null);
  const opened = await page.evaluate(() => window.__wasm.documentGeneration);
  await open(page); await page.click(confirm);
  await page.waitForFunction(() => document.querySelector('#pdf-import-password')?.checkVisibility());
  await page.type('#pdf-import-password', 'synthetic-open'); await page.keyboard.press('Escape');
  assert.equal((await done(page)).ok, false);
  assert.equal(await page.evaluate(() => window.__wasm.documentGeneration), opened, '암호 입력 취소 시 기존 문서 유지');
  assert.equal(await page.$('#pdf-import-password'), null, '취소 뒤 입력 DOM 제거');
  console.log('PASS: 실제 Worker 인증·오입력·Enter 재시도·편집 문서 게시·Escape 취소');
});
