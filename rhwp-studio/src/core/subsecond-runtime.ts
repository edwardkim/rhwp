export interface SubsecondRenderCapabilities {
  isSubsecondHotpatchEnabled(): boolean;
  getSubsecondPatchRevision(): string | null;
  invalidateSubsecondRenderCaches(): boolean;
}

export interface SubsecondWasmExports {
  /**
   * 데브서버 메시지 한 건을 처리하고 `src/subsecond_dev.rs` 의
   * `DevtoolsMessageOutcome::code()` 식별자를 돌려준다. `'patch-dispatched'` 는
   * "점프 테이블을 subsecond 에 넘겼다" 까지만 뜻한다 — 아래
   * {@link SUBSECOND_OUTCOMES} 주석 참고.
   */
  applySubsecondDevtoolsMessage?: (message: string) => string;
}

/**
 * 보고할 신호. **사실만** 담고 문구는 담지 않는다.
 *
 * 문구를 여기 넣으면 메시지 경로에서 참조되어 프로덕션 번들까지 따라온다(실측: 진단 문구가
 * `dist/assets/index-*.js` 에 그대로 들어갔다). 문구 생성은 개발 빌드에서만 살아남는
 * {@link describeSubsecondSignal} 로 미룬다.
 */
export type SubsecondSignal =
  | { kind: 'outcome'; code: string }
  | { kind: 'global-failure'; eventType: string; reason: string; dispatchedPatches: number };

/** 개발자에게 남길 한 줄. */
export type SubsecondDiagnostic = {
  level: 'debug' | 'info' | 'warn';
  message: string;
};

type AnimationFrameScheduler = {
  requestAnimationFrame(callback: FrameRequestCallback): number;
  cancelAnimationFrame(id: number): void;
};

type WebSocketConnection = {
  onmessage: ((event: MessageEvent) => void) | null;
  onclose: ((event: CloseEvent) => void) | null;
  close(): void;
};

/** 전역 오류·거부 이벤트를 듣는 대상. 기본값은 `window` 이고 테스트가 대체한다. */
type FailureEventTarget = {
  addEventListener(type: string, listener: (event: Event) => void): void;
  removeEventListener(type: string, listener: (event: Event) => void): void;
};

type SubsecondDevtoolsOptions = {
  location?: Pick<Location, 'protocol' | 'host'>;
  createWebSocket?: (url: string) => WebSocketConnection;
  setTimeout?: (callback: () => void, delay: number) => number;
  clearTimeout?: (id: number) => void;
  reportSignal?: (signal: SubsecondSignal) => void;
  errorEvents?: FailureEventTarget;
};

const RECONNECT_MIN_MS = 250;
const RECONNECT_MAX_MS = 4_000;

/**
 * `DevtoolsMessageOutcome::code()` → 그 결과가 뜻하는 것과 **다음에 볼 곳**.
 *
 * `'patch-dispatched'` 가 성공을 뜻하지 않는 이유: wasm32 의 `subsecond::apply_patch` 는
 * patch wasm 의 fetch/compile/instantiate future 를 띄우고 즉시 `Ok(())` 를 돌려주며
 * (subsecond 0.7.10 `src/lib.rs:551`, `:690`), future 안의 실패는 전부 `.unwrap()`/`panic!`
 * (`lib.rs:578-582`)이라 반환값이 되지 못한다. 그 실패를 붙잡는 곳이 전역 오류 청취다.
 *
 * 이 표는 {@link describeSubsecondSignal} 에서만 참조된다. 메시지 경로에서 참조하면 프로덕션
 * 번들까지 따라온다.
 */
