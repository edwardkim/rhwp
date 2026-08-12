/**
 * 플러그인 호스트 — 기능을 얹었다 내리는 자리.
 *
 * 코어는 플러그인을 정적으로 참조하지 않는다. 등록은 `resolve` 훅으로 바깥에서 들어오고,
 * 아무 플러그인도 올리지 않으면 이 파일은 상태만 들고 아무 일도 하지 않는다.
 *
 * 문서 조작의 규칙은 하나다 — **`transaction` 밖에서 문서를 바꾸지 않는다**. studio 의 undo 는
 * `executeOperation → CommandHistory` 경유가 전제이고, 그 밖의 뮤테이션은 undo 불가·redo 스택
 * 미무효화·스냅샷 복원에 동반 파괴를 함께 부른다(#2027 계급).
 *
 * 계획: `mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md` §2.2, §4, §7(P2)
 */
import type { HwpDocument } from '@wasm/rhwp.js';
import type { StudioAutomation } from '@/automation/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { InputHandler } from '@/engine/input-handler';
import {
  PLUGIN_API_VERSION,
  type DocumentLease,
  type PluginError,
  type PluginErrorCode,
  type PluginHost,
  type PluginLedger,
  type PluginSurface,
  type StudioPlugin,
  type Tx,
} from './types';

export function pluginError(code: PluginErrorCode, message: string): PluginError {
  const error = new Error(message) as PluginError;
  error.code = code;
  return error;
}

export interface PluginHostDeps {
  wasm: WasmBridge;
  automation: StudioAutomation;
  eventBus: EventBus;
  getInputHandler: () => InputHandler | null;
  /** 문서 교체 위임 — 원자 교체·세대 증가·이전 문서 해제는 studio 가 한다. */
  loadDocument: (bytes: Uint8Array, fileName?: string) => Promise<void>;
  createBlankDocument: () => void;
  /**
   * id 로 플러그인을 가져온다. **allowlist 는 이 훅이 쥔다** — 여기서 거절하면 호스트는
   * `PLUGIN_NOT_ALLOWED` 로 답한다. 임의 URL 로드 경로는 만들지 않는다.
   */
  resolve: (id: string) => Promise<StudioPlugin>;
}

/** unload 가 진행 중인 트랜잭션을 기다리는 상한. 넘기면 거절한다(문서는 그대로 남는다). */
const UNLOAD_TX_WAIT_MS = 2000;
const UNLOAD_TX_POLL_MS = 20;

function emptyLedger(): PluginLedger {
  return {
    commands: new Set(), menuItems: new Set(),
    eventUnsubs: [], swapUnsubs: [], activeTx: 0,
  };
}

interface LoadedPlugin {
  plugin: StudioPlugin;
  surface: PluginSurface;
  ledger: PluginLedger;
  facade: PluginHostFacade;
}

/**
 * 플러그인 하나에 주는 호스트 표면.
 *
 * 플러그인마다 별도 인스턴스를 만든다 — 원장이 플러그인별이어야 unload 가 남의 것까지 걷어가지
 * 않는다.
 */
class PluginHostFacade implements PluginHost {
  private swapListeners: Array<(lease: DocumentLease) => void> = [];

  constructor(
    private readonly pluginId: string,
    private readonly deps: PluginHostDeps,
    private readonly ledger: PluginLedger,
  ) {}

  borrowDocument(): DocumentLease | null {
    const handle = this.deps.wasm.borrowDocumentHandle();
    if (!handle) return null;
    return { handle, generation: this.deps.wasm.documentGeneration };
  }

  currentGeneration(): number {
    return this.deps.wasm.documentGeneration;
  }

  transaction<T>(label: string, fn: (tx: Tx) => T): T {
    if (this.ledger.activeTx > 0) {
      throw pluginError('NESTED_TX', `트랜잭션 중첩은 허용하지 않습니다: ${label}`);
    }
    const ih = this.deps.getInputHandler();
    if (!ih) throw pluginError('DOCUMENT_RELEASED', '문서가 로드되지 않았습니다');

    let deferred = false;
    let ran = false;
    let result!: T;

    const tx: Tx = {
      doc: () => this.requireDocument(),
      deferPagination: () => {
        if (deferred) return;
        deferred = true;
        this.deps.wasm.beginDeferredPagination();
      },
    };

    this.ledger.activeTx += 1;
    try {
      ih.executeOperation({
        kind: 'snapshot',
        operationType: `plugin:${this.pluginId}:${label}`,
        operation: () => {
          result = fn(tx);
          ran = true;
          // 조판은 커밋 직전에 한 번만 turn — 호출당 재조판을 없애는 자리다.
          if (deferred) this.deps.wasm.flushDeferredPagination();
          return ih.getCursorPosition();
        },
      });
    } finally {
      this.ledger.activeTx -= 1;
      if (deferred && !ran) {
        // 본문이 던져 롤백된 경우에도 미뤄 둔 조판은 정리한다.
        try { this.deps.wasm.cancelDeferredPagination(); } catch { /* noop */ }
      }
    }

    if (!ran) {
      // 편집 모드 게이트가 작업을 드롭한 경우다(양식 모드 등). 문서는 그대로다.
      throw pluginError('DOCUMENT_RELEASED', `현재 편집 모드에서 거절된 작업입니다: ${label}`);
    }
    return result;
  }

