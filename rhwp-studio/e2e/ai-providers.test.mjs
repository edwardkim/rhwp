/**
 * E2E 테스트 — AI 도우미의 OpenAI·Gemini 제공자
 *
 * 두 API 는 요청 가로채기로 흉내 낸다(SSE 응답). 실제 키·네트워크는 쓰지 않는다.
 *
 * 제공자마다:
 * 1. 설정에서 제공자를 고르면 모델 목록이 그 제공자의 것으로 바뀌고, 연결 테스트는 모델 조회 한 번
 *    (키는 헤더로만, URL 에 없음)
 * 2. 제공자가 바뀌면 전송 동의를 다시 묻는다
 * 3. 질문 → 모델이 read_page_text 를 부름 → 결과(문서 경계)를 보냄 → 답 표시
 *    - 첫 요청: 모델·응답 깊이·도구 41개·시스템 규칙
 *    - 둘째 요청: 첫 응답 원형(서명·암호화 추론 포함)을 그대로 되돌리고 도구 결과를 짝지어 보냄
 * 4. 대화 전후 문서 바이트가 같다(읽기만 했다)
 * 5. 401/400 키 오류는 키 오류 문구
 * 마지막으로 제공자를 바꾸면 대화가 비워지고 안내가 보인다.
 */

import { runTest, loadApp, loadHwpFile, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const lastDialog = '.modal-overlay:last-of-type .dialog-wrap';
const OPENAI_KEY = 'sk-proj-e2e-PROVIDERKEY-0123456789abcdef';
const GEMINI_KEY = 'AIzaSy-e2e-PROVIDERKEY-0123456789abcdef';

function openAiSse(output, text) {
  const events = [{ type: 'response.created', response: { status: 'in_progress' } }];
  for (const piece of text.match(/[\s\S]{1,10}/g) ?? []) events.push({ type: 'response.output_text.delta', delta: piece });
  for (const item of output) if (item.type === 'function_call') events.push({ type: 'response.output_item.added', item });
  events.push({ type: 'response.completed', response: {
    status: 'completed', output, usage: { input_tokens: 1500, output_tokens: 30, input_tokens_details: { cached_tokens: 1200 } },
  } });
  return events.map((event) => `event: ${event.type}\ndata: ${JSON.stringify(event)}\n\n`).join('');
}

function geminiSse(parts, finishReason = 'STOP') {
  const chunks = parts.map((part, index) => ({
    candidates: [{ content: { role: 'model', parts: [part] }, ...(index === parts.length - 1 ? { finishReason } : {}) }],
    ...(index === parts.length - 1 ? { usageMetadata: { promptTokenCount: 1500, cachedContentTokenCount: 1000, candidatesTokenCount: 20, thoughtsTokenCount: 10 } } : {}),
  }));
  return chunks.map((chunk) => `data: ${JSON.stringify(chunk)}\r\n\r\n`).join('');
}

const OPENAI_FIRST = [
  { type: 'reasoning', id: 'rs_e2e', summary: [], encrypted_content: 'ENC-E2E' },
  { type: 'message', id: 'msg_e2e', role: 'assistant', status: 'completed', content: [{ type: 'output_text', text: '표지를 읽어 볼게요.' }] },
  { type: 'function_call', id: 'fc_e2e', call_id: 'call_e2e_1', name: 'read_page_text', arguments: '{"page":1}', status: 'completed' },
];
const GEMINI_FIRST = [
  { text: '표지를 읽어 볼게요.' },
  { functionCall: { name: 'read_page_text', args: { page: 1 } }, thoughtSignature: 'SIG-E2E' },
];

runTest('AI 도우미 — OpenAI·Gemini 제공자', async ({ page }) => {
  /** 가로챈 요청 기록과 다음 응답 줄 */
  const seen = [];
  const queue = [];
  await page.setRequestInterception(true);
  page.on('request', (request) => {
    const url = request.url();
    const isOpenAi = url.startsWith('https://api.openai.com/');
    const isGemini = url.startsWith('https://generativelanguage.googleapis.com/');
    if (!isOpenAi && !isGemini) { request.continue(); return; }
    const cors = {
      'access-control-allow-origin': '*', 'access-control-allow-headers': '*',
      'access-control-allow-methods': 'GET, POST, OPTIONS',
    };
    if (request.method() === 'OPTIONS') { request.respond({ status: 204, headers: cors }); return; }
    seen.push({ url, method: request.method(), headers: request.headers(), body: request.postData() ? JSON.parse(request.postData()) : null });
    if (request.method() === 'GET') {
      request.respond({ status: 200, headers: { ...cors, 'content-type': 'application/json' },
        body: JSON.stringify(isOpenAi ? { id: 'gpt-6.1-sol', object: 'model' } : { name: 'models/gemini-3.8-flash', displayName: 'Gemini 3.8 Flash' }) });
      return;
    }
    const next = queue.shift() ?? { status: 500, body: '{}' };
    request.respond({
      status: next.status ?? 200,
      headers: { ...cors, 'content-type': next.status && next.status !== 200 ? 'application/json' : 'text/event-stream' },
      body: next.body,
    });
  });

  await loadApp(page);
  await page.waitForFunction(() => window.rhwpStudio?.plugins?.isLoaded?.('ai-assistant'), { timeout: 10000 });
  await loadHwpFile(page, 'biz_plan.hwp');
  const docHash = () => page.evaluate(async () => {
    const digest = await crypto.subtle.digest('SHA-256', window.__wasm.exportHwp());
    return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, '0')).join('');
  });
  const hashBefore = await docHash();
  const panelState = () => page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'panelState', []));
  const conversation = () => page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []));

  async function configure(provider, model, key) {
    await page.evaluate(() => window.rhwpStudio.automation.execute('ext:ai.settings', {}, { allowDialog: true }));
    await page.waitForSelector('#ai-settings-provider');
    await page.select('#ai-settings-provider', provider);
    const models = await page.$$eval('#ai-settings-model option', (options) => options.map((o) => o.value));
    await page.select('#ai-settings-model', model);
    await page.type('#ai-settings-key', key);
    const before = seen.length;
    await page.click('.ai-settings-actions .dialog-btn');
    await page.waitForFunction(() => document.querySelector('.ai-settings-status')?.dataset.kind === 'ok', { timeout: 5000 });
    const status = await page.$eval('.ai-settings-status', (el) => el.textContent);
    const test = seen.slice(before);
    await page.click(`${lastDialog} .dialog-btn-primary`);
    await page.waitForFunction(() => !document.querySelector('#ai-settings-key'));
    return { models, status, test };
  }

  async function giveConsent() {
    await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [true]));
    await page.waitForFunction(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'panelState', []) === 'consent'
      || window.rhwpStudio.plugins.invoke('ai-assistant', 'panelState', []) === 'ready');
    const asked = (await panelState()) === 'consent';
    if (asked) {
      // 패널을 처음 열 때만 저절로 묻는다. 이미 열려 있으면 안내의 「전송에 동의」를 누른다.
      const prompted = await page.waitForSelector(`${lastDialog} .dialog-btn-primary`, { timeout: 1500 }).catch(() => null);
      if (!prompted) {
        await page.click('.ai-panel-action');
        await page.waitForSelector(`${lastDialog} .dialog-btn-primary`);
      }
      await page.click(`${lastDialog} .dialog-btn-primary`);
      await page.waitForFunction(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'panelState', []) === 'ready');
    }
    return asked;
  }

  async function ask(text) {
    const before = seen.length;
    const answers = await page.$$eval('.ai-msg-assistant', (els) => els.length);
    await page.type('.ai-chat-input', text);
    await page.keyboard.press('Enter');
    await page.waitForFunction((n) => !window.rhwpStudio.plugins.invoke('ai-assistant', 'conversation', []).busy
      && document.querySelectorAll('.ai-msg-assistant:not([aria-busy])').length > n, { timeout: 15000 }, answers);
    const last = await page.$$eval('.ai-msg-assistant', (els) => els.at(-1).textContent);
    return { requests: seen.slice(before), last };
  }

  // ── OpenAI ─────────────────────────────────────────────
  {
    const { models, status, test } = await configure('openai', 'gpt-6.1-sol', OPENAI_KEY);
    assert(models.join() === 'gpt-6-astra,gpt-6.1-sol,gpt-6-luna', `OA1: 모델 목록 (${models})`);
    assert(test.length === 1 && test[0].method === 'GET' && test[0].url === 'https://api.openai.com/v1/models/gpt-6.1-sol',
      `OA1: 연결 테스트는 모델 조회 한 번 (${test.map((t) => t.url)})`);
    assert(test[0].headers.authorization === `Bearer ${OPENAI_KEY}` && !test[0].url.includes(OPENAI_KEY), 'OA1: 키는 헤더로만');
    assert(status.includes('GPT-6.1 Sol'), `OA1: 연결 결과 (${status})`);
    assert(await giveConsent(), 'OA2: 처음 고른 제공자는 전송 동의를 묻는다');
    const line = await page.$eval('.ai-panel', (el) => el.textContent);
    assert(line.includes('OpenAI') && line.includes('GPT-6.1 Sol'), 'OA2: 패널에 제공자·모델 표시');

    queue.push({ body: openAiSse(OPENAI_FIRST, '표지를 읽어 볼게요.') });
    queue.push({ body: openAiSse([{ type: 'message', id: 'msg_e2e_2', role: 'assistant', status: 'completed',
      content: [{ type: 'output_text', text: '**요약**: 표지에는 사업명이 있습니다.' }] }], '**요약**: 표지에는 사업명이 있습니다.') });
    const { requests, last } = await ask('표지에 뭐가 있어?');
    assert(requests.length === 2 && requests.every((r) => r.url === 'https://api.openai.com/v1/responses'), `OA3: 요청 2회 (${requests.length})`);
    const [first, second] = requests;
    assert(first.headers.authorization === `Bearer ${OPENAI_KEY}`, 'OA3: 키는 Authorization 헤더');
    assert(first.body.model === 'gpt-6.1-sol' && first.body.stream === true && first.body.store === false
      && first.body.reasoning.effort === 'medium', 'OA3: 모델·스트리밍·무상태·응답 깊이');
    assert(first.body.tools.length === 41 && first.body.tools.every((t) => t.type === 'function' && t.strict === false), 'OA3: 도구 41개');
    assert(first.body.instructions.includes('document_content'), 'OA3: 시스템에 문서 경계 규칙');
    const echoed = second.body.input.slice(1, 4);
    assert(JSON.stringify(echoed) === JSON.stringify(OPENAI_FIRST), 'OA3: 첫 응답 원형(암호화 추론 포함)을 그대로 되돌린다');
    const output = second.body.input.find((item) => item.type === 'function_call_output');
    assert(output?.call_id === 'call_e2e_1' && output.output.includes('<document_content id='), 'OA3: 도구 결과를 call_id 로 짝지어 경계 안에');
    assert(last.includes('요약') && last.includes('사업명'), `OA3: 답 표시 (${last.slice(0, 40)})`);
    assert((await conversation()).messages === 4, 'OA3: 이력 4칸');

    queue.push({ status: 401, body: JSON.stringify({ error: { message: 'Incorrect API key provided', type: 'invalid_request_error', code: 'invalid_api_key' } }) });
    const failed = await ask('다시');
    assert(failed.last.includes('API 키가 올바르지 않습니다'), `OA5: 401 키 오류 문구 (${failed.last})`);
  }

  // ── Gemini — 제공자를 바꾸면 대화가 새로 시작된다 ─────────
  {
    const { models, status, test } = await configure('gemini', 'gemini-3.8-flash', GEMINI_KEY);
    assert(models.join() === 'gemini-4-argon,gemini-3.8-flash,gemini-3.5-flash-lite', `GM1: 모델 목록 (${models})`);
    assert(test.length === 1 && test[0].url === 'https://generativelanguage.googleapis.com/v1beta/models/gemini-3.8-flash',
      `GM1: 연결 테스트는 모델 조회 한 번 (${test.map((t) => t.url)})`);
    assert(test[0].headers['x-goog-api-key'] === GEMINI_KEY && !test[0].url.includes(GEMINI_KEY), 'GM1: 키는 x-goog-api-key 헤더로만');
    assert(status.includes('Gemini 3.8 Flash'), `GM1: 연결 결과 (${status})`);
    assert((await conversation()).messages === 0, 'GM2: 제공자가 바뀌어 이력을 비웠다');
    const notice = await page.$eval('.ai-panel', (el) => el.textContent);
    assert(notice.includes('AI 제공자가 바뀌어 새 대화를 시작했습니다'), 'GM2: 새 대화 안내');
    assert(await giveConsent(), 'GM2: 제공자가 바뀌면 전송 동의를 다시 묻는다');

    queue.push({ body: geminiSse(GEMINI_FIRST) });
    queue.push({ body: geminiSse([{ text: '**요약**: 표지에는 ' }, { text: '사업명이 있습니다.' }]) });
    const { requests, last } = await ask('표지에 뭐가 있어?');
    const streamUrl = 'https://generativelanguage.googleapis.com/v1beta/models/gemini-3.8-flash:streamGenerateContent?alt=sse';
    assert(requests.length === 2 && requests.every((r) => r.url === streamUrl), `GM3: 요청 2회 (${requests.map((r) => r.url)})`);
    const [first, second] = requests;
    assert(first.headers['x-goog-api-key'] === GEMINI_KEY && !first.url.includes(GEMINI_KEY), 'GM3: 키는 헤더로만');
    assert(first.body.generationConfig.thinkingConfig.thinkingLevel === 'medium', 'GM3: 응답 깊이 → thinkingLevel');
    assert(first.body.tools[0].functionDeclarations.length === 41, 'GM3: 도구 41개');
    assert(!JSON.stringify(first.body.tools).includes('additionalProperties'), 'GM3: Gemini 가 받지 않는 schema 키는 뺐다');
    assert(first.body.systemInstruction.parts[0].text.includes('document_content'), 'GM3: 시스템에 문서 경계 규칙');
    const model = second.body.contents[1];
    assert(model.role === 'model' && JSON.stringify(model.parts) === JSON.stringify(GEMINI_FIRST), 'GM3: 첫 응답 part(서명 포함)를 그대로 되돌린다');
    const response = second.body.contents[2].parts[0].functionResponse;
    assert(response?.name === 'read_page_text' && response.response.output.includes('<document_content id='), 'GM3: 도구 결과를 이름으로 짝지어 경계 안에');
    assert(last.includes('요약') && last.includes('사업명이 있습니다'), `GM3: 답 표시 (${last.slice(0, 40)})`);

    queue.push({ status: 400, body: JSON.stringify({ error: { code: 400, message: 'API key not valid. Please pass a valid API key.',
      status: 'INVALID_ARGUMENT', details: [{ '@type': 'type.googleapis.com/google.rpc.ErrorInfo', reason: 'API_KEY_INVALID' }] } }) });
    const failed = await ask('다시');
    assert(failed.last.includes('API 키가 올바르지 않습니다'), `GM5: 400 API_KEY_INVALID 도 키 오류 문구 (${failed.last})`);
  }

  assert(await docHash() === hashBefore, 'TC4: 읽기만 한 대화 전후 문서 바이트가 같다');
});
