/** Studio 편집 오류 계약. 문자열·번들 위치로 복구 여부를 추측하지 않는다. */
export const COMMAND_ERROR_CAPABILITY = 'command-error-v1' as const;
export type RollbackOutcome = 'restored' | 'failed' | 'unknown' | 'rejected';
export interface CommandErrorV1 {
  schemaVersion: 1;
  capability: typeof COMMAND_ERROR_CAPABILITY;
  eventId: string;
  documentSessionId: string;
  operation: string;
  code: 'COMMAND_REJECTED' | 'COMMAND_ROLLED_BACK' | 'ROLLBACK_FAILED' | 'INTEGRITY_UNKNOWN';
  recovery: 'succeeded' | 'failed' | 'unknown';
  checks: { content: boolean; editing: boolean; history: boolean; dirty: boolean; ready: boolean };
}

/** 명령 호출자는 항상 실패를 받는다. 보고 완료는 success 응답이 아니다. */
export class StudioCommandError extends Error {
  readonly report: Promise<CommandErrorV1>;
  constructor(cause: unknown, report: Promise<CommandErrorV1>) {
    super('Studio 편집 명령을 완료하지 못했습니다.', { cause });
    this.name = 'StudioCommandError';
    this.report = report;
  }
}

export interface CommandFailureCheckpoint {
  /** 문서·history는 복원하지 않는다. 성공한 snapshot rollback 뒤 편집 상태만 복원한다. */
  restoreEditing(): void;
  editingUnchanged(): boolean;
  historyUnchanged(): boolean;
  dirtyUnchanged(): boolean;
}

function isTrap(error: unknown): boolean {
  return (typeof WebAssembly !== 'undefined' && error instanceof WebAssembly.RuntimeError)
    || (error instanceof AggregateError && error.errors.some(isTrap));
}

// 비보안 컨텍스트(LAN IP의 http 등)에는 crypto.randomUUID가 없다. getRandomValues는
// 보안 컨텍스트가 아니어도 있으므로 같은 128비트 난수로 세션 ID를 만든다.
function sessionInstanceId(): string {
  const uuid = globalThis.crypto.randomUUID?.();
  if (uuid) return uuid;
  const bytes = globalThis.crypto.getRandomValues(new Uint8Array(16));
  return `s_${Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('')}`;
}

export class CommandErrorBoundary {
  private sequence = 0;
  private readonly instance = sessionInstanceId();
  private blockedSession: string | null = null;
  private pendingSession: string | null = null;
  private lastError: StudioCommandError | null = null;
  private readonly options: {
    generation(): number;
    refresh(): Promise<void>;
    probe(): void;
    publish(payload: CommandErrorV1): void;
  };
  constructor(options: CommandErrorBoundary['options']) { this.options = options; }

  sessionId(): string { return `${this.instance}:${this.options.generation()}`; }
  state() {
    const documentSessionId = this.sessionId();
    return { schemaVersion: 1 as const, capability: COMMAND_ERROR_CAPABILITY, documentSessionId,
      status: this.pendingSession === documentSessionId ? 'recovering'
        : this.blockedSession === documentSessionId ? 'blocked' : 'ready' };
  }

  assertReady(): void {
    if (this.state().status !== 'ready' && this.lastError) throw this.lastError;
  }

