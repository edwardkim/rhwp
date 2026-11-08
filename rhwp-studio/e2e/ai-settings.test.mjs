/**
 * E2E 테스트 — AI 설정·키 보관·전송 동의 (P0 Stage 2)
 *
 * Anthropic API 는 요청 가로채기로 흉내 낸다 — 실제 키도 네트워크도 쓰지 않는다.
 *
 * 검증 항목:
 * 1. 도구 메뉴에 「AI 도우미」 → 「AI 설정」 순서로 항목이 있다
 * 2. 키가 없으면 패널은 설정 안내를 보인다
 * 3. 연결 테스트는 모델 조회 한 번으로 확인한다(브라우저 직접 접근 헤더, 키 헤더)
 *    성공 시 모델 이름, 401 이면 키 오류 문구
 * 4. 기억 + 암호로 저장하면 IndexedDB 에 암호문만 남고 설정·전역 객체·콘솔에 키가 없다
 * 5. 동의 전에는 「동의 필요」, 동의하면 「준비됨」
 * 6. 새로고침하면 「잠김」 — 틀린 암호는 거절, 맞는 암호로 「준비됨」, 동의는 유지
 * 7. 키 삭제 후 「키 없음」, IndexedDB 도 비워진다
 *
 * 계획: mydocs/plans/rhwp_studio_ai_chat_p0_impl.md §3.6, §3.7
 */