const SUBSECOND_OUTCOMES: Record<string, SubsecondDiagnostic | undefined> = {
  'patch-dispatched': {
    level: 'info',
    message:
      '점프 테이블을 subsecond 에 넘겼다. wasm 에서 적용은 비동기라 성공 여부는 여기서 알 수 없다 — ' +
      '화면이 그대로면 이어지는 경고와 콘솔의 Rust panic 메시지를 본다.',
  },
  'not-hot-reload': {
    level: 'debug',
    message: 'HotReload 가 아닌 데브서버 메시지다. 정상 제어 트래픽이므로 조치할 것이 없다.',
  },
  'not-json': {
    level: 'warn',
    message:
      '데브서버가 JSON 이 아닌 텍스트 프레임을 보냈다. `npm run subsecond:install` 이 고정한 ' +
      'dioxus-cli 0.7.10 이 아닌 dx 가 떠 있는지 `npm run subsecond:serve` 터미널에서 확인한다.',
  },
  'foreign-build-id': {
    level: 'warn',
    message:
      '다른 build_id 를 향한 패치라 무시했다. 스튜디오는 `/_dioxus?build_id=0` 으로만 접속하므로 ' +
      'dx serve 가 다른 빌드를 감시하고 있는지 확인한다.',
  },
  'missing-jump-table': {
    level: 'warn',
    message:
      'HotReload 에 jump_table 이 없다. dx 가 패치 링크에 실패하면 이 모양이 되므로 ' +
      '`npm run subsecond:serve` 터미널의 링크 오류부터 본다 (호스트가 unix 가 아니면 항상 실패한다).',
  },
  'undeserializable-jump-table': {
    level: 'warn',
    message:
      'jump_table 을 subsecond 0.7.10 의 JumpTable 로 읽지 못했다. dx 와 크레이트 버전이 어긋났는지 ' +
      '루트 `Cargo.toml` 의 `subsecond = "=0.7.10"` 과 dx 버전을 대조한다.',
  },
  'patch-rejected': {
    level: 'warn',
    message:
      'apply_patch 가 오류를 돌려줬다. wasm 빌드에서는 나올 수 없는 값이므로 ' +
      '이 번들이 정말 wasm32 용인지(`src/subsecond_dev.rs` 의 DevtoolsMessageOutcome 문서) 확인한다.',
  },
};

/** 신호 하나를 개발자가 읽을 한 줄로 만든다. */
export function describeSubsecondSignal(signal: SubsecondSignal): SubsecondDiagnostic {
  if (signal.kind === 'global-failure') {
    return {
      level: 'warn',
      message:
        `패치 ${signal.dispatchedPatches}건을 넘긴 뒤 처리되지 않은 ${signal.eventType} 가 도달했다: ` +
        `${signal.reason}. 핫패치와 무관한 오류일 수도 있지만, subsecond 의 wasm apply_patch 는 ` +
        '실패를 반환값으로 돌려주지 않으므로 이 이벤트가 유일한 신호일 수 있다. ' +
        '다음: 콘솔의 Rust panic 메시지 → Network 탭의 `/wasm/librhwp-subsecond-patch-*.wasm` 응답 → ' +
        '`npm run subsecond:serve` 터미널의 링크 오류.',
    };
  }
  return (
    SUBSECOND_OUTCOMES[signal.code] ?? {
      level: 'warn',
      message:
        `읽지 못한 결과 값 ${JSON.stringify(signal.code)}. ` +
        '`src/subsecond_dev.rs` 의 DevtoolsMessageOutcome::code() 와 이 표가 어긋났거나, ' +
        '옛 WASM 번들(불리언을 돌려주던 판)이 로드돼 있다 — `npm run subsecond:sync` 를 다시 돌린다.',
    }
  );
}

/** 전역 오류 이벤트에서 사람이 읽을 원인 문자열을 뽑는다. */
function readFailureReason(event: Event): string {
  const candidate =
    (event as PromiseRejectionEvent).reason ??
    (event as ErrorEvent).error ??
    (event as ErrorEvent).message;
  if (candidate === undefined || candidate === null) return '(원인 없음)';
  return candidate instanceof Error ? `${candidate.name}: ${candidate.message}` : String(candidate);
}

/**
 * 기본 진단 출력.
 *
 * 프로덕션 번들에서 Vite 가 `import.meta.env.DEV` 를 `false` 로 치환하므로 이 함수 본문이
 * 통째로 죽고, 그 결과 {@link describeSubsecondSignal} 과 진단 문구 표도 아무 데서도 참조되지
 * 않아 번들에서 사라진다(`dist/assets/index-*.js` 에서 실측). 런타임으로도
 * `applySubsecondDevtoolsMessage` 는 `subsecond-dev` 빌드에만 존재해 여기까지 오지 않는다.
 */
function reportToDevConsole(signal: SubsecondSignal): void {
  if (!import.meta.env.DEV) return;
  const { level, message } = describeSubsecondSignal(signal);
  console[level](`[subsecond] ${message}`);
}

export class SubsecondRevisionWatcher {
  private frameId: number | null = null;
  private running = false;
  private hasBaseline = false;
  private lastRevision: string | null = null;
  private capabilities: SubsecondRenderCapabilities;
  private onPatched: (revision: string) => void;
  private scheduler: AnimationFrameScheduler;

