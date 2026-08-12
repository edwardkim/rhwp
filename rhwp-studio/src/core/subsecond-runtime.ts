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
  onopen: ((event: Event) => void) | null;
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
  now?: () => number;
  patchAccumulation?: SubsecondPatchAccumulation;
};

export type SubsecondPatchAccumulationOptions = {
  warn?: (message: string) => void;
  warnEveryPatches?: number;
  /** wasm 선형 메모리 크기(byte). 측정할 수 없으면 null. 경고에 실측값을 얹는 용도다. */
  measureHeapBytes?: () => number | null;
};

const RECONNECT_MIN_MS = 250;
const RECONNECT_MAX_MS = 4_000;
/**
 * 이 시간보다 오래 붙어 있었던 연결만 "살아 있었다"고 보고 백오프를 되돌린다.
 * 재연결 상한과 우연히 같은 값이지만 다른 개념이다 — 상한을 바꾼다고 같이 바뀌면 안 된다.
 */
const STABLE_CONNECTION_MS = 4_000;
const PATCH_WARNING_INTERVAL = 32;
const BYTES_PER_MB = 1024 * 1024;

/**
 * 세션에 쌓인 핫패치 수를 세고, 셀 때마다가 아니라 임계값마다 경고한다.
 *
 * subsecond 는 적용한 패치를 회수하지 못한다. `commit_patch` 는 이전 `JumpTable` 을 그대로 버리고
 * (`subsecond-0.7.10/src/lib.rs:308-312`, 상류가 문서화한 의도적 누수), wasm 의 선형 메모리와
 * 간접 함수 테이블은 `memory.grow`/`funcs.grow` 로 늘기만 하며 줄어들 수 없다(`:628-632`).
 * 즉 패치 하나가 더한 코드·데이터는 세션이 끝날 때까지 남는다 — 회수는 플랫폼 제약상 불가능하고,
 * 유일한 회수 지점은 새로고침이다.
 *
 * 그래서 이 클래스는 고치지 못하는 것을 보이게만 한다. 경고 없이 쌓이면 편집 도중
 * `RangeError: WebAssembly.Memory.grow(): Unable to grow instance memory` 로 탭이 죽고,
 * 핫패치가 원인이라는 표시가 어디에도 남지 않는다.
 *
 * 경고 기준은 패치 수다. 선형 메모리 크기는 경고에 실측 근거로 얹기만 하고 기준으로 쓰지 않는다 —
 * 큰 문서를 열어도 같이 커지는 값이라, 그것으로 임계를 잡으면 핫패치가 아닌 사용을 핫패치 탓으로 돌린다.
 */
export class SubsecondPatchAccumulation {
  private applied = 0;
  private readonly warn: (message: string) => void;
  private readonly warnEveryPatches: number;
  private readonly measureHeapBytes: () => number | null;

  constructor(options: SubsecondPatchAccumulationOptions = {}) {
    this.warn = options.warn ?? (message => console.warn(message));
    this.warnEveryPatches = Math.max(1, options.warnEveryPatches ?? PATCH_WARNING_INTERVAL);
    this.measureHeapBytes = options.measureHeapBytes ?? (() => null);
  }

  /**
   * 패치 하나가 적용에 들어갔음을 기록한다.
   *
   * 정확히는 "이 build 를 위한 `HotReload` 중 점프 테이블 역직렬화까지 성공한 메시지 수"다.
   * wasm 에서 `apply_patch` 는 fetch·instantiate future 를 띄우고 바로 `Ok(())` 를 돌려주며
   * (`subsecond-0.7.10/src/lib.rs:551`), 그 future 는 `.wasm` 이 아닌 경로에서 조용히 빠져나갈 수도
   * 있다(`:565-567`). 그래서 이 수는 실제 메모리 증가 횟수의 상한이다.
   */
  recordApplied(): void {
    this.applied += 1;
    if (this.applied % this.warnEveryPatches !== 0) return;
    this.warn(
      `[subsecond] 이 세션에 핫패치 ${this.applied}건(적용 요청 기준)이 쌓였다${this.heapSuffix()}. `
      + '적용한 패치는 회수되지 않는다(wasm 선형 메모리·간접 함수 테이블은 축소 불가). '
      + '세션이 길어지면 WebAssembly.Memory.grow() 실패로 탭이 죽으므로 새로고침으로 세션을 끊어라.',
    );
  }

  private heapSuffix(): string {
    // 경고는 패치 배달 경로 안에서 만들어진다. 측정이 던져 그 경로로 예외가 새어 나가면 안 된다.
    try {
      const bytes = this.measureHeapBytes();
      if (bytes === null || !Number.isFinite(bytes)) return '';
      return ` — wasm 선형 메모리 ${Math.round(bytes / BYTES_PER_MB)}MB`;
    } catch {
      return '';
    }
  }
}

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