  /**
   * 문서를 바꾸지 않는 작업.
   *
   * 히스토리를 건드리지 않으므로 스냅샷 비용도 undo 항목도 없다. **여기서 문서를 바꾸면
   * 미기록 편집이 되어 undo 계약이 깨진다** — 무엇이 읽기인지는 호출자(어댑터)가 자기 원장으로
   * 판정하고, 그 판정의 정확성은 모드 동등성 테스트가 지킨다.
   */
  read<T>(fn: (doc: HwpDocument) => T): T {
    return fn(this.requireDocument());
  }

  async loadDocument(bytes: Uint8Array, fileName?: string): Promise<void> {
    await this.deps.loadDocument(bytes, fileName);
    this.notifySwap();
  }

  createBlankDocument(): void {
    this.deps.createBlankDocument();
    this.notifySwap();
  }

  /**
   * 자동화 표면 — 등록물을 **원장에 적으면서** 위임한다.
   *
   * 플러그인이 심은 커맨드·메뉴 항목은 unload 때 호스트가 걷어야 한다. 공유 인스턴스를 그대로
   * 넘기면 누가 무엇을 심었는지 잃어버린다.
   */
  get automation(): StudioAutomation {
    const base = this.deps.automation;
    const ledger = this.ledger;
    return {
      listCommands: () => base.listCommands(),
      getMenuModel: () => base.getMenuModel(),
      isEnabled: (id) => base.isEnabled(id),
      execute: (id, params) => base.execute(id, params),
      getContext: () => base.getContext(),
      registerCommand: (def) => { base.registerCommand(def); ledger.commands.add(def.id); },
      unregisterCommand: (id) => { base.unregisterCommand(id); ledger.commands.delete(id); },
      addMenuItem: (spec) => { base.addMenuItem(spec); ledger.menuItems.add(spec.commandId); },
      removeMenuItem: (id) => { base.removeMenuItem(id); ledger.menuItems.delete(id); },
    };
  }

  events = {
    on: (name: string, cb: (payload?: unknown) => void): (() => void) => {
      const off = this.deps.eventBus.on(name, cb);
      this.ledger.eventUnsubs.push(off);
      return off;
    },
  };

  onDocumentSwap(cb: (lease: DocumentLease) => void): () => void {
    this.swapListeners.push(cb);
    const off = () => {
      this.swapListeners = this.swapListeners.filter((fn) => fn !== cb);
    };
    this.ledger.swapUnsubs.push(off);
    return off;
  }

  /** 문서가 교체됐음을 이 플러그인에 알린다. 호스트도 외부 교체 때 부른다. */
  notifySwap(): void {
    const lease = this.borrowDocument();
    if (!lease) return;
    for (const cb of [...this.swapListeners]) {
      try { cb(lease); } catch (error) { console.error('[PluginHost] onDocumentSwap 실패', error); }
    }
  }

  /**
   * 유효한 문서 핸들.
   *
   * 세대와 포인터를 함께 본다. 해제된 문서에 대한 호출은 방어적 optional-chaining 을 거치면
   * 예외가 아니라 `0`·빈 문자열이라는 **조용한 오답**으로 돌아온다 — 그것을 여기서 끊는다.
   */
  private requireDocument(): HwpDocument {
    const handle = this.deps.wasm.borrowDocumentHandle();
    if (!handle || (handle as unknown as { __wbg_ptr?: number }).__wbg_ptr === 0) {
      throw pluginError('DOCUMENT_RELEASED', '문서가 이미 해제되었습니다');
    }
    return handle;
  }
}

export class PluginHostRegistry {
  private readonly loaded = new Map<string, LoadedPlugin>();

  constructor(private readonly deps: PluginHostDeps) {}

  list(): Array<{ id: string; apiVersion: number; active: boolean }> {
    return Array.from(this.loaded.values()).map(({ plugin }) => ({
      id: plugin.id, apiVersion: plugin.apiVersion, active: true,
    }));
  }

