import test from 'node:test';
import assert from 'node:assert/strict';

import { APIConnectionError, APIError, APIUserAbortError } from '@anthropic-ai/sdk';
import { classifyClaudeError, testClaudeConnection } from '../src/ai/provider/claude.ts';

const apiError = (status: number, headers: Record<string, string> = {}) =>
  APIError.generate(status, { type: 'error', error: { type: 'x', message: 'm' } }, 'm', new Headers(headers));

test('SDK 오류를 화면 갈래로 옮긴다', () => {
  assert.deepEqual(classifyClaudeError(apiError(401)), { reason: 'auth', status: 401 });
  assert.deepEqual(classifyClaudeError(apiError(403)), { reason: 'permission', status: 403 });
  assert.deepEqual(classifyClaudeError(apiError(404)), { reason: 'model', status: 404 });
  assert.deepEqual(classifyClaudeError(apiError(400)), { reason: 'bad-request', status: 400, detail: 'm' });
  assert.deepEqual(classifyClaudeError(apiError(429, { 'retry-after': '12' })),
    { reason: 'rate', status: 429, retryAfterSeconds: 12 });
  assert.deepEqual(classifyClaudeError(apiError(429)), { reason: 'rate', status: 429 });
  assert.deepEqual(classifyClaudeError(apiError(529)), { reason: 'overloaded', status: 529 });
  assert.deepEqual(classifyClaudeError(new APIConnectionError({ message: 'down' })), { reason: 'network' });
  assert.deepEqual(classifyClaudeError(new APIUserAbortError()), { reason: 'aborted' });
  assert.deepEqual(classifyClaudeError(new Error('?')), { reason: 'unknown' });
});

test('연결 테스트는 모델 정보 조회로 확인하고 오류를 갈래로 돌려준다', async () => {
  const calls: unknown[][] = [];
  const ok = await testClaudeConnection('k', 'claude-opus-5-5', undefined, {
    models: { retrieve: async (...args: unknown[]) => { calls.push(args); return { display_name: 'Claude Opus 5.5' }; } },
  } as never);
  assert.deepEqual(ok, { ok: true, modelName: 'Claude Opus 5.5' });
  assert.equal(calls[0][0], 'claude-opus-5-5');

  const denied = await testClaudeConnection('k', 'claude-opus-5-5', undefined, {
    models: { retrieve: async () => { throw apiError(401); } },
  } as never);
  assert.deepEqual(denied, { ok: false, reason: 'auth', status: 401 });
});

test('400 은 무엇이 거절됐는지 알 수 있게 제공자 사유와 request id 를 붙인다', () => {
  const error = APIError.generate(400,
    { type: 'error', error: { type: 'invalid_request_error', message: 'fallbacks: Extra inputs are not permitted' } },
    undefined, new Headers({ 'request-id': 'req_123' }));
  assert.deepEqual(classifyClaudeError(error), {
    reason: 'bad-request', status: 400, detail: 'fallbacks: Extra inputs are not permitted · request-id req_123',
  });
  assert.deepEqual(classifyClaudeError(APIError.generate(400, undefined, undefined, new Headers())),
    { reason: 'bad-request', status: 400 }, '본문이 없으면 사유도 없다');
  assert.deepEqual(classifyClaudeError(apiError(500)), { reason: 'unknown', status: 500, detail: 'm' });
});

test('크레딧 부족 400 은 billing 갈래로 옮긴다', () => {
  const error = APIError.generate(400,
    { type: 'error', error: { type: 'invalid_request_error',
      message: 'Your credit balance is too low to access the Anthropic API. Please go to Plans & Billing to upgrade or purchase credits.' } },
    undefined, new Headers({ 'request-id': 'req_9' }));
  const failure = classifyClaudeError(error);
  assert.equal(failure.reason, 'billing');
  assert.equal(failure.status, 400);
  assert.match(failure.detail ?? '', /request-id req_9/);
});
