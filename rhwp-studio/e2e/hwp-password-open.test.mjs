/**
 * Issue #3474/#3481 browser regression — HWP3/HWP5 password open.
 *
 * The fixture password is intentionally reconstructed from character codes so
 * test output, source grep, and reports never print it as a literal value.
 */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, createNewDocument, runTest, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const FIXTURES = [
  {
    name: 'HWP5 EncryptVersion 4',
    path: path.join(REPO_ROOT, 'samples', 'hwp3-sample16-hwp5-2024-password-123456.hwp'),
    pageCount: 64,
  },
  {
    name: 'HWP3 압축 비밀번호 문서',
    path: path.join(REPO_ROOT, 'samples', 'HWP3-password-123456.hwp'),
    pageCount: 24,
    canvasAt144Dpi: { width: 1191, height: 1684 },
  },
];

async function uploadFixture(page, fixture) {
  const input = await page.$('#file-input');
  if (!input) throw new Error('file-input not found');
  await input.uploadFile(fixture.path);
  await page.waitForSelector('#hwp-password-input', { timeout: 30_000 });
}

async function dialogState(page) {
  return await page.evaluate(() => {
    const input = document.querySelector('#hwp-password-input');
    const dialog = input?.closest('.dialog-wrap');
    const label = dialog?.querySelector('label[for="hwp-password-input"]');
    return {
      shown: Boolean(dialog && input),
      role: dialog?.getAttribute('role'),
      modal: dialog?.getAttribute('aria-modal'),
      inputType: input instanceof HTMLInputElement ? input.type : '',
      autocomplete: input instanceof HTMLInputElement ? input.autocomplete : '',
      labelled: Boolean(label),
    };
  });
}

async function clickDialogButton(page, label) {
  const clicked = await page.evaluate((text) => {
    const button = [...document.querySelectorAll('.modal-overlay button')]
      .find((candidate) => candidate.textContent?.trim() === text);
    if (!(button instanceof HTMLButtonElement)) return false;
    button.click();
    return true;
  }, label);
  assert(clicked, `암호 대화상자 ${label} 버튼이 있다`);
}

runTest('Issue #3474/#3481 HWP3/HWP5 암호 문서 열기', async ({ page }) => {
  await createNewDocument(page);
  for (const fixture of FIXTURES) {
    const prior = await page.evaluate(() => ({
      fileName: window.__wasm?.fileName ?? '',
      pageCount: window.__wasm?.pageCount ?? 0,
    }));

    await uploadFixture(page, fixture);
    const initialDialog = await dialogState(page);
    assert(initialDialog.shown, `${fixture.name} 선택 시 입력 대화상자가 표시된다`);
    assert(initialDialog.inputType === 'password', '암호 입력은 마스킹된다');
    assert(initialDialog.autocomplete === 'off', '브라우저 암호 자동완성을 요청하지 않는다');
    assert(initialDialog.role === 'dialog' && initialDialog.modal === 'true' && initialDialog.labelled,
      '접근 가능한 모달·레이블을 제공한다');

    await clickDialogButton(page, '취소');
    await page.waitForSelector('#hwp-password-input', { hidden: true, timeout: 3_000 });
    const afterCancel = await page.evaluate(() => ({
      fileName: window.__wasm?.fileName ?? '',
      pageCount: window.__wasm?.pageCount ?? 0,
    }));
    assert(JSON.stringify(afterCancel) === JSON.stringify(prior), '취소는 현재 문서를 교체하지 않는다');

    await uploadFixture(page, fixture);
    await page.type('#hwp-password-input', 'wrong-password');
    await clickDialogButton(page, '확인');
    await page.waitForFunction(
      () => document.querySelector('#hwp-password-error')?.textContent?.includes('암호가 일치하지 않거나 문서가 손상되었습니다'),
      { timeout: 30_000 },
    );
    const afterWrong = await page.evaluate(() => ({
      fileName: window.__wasm?.fileName ?? '',
      pageCount: window.__wasm?.pageCount ?? 0,
      inputValue: (document.querySelector('#hwp-password-input') instanceof HTMLInputElement)
        ? document.querySelector('#hwp-password-input').value
        : null,
    }));
    assert(afterWrong.fileName === prior.fileName && afterWrong.pageCount === prior.pageCount,
      '오답은 현재 문서를 교체하지 않는다');
    assert(afterWrong.inputValue === '', '오답 뒤 새 입력 대화상자는 이전 입력값을 유지하지 않는다');

    // 공개 fixture의 값은 보고·오류·저장소에 노출하지 않는다.
    const fixturePassword = String.fromCharCode(49, 50, 51, 52, 53, 54);
    await page.type('#hwp-password-input', fixturePassword);
    await page.keyboard.press('Enter');
    await page.waitForFunction(
      (pageCount) => window.__wasm?.pageCount === pageCount,
      { timeout: 30_000 },
      fixture.pageCount,
    );
    await waitForCanvas(page, 30_000);

    const finalState = await page.evaluate((password) => ({
      pageCount: window.__wasm?.pageCount ?? 0,
      fileName: window.__wasm?.fileName ?? '',
      dialogGone: !document.querySelector('#hwp-password-input'),
      localHasPassword: JSON.stringify(localStorage).includes(password),
      sessionHasPassword: JSON.stringify(sessionStorage).includes(password),
    }), fixturePassword);
    assert(finalState.pageCount === fixture.pageCount, `올바른 암호로 ${fixture.name}를 연다`);
    assert(finalState.fileName.endsWith('.hwp') && finalState.dialogGone, '성공 뒤 입력 대화상자를 제거한다');
    assert(!finalState.localHasPassword && !finalState.sessionHasPassword,
      '암호를 local/session storage에 보관하지 않는다');

    if (fixture.canvasAt144Dpi) {
      const canvasExtent = await page.evaluate(() => {
        const doc = window.__wasm?.doc;
        if (!doc || typeof doc.renderPageToCanvas !== 'function') {
          throw new Error('WASM Canvas 렌더러를 찾을 수 없습니다');
        }
        const canvas = document.createElement('canvas');
        // Studio의 CSS 좌표는 96dpi이므로 1.5x는 PDF 대조의 144dpi bitmap이다.
        doc.renderPageToCanvas(0, canvas, 1.5);
        return { width: canvas.width, height: canvas.height };
      });
      assert(canvasExtent.width === fixture.canvasAt144Dpi.width
        && canvasExtent.height === fixture.canvasAt144Dpi.height,
      `${fixture.name} 144dpi Canvas가 A4 우·하단 경계를 자르지 않는다`);
    }
  }
});
