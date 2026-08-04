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
