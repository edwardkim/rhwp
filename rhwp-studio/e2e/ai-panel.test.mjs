/**
 * E2E 테스트 — AI 대화 패널 (P0 Stage 4)
 *
 * Anthropic Messages API 는 요청 가로채기로 흉내 낸다(SSE 응답). 실제 키·네트워크는 쓰지 않는다.
 *
 * 검증 항목:
 * 1. 질문 → 모델이 read_page_text 를 부름 → 결과(표 칸 글 포함, 문서 경계)를 보냄 → 답 표시
 *    - 첫 요청: 모델·도구 7개(strict)·시스템 규칙·서버 대체·스트리밍
 *    - 둘째 요청: 첫 응답을 그대로 되돌리고 도구 결과는 한 user 메시지
 * 2. 답의 Markdown 은 정해진 요소로, HTML 은 글자로 보인다(<script> 요소 없음)
 * 3. 대화 전후 문서 바이트가 같다(P0 는 읽기 전용)
 * 4. 패널 입력란의 Ctrl+Z 는 문서 undo 로 새지 않는다
 * 5. 선택 영역이 있으면 빠른 동작이 보이고, 누르면 그 프롬프트로 묻는다
 * 6. 중단 단추는 진행 중인 요청을 끊고 「중단했습니다」를 보인다
 * 7. 401 이면 키 오류 문구, 400 이면 거절 문구와 제공자 사유
 * 8. 새 대화 / 문서 교체는 이력을 비운다
 * 9. 변경 제안 카드 → 적용하면 표 칸 글이 바뀌고 undo 한 번으로 돌아간다
 * 10. 거절하면 문서는 그대로이고 모델에 거절을 알린다
 * 11. 새 문서 만들기(저장 확인 포함) → 문단 쓰기, 대화는 이어진다
 * 12. 글자·문단 서식과 표 — 한 응답의 여러 변경을 차례로 승인, undo 는 변경마다 한 단계
 *
 * 계획: mydocs/plans/rhwp_studio_ai_chat_p0_impl.md §5.2
 */

import {
  runTest, loadApp, loadHwpFile, createNewDocument, clickEditArea, typeText, screenshot, assert,
} from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const KEY = 'sk-ant-api03-e2e-PANELKEY-0123456789abcdef';
const lastDialog = '.modal-overlay:last-of-type .dialog-wrap';
const wait = (page, ms) => page.evaluate((t) => new Promise((r) => setTimeout(r, t)), ms);

/** Anthropic SSE 본문을 만든다. blocks: [{type:'text', text}|{type:'tool_use', id, name, input}] */
function sse(blocks, stopReason, usage = { input_tokens: 1200, output_tokens: 40, cache_read_input_tokens: 900 }) {
  const events = [];
  const push = (type, data) => events.push(`event: ${type}\ndata: ${JSON.stringify({ type, ...data })}\n\n`);
  push('message_start', { message: {
    id: 'msg_e2e', type: 'message', role: 'assistant', model: 'claude-opus-5-5', content: [],
    stop_reason: null, stop_sequence: null,
    usage: { input_tokens: usage.input_tokens, output_tokens: 1, cache_read_input_tokens: usage.cache_read_input_tokens, cache_creation_input_tokens: 0 },
  } });
  blocks.forEach((block, index) => {
    if (block.type === 'text') {
      push('content_block_start', { index, content_block: { type: 'text', text: '' } });
      // 글자를 몇 조각으로 나눠 스트리밍처럼 보낸다.
      for (const piece of block.text.match(/[\s\S]{1,12}/g) ?? []) {
        push('content_block_delta', { index, delta: { type: 'text_delta', text: piece } });
      }
    } else {
      push('content_block_start', { index, content_block: { type: 'tool_use', id: block.id, name: block.name, input: {} } });
      push('content_block_delta', { index, delta: { type: 'input_json_delta', partial_json: JSON.stringify(block.input) } });
    }
    push('content_block_stop', { index });
  });
  push('message_delta', { delta: { stop_reason: stopReason, stop_sequence: null }, usage: { output_tokens: usage.output_tokens } });
  push('message_stop', {});
  return events.join('');
}

