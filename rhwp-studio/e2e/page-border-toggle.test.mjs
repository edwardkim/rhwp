/**
 * E2E 테스트 — 쪽 테두리/배경 미리보기 버튼 토글
 *
 * 검증 항목:
 * 1. 개별 방향 버튼은 두 번 클릭하면 원래 상태로 복귀한다.
 * 2. 전체 버튼은 전체 적용과 전체 해제를 토글한다.
 * 3. 선 모양 바로 적용은 현재 켜진 방향에만 반영된다.
 * 4. 테두리 사용 안 함 상태에서 전체/개별 버튼은 첫 클릭으로 기대 상태를 적용한다.
 */

import {
  runTest, createNewDocument, assert,
} from './helpers.mjs';

const delay = (ms = 150) => new Promise((resolve) => setTimeout(resolve, ms));

async function openMenuCommand(page, commandId) {
  await page.evaluate((cmd) => {
    const item = document.querySelector(`.md-item[data-cmd="${cmd}"]`);
    if (!item) throw new Error(`메뉴 항목을 찾을 수 없습니다: ${cmd}`);

    const menu = item.closest('.menu-item');
    const title = menu?.querySelector('.menu-title');
    title?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));

    if (item.classList.contains('disabled')) {
      throw new Error(`메뉴 항목이 비활성 상태입니다: ${cmd}`);
    }
    item.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  }, commandId);
  await delay(300);
}

async function openPageBorderDialog(page) {
  await openMenuCommand(page, 'page:page-border');
  await page.waitForSelector('.page-border-side-btn', { timeout: 5000 });
}

async function clickPageBorderButton(page, title) {
  await page.evaluate((buttonTitle) => {
    const button = Array.from(document.querySelectorAll('.page-border-side-btn'))
      .find((candidate) => candidate.getAttribute('title') === buttonTitle);
    if (!button) throw new Error(`쪽 테두리 버튼을 찾을 수 없습니다: ${buttonTitle}`);
    button.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  }, title);
  await delay();
}

async function getPageBorderState(page) {
  return await page.evaluate(() => {
    const svg = document.querySelector('.dialog-wrap svg');
    const noneCheck = document.querySelector('input[data-label="테두리 사용 안 함"]');
    return {
      lineCount: svg?.querySelectorAll('line').length ?? -1,
      noneChecked: Boolean(noneCheck?.checked),
    };
  });
}

async function setBorderNone(page, checked) {
  await page.evaluate((nextChecked) => {
    const noneCheck = document.querySelector('input[data-label="테두리 사용 안 함"]');
    if (!noneCheck) throw new Error('테두리 사용 안 함 체크박스를 찾을 수 없습니다');
    noneCheck.checked = nextChecked;
    noneCheck.dispatchEvent(new Event('change', { bubbles: true }));
  }, checked);
  await delay();
}

async function setLineType(page, value) {
  await page.evaluate((nextValue) => {
    const lineTypeSelect = document.querySelector('.dialog-wrap select.dialog-select');
    if (!lineTypeSelect) throw new Error('선 종류 select를 찾을 수 없습니다');
    lineTypeSelect.value = nextValue;
    lineTypeSelect.dispatchEvent(new Event('change', { bubbles: true }));
  }, value);
  await delay();
}

runTest('쪽 테두리/배경 미리보기 버튼 토글', async ({ page }) => {
  await createNewDocument(page);
  await openPageBorderDialog(page);

  const initial = await getPageBorderState(page);
  assert(initial.lineCount === 0, 'TC1: 새 문서의 쪽 테두리 preview는 선 없음 상태다');
  assert(initial.noneChecked, 'TC1: 새 문서의 테두리 사용 안 함 체크가 켜져 있다');

  await clickPageBorderButton(page, '위쪽');
  const topOn = await getPageBorderState(page);
  assert(topOn.lineCount === 1, 'TC2: 위쪽 버튼 1회 클릭 시 preview 선이 1개가 된다');
  assert(!topOn.noneChecked, 'TC2: 위쪽 버튼 1회 클릭 시 테두리 사용 안 함 체크가 꺼진다');

  await clickPageBorderButton(page, '위쪽');
  const topOff = await getPageBorderState(page);
  assert(topOff.lineCount === 0, 'TC2: 위쪽 버튼 2회 클릭 시 preview 선이 다시 0개가 된다');
  assert(topOff.noneChecked, 'TC2: 위쪽 버튼 2회 클릭 시 테두리 사용 안 함 체크가 다시 켜진다');

  await clickPageBorderButton(page, '모두');
  const allOn = await getPageBorderState(page);
  assert(allOn.lineCount === 4, 'TC3: 모두 버튼 1회 클릭 시 사방 선이 켜진다');
  assert(!allOn.noneChecked, 'TC3: 모두 버튼 1회 클릭 시 테두리 사용 안 함 체크가 꺼진다');

  await clickPageBorderButton(page, '모두');
  const allOff = await getPageBorderState(page);
  assert(allOff.lineCount === 0, 'TC3: 모두 버튼 2회 클릭 시 사방 선이 모두 꺼진다');
  assert(allOff.noneChecked, 'TC3: 모두 버튼 2회 클릭 시 테두리 사용 안 함 체크가 다시 켜진다');

  await clickPageBorderButton(page, '위쪽');
  await setLineType(page, '2');
  const immediate = await getPageBorderState(page);
  assert(immediate.lineCount === 1, 'TC4: 선 모양 바로 적용은 켜진 방향 1개에만 반영된다');
  assert(!immediate.noneChecked, 'TC4: 선 모양 바로 적용 후에도 활성 테두리 상태가 유지된다');

  await clickPageBorderButton(page, '모두');
  const allOnBeforeNone = await getPageBorderState(page);
  assert(allOnBeforeNone.lineCount === 4, 'TC5: 테두리 사용 안 함 확인 전 사방 선을 켠다');

  await setBorderNone(page, true);
  const noneAfterAllOn = await getPageBorderState(page);
  assert(noneAfterAllOn.lineCount === 0, 'TC5: 테두리 사용 안 함 체크 시 preview 선이 모두 사라진다');
  assert(noneAfterAllOn.noneChecked, 'TC5: 테두리 사용 안 함 체크 상태가 유지된다');

  await clickPageBorderButton(page, '모두');
  const allOnAfterNone = await getPageBorderState(page);
  assert(allOnAfterNone.lineCount === 4, 'TC5: 테두리 사용 안 함 상태에서 모두 버튼 1회 클릭으로 사방 선이 켜진다');
  assert(!allOnAfterNone.noneChecked, 'TC5: 모두 버튼 적용 시 테두리 사용 안 함 체크가 꺼진다');

  await setBorderNone(page, true);
  await clickPageBorderButton(page, '위쪽');
  const topOnlyAfterNone = await getPageBorderState(page);
  assert(topOnlyAfterNone.lineCount === 1, 'TC6: 테두리 사용 안 함 상태에서 위쪽 버튼 클릭 시 위쪽 선만 켜진다');
  assert(!topOnlyAfterNone.noneChecked, 'TC6: 위쪽 버튼 적용 시 테두리 사용 안 함 체크가 꺼진다');
});
