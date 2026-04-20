import type { EventBus } from '@/core/event-bus';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { InputHandler } from '@/engine/input-handler';
import type { ViewportManager } from '@/view/viewport-manager';

/** 커맨드 실행 가능 여부 판단용 에디터 상태 스냅샷 */
export interface EditorContext {
  /** 문서가 로드되어 있는가? */
  hasDocument: boolean;
  /** 선택 영역이 있는가? */
  hasSelection: boolean;
  /** 커서가 표 셀 내부인가? */
  inTable: boolean;
  /** F5 셀 선택 모드인가? */
  inCellSelectionMode: boolean;
  /** 표 객체 선택 모드인가? */
  inTableObjectSelection: boolean;
  /** 그림 객체 선택 모드인가? */
  inPictureObjectSelection: boolean;
  /** 커서가 누름틀 필드 내부인가? */
  inField: boolean;
  /** 편집 가능 모드인가? (vs 읽기 전용) */
  isEditable: boolean;
  /** Undo 가능한가? */
  canUndo: boolean;
  /** Redo 가능한가? */
  canRedo: boolean;
  /** 현재 줌 레벨 (0.5 ~ 4.0) */
  zoom: number;
  /** 조판부호 보이기 모드인가? */
  showControlCodes: boolean;
  /** 원본 파일 형식 (#196 — HWPX 출처는 저장 비활성화) */
  sourceFormat?: 'hwp' | 'hwpx';
}

/** 개별 커맨드 정의 */
export interface CommandDef {
  /** 네임스페이스 ID: "카테고리:액션" (예: "edit:copy") */
  readonly id: string;
  /** 표시 레이블 (한국어) */
  readonly label: string;
  /** 단축키 표시 문자열 (예: "Ctrl+C"). 표시 전용 */
  readonly shortcutLabel?: string;
  /** 아이콘 CSS 클래스명 (기존 icon-* 클래스) */
  readonly icon?: string;
  /**
   * 현재 컨텍스트에서 실행 가능한지 판단.
   * 생략 시 항상 활성.
   */
  canExecute?: (ctx: EditorContext) => boolean;
  /** 커맨드 실행 */
  execute: (services: CommandServices, params?: Record<string, unknown>) => void;
}

/** 커맨드 execute()에 주입되는 서비스 */
export interface CommandServices {
  eventBus: EventBus;
  wasm: WasmBridge;
  /** 현재 에디터 상태 스냅샷 */
  getContext: () => EditorContext;
  /** InputHandler 접근 (문서 미로드 시 null) */
  getInputHandler: () => InputHandler | null;
  /** ViewportManager 접근 (문서 미로드 시 null) */
  getViewportManager: () => ViewportManager | null;
}
