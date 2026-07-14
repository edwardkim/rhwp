import type { EventBus } from '@/core/event-bus';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { DocumentDirtyState } from '@/core/document-dirty-state';
import type { InputHandler } from '@/engine/input-handler';
import type { ViewportManager } from '@/view/viewport-manager';

export type EditorEditMode = 'normal' | 'form';

/** 커맨드 실행 가능 여부 판단용 에디터 상태 스냅샷 */
export interface EditorContext {
  /** 문서가 로드되어 있는가? */
  hasDocument: boolean;
  /** 선택 영역이 있는가? */
  hasSelection: boolean;
  /** 모양 복사 상태가 있는가? */
  hasCopiedFormat: boolean;
  /** 커서가 표 셀 내부인가? */
  inTable: boolean;
  /** F5 셀 선택 모드인가? */
  inCellSelectionMode: boolean;
  /** 여러 셀이 선택된 상태인가? */
  hasMultiCellSelection: boolean;
  /** 행/열 바꿈 복사 버퍼가 있는가? */
  hasTableTransposeClipboard: boolean;
  /** 표 객체 선택 모드인가? */
  inTableObjectSelection: boolean;
  /** 그림 객체 선택 모드인가? */
  inPictureObjectSelection: boolean;
  /** 커서가 누름틀 필드 내부인가? */
  inField: boolean;
  /** 편집 가능 모드인가? (vs 읽기 전용) */
  isEditable: boolean;
  /** 현재 편집 모드 */
  editMode: EditorEditMode;
  /** 양식 모드인가? */
  isFormMode: boolean;
  /** 현재 커서 위치가 양식 모드에서 수정 가능한 누름틀인가? */
  canEditFormField: boolean;
  /** Undo 가능한가? */
  canUndo: boolean;
  /** Redo 가능한가? */
  canRedo: boolean;
  /** 현재 줌 레벨 (0.5 ~ 4.0) */
  zoom: number;
  /** 조판부호 보이기 모드인가? */
  showControlCodes: boolean;
  /** 문단부호 보이기 모드인가? */
  showParagraphMarks: boolean;
  /** 저장되지 않은 문서 변경사항이 있는가? */
  isDirty: boolean;
  /** 원본 파일 형식 — 저장 시 출처 포맷 유지(HWPX→HWPX, HWP→HWP). 다른 포맷 저장은 별도 메뉴(#1613). */
  sourceFormat?: 'hwp' | 'hwpx' | 'hml';
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
  /** 저장되지 않은 문서 변경 상태 */
  documentState: DocumentDirtyState;
  /** 현재 에디터 상태 스냅샷 */
  getContext: () => EditorContext;
  /** InputHandler 접근 (문서 미로드 시 null) */
  getInputHandler: () => InputHandler | null;
  /** ViewportManager 접근 (문서 미로드 시 null) */
  getViewportManager: () => ViewportManager | null;
  /** 에디터 편집 모드 변경 */
  setEditMode: (mode: EditorEditMode) => void;
}
