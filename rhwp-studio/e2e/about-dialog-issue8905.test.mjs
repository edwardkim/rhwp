/** #8905: 낮거나 좁은 화면에서도 제품 정보 창의 닫기 수단이 화면 안에 있고 실제로 닫힌다. UI 계약이며 문서 조판 검사가 아니다. */
import { runTest, assert } from './helpers.mjs';

// 구현 class가 아니라 구조로 고른다. 수정 전 코드에서도 같은 창을 찾아 배치 결함으로 실패한다.
const DIALOG = '.dialog-wrap:has(> .about-body)';

const VIEWPORTS = [
  { name: '가로로 눕힌 휴대폰', width: 800, height: 360 },
  { name: '세로 휴대폰', width: 360, height: 640 },
  { name: '데스크톱', width: 1280, height: 900 },
];

async function openAbout(page) {
  await page.evaluate(async () => {
    const { AboutDialog } = await import('/src/ui/about-dialog.ts');
    new AboutDialog().show();
  });
  await page.waitForSelector(DIALOG);
}

/** 대화상자와 닫기 수단의 화면 좌표, 본문 스크롤 상태를 읽는다. */
async function measure(page) {
  return page.evaluate((dialog) => {
    const rect = (selector) => {
      const r = document.querySelector(`${dialog}${selector}`).getBoundingClientRect();
      return { top: r.top, bottom: r.bottom, left: r.left, right: r.right };
    };
    const body = document.querySelector(`${dialog} > .about-body`);
    return {
      viewport: { width: innerWidth, height: innerHeight },
      dialog: rect(''),
      titleClose: rect(' .dialog-close'),
      footerClose: rect(' .dialog-footer .dialog-btn'),
      bodyScrollHeight: body.scrollHeight,
      bodyClientHeight: body.clientHeight,
      bodyOverflowY: getComputedStyle(body).overflowY,
    };
  }, DIALOG);
}

function inside(box, viewport) {
  return box.top >= 0 && box.left >= 0 && box.bottom <= viewport.height && box.right <= viewport.width;
}

const closed = (page) => page.waitForSelector(DIALOG, { hidden: true, timeout: 3000 });

await runTest('Issue #8905 제품 정보 창이 작은 화면에서도 화면 안에 있고 닫힌다', async ({ page }) => {
  for (const vp of VIEWPORTS) {
    await page.setViewport({ width: vp.width, height: vp.height });
    await openAbout(page);
    const m = await measure(page);
    const detail = `${vp.name} ${vp.width}x${vp.height}: ${JSON.stringify(m)}`;

    assert(inside(m.dialog, m.viewport), `대화상자 전체가 화면 안에 있다 — ${detail}`);
    assert(inside(m.titleClose, m.viewport), `제목 줄 닫기(×) 버튼이 화면 안에 있다 — ${detail}`);
    assert(inside(m.footerClose, m.viewport), `하단 닫기 버튼이 화면 안에 있다 — ${detail}`);
    if (m.bodyScrollHeight > m.bodyClientHeight) {
      assert(m.bodyOverflowY === 'auto', `본문이 넘치면 본문 안에서 스크롤한다 — ${detail}`);
      const scrolled = await page.evaluate((dialog) => {
        const body = document.querySelector(`${dialog} > .about-body`);
        body.scrollTop = body.scrollHeight;
        const last = body.querySelector('.about-copyright').getBoundingClientRect();
        const box = body.getBoundingClientRect();
        return { scrollTop: body.scrollTop, lastVisible: last.bottom <= box.bottom + 1 && last.top >= box.top - 1 };
      }, DIALOG);
      assert(scrolled.scrollTop > 0 && scrolled.lastVisible, `스크롤하면 본문 끝(저작권 줄)까지 볼 수 있다 — ${vp.name}`);
    }

    // 1) 하단 닫기 버튼: 실제 좌표를 눌러 닫는다. 가려져 있으면 다른 요소가 눌려 실패한다.
    await page.mouse.click((m.footerClose.left + m.footerClose.right) / 2, (m.footerClose.top + m.footerClose.bottom) / 2);
    await closed(page);

    // 2) 제목 줄 닫기(×) 버튼
    await openAbout(page);
    await page.mouse.click((m.titleClose.left + m.titleClose.right) / 2, (m.titleClose.top + m.titleClose.bottom) / 2);
    await closed(page);

    // 3) 배경: 대화상자 밖 모서리를 누르면 닫히고, 대화상자 안을 눌러서는 닫히지 않는다.
    await openAbout(page);
    await page.mouse.click((m.dialog.left + m.dialog.right) / 2, m.dialog.top + 40);
    assert(await page.$(DIALOG) !== null, `대화상자 안을 눌러서는 닫히지 않는다 — ${vp.name}`);
    await page.mouse.click(3, 3);
    await closed(page);

    // 4) Escape
    await openAbout(page);
    await page.keyboard.press('Escape');
    await closed(page);

    assert(await page.$('.modal-overlay') === null, `닫은 뒤 남은 overlay가 없다 — ${vp.name}`);
  }
});