const ANSWER = '**요약**: 표지에는 사업명이 있습니다.\n\n- 첫째\n- 둘째\n\n<script>window.__aiPwned = 1</script>';

runTest('AI 대화 패널', async ({ page }) => {
  const requests = [];
  /** 다음 messages 요청에 줄 응답들. 함수면 (request) => 응답 설명 */
  const queue = [];
  await page.setRequestInterception(true);
  page.on('request', async (request) => {
    const url = request.url();
    if (!url.startsWith('https://api.anthropic.com/')) { request.continue(); return; }
    const cors = {
      'access-control-allow-origin': '*', 'access-control-allow-headers': '*',
      'access-control-allow-methods': 'GET, POST, OPTIONS',
    };
    if (request.method() === 'OPTIONS') { request.respond({ status: 204, headers: cors }); return; }
    if (url.includes('/v1/models/')) {
      request.respond({ status: 200, headers: { ...cors, 'content-type': 'application/json' },
        body: JSON.stringify({ type: 'model', id: 'claude-opus-5-5', display_name: 'Claude Opus 5.5', created_at: '2026-01-01T00:00:00Z' }) });
      return;
    }
    requests.push({ headers: request.headers(), body: JSON.parse(request.postData() ?? '{}') });
    const next = queue.shift() ?? { status: 500, body: '{}' };
    if (next.delayMs) await new Promise((r) => setTimeout(r, next.delayMs));
    try {
      request.respond({
        status: next.status ?? 200,
        headers: { ...cors, 'content-type': next.status && next.status !== 200 ? 'application/json' : 'text/event-stream' },
        body: next.body,
      });
    } catch { /* 중단된 요청 */ }
  });

  await loadApp(page);
  await page.waitForFunction(() => window.rhwpStudio?.plugins?.isLoaded?.('ai-assistant'), { timeout: 10000 });
  await loadHwpFile(page, 'biz_plan.hwp');

  // ── 준비: 키(세션) + 동의 ─────────────────────────────
  await page.evaluate(() => window.rhwpStudio.automation.execute('ext:ai.settings', {}, { allowDialog: true }));
  await page.waitForSelector('#ai-settings-key');
  await page.type('#ai-settings-key', KEY);
  await page.click(`${lastDialog} .dialog-btn-primary`);
  await page.waitForFunction(() => !document.querySelector('#ai-settings-key'));
  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [true]));
  await page.waitForSelector(`${lastDialog} .dialog-btn-primary`);
  await page.click(`${lastDialog} .dialog-btn-primary`);
  await page.waitForFunction(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'panelState', []) === 'ready');
  const focused = await page.evaluate(() => document.activeElement?.classList.contains('ai-chat-input'));
  assert(focused, 'TC0: 준비되면 입력란에 포커스');

  const docHash = () => page.evaluate(async () => {
    const bytes = window.__wasm.exportHwp();
    const digest = await crypto.subtle.digest('SHA-256', bytes);
    return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, '0')).join('');
  });
  const hashBefore = await docHash();

  // ── TC1: 질문 → 도구 → 답 ─────────────────────────────
  queue.push({ body: sse([
    { type: 'text', text: '표지를 읽어 볼게요.' },
    { type: 'tool_use', id: 'toolu_e2e_1', name: 'read_page_text', input: { page: 1 } },
  ], 'tool_use') });
  queue.push({ body: sse([{ type: 'text', text: ANSWER }], 'end_turn') });
  await page.type('.ai-chat-input', '표지에 뭐가 있어?');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy
    && document.querySelector('.ai-msg-assistant:not([aria-busy])'), { timeout: 15000 });

  assert(requests.length === 2, `TC1: 요청 2회 (${requests.length})`);
  const [first, second] = requests;
  assert(first.body.model === 'claude-opus-5-5' && first.body.stream === true, 'TC1: 모델·스트리밍');
  assert(first.body.tools.length === 14 && first.body.tools.every((t) => t.strict === true), 'TC1: 도구 14개(읽기 7·쓰기 7) strict');
  assert(first.body.system[0].text.includes('document_content'), 'TC1: 시스템에 문서 경계 규칙');
  assert(first.body.fallbacks === 'default'
    && (first.headers['anthropic-beta'] ?? '').includes('server-side-fallback-2026-07-01'), 'TC1: 서버 대체');
  assert(first.body.output_config?.effort === 'medium' && !('thinking' in first.body), 'TC1: 응답 깊이, thinking 생략');
  assert(first.headers['x-api-key'] === KEY, 'TC1: 키 헤더');
  assert(JSON.stringify(first.body.messages) === JSON.stringify([{ role: 'user', content: [{ type: 'text', text: '표지에 뭐가 있어?' }] }]),
    'TC1: 첫 요청은 질문 하나');
  assert(second.body.messages[1].role === 'assistant'
    && second.body.messages[1].content.some((b) => b.type === 'tool_use' && b.id === 'toolu_e2e_1'),
  'TC1: 첫 응답(tool_use)을 그대로 되돌림');
  const toolResult = second.body.messages[2].content;
  assert(toolResult.length === 1 && toolResult[0].type === 'tool_result' && toolResult[0].tool_use_id === 'toolu_e2e_1',
    'TC1: 도구 결과는 한 user 메시지');
  const resultText = toolResult[0].content;
  assert(/^<document_content id="[0-9a-f]{12}">/.test(resultText), 'TC1: 결과는 문서 경계로 감쌈');
  assert(JSON.parse(resultText.split('\n')[1]).text.includes('[table 1 / cell 0] OO교육대학교 OXX사업단 홈페이지 구축 사업'),
    'TC1: 표 칸 글이 결과에 포함');
  assert(JSON.stringify(second.body.messages.slice(0, 1)) === JSON.stringify(first.body.messages), 'TC1: 이력은 접두 유지');

  const view = await page.evaluate(() => {
    const log = document.querySelector('.ai-chat-log');
    return {
      user: log.querySelector('.ai-msg-user')?.textContent,
      chips: [...log.querySelectorAll('.ai-tool-chip')].map((c) => [c.textContent, c.dataset.state]),
      strong: log.querySelector('.ai-msg-body strong')?.textContent,
      items: log.querySelectorAll('.ai-msg-body li').length,
      scripts: log.querySelectorAll('script').length,
      scriptText: log.textContent.includes('<script>window.__aiPwned = 1</script>'),
      pwned: window.__aiPwned === 1,
      usage: document.querySelector('.ai-chat-usage')?.textContent,
    };
  });
  assert(view.user === '표지에 뭐가 있어?', 'TC1: 사용자 말 표시');
  assert(view.chips.length === 1 && view.chips[0][1] === 'done', `TC1: 도구 표시 (${JSON.stringify(view.chips)})`);
  assert(view.strong === '요약' && view.items === 2, 'TC2: 굵게·목록 렌더');
  assert(view.scripts === 0 && view.scriptText && !view.pwned, 'TC2: HTML 은 글자로, 실행 없음');
  assert(view.usage.includes('4,200') && view.usage.includes('1,800') && view.usage.includes('80'),
    `TC1: 토큰 사용량 누적 (${view.usage})`);
  await screenshot(page, 'ai-panel-answer');

  // ── TC3: 문서 무변경 ──────────────────────────────────
  assert(await docHash() === hashBefore, 'TC3: 대화 전후 문서 바이트 동일');

  // ── TC9: 문서 변경 제안 → 적용 → 문서 반영 → undo ─────
  const cellText = () => page.evaluate(() => {
    const w = window.__wasm;
    const len = w.getCellParagraphLength(0, 2, 0, 1, 0);
    return w.getTextInCell(0, 2, 0, 1, 0, 0, len);
  });
  assert(await cellText() === '사 업 수 행 계 획 서', 'TC9: 고치기 전 표 칸 글');
  queue.push({ body: sse([
    { type: 'tool_use', id: 'toolu_edit_1', name: 'replace_text', input: { find: '사 업 수 행 계 획 서', replace: '사 업 계 획 서' } },
  ], 'tool_use') });
  queue.push({ body: sse([{ type: 'text', text: '제목을 바꿨습니다.' }], 'end_turn') });
  await page.type('.ai-chat-input', '제목을 사업계획서로 바꿔 줘');
  await page.keyboard.press('Enter');
  await page.waitForSelector('.ai-change-card[data-state="pending"] .ai-change-apply', { timeout: 15000 });
  const card = await page.$eval('.ai-change-card', (el) => ({
    title: el.querySelector('.ai-change-title').textContent,
    del: el.querySelector('.ai-change-del').textContent,
    ins: el.querySelector('.ai-change-ins').textContent,
  }));
  assert(card.title.startsWith('글 바꾸기') && card.title.includes('표 안'), `TC9: 카드 제목 (${card.title})`);
  assert(card.del === '사 업 수 행 계 획 서' && card.ins === '사 업 계 획 서', 'TC9: 전·후 표시');
  assert(await cellText() === '사 업 수 행 계 획 서', 'TC9: 승인 전에는 문서가 그대로');
  await page.click('.ai-change-apply');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy
    && document.querySelector('.ai-change-card')?.dataset.state === 'applied', { timeout: 15000 });
  assert(await cellText() === '사 업 계 획 서', 'TC9: 적용 후 표 칸 글이 바뀜');
  const editResult = requests.at(-1).body.messages.at(-1).content[0];
  assert(editResult.tool_use_id === 'toolu_edit_1' && !editResult.is_error
    && JSON.parse(editResult.content.split('\n')[1]).applied === true, 'TC9: 모델에 적용 결과 전달');
  await screenshot(page, 'ai-panel-change');
  await page.evaluate(() => window.rhwpStudio.automation.execute('edit:undo'));
  await wait(page, 300);
  assert(await cellText() === '사 업 수 행 계 획 서', 'TC9: undo 한 번으로 되돌아감');

  // ── TC10: 거절하면 문서는 그대로이고 모델에 거절을 알린다 ──
  const hashBeforeReject = await docHash();
  queue.push({ body: sse([
    { type: 'tool_use', id: 'toolu_edit_2', name: 'replace_text', input: { find: '사 업 수 행 계 획 서', replace: '바꾸면 안 됨' } },
  ], 'tool_use') });
  queue.push({ body: sse([{ type: 'text', text: '알겠습니다.' }], 'end_turn') });
  await page.type('.ai-chat-input', '다른 제목 제안해 줘');
  await page.keyboard.press('Enter');
  await page.waitForSelector('.ai-change-card[data-state="pending"] .ai-change-reject', { timeout: 15000 });
  await page.click('.ai-change-card[data-state="pending"] .ai-change-reject');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy, { timeout: 15000 });
  assert(await docHash() === hashBeforeReject, 'TC10: 거절하면 문서 바이트 그대로');
  const rejected = requests.at(-1).body.messages.at(-1).content[0];
  assert(rejected.tool_use_id === 'toolu_edit_2' && rejected.content.includes('rejected'), 'TC10: 모델에 거절 전달');

  // ── TC11: 새 문서 만들기 → 저장 확인 → 문단 쓰기 (대화는 이어진다) ──
  queue.push({ body: sse([{ type: 'tool_use', id: 'toolu_new_1', name: 'create_document', input: {} }], 'tool_use') });
  queue.push({ body: sse([{ type: 'tool_use', id: 'toolu_new_2', name: 'insert_paragraphs',
    input: { position: 'end', paragraphs: ['회의록', '', '일시: 2026년 10월 23일', '안건: AI 도우미 시연'] } }], 'tool_use') });
  queue.push({ body: sse([{ type: 'text', text: '새 문서에 회의록을 작성했습니다.' }], 'end_turn') });
  await page.type('.ai-chat-input', '새 문서에 회의록 양식을 써 줘');
  await page.keyboard.press('Enter');
  await page.waitForSelector('.ai-change-card[data-state="pending"] .ai-change-apply', { timeout: 15000 });
  await page.click('.ai-change-card[data-state="pending"] .ai-change-apply');
  // 고친 뒤 되돌린 문서라 저장 확인이 뜰 수 있다 — 뜨면 저장하지 않고 진행한다.
  const guard = await page.waitForFunction(() => {
    const buttons = [...document.querySelectorAll('.modal-overlay .dialog-btn')];
    return buttons.find((b) => b.textContent === '저장 안 함') ?? document.querySelector('.ai-change-card[data-state="pending"] .ai-change-apply');
  }, { timeout: 15000 });
  const guardText = await guard.evaluate((el) => el.textContent);
  if (guardText === '저장 안 함') await guard.evaluate((el) => el.click());
  await page.waitForSelector('.ai-change-card[data-state="pending"] .ai-change-apply', { timeout: 15000 });
  const insertCard = await page.$eval('.ai-change-card[data-state="pending"]', (el) => ({
    title: el.querySelector('.ai-change-title').textContent,
    text: el.querySelector('.ai-change-ins').textContent,
  }));
  assert(insertCard.title === '문단 추가 · 문서 끝' && insertCard.text.includes('안건: AI 도우미 시연'), `TC11: 문단 추가 카드 (${insertCard.title})`);
  await page.click('.ai-change-card[data-state="pending"] .ai-change-apply');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy, { timeout: 15000 });
  const newDoc = await page.evaluate(() => {
    const w = window.__wasm;
    const n = w.getParagraphCount(0);
    return Array.from({ length: n }, (_, i) => w.getTextRange(0, i, 0, w.getParagraphLength(0, i)));
  });
  assert(JSON.stringify(newDoc) === JSON.stringify(['회의록', '', '일시: 2026년 10월 23일', '안건: AI 도우미 시연']),
    `TC11: 새 문서 문단 (${JSON.stringify(newDoc)})`);
  const states = await page.$$eval('.ai-change-card', (els) => els.map((el) => el.dataset.state));
  assert(states.slice(-2).join() === 'applied,applied', `TC11: 두 변경 모두 적용 (${states.join()})`);
  const createdStatus = await page.$$eval('.ai-change-status', (els) => els.at(-2)?.textContent);
  assert(createdStatus === '새 문서를 열었습니다.', `TC11: 새 문서 카드는 undo 안내를 하지 않음 (${createdStatus})`);
  const conv11 = await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []));
  assert(conv11.messages > 2, 'TC11: AI 가 만든 새 문서로는 대화가 끊기지 않음');
  await screenshot(page, 'ai-panel-new-document');

  // ── TC12: 서식(글자·문단)과 표 — 한 응답의 여러 변경을 차례로 승인 ──
  queue.push({ body: sse([
    { type: 'tool_use', id: 'toolu_fmt_1', name: 'format_text', input: { find: '회의록', bold: true, fontSize: 20 } },
    { type: 'tool_use', id: 'toolu_fmt_2', name: 'format_paragraph', input: { find: '회의록', alignment: 'center' } },
    { type: 'tool_use', id: 'toolu_tbl_1', name: 'insert_table',
      input: { position: 'end', columns: 2, cells: ['참석자', '역할', '홍길동', '진행'] } },
  ], 'tool_use') });
  queue.push({ body: sse([{ type: 'text', text: '제목을 꾸미고 참석자 표를 넣었습니다.' }], 'end_turn') });
  await page.type('.ai-chat-input', '제목을 크게 가운데로, 참석자 표도 넣어 줘');
  await page.keyboard.press('Enter');
  const summaries = [];
  for (let i = 0; i < 3; i += 1) {
    await page.waitForSelector('.ai-change-card[data-state="pending"] .ai-change-apply', { timeout: 15000 });
    summaries.push(await page.$eval('.ai-change-card[data-state="pending"]', (el) =>
      `${el.querySelector('.ai-change-title').textContent} / ${el.querySelector('.ai-change-summary')?.textContent ?? ''}`));
    await page.click('.ai-change-card[data-state="pending"] .ai-change-apply');
  }
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy, { timeout: 15000 });
  assert(summaries[0].includes('굵게') && summaries[0].includes('20pt'), `TC12: 글자 모양 카드 (${summaries[0]})`);
  assert(summaries[1].includes('정렬 가운데'), `TC12: 문단 모양 카드 (${summaries[1]})`);
  assert(summaries[2].startsWith('표 만들기') && summaries[2].includes('2행 × 2열'), `TC12: 표 카드 (${summaries[2]})`);
  const formatted = await page.evaluate(() => {
    const w = window.__wasm;
    const ch = w.getCharPropertiesAt(0, 0, 0);
    const para = w.getParaPropertiesAt(0, 0);
    const inTable = ['참석자', '역할', '홍길동', '진행']
      .map((cell) => w.searchAllText(cell, true, true).some((hit) => hit.cellContext));
    return { bold: ch.bold, size: ch.fontSize, align: para.alignment, inTable };
  });
  assert(formatted.bold === true && formatted.size === 2000, `TC12: 제목 굵게·20pt (${JSON.stringify(formatted)})`);
  assert(formatted.align === 'center', 'TC12: 제목 가운데 정렬');
  assert(formatted.inTable.every(Boolean), `TC12: 네 칸 글이 모두 표 칸 안에 있음 (${formatted.inTable})`);
  await screenshot(page, 'ai-panel-format-table');
  await page.evaluate(() => window.rhwpStudio.automation.execute('edit:undo'));
  await wait(page, 300);
  const afterUndo12 = await page.evaluate(() => window.__wasm.searchAllText('참석자', true, true).length > 0);
  assert(!afterUndo12, 'TC12: undo 한 번으로 표만 사라짐');

  // ── TC4: 입력란 Ctrl+Z 격리 ───────────────────────────
  await createNewDocument(page);
  await page.waitForFunction(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).messages === 0);
  const swapNotice = await page.$eval('.ai-chat-log', (el) => el.textContent);
  assert(swapNotice.includes('문서가 바뀌어'), 'TC8: 문서 교체 시 새 대화 안내');
  await clickEditArea(page);
  await typeText(page, '가나다라마');
  await wait(page, 300);
  const undoBefore = await page.evaluate(() => window.rhwpStudio.automation.getContext().canUndo);
  await page.focus('.ai-chat-input');
  await page.keyboard.type('메모');
  const mod = process.platform === 'darwin' ? 'Meta' : 'Control';
  await page.keyboard.down(mod);
  await page.keyboard.press('KeyZ');
  await page.keyboard.up(mod);
  await wait(page, 300);
  const afterUndo = await page.evaluate(() => ({
    canUndo: window.rhwpStudio.automation.getContext().canUndo,
    text: window.__wasm.getTextRange(0, 0, 0, 5),
  }));
  assert(undoBefore && afterUndo.canUndo && afterUndo.text === '가나다라마',
    `TC4: 입력란 Ctrl+Z 가 문서로 새지 않음 (${JSON.stringify(afterUndo)})`);
  await page.$eval('.ai-chat-input', (el) => { el.value = ''; });

  // ── TC5: 빠른 동작 ────────────────────────────────────
  const quickHiddenBefore = await page.$eval('.ai-chat-quick', (el) => el.hidden);
  await clickEditArea(page);
  await page.keyboard.press('End');
  await page.keyboard.down('Shift');
  for (let i = 0; i < 3; i += 1) await page.keyboard.press('ArrowLeft');
  await page.keyboard.up('Shift');
  await page.waitForFunction(() => !document.querySelector('.ai-chat-quick').hidden, { timeout: 3000 });
  assert(quickHiddenBefore, 'TC5: 선택이 없으면 빠른 동작 숨김');
  queue.push({ body: sse([{ type: 'text', text: '요약입니다.' }], 'end_turn') });
  await page.click('.ai-quick-btn[data-action="summarize"]');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy
    && document.querySelector('.ai-msg-assistant:not([aria-busy])'), { timeout: 15000 });
  assert(requests.at(-1).body.messages[0].content[0].text === '선택한 부분을 요약해 줘.', 'TC5: 빠른 동작 프롬프트로 질문');

  // ── TC6: 중단 ─────────────────────────────────────────
  queue.push({ delayMs: 4000, body: sse([{ type: 'text', text: '늦은 답' }], 'end_turn') });
  await page.type('.ai-chat-input', '천천히 답해 줘');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => document.querySelector('.ai-send-btn')?.classList.contains('ai-stop-btn'));
  await page.click('.ai-send-btn');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy, { timeout: 5000 });
  const stoppedNote = await page.$$eval('.ai-msg-note', (els) => els.at(-1)?.textContent);
  assert(stoppedNote === '중단했습니다.', `TC6: 중단 표시 (${stoppedNote})`);

  // ── TC7: 401 ──────────────────────────────────────────
  queue.push({ status: 401, body: JSON.stringify({ type: 'error', error: { type: 'authentication_error', message: 'invalid x-api-key' } }) });
  await page.type('.ai-chat-input', '다시');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy
    && document.querySelectorAll('.ai-msg-note').length >= 2, { timeout: 10000 });
  const authNote = await page.$$eval('.ai-msg-note', (els) => els.at(-1)?.textContent);
  assert(authNote === 'API 키가 올바르지 않습니다.', `TC7: 401 문구 (${authNote})`);
  const conv = await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []));
  assert(conv.messages % 2 === 0, `TC7: 실패 뒤에도 이력은 user/assistant 짝 (${conv.messages})`);

  // ── TC7b: 400 은 제공자 사유를 함께 보인다 ──────────────
  queue.push({ status: 400, body: JSON.stringify({ type: 'error',
    error: { type: 'invalid_request_error', message: 'tools.0: probe reason' } }) });
  await page.type('.ai-chat-input', '또');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy
    && document.querySelectorAll('.ai-msg-note').length >= 3, { timeout: 10000 });
  const badNote = await page.$$eval('.ai-msg-note', (els) => els.at(-1)?.textContent);
  assert(badNote.startsWith('요청이 거절되었습니다.') && badNote.includes('tools.0: probe reason'),
    `TC7b: 400 사유 표시 (${badNote})`);

  // ── TC7c: 크레딧 부족은 전용 안내 ──────────────────────
  queue.push({ status: 400, body: JSON.stringify({ type: 'error', error: { type: 'invalid_request_error',
    message: 'Your credit balance is too low to access the Anthropic API. Please go to Plans & Billing to upgrade or purchase credits.' } }) });
  await page.type('.ai-chat-input', '크레딧');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy
    && document.querySelectorAll('.ai-msg-note').length >= 4, { timeout: 10000 });
  const billingNote = await page.$$eval('.ai-msg-note', (els) => els.at(-1)?.textContent);
  assert(billingNote.startsWith('API 계정의 크레딧이 부족합니다'), `TC7c: 크레딧 부족 안내 (${billingNote})`);

  // ── TC8: 새 대화 ──────────────────────────────────────
  await page.click('.ai-new-chat');
  const cleared = await page.evaluate(() => ({
    messages: window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).messages,
    bubbles: document.querySelectorAll('.ai-msg').length,
  }));
  assert(cleared.messages === 0 && cleared.bubbles === 0, 'TC8: 새 대화는 이력·화면을 비움');
});
