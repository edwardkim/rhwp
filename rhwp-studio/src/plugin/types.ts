/**
 * 플러그인 계약 — studio 위에 기능을 얹었다 내리는 자리.
 *
 * 두 방향의 독립이 이 파일의 존재 이유다.
 *  - 코어는 플러그인을 **정적으로 참조하지 않는다**. 등록은 바깥에서 들어온다.
 *  - 플러그인은 studio 를 **import 하지 않는다**. `PluginHost` 의 모양만 안다.
 *
 * 계획: `mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md` §2.2
 */
import type { HwpDocument } from '@wasm/rhwp.js';
import type { StudioAutomation } from '@/automation/types';

/** 현재 플러그인 API 세대. `activate` 는 이 값을 선언한 플러그인만 받는다. */
export const PLUGIN_API_VERSION = 1 as const;

/**
 * 빌린 문서 핸들.
 *
 * **소유권이 아니라 차용이다** — `handle.free()` 는 소유자(`WasmBridge`)만 부른다.
 * `generation` 은 빌린 시점의 `WasmBridge` 문서 세대다. 문서가 교체·해제되면 세대가 올라가고
 * 이 lease 는 그 순간 무효가 된다.
 *
 * 세대 검사가 필요한 이유: 해제된 문서에 대한 호출이 **예외가 아니라 조용한 오답**으로 돌아오는
 * 경로가 실제로 있다. wasm-bindgen 은 `null pointer passed to rust` 를 던지지만, 방어적
 * optional-chaining 을 거치면 그것이 `0`·빈 문자열로 바뀐다. 그래서 매 호출 진입에서 검사하고
 * 무효면 `DOCUMENT_RELEASED` 로 **던진다**.
 */
export interface DocumentLease {
  readonly handle: HwpDocument;
  readonly generation: number;
}

/** 트랜잭션 안에서만 쥘 수 있는 문서 접근권. */
export interface Tx {
  /** 유효한 문서 핸들. lease 가 무효면 `DOCUMENT_RELEASED` 를 던진다. */
  doc(): HwpDocument;
  /**
   * 이 트랜잭션의 조판을 미룬다. 종료 시 한 번에 flush 한다.
   * 호출당 재조판을 없애는 자리 — 배치 1건이 조판 1회다.
   */
  deferPagination(): void;
}

/**
 * 플러그인이 studio 에 요청할 수 있는 전부.
 *
 * 여기 없는 것은 **의도적으로 없다**. 특히 wasm 모듈 네임스페이스를 주지 않는다 — 주면
 * 플러그인이 `new HwpDocument(...)` 로 문서를 따로 만들 수 있고, 그 순간 studio 와 플러그인이
 * 서로 다른 문서를 만지게 된다. 문서 교체는 `loadDocument`/`createBlankDocument` 위임뿐이다.
 */
export interface PluginHost {
  /** 지금 문서를 빌린다. 문서가 없으면 `null`. */
  borrowDocument(): DocumentLease | null;
  /** 현재 문서 세대. lease 유효성 검사에 쓴다. */
  currentGeneration(): number;

  /**
   * 문서를 바꾸는 **유일한 통로**.
   *
   * studio 의 undo 는 `executeOperation → CommandHistory` 경유가 전제다. 이 밖에서 문서를
   * 만지면 undo 불가·redo 스택 미무효화·스냅샷 복원에 동반 파괴가 함께 온다.
   * 트랜잭션 1건 = undo 1스텝 = 재조판 1회이며, 본문이 던지면 진입 시점으로 롤백된다.
   * 중첩 호출은 `NESTED_TX` 로 거절한다 — 스냅샷이 이중으로 쌓이면 1스텝 계약이 깨진다.
   */
  transaction<T>(label: string, fn: (tx: Tx) => T): T;