  isLoaded(id: string): boolean {
    return this.loaded.has(id);
  }

  async load(id: string): Promise<{ id: string; methods: string[] }> {
    const already = this.loaded.get(id);
    if (already) return { id, methods: Object.keys(already.surface) };

    let plugin: StudioPlugin;
    try {
      plugin = await this.deps.resolve(id);
    } catch (error) {
      throw pluginError('PLUGIN_NOT_ALLOWED', `허용되지 않은 플러그인입니다: ${id}`);
    }
    if (plugin.apiVersion !== PLUGIN_API_VERSION) {
      throw pluginError(
        'PLUGIN_ACTIVATE_FAILED',
        `지원하지 않는 플러그인 API 세대: ${plugin.apiVersion} (지원 ${PLUGIN_API_VERSION})`,
      );
    }

    const ledger = emptyLedger();
    const facade = new PluginHostFacade(plugin.id, this.deps, ledger);
    let surface: PluginSurface;
    try {
      surface = await plugin.activate(facade);
    } catch (error) {
      // 활성화 실패가 studio 를 죽이지 않는다. 부분 등록물은 원장으로 걷어낸다.
      this.reclaim(ledger);
      throw pluginError(
        'PLUGIN_ACTIVATE_FAILED',
        error instanceof Error ? error.message : String(error),
      );
    }

    this.loaded.set(plugin.id, { plugin, surface, ledger, facade });
    return { id: plugin.id, methods: Object.keys(surface) };
  }

  async unload(id: string): Promise<void> {
    const entry = this.loaded.get(id);
    if (!entry) throw pluginError('PLUGIN_NOT_LOADED', `올라와 있지 않은 플러그인입니다: ${id}`);

    // 진행 중 트랜잭션을 기다린다. 반쯤 적용된 문서를 남기고 내려가지 않는다.
    const deadline = Date.now() + UNLOAD_TX_WAIT_MS;
    while (entry.ledger.activeTx > 0) {
      if (Date.now() > deadline) {
        throw pluginError('TX_TIMEOUT', `진행 중인 트랜잭션이 끝나지 않았습니다: ${id}`);
      }
      await new Promise((resolve) => setTimeout(resolve, UNLOAD_TX_POLL_MS));
    }

    try { entry.plugin.deactivate?.(); }
    catch (error) { console.error(`[PluginHost] ${id} deactivate 실패`, error); }

    this.reclaim(entry.ledger);
    this.loaded.delete(id);
  }

  /** 플러그인 표면 호출. 문서는 플러그인이 트랜잭션 경유로만 바꾼다. */
  invoke(id: string, method: string, args: unknown[]): unknown {
    const entry = this.loaded.get(id);
    if (!entry) throw pluginError('PLUGIN_NOT_LOADED', `올라와 있지 않은 플러그인입니다: ${id}`);
    const fn = entry.surface[method];
    if (typeof fn !== 'function') {
      throw new Error(`${id} 플러그인에 없는 메서드입니다: ${method}`);
    }
    return (fn as (...a: unknown[]) => unknown)(...args);
  }

  /** studio 가 문서를 갈아끼웠을 때 올라와 있는 플러그인 전부에 알린다. */
  notifyDocumentSwap(): void {
    for (const entry of this.loaded.values()) entry.facade.notifySwap();
  }

  /** 모든 플러그인을 내린다. 브리지 destroy 가 부른다. */
  async unloadAll(): Promise<void> {
    for (const id of Array.from(this.loaded.keys())) {
      try { await this.unload(id); }
      catch (error) { console.error(`[PluginHost] ${id} unload 실패`, error); }
    }
  }

  /**
   * 원장에 담긴 것을 **역순으로** 철거한다.
   *
   * 플러그인의 `deactivate()` 성실성에 기대지 않는다 — 기대면 죽은 메뉴 항목이 남는다.
   */
  private reclaim(ledger: PluginLedger): void {
    for (const off of ledger.swapUnsubs.reverse()) {
      try { off(); } catch { /* noop */ }
    }
    for (const off of ledger.eventUnsubs.reverse()) {
      try { off(); } catch { /* noop */ }
    }
    for (const commandId of Array.from(ledger.menuItems)) {
      this.deps.automation.removeMenuItem(commandId);
    }
    for (const commandId of Array.from(ledger.commands)) {
      this.deps.automation.unregisterCommand(commandId);
    }
    ledger.swapUnsubs.length = 0;
    ledger.eventUnsubs.length = 0;
    ledger.menuItems.clear();
    ledger.commands.clear();
  }
}