  fail(operation: string, cause: unknown, rollback: RollbackOutcome,
    checkpoint: CommandFailureCheckpoint, sessionId: string): StudioCommandError {
    if (cause instanceof StudioCommandError) return cause;
    this.pendingSession = sessionId;
    // Microtask까지 기다려 caller에 typed failure를 먼저 반환한다.
    const report = Promise.resolve().then(async (): Promise<CommandErrorV1> => {
      const checks = { content: false, editing: false, history: false, dirty: false, ready: false };
      let recovery: CommandErrorV1['recovery'] = rollback === 'failed' ? 'failed' : 'unknown';
      if (!isTrap(cause) && (rollback === 'restored' || rollback === 'rejected')
          && sessionId === this.sessionId()) {
        try {
          checks.content = true;
          checkpoint.restoreEditing();
          let timer: ReturnType<typeof setTimeout> | undefined;
          try {
            await Promise.race([
              this.options.refresh(),
              new Promise<never>((_, reject) => {
                timer = setTimeout(() => reject(new Error('Recovery refresh timed out')), 10_000);
              }),
            ]);
          } finally { clearTimeout(timer); }
          if (sessionId !== this.sessionId()) throw new Error('Document replaced during recovery');
          this.options.probe();
          checks.editing = checkpoint.editingUnchanged();
          checks.history = checkpoint.historyUnchanged();
          checks.dirty = checkpoint.dirtyUnchanged();
          checks.ready = checks.editing && checks.history && checks.dirty;
          if (Object.values(checks).every(Boolean)) recovery = 'succeeded';
        } catch {
          // 조회·화면 갱신 실패도 복구 성공으로 승격하지 않는다.
          checks.ready = false;
        }
      }
      const payload: CommandErrorV1 = {
        schemaVersion: 1, capability: COMMAND_ERROR_CAPABILITY,
        eventId: `${sessionId}:${++this.sequence}`, documentSessionId: sessionId, operation,
        code: recovery === 'succeeded' ? (rollback === 'rejected' ? 'COMMAND_REJECTED' : 'COMMAND_ROLLED_BACK')
          : recovery === 'failed' ? 'ROLLBACK_FAILED' : 'INTEGRITY_UNKNOWN',
        recovery, checks,
      };
      if (this.pendingSession === sessionId) this.pendingSession = null;
      if (sessionId === this.sessionId() && recovery !== 'succeeded') this.blockedSession = sessionId;
      Object.freeze(checks);
      Object.freeze(payload);
      // 실패한 observer가 보고 Promise를 reject시켜 두 번째 unhandled rejection을 만들지 않는다.
      try { this.options.publish(payload); } catch (error) { console.error('[command-error] notification failed', error); }
      return payload;
    });
    this.lastError = new StudioCommandError(cause, report);
    return this.lastError;
  }
}

/** 알려진 명령 오류만 별도 통지 경로가 소유한다. 일반 오류/초기화 오류는 그대로 전파한다. */
export function installCommandErrorGlobalRouting(target: Window): () => void {
  const error = (event: ErrorEvent) => { if (event.error instanceof StudioCommandError) event.preventDefault(); };
  const rejection = (event: PromiseRejectionEvent) => {
    if (event.reason instanceof StudioCommandError) event.preventDefault();
  };
  target.addEventListener('error', error);
  target.addEventListener('unhandledrejection', rejection);
  return () => {
    target.removeEventListener('error', error);
    target.removeEventListener('unhandledrejection', rejection);
  };
}

/** 수신자의 현재 세션은 handshake/getCommandErrorState에서 얻어야 한다. */
export function inspectCommandError(value: unknown, sessionId: string, seen: Set<string>):
  'accept' | 'invalid' | 'unsupported' | 'stale' | 'duplicate' {
  if (!value || typeof value !== 'object') return 'invalid';
  const p = value as Partial<CommandErrorV1>;
  if (p.schemaVersion !== 1 || p.capability !== COMMAND_ERROR_CAPABILITY) return 'unsupported';
  if (![p.eventId, p.documentSessionId, p.operation].every(v => typeof v === 'string' && v.length > 0)
      || !['COMMAND_REJECTED', 'COMMAND_ROLLED_BACK', 'ROLLBACK_FAILED', 'INTEGRITY_UNKNOWN'].includes(p.code!)
      || !['succeeded', 'failed', 'unknown'].includes(p.recovery!) || !p.checks
      || !['content', 'editing', 'history', 'dirty', 'ready'].every(k =>
        typeof (p.checks as unknown as Record<string, unknown>)[k] === 'boolean')) return 'invalid';
  if ((p.recovery === 'succeeded') !== ['COMMAND_REJECTED', 'COMMAND_ROLLED_BACK'].includes(p.code!)
      || (p.recovery === 'failed') !== (p.code === 'ROLLBACK_FAILED')
      || (p.recovery === 'succeeded' && !Object.values(p.checks).every(Boolean))) return 'invalid';
  if (p.documentSessionId !== sessionId) return 'stale';
  if (seen.has(p.eventId!)) return 'duplicate';
  seen.add(p.eventId!);
  return 'accept';
}