  /**
   * 문서를 바꾸지 않는 작업. 히스토리를 건드리지 않아 스냅샷 비용도 undo 항목도 없다.
   *
   * **여기서 문서를 바꾸면 미기록 편집이 되어 undo 계약이 깨진다.** 무엇이 읽기인지는 호출자가
   * 자기 원장으로 판정한다 — studio 는 그것을 강제할 수단이 없고, 대신 두 모드의 산출을 비교하는
   * 동등성 테스트가 오분류를 잡는다.
   */
  read<T>(fn: (doc: HwpDocument) => T): T;

  /**
   * 문서 교체 위임. 원자 교체·세대 증가·이전 문서 해제는 studio 가 한다.
   * 비동기인 이유는 studio 의 열기 경로가 저장 안 된 변경 확인·복구 흐름을 포함하기 때문이다.
   */
  loadDocument(bytes: Uint8Array, fileName?: string): Promise<void>;
  createBlankDocument(): void;

  /** 커맨드·메뉴 조작 표면. */
  readonly automation: StudioAutomation;

  /** 이벤트 구독. 반환값은 해지 함수이며 호스트가 원장에 담아 unload 시 회수한다. */
  events: { on(name: string, cb: (payload?: unknown) => void): () => void };

  /** 문서가 교체되면 새 lease 를 받는다. 반환값은 해지 함수다. */
  onDocumentSwap(cb: (lease: DocumentLease) => void): () => void;
}

/**
 * 플러그인이 노출하는 메서드 집합. RPC 는 이 표면만 호출한다.
 *
 * 반환값은 구조화 복제 가능해야 한다 — 부모 페이지로 건너가기 때문이다.
 * `Uint8Array` 는 transferable 로 넘어간다.
 */
export type PluginSurface = Record<string, (...args: never[]) => unknown>;

export interface StudioPlugin {
  /** 플러그인 식별자. RPC 의 `plugin.load({id})` 와 같은 값이다. */
  readonly id: string;
  readonly apiVersion: typeof PLUGIN_API_VERSION;
  activate(host: PluginHost): PluginSurface | Promise<PluginSurface>;
  /**
   * 정리 훅. **호출되지 않아도 누수가 없어야 한다** — 커맨드·메뉴·리스너 회수는 호스트의
   * 등록 원장이 책임진다(`PluginLedger`). 이 훅은 플러그인 자신의 내부 상태용이다.
   */
  deactivate?(): void;
}

/**
 * 호스트가 플러그인별로 들고 있는 회수 원장.
 *
 * unload 는 여기 담긴 것을 **역순으로** 철거한다. 플러그인의 `deactivate()` 성실성에 기대면
 * 죽은 메뉴 항목이 남는다.
 */
export interface PluginLedger {
  /** `registerCommand` 로 들어온 `ext:` id */
  commands: Set<string>;
  /** `addMenuItem` 이 심은 commandId */
  menuItems: Set<string>;
  /** `events.on` 이 돌려준 해지 함수 */
  eventUnsubs: Array<() => void>;
  /** `onDocumentSwap` 이 돌려준 해지 함수 */
  swapUnsubs: Array<() => void>;
  /** 진행 중인 트랜잭션 수. 0 이 될 때까지 unload 는 기다린다. */
  activeTx: number;
}

/** 플러그인 층이 던지는 오류 코드. RPC 는 이 값을 그대로 실어 보낸다. */
export type PluginErrorCode =
  | 'PLUGIN_NOT_LOADED'
  | 'PLUGIN_NOT_ALLOWED'
  | 'PLUGIN_ACTIVATE_FAILED'
  | 'DOCUMENT_RELEASED'
  | 'NESTED_TX'
  | 'TX_TIMEOUT';

/** 코드를 실어 나르는 오류. `instanceof` 없이 `code` 로 판별할 수 있게 평범한 필드로 둔다. */
export interface PluginError extends Error {
  code: PluginErrorCode;
}

export function isPluginError(value: unknown): value is PluginError {
  return value instanceof Error && typeof (value as PluginError).code === 'string';
}
