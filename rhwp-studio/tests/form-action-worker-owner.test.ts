import test from 'node:test';
import assert from 'node:assert/strict';
import { planFormAction, executeFormButton } from '../src/engine/input-handler-form-action.ts';
import { setFormActionsEnabled } from '../src/core/form-action-policy.ts';
import type { FormActionContext } from '../../rhwp-shared/forms/action-runtime.ts';

test('a read-only host rejects button execution before reading its document or spawning a Worker', async () => {
  setFormActionsEnabled(false);
  try {
    await assert.rejects(executeFormButton(null as unknown as Parameters<typeof executeFormButton>[0],
      { found: false }, 0), /FORM_ACTION_READ_ONLY/);
  } finally { setFormActionsEnabled(true); }
});

test('owner terminates silent, cancelled and failed form Workers; a subsequent action succeeds', async () => {
  const context = {} as FormActionContext;
  let terminated = 0;
  const fake = () => ({ terminate() { terminated++; }, postMessage() {}, onerror: null, onmessageerror: null, onmessage: null });
  const signal = new AbortController().signal;
  await assert.rejects(planFormAction(context, signal, 5, fake as unknown as () => Worker), /FORM_ACTION_TIMEOUT/);
  const controller = new AbortController();
  const pending = planFormAction(context, controller.signal, 1000, fake as unknown as () => Worker);
  controller.abort(); await assert.rejects(pending, /FORM_ACTION_CANCELLED/);
  const broken = fake();
  broken.postMessage = () => { throw new Error('clone failed'); };
  await assert.rejects(planFormAction(context, signal, 1000, () => broken as unknown as Worker), /FORM_ACTION_WORKER_FAILED/);
  const working = fake();
  working.postMessage = () => queueMicrotask(() => (working.onmessage as unknown as (event: unknown) => void)({ data: { result: { updates: [], commands: [] } } }));
  assert.deepEqual(await planFormAction(context, signal, 1000, () => working as unknown as Worker), { updates: [], commands: [] });
  assert.equal(terminated, 4);
});