import { runTest, loadApp, screenshot, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const GOOD_KEY = 'sk-ant-api03-e2e-GOODKEY-0123456789abcdef';
const BAD_KEY = 'sk-ant-api03-e2e-BADKEY-0123456789abcdef';
const PASSPHRASE = 'e2e 암호';

const wait = (page, ms) => page.evaluate((t) => new Promise((r) => setTimeout(r, t)), ms);
const panelState = (page) => page.evaluate(() =>
  window.rhwpStudio.plugins.invoke('ai-assistant', 'panelState', []));
const waitState = (page, state) => page.waitForFunction(
  (s) => window.rhwpStudio.plugins.invoke('ai-assistant', 'panelState', []) === s,
  { timeout: 5000 }, state,
);
const lastDialog = '.modal-overlay:last-of-type .dialog-wrap';
// 비밀번호 입력란은 세 번 클릭으로 전체 선택되지 않는다 — 값을 비운 뒤 입력한다.
const retype = async (page, selector, text) => {
  await page.$eval(selector, (el) => { el.value = ''; });
  await page.type(selector, text);
};

async function ready(page) {
  await page.waitForFunction(
    () => window.rhwpStudio?.plugins?.isLoaded?.('ai-assistant'),
    { timeout: 10000 },
  );
}

runTest('AI 설정과 키 보관', async ({ page }) => {
  const apiCalls = [];
  const consoleText = [];
  page.on('console', (msg) => consoleText.push(msg.text()));
  await page.setRequestInterception(true);
  page.on('request', (request) => {
    const url = request.url();
    if (!url.startsWith('https://api.anthropic.com/')) {
      request.continue();
      return;
    }
    const headers = request.headers();
    apiCalls.push({ url, method: request.method(), headers });
    const cors = {
      'access-control-allow-origin': '*',
      'access-control-allow-headers': '*',
      'access-control-allow-methods': 'GET, POST, OPTIONS',
    };
    if (request.method() === 'OPTIONS') {
      request.respond({ status: 204, headers: cors });
      return;
    }
    const ok = headers.authorization === `Bearer ${GOOD_KEY}`;
    request.respond({
      status: ok ? 200 : 401,
      headers: { ...cors, 'content-type': 'application/json' },
      body: JSON.stringify(ok
        ? { type: 'model', id: 'claude-opus-5-5', display_name: 'Claude Opus 5.5', created_at: '2026-01-01T00:00:00Z' }
        : { type: 'error', error: { type: 'authentication_error', message: 'invalid credentials' } }),
    });
  });

  await loadApp(page);
  await ready(page);

  // ── TC1: 메뉴 순서 ────────────────────────────────────
  const menu = await page.evaluate(() => [...document.querySelectorAll(
    '.menu-item[data-menu="tool"] .md-item[data-cmd]')].map((el) => el.dataset.cmd));
  assert(menu[0] === 'ext:ai.toggle-panel' && menu[1] === 'ext:ai.settings',
    `TC1: 도구 메뉴 순서 (${menu.join(', ')})`);

  // ── TC2: 키 없음 ──────────────────────────────────────
  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [true]));
  await waitState(page, 'none');
  assert(await panelState(page) === 'none', 'TC2: 키가 없으면 none');

  // ── TC3: 연결 테스트 ──────────────────────────────────
  await page.evaluate(() => window.rhwpStudio.automation.execute('ext:ai.settings', {}, { allowDialog: true }));
  await page.waitForSelector('#ai-settings-key');
  await page.type('#ai-settings-key', BAD_KEY);
  await page.click(`${lastDialog} .ai-settings-actions .dialog-btn`);
  await page.waitForFunction(() => document.querySelector('.ai-settings-status')?.dataset.kind === 'error');
  const badText = await page.$eval('.ai-settings-status', (el) => el.textContent);
  assert(badText.includes('API 키가 올바르지 않습니다'), `TC3: 401 → 키 오류 문구 (${badText})`);

  await retype(page, '#ai-settings-key', GOOD_KEY);
  await page.click(`${lastDialog} .ai-settings-actions .dialog-btn`);
  await page.waitForFunction(() => document.querySelector('.ai-settings-status')?.dataset.kind === 'ok');
  const okText = await page.$eval('.ai-settings-status', (el) => el.textContent);
  assert(okText.includes('Claude Opus 5.5'), `TC3: 성공 → 모델 이름 (${okText})`);
  const getCalls = apiCalls.filter((c) => c.method === 'GET');
  assert(getCalls.length === 2 && getCalls.every((c) => c.url.includes('/v1/models/claude-opus-5-5')),
    `TC3: 모델 조회만 호출 (${getCalls.map((c) => c.url).join(', ')})`);
  assert(getCalls.every((c) => c.headers['anthropic-dangerous-direct-browser-access'] === 'true'),
    'TC3: 브라우저 직접 접근 헤더');
  assert(getCalls[0].headers.authorization === `Bearer ${BAD_KEY}`
    && getCalls[1].headers.authorization === `Bearer ${GOOD_KEY}`
    && getCalls.every((c) => !('x-api-key' in c.headers)
      && c.headers['anthropic-version'] === '2026-10-21'), 'TC3: SDK 1.x 인증과 API 버전');
  assert(apiCalls.every((c) => c.method !== 'POST'), 'TC3: 메시지 요청(토큰 사용) 없음');

  // ── TC4: 기억 + 암호로 저장 ────────────────────────────
  await page.click('#ai-settings-remember');
  await page.type('#ai-settings-passphrase', PASSPHRASE);
  await screenshot(page, 'ai-settings-dialog');
  await page.click(`${lastDialog} .dialog-btn-primary`);
  await page.waitForFunction(() => !document.querySelector('#ai-settings-key'), { timeout: 15000 });
  await waitState(page, 'consent');

  const stored = await page.evaluate(() => new Promise((resolve) => {
    const open = indexedDB.open('rhwp-ai');
    open.onsuccess = () => {
      const req = open.result.transaction('keys').objectStore('keys').get('claude');
      req.onsuccess = () => resolve(req.result ?? null);
    };
  }));
  const settingsRaw = await page.evaluate(() => localStorage.getItem('rhwp-ai-settings'));
  assert(stored?.mode === 'encrypted', `TC4: 암호화 저장 (${stored?.mode})`);
  assert(!JSON.stringify(stored).includes(GOOD_KEY), 'TC4: IndexedDB 에 키 원문 없음');
  assert(JSON.parse(settingsRaw).rememberKey === true && !settingsRaw.includes('sk-ant'),
    'TC4: 설정에는 기억 여부만, 키 없음');
  const leaked = await page.evaluate((key) => {
    try { return JSON.stringify(window.rhwpStudio, (_k, v) => (typeof v === 'function' ? undefined : v)).includes(key); }
    catch { return false; }
  }, GOOD_KEY);
  assert(!leaked, 'TC4: 전역 객체에 키 없음');

  // ── TC5: 동의 ─────────────────────────────────────────
  await page.click('#side-panel-host .ai-panel-action');
  await page.waitForSelector(`${lastDialog} .dialog-btn-primary`);
  const consentText = await page.$eval(`${lastDialog} .dialog-body`, (el) => el.textContent);
  assert(consentText.includes('Claude (Anthropic)'), 'TC5: 동의 문구에 제공자 이름');
  await page.click(`${lastDialog} .dialog-btn-primary`);
  await waitState(page, 'ready');
  assert(await panelState(page) === 'ready', 'TC5: 동의 후 ready');
  await screenshot(page, 'ai-panel-ready');

  // ── TC6: 새로고침 → 잠김 → 잠금 해제 ────────────────────
  await loadApp(page);
  await ready(page);
  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [true]));
  await waitState(page, 'locked');
  assert(await panelState(page) === 'locked', 'TC6: 새 실행에서는 잠김');
  await page.click('#side-panel-host .ai-panel-action');
  await page.waitForSelector(`${lastDialog} input[type="password"]`);
  await page.type(`${lastDialog} input[type="password"]`, '틀린 암호');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => document.querySelector('.ai-settings-status')?.dataset.kind === 'error',
    { timeout: 15000 });
  assert(await panelState(page) === 'locked', 'TC6: 틀린 암호는 거절 (Enter 로 제출)');
  await retype(page, `${lastDialog} input[type="password"]`, PASSPHRASE);
  await page.click(`${lastDialog} .dialog-btn-primary`);
  await waitState(page, 'ready');
  assert(await panelState(page) === 'ready', 'TC6: 맞는 암호로 ready, 동의 유지');

  // ── TC7: 키 삭제 ──────────────────────────────────────
  await page.evaluate(() => window.rhwpStudio.automation.execute('ext:ai.settings', {}, { allowDialog: true }));
  await page.waitForSelector('#ai-settings-key');
  const keyState = await page.$eval('.ai-settings-key-state', (el) => el.textContent);
  assert(keyState.includes('sk-ant-…') && !keyState.includes(GOOD_KEY), `TC7: 가린 키 표시 (${keyState})`);
  await page.click(`${lastDialog} .ai-settings-actions .dialog-btn:nth-child(2)`);
  await page.waitForFunction(() => document.querySelector('.ai-settings-key-state')?.textContent.includes('없습니다'));
  await page.click(`${lastDialog} .dialog-close`);
  await waitState(page, 'none');
  const afterClear = await page.evaluate(() => new Promise((resolve) => {
    const open = indexedDB.open('rhwp-ai');
    open.onsuccess = () => {
      const req = open.result.transaction('keys').objectStore('keys').get('claude');
      req.onsuccess = () => resolve(req.result ?? null);
    };
  }));
  assert(afterClear === null, 'TC7: 저장소 비워짐');
  assert(!consoleText.some((line) => line.includes(GOOD_KEY) || line.includes(BAD_KEY)), 'TC7: 콘솔에 키 없음');
  await wait(page, 100);
});
