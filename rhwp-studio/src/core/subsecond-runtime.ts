export interface SubsecondRenderCapabilities {
  isSubsecondHotpatchEnabled(): boolean;
  getSubsecondPatchRevision(): string | null;
  invalidateSubsecondRenderCaches(): boolean;
}

export interface SubsecondWasmExports {
  applySubsecondDevtoolsMessage?: (message: string) => boolean;
}

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

type SubsecondDevtoolsOptions = {
  location?: Pick<Location, 'protocol' | 'host'>;
  createWebSocket?: (url: string) => WebSocketConnection;
  setTimeout?: (callback: () => void, delay: number) => number;
  clearTimeout?: (id: number) => void;
  patchBudget?: SubsecondPatchBudget;
};

export type SubsecondPatchBudgetOptions = {
  warn?: (message: string) => void;
  warnEveryPatches?: number;
  /** wasm 선형 메모리 크기(byte). 측정할 수 없으면 null. */
  measureHeapBytes?: () => number | null;
};

const RECONNECT_MIN_MS = 250;
const RECONNECT_MAX_MS = 4_000;
const PATCH_WARNING_INTERVAL = 32;
const BYTES_PER_MIB = 1024 * 1024;

/**
 * 적용한 핫패치가 세션 안에 얼마나 쌓였는지 세고, 임계값마다 경고한다.
 *
 * subsecond 는 적용한 패치를 회수하지 못한다. `commit_patch` 는 이전 `JumpTable` 을 그대로 버리고
 * (`subsecond-0.7.10/src/lib.rs:308-312`, 상류가 문서화한 의도적 누수), wasm 의 선형 메모리와
 * 간접 함수 테이블은 `memory.grow`/`funcs.grow` 로 늘기만 하며 줄어들 수 없다(`:628-632`).
 * 즉 패치 하나가 더한 코드·데이터는 세션이 끝날 때까지 남는다 — 회수는 플랫폼 제약상 불가능하다.
 *
 * 그래서 이 클래스는 고치지 못하는 것을 보이게만 한다. 경고 없이 쌓이면 편집 도중
 * `RangeError: WebAssembly.Memory.grow(): Unable to grow instance memory` 로 탭이 죽고,
 * 핫패치가 원인이라는 표시가 어디에도 남지 않는다. 유일한 회수 지점은 새로고침이다.
 */
export class SubsecondPatchBudget {
  private applied = 0;
  private readonly warn: (message: string) => void;
  private readonly warnEveryPatches: number;
  private readonly measureHeapBytes: () => number | null;

  constructor(options: SubsecondPatchBudgetOptions = {}) {
    this.warn = options.warn ?? (message => console.warn(message));
    this.warnEveryPatches = Math.max(1, options.warnEveryPatches ?? PATCH_WARNING_INTERVAL);
    this.measureHeapBytes = options.measureHeapBytes ?? (() => null);
  }

  /**
   * 패치 하나가 적용에 들어갔음을 기록한다. wasm 에서 적용은 비동기라 이 수는 "적용을 시작한
   * 패치 수"이며, 메모리가 실제로 늘어난 시점보다 조금 앞선다.
   */
  recordApplied(): void {
    this.applied += 1;
    if (this.applied % this.warnEveryPatches !== 0) return;
    this.warn(
      `[subsecond] 이 세션에 핫패치 ${this.applied}건이 쌓였다${this.heapSuffix()}. `
      + '적용한 패치는 회수되지 않는다(wasm 선형 메모리·간접 함수 테이블은 축소 불가). '
      + '세션이 길어지면 WebAssembly.Memory.grow() 실패로 탭이 죽으므로 새로고침으로 세션을 끊어라.',
    );
  }

  private heapSuffix(): string {
    const bytes = this.measureHeapBytes();
    if (bytes === null || !Number.isFinite(bytes)) return '';
    return ` — wasm 선형 메모리 ${Math.round(bytes / BYTES_PER_MIB)}MB`;
  }
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
  const patchBudget = options.patchBudget ?? new SubsecondPatchBudget();
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${protocol}//${location.host}/_dioxus?build_id=0`;

  let active = true;
  let socket: WebSocketConnection | null = null;
  let reconnectTimer: number | null = null;
  let reconnectDelay = RECONNECT_MIN_MS;

  const connect = (): void => {
    if (!active) return;
    socket = createWebSocket(url);
    socket.onmessage = event => {
      if (typeof event.data === 'string' && applyMessage(event.data)) {
        patchBudget.recordApplied();
      }
    };
    // 연결이 살아난 순간 백오프를 되돌린다. 되돌리지 않으면 dx serve 를 껐다 켠 뒤에도
    // 다음 끊김마다 최대 4초를 기다려, 남은 세션 내내 첫 패치가 그만큼 늦는다.
    socket.onopen = () => {
      reconnectDelay = RECONNECT_MIN_MS;
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
    if (reconnectTimer !== null) {
      cancelTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    socket?.close();
    socket = null;
  };
}
