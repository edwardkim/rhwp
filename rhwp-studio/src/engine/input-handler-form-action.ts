import type { FormActionContext, FormActionPlan, FormAddress } from '../../../rhwp-shared/forms/action-runtime';
import { assertFormActionsEnabled } from '../core/form-action-policy.ts';
import type { FormObjectHitResult } from '../core/types';
import type { DocumentPosition } from '../core/types';
import type { EventBus } from '../core/event-bus';
import type { WasmBridge } from '../core/wasm-bridge';

const active = new WeakSet<object>();
export function planFormAction(context: FormActionContext, signal: AbortSignal, timeoutMs = 1000,
  workerFactory = () => new Worker(new URL('./form-action-worker.ts', import.meta.url), { type: 'module' })): Promise<FormActionPlan> {
  if (!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 5000) return Promise.reject(new Error('FORM_ACTION_TIMEOUT'));
  if (signal.aborted) return Promise.reject(new Error('FORM_ACTION_CANCELLED'));
  return new Promise((resolve, reject) => {
    const worker = workerFactory();
    let settled = false;
    const finish = (error?: Error, plan?: FormActionPlan) => {
      if (settled) return; settled = true;
      clearTimeout(timer); signal.removeEventListener('abort', cancel); worker.terminate();
      if (error) reject(error); else resolve(plan!);
    };
    const cancel = () => finish(new Error('FORM_ACTION_CANCELLED'));
    const timer = setTimeout(() => finish(new Error('FORM_ACTION_TIMEOUT')), timeoutMs);
    signal.addEventListener('abort', cancel, { once: true });
    worker.onerror = worker.onmessageerror = () => finish(new Error('FORM_ACTION_WORKER_FAILED'));
    worker.onmessage = ({ data }) => data.error ? finish(new Error(data.error)) : finish(undefined, data.result);
    try { worker.postMessage(context); } catch { finish(new Error('FORM_ACTION_WORKER_FAILED')); }
  });
}

export function applyFormActionPlan(wasm: WasmBridge, plan: FormActionPlan, initial: DocumentPosition) {
  wasm.applyFormActionUpdates(plan.updates);
  let cursor = { ...initial };
  for (const command of plan.commands) {
    if (command.name === 'MoveDocBegin') cursor = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
    else if (command.name === 'MoveDocEnd') {
      const sectionIndex = wasm.getDocumentInfo().sectionCount - 1;
      const paragraphIndex = wasm.getParagraphCount(sectionIndex) - 1;
      cursor = { sectionIndex, paragraphIndex, charOffset: wasm.getParagraphLength(sectionIndex, paragraphIndex) };
    } else if (command.name === 'InsertText') {
      if (cursor.cellPath?.length || cursor.parentParaIndex !== undefined) throw new Error('FORM_ACTION_BODY_TARGET_REQUIRED');
      const result = JSON.parse(wasm.insertText(cursor.sectionIndex, cursor.paragraphIndex, cursor.charOffset, command.text ?? ''));
      if (result.ok === false) throw new Error('FORM_ACTION_EDIT_FAILED');
      cursor = { ...cursor, charOffset: result.charOffset ?? cursor.charOffset + Array.from(command.text ?? '').length };
    }
  }
  return cursor;
}

export async function executeFormButton(self: { owner: object; wasm: WasmBridge; eventBus: EventBus; executePlan(plan: FormActionPlan): void }, hit: FormObjectHitResult, pageIndex: number): Promise<void> {
  assertFormActionsEnabled();
  if (active.has(self.owner)) throw new Error('FORM_ACTION_BUSY: 다른 단추 작업이 진행 중입니다.');
  if (hit.sec === undefined || hit.para === undefined || hit.ci === undefined) throw new Error('FORM_ACTION_TARGET');
  const address: FormAddress = { section: hit.sec, para: hit.para, control: hit.ci,
    cellPath: hit.inCell ? [[hit.tableCi!, hit.cellIdx!, hit.cellPara!]] : [] };
  if (hit.inCell && hit.bbox) {
    const point = self.wasm.hitTest(pageIndex, hit.bbox.x + hit.bbox.w / 2, hit.bbox.y + hit.bbox.h / 2);
    if (point.cellPath?.length) {
      address.para = point.parentParaIndex ?? hit.para;
      address.cellPath = point.cellPath.map(entry => [entry.controlIndex, entry.cellIndex, entry.cellParaIndex]);
    }
  }
  const generation = self.wasm.documentGeneration;
  const context = self.wasm.getFormActionContext(address);
  const selected = context.forms.find(form => JSON.stringify(form.address) === JSON.stringify(context.button));
  if (!selected || (hit.name !== undefined && selected.name !== hit.name)) throw new Error('FORM_ACTION_TARGET_MISMATCH');
  const controller = new AbortController();
  const off = self.eventBus.on('document-swapped', () => controller.abort());
  const cancel = (event: KeyboardEvent) => { if (event.key === 'Escape') controller.abort(); };
  window.addEventListener('keydown', cancel, true); active.add(self.owner);
  try {
    const plan = await planFormAction(context, controller.signal);
    if (controller.signal.aborted || self.wasm.documentGeneration !== generation) throw new Error('FORM_ACTION_STALE');
    self.executePlan(plan);
  } finally {
    off(); window.removeEventListener('keydown', cancel, true); active.delete(self.owner);
  }
}
