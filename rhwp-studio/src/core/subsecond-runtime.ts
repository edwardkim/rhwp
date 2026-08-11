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
};

const RECONNECT_MIN_MS = 250;
const RECONNECT_MAX_MS = 4_000;

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
      if (typeof event.data === 'string') {
        applyMessage(event.data);
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