  constructor(
    capabilities: SubsecondRenderCapabilities,
    onPatched: (revision: string) => void,
    scheduler: AnimationFrameScheduler = {
      requestAnimationFrame: callback => requestAnimationFrame(callback),
      cancelAnimationFrame: id => cancelAnimationFrame(id),
    },
  ) {
    this.capabilities = capabilities;
    this.onPatched = onPatched;
    this.scheduler = scheduler;
  }

  start(): boolean {
    if (this.running) return true;
    if (!this.capabilities.isSubsecondHotpatchEnabled()) return false;
    this.running = true;
    this.schedule();
    return true;
  }

  stop(): void {
    this.running = false;
    if (this.frameId !== null) {
      this.scheduler.cancelAnimationFrame(this.frameId);
      this.frameId = null;
    }
  }

  private schedule(): void {
    this.frameId = this.scheduler.requestAnimationFrame(() => {
      this.frameId = null;
      this.checkRevision();
      if (this.running) this.schedule();
    });
  }

  private checkRevision(): void {
    const revision = this.capabilities.getSubsecondPatchRevision();
    if (revision === null) return;
    if (!this.hasBaseline) {
      this.hasBaseline = true;
      this.lastRevision = revision;
      return;
    }
    if (revision === this.lastRevision) return;
    this.lastRevision = revision;
    if (this.capabilities.invalidateSubsecondRenderCaches()) {
      this.onPatched(revision);
    }
  }
}

export function connectSubsecondDevtools(
  wasm: SubsecondWasmExports,
  options: SubsecondDevtoolsOptions = {},
): (() => void) | null {
  const applyMessage = wasm.applySubsecondDevtoolsMessage;
  if (typeof applyMessage !== 'function') return null;

  const location = options.location ?? window.location;
  const createWebSocket = options.createWebSocket ?? (url => new WebSocket(url));
  const scheduleTimeout = options.setTimeout ?? ((callback, delay) => window.setTimeout(callback, delay));
  const cancelTimeout = options.clearTimeout ?? (id => window.clearTimeout(id));
  const report = options.reportSignal ?? reportToDevConsole;
  const errorEvents = options.errorEvents ?? window;
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${protocol}//${location.host}/_dioxus?build_id=0`;

  let active = true;
  let socket: WebSocketConnection | null = null;
  let reconnectTimer: number | null = null;
  let reconnectDelay = RECONNECT_MIN_MS;
  let dispatchedPatches = 0;

  // wasm 의 apply_patch 실패는 반환값이 아니라 panic → trap 으로만 나간다. trap 은 microtask
  // 경계에서 전역 `error`(queueMicrotask 경로) 또는 `unhandledrejection`(promise 폴백 경로)이
  // 되므로 양쪽을 모두 듣는다. 어느 오류가 패치 탓인지는 알 수 없으니 단정하지 않고,
  // "패치를 넘긴 뒤였다" 는 사실과 다음에 볼 곳만 말한다.
  const onGlobalFailure = (event: Event): void => {
    if (dispatchedPatches === 0) return;
    report({
      kind: 'global-failure',
      eventType: event.type,
      reason: readFailureReason(event),
      dispatchedPatches,
    });
  };
  errorEvents.addEventListener('error', onGlobalFailure);
  errorEvents.addEventListener('unhandledrejection', onGlobalFailure);

  const connect = (): void => {
    if (!active) return;
    socket = createWebSocket(url);
    socket.onmessage = event => {
      if (typeof event.data !== 'string') return;
      const outcome = applyMessage(event.data);
      if (outcome === 'patch-dispatched') dispatchedPatches += 1;
      report({ kind: 'outcome', code: outcome });
    };
    socket.onclose = event => {
      if (!active || event.code === 1001) return;
      reconnectTimer = scheduleTimeout(() => {
        reconnectTimer = null;
        connect();
      }, reconnectDelay);
      reconnectDelay = Math.min(RECONNECT_MAX_MS, reconnectDelay * 2);
    };
  };

  connect();

  return () => {
    active = false;
    errorEvents.removeEventListener('error', onGlobalFailure);
    errorEvents.removeEventListener('unhandledrejection', onGlobalFailure);
    if (reconnectTimer !== null) {
      cancelTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    socket?.close();
    socket = null;
  };
}