/**
 * 핫패치가 렌더 함수를 바꿨는지 애니메이션 프레임마다 확인하고, 바뀌었으면 재도색을 알린다.
 *
 * 수명은 realm 과 같다. 스튜디오에는 문서 닫기도 뷰 폐기도 없어서 `stop()` 을 부를 시점이
 * `CanvasView.dispose()` 밖에 없고, 그 시점 자체가 오지 않는다. 그래서 이 루프는 스스로 멎지
 * 않는 것이 정상 동작이며, 특히 재도색이 던져도 다음 프레임 예약을 건너뛰어서는 안 된다.
 */
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
    // 예약은 언제나 한 개다. 재도색 안에서 stop()·start() 가 불리면 그 start() 가 이미 예약한
    // 프레임을 아래 finally 가 덮어써, 취소할 수 없는 두 번째 루프가 남는다.
    if (this.frameId !== null) return;
    this.frameId = this.scheduler.requestAnimationFrame(() => {
      this.frameId = null;
      // 재도색은 패치된 코드를 실행하므로 던질 수 있다 — 패치에 버그를 넣는 것이 정상 상황이다.
      // 예외는 그대로 흘려보내(rAF 콜백 밖에서 브라우저가 보고한다) 다음 프레임만 되살린다.
      // 여기서 재무장을 건너뛰면 running=true, frameId=null 로 남아 start()·stop() 둘 다
      // 손댈 것을 못 찾고, 세션이 끝날 때까지 감시가 영구히 멎는다.
      try {
        this.checkRevision();
      } finally {
        if (this.running) this.schedule();
      }
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
    // 재도색보다 먼저 올린다. 던지는 패치를 매 프레임 다시 그리면 초당 60번 실패하므로,
    // 실패한 리비전은 다음 패치가 올 때까지 다시 시도하지 않는다 — 세션은 살아 있고, 재도색이
    // 던지는 패치 버그는 다음 저장이 만드는 새 리비전에서 정상으로 돌아온다.
    // 아래 무효화 자체가 계속 트랩하면 어느 리비전도 그리지 못하지만, 그건 핫패치 경계 밖의
    // 고장이라 새 패치로도 안 고쳐진다 — 보고는 #4578, 경계는 #4577 의 몫이다.
    this.lastRevision = revision;
    if (this.capabilities.invalidateSubsecondRenderCaches()) {
      this.onPatched(revision);
    }
  }
}

/**
 * dx devserver 소켓에 붙어 도착한 패치를 wasm 에 넘긴다.
 *
 * 돌려주는 해제 함수는 소켓과 재연결 타이머를 함께 내린다. 스튜디오에서는 realm 이 끝날 때까지
 * 부를 시점이 없고(문서 닫기·뷰 폐기가 없다) 호출부는 `wasm-bridge.ts` 의 중복 연결 guard 하나뿐이지만,
 * 소켓을 연 곳이 내리는 방법을 함께 돌려주는 형태는 유지한다 — 테스트와 이후 종료 경로의 유일한 해제선이다.
 */
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
  // Node 기반 단위 테스트에서는 Vite가 주입하는 import.meta.env가 없으므로 기본 진단을 비활성화한다.
  const report = options.reportSignal ?? (typeof window === 'undefined' ? () => {} : reportToDevConsole);
  // Node 기반 단위 테스트는 소켓·타이머만 대체해도 되도록 전역 오류 대상은 브라우저에서만 기본 연결한다.
  const errorEvents = options.errorEvents ?? (typeof window === 'undefined' ? null : window);
  // 연결이 버틴 시간만 재므로 단조 시계를 쓴다. Date.now() 는 시스템 시각 변경에 흔들린다.
  const now = options.now ?? (() => performance.now());
  const patchAccumulation = options.patchAccumulation ?? new SubsecondPatchAccumulation();
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
  errorEvents?.addEventListener('error', onGlobalFailure);
  errorEvents?.addEventListener('unhandledrejection', onGlobalFailure);
  let openedAt: number | null = null;

  const connect = (): void => {
    if (!active) return;
    socket = createWebSocket(url);
    socket.onmessage = event => {
      if (typeof event.data !== 'string') return;
      const outcome = applyMessage(event.data);
      if (outcome === 'patch-dispatched') {
        dispatchedPatches += 1;
        patchAccumulation.recordApplied();
      }
      report({ kind: 'outcome', code: outcome });
    };
    socket.onopen = () => {
      openedAt = now();
    };
    socket.onclose = event => {
      // openedAt 은 언제나 "지금 열려 있는 소켓"만 가리킨다 — 재연결하지 않는 경로에서도 지운다.
      const lasted = openedAt === null ? 0 : now() - openedAt;
      openedAt = null;
      if (!active || event.code === 1001) return;
      // 살아 있었던 연결이 끊긴 것이면 백오프를 되돌린다. 되돌리지 않으면 dx serve 를 껐다 켠
      // 뒤에도 끊김마다 최대 4초를 기다려, 남은 세션 내내 첫 패치가 그만큼 늦는다.
      // 반대로 핸드셰이크만으로 되돌려서도 안 된다 — dx serve 가 꺼져 Vite 프록시가 열자마자
      // 끊는 상황에서 250ms 재연결을 영원히 돌게 된다. 버틴 시간으로 둘을 가른다.
      if (lasted >= STABLE_CONNECTION_MS) reconnectDelay = RECONNECT_MIN_MS;
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
    errorEvents?.removeEventListener('error', onGlobalFailure);
    errorEvents?.removeEventListener('unhandledrejection', onGlobalFailure);
    if (reconnectTimer !== null) {
      cancelTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    socket?.close();
    socket = null;
  };
}
