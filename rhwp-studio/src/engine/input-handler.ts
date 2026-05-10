import { WasmBridge } from '@/core/wasm-bridge';
import { EventBus } from '@/core/event-bus';
import { CursorState } from './cursor';
import { CaretRenderer } from './caret-renderer';
import { FieldMarkerRenderer } from './field-marker-renderer';
import { SelectionRenderer } from './selection-renderer';
import { CommandHistory } from './history';
import { DeleteSelectionCommand, ApplyCharFormatCommand, SnapshotCommand } from './command';
import type { OperationDescriptor } from './command';
import { VirtualScroll } from '@/view/virtual-scroll';
import { ViewportManager } from '@/view/viewport-manager';
import type { DocumentPosition, CharProperties, ParaProperties, CursorRect, FormObjectHitResult } from '@/core/types';
import type { CommandDispatcher } from '@/command/dispatcher';
import { matchShortcut, defaultShortcuts } from '@/command/shortcut-map';
import type { ContextMenu, ContextMenuItem } from '@/ui/context-menu';
import type { CommandPalette } from '@/ui/command-palette';
import type { CellSelectionRenderer } from './cell-selection-renderer';
import type { TableObjectRenderer } from './table-object-renderer';
import type { TableResizeRenderer, BorderEdge } from './table-resize-renderer';
import type { CellBbox } from '@/core/types';
import * as _mouse from './input-handler-mouse';
import * as _table from './input-handler-table';
import * as _keyboard from './input-handler-keyboard';
import * as _text from './input-handler-text';
import * as _picture from './input-handler-picture';

const SVG_NS = 'http://www.w3.org/2000/svg';
const DRAG_SCROLL_EDGE_PX = 48;
const DRAG_SCROLL_MIN_STEP_PX = 2;
const DRAG_SCROLL_MAX_STEP_PX = 20;

function createOverlaySvg(): SVGSVGElement {
  const svg = document.createElementNS(SVG_NS, 'svg');
  svg.style.width = '100%';
  svg.style.height = '100%';
  svg.style.overflow = 'visible';
  return svg;
}

function setSvgAttrs(el: SVGElement, attrs: Record<string, string | number>): void {
  for (const [key, value] of Object.entries(attrs)) {
    el.setAttribute(key, String(value));
  }
}

function appendOverlayLine(
  svg: SVGSVGElement,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  dashed = false,
): void {
  const line = document.createElementNS(SVG_NS, 'line');
  setSvgAttrs(line, {
    x1,
    y1,
    x2,
    y2,
    stroke: '#333',
    'stroke-width': 2,
  });
  if (dashed) line.setAttribute('stroke-dasharray', '6,3');
  svg.appendChild(line);
}

function createOverlayLabel(x: number, y: number, text: string): HTMLDivElement {
  const label = document.createElement('div');
  label.style.cssText =
    `position:fixed;left:${x}px;top:${y}px;` +
    'background:rgba(0,0,0,0.75);color:#fff;font-size:11px;padding:2px 6px;' +
    'border-radius:3px;white-space:nowrap;pointer-events:none';
  label.textContent = text;
  return label;
}

/** 클릭 커서 배치 + 키보드 입력을 처리한다 */
export class InputHandler {
  private cursor: CursorState;
  private caret: CaretRenderer;
  private fieldMarker: FieldMarkerRenderer;
  private selectionRenderer: SelectionRenderer;
  private history: CommandHistory;
  private textarea: HTMLTextAreaElement;
  private active = false;
  private insertMode = true;  // true=삽입, false=수정(덮어쓰기)
  /** 마지막 셀 키 (눈금자 셀 bbox 중복 조회 방지) */
  private lastCellKey: string | null = null;
  private dispatcher: CommandDispatcher | null = null;
  private contextMenu: ContextMenu | null = null;
  private commandPalette: CommandPalette | null = null;
  private cellSelectionRenderer: CellSelectionRenderer | null = null;
  private tableObjectRenderer: TableObjectRenderer | null = null;
  private tableResizeRenderer: TableResizeRenderer | null = null;
  private pictureObjectRenderer: TableObjectRenderer | null = null;

  // 마우스 드래그 선택 상태
  private isDragging = false;
  private dragRafId = 0; // requestAnimationFrame throttle용
  private dragAutoScrollRafId = 0;
  private dragLastClientX = 0;
  private dragLastClientY = 0;

  // 표 경계선 hover 상태
  private resizeHoverRafId = 0;
  private cachedTableRef: { sec: number; ppi: number; ci: number } | null = null;
  private cachedCellBboxes: CellBbox[] | null = null;

  // 표 경계선 리사이즈 드래그 상태
  private isResizeDragging = false;
  private resizeDragState: {
    edge: BorderEdge;
    tableRef: { sec: number; ppi: number; ci: number };
    bboxes: CellBbox[];
    pageBboxes: CellBbox[];
    affectedCellIndices: number[];
    borderOriginalPos: number;
  } | null = null;

  // 표 이동 드래그 상태
  private isMoveDragging = false;
  private moveDragState: {
    tableRef: { sec: number; ppi: number; ci: number };
    startPpi: number;  // 드래그 시작 시 ppi (Undo용)
    startPageX: number;
    startPageY: number;
    lastPageX: number;
    lastPageY: number;
    totalDeltaH: number;  // 누적 HWPUNIT 델타 (Undo용)
    totalDeltaV: number;
  } | null = null;

  // 그림 삽입 배치 모드 상태
  private imagePlacementMode = false;
  private imagePlacementData: {
    data: Uint8Array; ext: string; fileName: string;
    naturalWidth: number; naturalHeight: number;
  } | null = null;
  private imagePlacementDrag: {
    startClientX: number; startClientY: number;
    currentClientX: number; currentClientY: number;
    isDragging: boolean;
  } | null = null;
  private imagePlacementOverlay: HTMLDivElement | null = null;

  // 도형/글상자 삽입 배치 모드 상태
  private shapePlacementType: string = 'rectangle'; // 'rectangle' | 'ellipse' | 'line'
  private textboxPlacementMode = false;
  private textboxPlacementDrag: {
    startClientX: number; startClientY: number;
    currentClientX: number; currentClientY: number;
    isDragging: boolean;
  } | null = null;
  private textboxPlacementOverlay: HTMLDivElement | null = null;

  // 연결선 드로잉 모드 상태
  private connectorDrawingMode = false;
  private connectorType: string = 'connector-straight';
  private connectorStartRef: { sec: number; ppi: number; ci: number; pointIndex: number; x: number; y: number } | null = null;
  private connectorOverlay: HTMLDivElement | null = null;

  // 다각형 그리기 모드 상태
  private polygonDrawingMode = false;
  private polygonPoints: { x: number; y: number }[] = [];
  private polygonOverlay: HTMLDivElement | null = null;
  private polygonMousePos: { x: number; y: number } | null = null;

  // 그림/글상자 핸들 드래그 리사이즈 상태
  private isPictureResizeDragging = false;
  private pictureResizeState: {
    dir: string;
    ref: { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' };
    origWidth: number;
    origHeight: number;
    origHorzOffset?: number;
    origVertOffset?: number;
    startClientX: number;
    startClientY: number;
    pageIndex: number;
    bbox: { x: number; y: number; w: number; h: number };
    /** 다중 선택 리사이즈 시 각 개체의 원래 크기/위치 */
    multiRefs?: { sec: number; ppi: number; ci: number; type: string; origWidth: number; origHeight: number; origHorzOffset: number; origVertOffset: number; bboxX: number; bboxY: number }[];
  } | null = null;

  // 그림/글상자 이동 드래그 상태
  private isPictureMoveDragging = false;
  private pictureMoveState: {
    ref: { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' };
    origHorzOffset: number;
    origVertOffset: number;
    startPageX: number;
    startPageY: number;
    lastPageX: number;
    lastPageY: number;
    totalDeltaH: number;
    totalDeltaV: number;
    pageIndex: number;
    /** 다중 선택 이동 시 각 개체의 원래 offset 기록 */
    multiRefs?: { sec: number; ppi: number; ci: number; type: string; origHorzOffset: number; origVertOffset: number }[];
  } | null = null;

  // 그림/글상자 회전 드래그 상태
  private isPictureRotateDragging = false;
  private pictureRotateState: {
    ref: { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' };
    origAngle: number;      // 드래그 시작 시 원래 회전각 (도)
    centerX: number;        // 도형 중심 (scroll-content 좌표, px)
    centerY: number;
    startAngle: number;     // 드래그 시작 시 마우스→중심 각도 (rad)
    pageIndex: number;
  } | null = null;

  // 직선 끝점 드래그 상태
  private isLineEndpointDragging = false;
  private lineEndpointState: {
    ref: { sec: number; ppi: number; ci: number; type: string };
    endpoint: 'start' | 'end';
    pageIndex: number;
    pageLeft: number;
    pageOffset: number;
    zoom: number;
  } | null = null;

  // 양식 개체 오버레이
  private formOverlay: HTMLElement | null = null;

  // [Task #394] 셀 진입 자동 ON 로직 비활성화 — checkTransparentBordersTransition 와 동시 주석 처리.
  // 되돌리려면 아래 3 개 변수 + 호출 지점 + 메서드 본체 + 이벤트 핸들러의 주석을 동시에 해제.
  // // 투명선 자동 활성화 상태
  // private wasInCell = false;
  // private manualTransparentBorders = false;
  // private autoTransparentBorders = false;

  // IME 조합 상태
  private isComposing = false;
  private compositionAnchor: DocumentPosition | null = null;
  private compositionLength = 0; // 문서에 삽입된 조합 텍스트 길이
  // iOS 폴백: composition 이벤트 없이 input만으로 한글 조합 처리
  private _iosComposing = false;
  private _iosAnchor: DocumentPosition | null = null;
  private _iosLength = 0;
  private _iosPrevText = '';
  private _iosInputTimer: any = null;
  private _isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) ||
    (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1);

  private onClickBound: (e: MouseEvent) => void;
  private onDblClickBound: (e: MouseEvent) => void;
  private onKeyDownBound: (e: KeyboardEvent) => void;
  private onInputBound: (e?: Event) => void;
  private onCompositionStartBound: () => void;
  private onCompositionEndBound: () => void;
  private onCopyBound: (e: ClipboardEvent) => void;
  private onCutBound: (e: ClipboardEvent) => void;
  private onPasteBound: (e: ClipboardEvent) => void;
  private onContextMenuBound: (e: MouseEvent) => void;
  private onMouseMoveBound: (e: MouseEvent) => void;
  private onMouseUpBound: (e: MouseEvent) => void;
  private onF11InterceptBound: (e: KeyboardEvent) => void;

  constructor(
    private container: HTMLElement,
    private wasm: WasmBridge,
    private eventBus: EventBus,
    private virtualScroll: VirtualScroll,
    private viewportManager: ViewportManager,
  ) {
    this.cursor = new CursorState(wasm);
    this.caret = new CaretRenderer(container, virtualScroll);
    this.fieldMarker = new FieldMarkerRenderer(container, virtualScroll);
    this.selectionRenderer = new SelectionRenderer(container, virtualScroll);
    this.history = new CommandHistory();

    // Hidden input 요소 생성
    // iOS WebKit에서는 <textarea>로 composition 이벤트가 발생하지 않으므로
    // contentEditable <div>를 사용하고 .value 프록시를 추가한다.
    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) ||
      (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1);

    if (isIOS) {
      const div = document.createElement('div');
      div.contentEditable = 'true';
      div.style.cssText =
        'position:absolute;left:0;top:0;width:2em;height:1.5em;' +
        'color:transparent;background:transparent;caret-color:transparent;' +
        'border:none;outline:none;overflow:hidden;white-space:nowrap;' +
        'z-index:10;font-size:16px;padding:0;margin:0;';
      div.setAttribute('autocomplete', 'off');
      div.setAttribute('autocorrect', 'off');
      div.setAttribute('autocapitalize', 'off');
      div.setAttribute('spellcheck', 'false');
      div.setAttribute('inputmode', 'text');
      document.body.appendChild(div);
      // textarea 인터페이스 호환을 위한 프록시
      Object.defineProperty(div, 'value', {
        get() { return div.textContent || ''; },
        set(v: string) { div.textContent = v; },
      });
      this.textarea = div as unknown as HTMLTextAreaElement;
    } else {
      this.textarea = document.createElement('textarea');
      this.textarea.style.cssText =
        'position:fixed;left:-9999px;top:0;width:1px;height:1px;opacity:0;';
      this.textarea.setAttribute('autocomplete', 'off');
      this.textarea.setAttribute('autocorrect', 'off');
      this.textarea.setAttribute('autocapitalize', 'off');
      this.textarea.setAttribute('spellcheck', 'false');
      document.body.appendChild(this.textarea);
    }

    this.onClickBound = this.onClick.bind(this);
    this.onDblClickBound = this.onDblClick.bind(this);
    this.onKeyDownBound = this.onKeyDown.bind(this);
    this.onInputBound = this.onInput.bind(this);
    this.onCompositionStartBound = this.onCompositionStart.bind(this);
    this.onCompositionEndBound = this.onCompositionEnd.bind(this);
    this.onCopyBound = this.onCopy.bind(this);
    this.onCutBound = this.onCut.bind(this);
    this.onPasteBound = this.onPaste.bind(this);
    this.onContextMenuBound = this.onContextMenu.bind(this);
    this.onMouseMoveBound = this.onMouseMove.bind(this);
    this.onMouseUpBound = this.onMouseUp.bind(this);

    // F11 브라우저 fullscreen 방지 (capture 단계에서 차단) + 컨트롤 선택 실행
    this.onF11InterceptBound = (e: KeyboardEvent) => {
      if (e.key === 'F11') {
        e.preventDefault();
        e.stopPropagation();
        if (e.shiftKey) {
          _keyboard.handleShiftF11.call(this);
        } else {
          _keyboard.handleF11.call(this);
        }
      }
    };
    document.addEventListener('keydown', this.onF11InterceptBound, true);

    container.addEventListener('mousedown', this.onClickBound);
    container.addEventListener('dblclick', this.onDblClickBound);
    container.addEventListener('contextmenu', this.onContextMenuBound);
    container.addEventListener('mousemove', this.onMouseMoveBound);
    this.textarea.addEventListener('keydown', this.onKeyDownBound);
    this.textarea.addEventListener('input', this.onInputBound);
    this.textarea.addEventListener('compositionstart', this.onCompositionStartBound);
    this.textarea.addEventListener('compositionend', this.onCompositionEndBound);
    this.textarea.addEventListener('copy', this.onCopyBound);
    this.textarea.addEventListener('cut', this.onCutBound);
    this.textarea.addEventListener('paste', this.onPasteBound);

    // 줌 변경 시 캐럿/선택 마커 위치 갱신
    eventBus.on('zoom-changed', () => {
      if (this.active) {
        const rect = this.cursor.getRect();
        if (rect) {
          this.caret.updatePosition(this.viewportManager.getZoom());
        }
        // 필드 마커도 줌에 맞게 갱신
        if (this.fieldMarker.isVisible) {
          this.updateFieldMarkers();
        }
      }
      // 텍스트 블럭 선택 줌 동기화
      if (this.cursor.hasSelection()) {
        this.updateSelection();
      }
      // F5 셀 선택 줌 동기화
      if (this.cursor.isInCellSelectionMode()) {
        this.updateCellSelection();
      }
      // 도형/표 선택 핸들 줌 동기화
      if (this.cursor.isInPictureObjectSelection()) {
        this.renderPictureObjectSelection();
      }
      if (this.cursor.isInTableObjectSelection()) {
        this.renderTableObjectSelection();
      }
    });

    // 표 객체 선택 변경 시 렌더링
    eventBus.on('table-object-selection-changed', (selected) => {
      if (selected) {
        this.renderTableObjectSelection();
      } else {
        this.tableObjectRenderer?.clear();
      }
    });

    // 문서 변경 후 그림/표 선택 마커 재렌더링
    eventBus.on('document-changed', () => {
      requestAnimationFrame(() => {
        if (this.cursor.isInPictureObjectSelection()) {
          this.renderPictureObjectSelection();
        }
        if (this.cursor.isInTableObjectSelection()) {
          this.renderTableObjectSelection();
        }
      });
    });

    // [Task #394] 셀 진입 자동 ON 로직 비활성화 — manual 추적 불필요.
    // transparent-borders-changed 이벤트 자체는 view.ts 에서 emit 되므로 보존됨 (다른 구독자가 사용 가능).
    // // 투명선 수동 토글 상태 추적
    // eventBus.on('transparent-borders-changed', (show) => {
    //   this.manualTransparentBorders = show as boolean;
    // });

    // Toolbar에서 서식 적용 요청 수신 (글꼴명, 크기, 색상 — 커맨드 시스템 미경유)
    eventBus.on('format-char', (props) => {
      if (!this.active) return;
      if (this.cursor.hasSelection()) {
        this.applyCharFormat(props as Partial<CharProperties>);
      }
      // 서식바 조작으로 빠진 포커스를 항상 복원
      this.focusTextarea();
    });
  }

  /** 클릭 이벤트 처리 — hitTest로 커서 배치 */
  private onClick(e: MouseEvent): void {
    _mouse.onClick.call(this, e);
  }

  /** 우클릭 컨텍스트 메뉴 처리 */
  private onContextMenu(e: MouseEvent): void {
    _mouse.onContextMenu.call(this, e);
  }

  /** 더블클릭: 글상자 객체 선택 → 텍스트 편집 진입 */
  private onDblClick(e: MouseEvent): void {
    _mouse.onDblClick.call(this, e);
  }

  /** 마우스 이동: 드래그 선택 또는 표 객체 선택 중 핸들 위 커서 변경 */
  private onMouseMove(e: MouseEvent): void {
    _mouse.onMouseMove.call(this, e);
  }

  /** 표 경계선 hover 감지 처리 */
  private handleResizeHover(e: MouseEvent): void {
    _mouse.handleResizeHover.call(this, e);
  }

  /** 리사이즈 드래그를 시작한다 */
  private startResizeDrag(
    edge: BorderEdge,
    pageX: number, pageY: number,
    pageBboxes: CellBbox[],
  ): void {
    _table.startResizeDrag.call(this, edge, pageX, pageY, pageBboxes);
  }

  /** 리사이즈 드래그 중 마커 위치를 갱신한다 */
  private updateResizeDrag(e: MouseEvent): void {
    _table.updateResizeDrag.call(this, e);
  }

  /** 리사이즈 드래그를 완료하고 셀 크기를 적용한다 */
  private finishResizeDrag(e: MouseEvent): void {
    _table.finishResizeDrag.call(this, e);
  }

  /** 리사이즈 드래그 상태를 초기화한다 */
  private cleanupResizeDrag(): void {
    _table.cleanupResizeDrag.call(this);
  }

  // ─── 격자 이동 크기 (mm) ───────────────────────────────
  private gridStepMm = 3; // 기본 3mm

  /** 격자 간격 설정 (mm 단위) */
  setGridStep(mm: number): void { this.gridStepMm = mm; }

  /** 현재 격자 간격 반환 (mm 단위) */
  getGridStepMm(): number { return this.gridStepMm; }

  // ─── 그림 삽입 배치 모드 ───────────────────────────────

  /** 그림 배치 모드 진입: 파일 선택 후 호출. 마우스로 영역 지정 대기 */
  enterImagePlacementMode(data: Uint8Array, ext: string, naturalWidth: number, naturalHeight: number, fileName: string = ''): void {
    this.imagePlacementMode = true;
    this.imagePlacementData = { data, ext, fileName, naturalWidth, naturalHeight };
    this.imagePlacementDrag = null;
    this.container.style.cursor = 'crosshair';
  }

  /** 그림 배치 모드 취소 */
  private cancelImagePlacement(): void {
    _table.cancelImagePlacement.call(this);
  }

  /** 그림 배치 사각형 오버레이 표시/갱신 */
  private showImagePlacementOverlay(x1: number, y1: number, x2: number, y2: number): void {
    _table.showImagePlacementOverlay.call(this, x1, y1, x2, y2);
  }

  /** 그림 배치 오버레이 제거 */
  private hideImagePlacementOverlay(): void {
    _table.hideImagePlacementOverlay.call(this);
  }

  /** 그림 배치 완료: 마우스업 시 호출 */
  private finishImagePlacement(e: MouseEvent): void {
    _table.finishImagePlacement.call(this, e);
  }

  // ─── 글상자 삽입 배치 모드 ───────────────────────────────

  /** 글상자 배치 모드 진입: 메뉴에서 호출. 마우스로 영역 지정 대기 */
  enterTextboxPlacementMode(): void {
    this.shapePlacementType = 'rectangle';
    this.textboxPlacementMode = true;
    this.textboxPlacementDrag = null;
    this.container.style.cursor = 'crosshair';
  }

  /** 도형 배치 모드 진입 (도형 타입 지정) */
  enterShapePlacementMode(shapeType: string): void {
    this.shapePlacementType = shapeType;
    if (shapeType.startsWith('connector-')) {
      // 연결선: 개체 연결점 클릭→드래그→연결점 모드
      this.connectorDrawingMode = true;
      this.connectorType = shapeType;
      this.connectorStartRef = null;
      this.container.style.cursor = 'crosshair';
    } else if (shapeType === 'polygon') {
      // 다각형: 클릭-클릭-더블클릭 모드
      this.polygonDrawingMode = true;
      this.polygonPoints = [];
      this.polygonMousePos = null;
      this.container.style.cursor = 'crosshair';
    } else {
      this.textboxPlacementMode = true;
      this.textboxPlacementDrag = null;
      this.container.style.cursor = 'crosshair';
    }
  }

  /** 다각형 그리기: 꼭짓점 추가 (클릭) */
  private polygonAddPoint(clientX: number, clientY: number): void {
    this.polygonPoints.push({ x: clientX, y: clientY });
    this.updatePolygonOverlay(clientX, clientY);
  }

  /** 다각형 그리기: 마우스 이동 시 프리뷰 갱신 */
  private updatePolygonOverlay(mx: number, my: number): void {
    this.polygonMousePos = { x: mx, y: my };
    if (!this.polygonOverlay) {
      this.polygonOverlay = document.createElement('div');
      this.polygonOverlay.style.cssText =
        'position:fixed;left:0;top:0;width:100vw;height:100vh;pointer-events:none;z-index:9999;';
      document.body.appendChild(this.polygonOverlay);
    }
    const pts = this.polygonPoints;
    if (pts.length === 0) {
      this.polygonOverlay.replaceChildren();
      return;
    }

    const svg = createOverlaySvg();
    // 확정된 변
    for (let i = 0; i < pts.length - 1; i++) {
      appendOverlayLine(svg, pts[i].x, pts[i].y, pts[i + 1].x, pts[i + 1].y);
    }
    // 마지막 점 → 마우스 위치 (프리뷰)
    const last = pts[pts.length - 1];
    appendOverlayLine(svg, last.x, last.y, mx, my, true);
    // 꼭짓점 마커
    for (const p of pts) {
      const circle = document.createElementNS(SVG_NS, 'circle');
      setSvgAttrs(circle, {
        cx: p.x,
        cy: p.y,
        r: 3,
        fill: '#fff',
        stroke: '#333',
        'stroke-width': 1,
      });
      svg.appendChild(circle);
    }
    // 크기 표시
    const allX = [...pts.map(p => p.x), mx];
    const allY = [...pts.map(p => p.y), my];
    const minX = Math.min(...allX), maxX = Math.max(...allX);
    const minY = Math.min(...allY), maxY = Math.max(...allY);
    const zoom = this.viewportManager.getZoom();
    const wMm = ((maxX - minX) / zoom * 25.4 / 96).toFixed(1);
    const hMm = ((maxY - minY) / zoom * 25.4 / 96).toFixed(1);
    const sizeLabel = createOverlayLabel(maxX + 4, maxY + 4, `${wMm} × ${hMm} mm`);

    this.polygonOverlay.replaceChildren(svg, sizeLabel);
  }

  /** 다각형 그리기: 완료 (더블클릭 또는 시작점 근접) */
  private finishPolygonDrawing(): void {
    const pts = this.polygonPoints;
    if (pts.length < 2) { this.cancelPolygonDrawing(); return; }

    // 화면 좌표 → 종이 좌표 (HWPUNIT)
    const zoom = this.viewportManager.getZoom();
    const scrollContent = this.container.querySelector('#scroll-content');
    const contentRect = scrollContent?.getBoundingClientRect();
    if (!contentRect) { this.cancelPolygonDrawing(); return; }

    // bbox 계산
    const xs = pts.map(p => p.x), ys = pts.map(p => p.y);
    const minX = Math.min(...xs), minY = Math.min(...ys);
    const maxX = Math.max(...xs), maxY = Math.max(...ys);
    const wPx = (maxX - minX) / zoom;
    const hPx = (maxY - minY) / zoom;
    const wHwp = Math.round(wPx * 75);
    const hHwp = Math.round(hPx * 75);

    // 종이 좌표로 오프셋 계산
    const centerX = (minX + maxX) / 2;
    const centerY = (minY + maxY) / 2;
    const cX = centerX - contentRect.left;
    const cY = centerY - contentRect.top;
    const pageIdx = this.virtualScroll.getPageAtPoint(cX, cY);
    const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
    const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, (scrollContent as HTMLElement).clientWidth);
    const paperX = ((cX - pageLeft) / zoom) * 75;
    const paperY = ((cY - pageOffset) / zoom) * 75;
    const horzOffset = Math.max(0, Math.round(paperX - wHwp / 2));
    const vertOffset = Math.max(0, Math.round(paperY - hHwp / 2));

    // 꼭짓점을 HWPUNIT 로컬 좌표로 변환 (bbox 기준)
    const pointsHwp = pts.map(p => ({
      x: Math.round(((p.x - minX) / zoom) * 75),
      y: Math.round(((p.y - minY) / zoom) * 75),
    }));

    // 커서 위치
    const cursorPos = this.cursor.getPosition();
    const sec = cursorPos.sectionIndex;
    const paraIdx = cursorPos.paragraphIndex;
    const charOffset = cursorPos.charOffset;

    try {
      const result = this.wasm.createShapeControl({
        sectionIdx: sec,
        paraIdx,
        charOffset,
        width: wHwp || 2250,
        height: hHwp || 2250,
        horzOffset,
        vertOffset,
        shapeType: 'polygon',
        polygonPoints: pointsHwp,
      });
      if (result.ok) {
        this.eventBus.emit('document-changed');
        this.cursor.enterPictureObjectSelectionDirect(sec, result.paraIdx, result.controlIdx, 'shape');
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderPictureObjectSelection();
        this.eventBus.emit('picture-object-selection-changed', true);
      }
    } catch (err) {
      console.warn('[InputHandler] 다각형 삽입 실패:', err);
    }

    this.cancelPolygonDrawing();
  }

  /** 다각형 그리기: 취소 */
  private cancelPolygonDrawing(): void {
    this.polygonDrawingMode = false;
    this.polygonPoints = [];
    this.polygonMousePos = null;
    if (this.polygonOverlay) {
      this.polygonOverlay.remove();
      this.polygonOverlay = null;
    }
    this.container.style.cursor = '';
  }

  /** 글상자 배치 모드 취소 */
  private cancelTextboxPlacement(): void {
    this.textboxPlacementMode = false;
    this.textboxPlacementDrag = null;
    this.hideTextboxPlacementOverlay();
    this.container.style.cursor = '';
  }

  /** 도형 배치 오버레이 표시/갱신 (도형 타입별 SVG) */
  private showTextboxPlacementOverlay(x1: number, y1: number, x2: number, y2: number, shiftKey = false): void {
    if (!this.textboxPlacementOverlay) {
      this.textboxPlacementOverlay = document.createElement('div');
      this.textboxPlacementOverlay.style.cssText =
        'position:fixed;left:0;top:0;width:100vw;height:100vh;pointer-events:none;z-index:9999;';
      document.body.appendChild(this.textboxPlacementOverlay);
    }
    const type = this.shapePlacementType;

    const zoom = this.viewportManager.getZoom();
    const left = Math.min(x1, x2);
    const top = Math.min(y1, y2);
    const w = Math.abs(x2 - x1);
    const h = Math.abs(y2 - y1);
    // mm 크기 계산 (96dpi 기준: 1px = 25.4/96 mm)
    const wMm = (w / zoom * 25.4 / 96).toFixed(1);
    const hMm = (h / zoom * 25.4 / 96).toFixed(1);
    const sizeLabel = createOverlayLabel(left + w + 4, top + h + 4, `${wMm} × ${hMm} mm`);

    const svg = createOverlaySvg();
    let customLabel: HTMLDivElement | null = null;
    if (type === 'line') {
      let ex = x2, ey = y2;
      if (shiftKey) {
        const dx = x2 - x1, dy = y2 - y1;
        const angle = Math.atan2(dy, dx);
        const snapAngle = Math.round(angle / (Math.PI / 4)) * (Math.PI / 4);
        const dist = Math.sqrt(dx * dx + dy * dy);
        ex = x1 + dist * Math.cos(snapAngle);
        ey = y1 + dist * Math.sin(snapAngle);
      }
      if (this.textboxPlacementDrag && shiftKey) {
        this.textboxPlacementDrag.currentClientX = ex;
        this.textboxPlacementDrag.currentClientY = ey;
      }
      appendOverlayLine(svg, x1, y1, ex, ey, true);
      // 직선: 길이 표시
      const lenPx = Math.hypot(ex - x1, ey - y1);
      const lenMm = (lenPx / zoom * 25.4 / 96).toFixed(1);
      const mx = (x1 + ex) / 2, my = (y1 + ey) / 2;
      customLabel = createOverlayLabel(mx + 8, my + 8, `${lenMm} mm`);
    } else if (type === 'ellipse') {
      const cx = left + w / 2, cy = top + h / 2;
      const ellipse = document.createElementNS(SVG_NS, 'ellipse');
      setSvgAttrs(ellipse, {
        cx,
        cy,
        rx: w / 2,
        ry: h / 2,
        fill: 'rgba(0,0,0,0.05)',
        stroke: '#333',
        'stroke-width': 2,
        'stroke-dasharray': '6,3',
      });
      svg.appendChild(ellipse);
    } else if (type === 'arc') {
      // 호: 사각형에 내접하는 타원의 1/4 호
      // 우상 사분면: 상단 중앙 → 우측 중앙
      const rx = w / 2, ry = h / 2;
      if (rx > 1 && ry > 1) {
        const cx = left + w / 2, cy = top + h / 2;
        // 시작: 상단 중앙 (cx, top), 끝: 우측 중앙 (left+w, cy)
        const path = document.createElementNS(SVG_NS, 'path');
        setSvgAttrs(path, {
          d: `M ${cx} ${top} A ${rx} ${ry} 0 0 1 ${left + w} ${cy}`,
          fill: 'none',
          stroke: '#333',
          'stroke-width': 2,
          'stroke-dasharray': '6,3',
        });
        svg.appendChild(path);
        // 보조선: 내접 사각형
        const guide = document.createElementNS(SVG_NS, 'rect');
        setSvgAttrs(guide, {
          x: left,
          y: top,
          width: w,
          height: h,
          fill: 'none',
          stroke: '#ccc',
          'stroke-width': 1,
          'stroke-dasharray': '3,3',
        });
        svg.appendChild(guide);
      }
    } else if (type === 'polygon') {
      // 다각형: 삼각형 프리뷰
      const tx = left + w / 2, ty = top;
      const polygon = document.createElementNS(SVG_NS, 'polygon');
      setSvgAttrs(polygon, {
        points: `${tx},${ty} ${left + w},${top + h} ${left},${top + h}`,
        fill: 'rgba(0,0,0,0.05)',
        stroke: '#333',
        'stroke-width': 2,
        'stroke-dasharray': '6,3',
      });
      svg.appendChild(polygon);
    } else {
      // rectangle / textbox
      const rect = document.createElementNS(SVG_NS, 'rect');
      setSvgAttrs(rect, {
        x: left,
        y: top,
        width: w,
        height: h,
        fill: 'rgba(0,0,0,0.05)',
        stroke: '#333',
        'stroke-width': 2,
        'stroke-dasharray': '6,3',
      });
      svg.appendChild(rect);
    }

    const label = customLabel || (w > 5 || h > 5 ? sizeLabel : null);
    this.textboxPlacementOverlay.replaceChildren(...(label ? [svg, label] : [svg]));
  }

  /** 도형 배치 오버레이 제거 */
  private hideTextboxPlacementOverlay(): void {
    if (this.textboxPlacementOverlay) {
      this.textboxPlacementOverlay.remove();
      this.textboxPlacementOverlay = null;
    }
  }

  /** 글상자 배치 완료: 마우스업 시 호출 */
  private finishTextboxPlacement(e: MouseEvent): void {
    const drag = this.textboxPlacementDrag;
    if (!drag) { this.cancelTextboxPlacement(); return; }

    this.hideTextboxPlacementOverlay();

    // 커서 위치에 도형 컨트롤 삽입 (한컴 동작: 커서 위치에 인라인 컨트롤 배치)
    const cursorPos = this.cursor.getPosition();
    const hit = {
      sectionIndex: cursorPos.sectionIndex,
      paragraphIndex: cursorPos.paragraphIndex,
      charOffset: cursorPos.charOffset,
    };
    if (hit.sectionIndex === undefined) { this.cancelTextboxPlacement(); return; }

    const sec = hit.sectionIndex;
    const paraIdx = hit.paragraphIndex;
    const charOffset = hit.charOffset;

    // 크기 결정
    const zoom = this.viewportManager.getZoom();
    let wPx: number, hPx: number;
    if (drag.isDragging) {
      wPx = Math.abs(drag.currentClientX - drag.startClientX) / zoom;
      hPx = Math.abs(drag.currentClientY - drag.startClientY) / zoom;
      const isLineType = this.shapePlacementType === 'line' || this.shapePlacementType.startsWith('connector-');
      if (!isLineType) {
        if (wPx < 10) wPx = 10;
        if (hPx < 10) hPx = 10;
      }
    } else {
      // 클릭만 한 경우
      const mm30 = 30 * 96 / 25.4; // ≈113.4 px
      if (this.shapePlacementType === 'line' || this.shapePlacementType.startsWith('connector-')) {
        wPx = mm30; hPx = 0; // 수평 직선/연결선
      } else {
        wPx = mm30; hPx = mm30;
      }
    }

    // px → HWPUNIT (1px = 75 HWPUNIT at 96 DPI)
    let wHwp = Math.round(wPx * 75);
    let hHwp = Math.round(hPx * 75);

    // 열 폭 초과 시 비례 축소
    try {
      const pageDef = this.wasm.getPageDef(sec);
      const colWidth = pageDef.width - pageDef.marginLeft - pageDef.marginRight;
      if (wHwp > colWidth) {
        const ratio = colWidth / wHwp;
        wHwp = Math.round(colWidth);
        hHwp = Math.round(hHwp * ratio);
      }
    } catch { /* 페이지 정보 없으면 그대로 */ }

    // 도형 위치 계산 (종이 기준 오프셋, HWPUNIT)
    let horzOffset = 0;
    let vertOffset = 0;
    if (this.shapePlacementType !== 'textbox') {
      // 드래그 영역 중심점의 화면 좌표
      const centerX = (drag.startClientX + drag.currentClientX) / 2;
      const centerY = (drag.startClientY + drag.currentClientY) / 2;
      // 화면 좌표 → 종이 좌표 (px, 줌 보정 전)
      const scrollContent = this.container.querySelector('#scroll-content');
      if (scrollContent) {
        const contentRect = scrollContent.getBoundingClientRect();
        const cX = centerX - contentRect.left;
        const cY = centerY - contentRect.top;
        const pageIdx = this.virtualScroll.getPageAtPoint(cX, cY);
        const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
        const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
        const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
        // 종이 좌표 (px → HWPUNIT)
        const paperX = ((cX - pageLeft) / zoom) * 75;
        const paperY = ((cY - pageOffset) / zoom) * 75;
        // 도형 좌상단 = 중심점 - 반폭/반높이
        horzOffset = Math.max(0, Math.round(paperX - wHwp / 2));
        vertOffset = Math.max(0, Math.round(paperY - hHwp / 2));
      }
    }

    // 직선 방향 결정: 드래그 시작→끝의 X/Y 방향
    let lineFlipX = false;
    let lineFlipY = false;
    if ((this.shapePlacementType === 'line' || this.shapePlacementType.startsWith('connector-')) && drag.isDragging) {
      lineFlipX = drag.currentClientX < drag.startClientX;
      lineFlipY = drag.currentClientY < drag.startClientY;
    }

    // WASM 호출로 도형 생성
    try {
      const result = this.wasm.createShapeControl({
        sectionIdx: sec,
        paraIdx,
        charOffset,
        width: wHwp,
        height: hHwp,
        horzOffset,
        vertOffset,
        shapeType: this.shapePlacementType,
        lineFlipX,
        lineFlipY,
      });
      if (result.ok) {
        this.eventBus.emit('document-changed');
        // 생성된 도형을 선택 상태로 진입
        const selType = (this.shapePlacementType === 'line' || this.shapePlacementType.startsWith('connector-')) ? 'line' : 'shape';
        this.cursor.enterPictureObjectSelectionDirect(sec, result.paraIdx, result.controlIdx, selType);
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderPictureObjectSelection();
        this.eventBus.emit('picture-object-selection-changed', true);
      }
    } catch (err) {
      console.warn('[InputHandler] 글상자 삽입 실패:', err);
    }

    // 모드 종료
    this.textboxPlacementMode = false;
    this.textboxPlacementDrag = null;
    this.container.style.cursor = '';
  }

  /** 표 객체 선택 모드에서 방향키로 표 위치 이동 */
  private moveSelectedTable(key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
    _table.moveSelectedTable.call(this, key);
  }

  /** 그림 객체 선택 모드에서 방향키로 그림 위치 이동 */
  private moveSelectedPicture(key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
    _table.moveSelectedPicture.call(this, key);
  }

  /** 마우스 드래그로 표 이동 — 드래그 중 갱신 */
  private updateMoveDrag(e: MouseEvent): void {
    _table.updateMoveDrag.call(this, e);
  }

  /** 마우스 드래그로 표 이동 — 드래그 종료 */
  private finishMoveDrag(): void {
    _table.finishMoveDrag.call(this);
  }

  /** 셀 선택 모드에서 Ctrl+방향키로 셀 크기 조절 */
  private resizeCellByKeyboard(key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
    _table.resizeCellByKeyboard.call(this, key);
  }

  private resizeTableProportional(key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
    _table.resizeTableProportional.call(this, key);
  }

  /** 마우스 버튼 놓기: 드래그 선택 종료 */
  private onMouseUp(_e: MouseEvent): void {
    _mouse.onMouseUp.call(this, _e);
  }

  /** 마우스 이벤트에서 hitTest 결과를 반환한다 */
  private hitTestFromEvent(e: MouseEvent): DocumentPosition | null {
    return this.hitTestFromClientPoint(e.clientX, e.clientY);
  }

  /** 화면 좌표에서 hitTest 결과를 반환한다 */
  private hitTestFromClientPoint(clientX: number, clientY: number): DocumentPosition | null {
    const zoom = this.viewportManager.getZoom();
    const scrollContent = this.container.querySelector('#scroll-content');
    if (!scrollContent) return null;
    const contentRect = scrollContent.getBoundingClientRect();
    // [Task #661 + #685+#689 통합] PR #718 영역 의 clientX/Y parameter 영역 +
    // PR #693 영역 의 getPageAtPoint (그리드 모드 click 좌표 정합) 보존.
    const contentX = clientX - contentRect.left;
    const contentY = clientY - contentRect.top;
    const pageIdx = this.virtualScroll.getPageAtPoint(contentX, contentY);
    const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
    const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
    const pageX = (contentX - pageLeft) / zoom;
    const pageY = (contentY - pageOffset) / zoom;
    try {
      return this.wasm.hitTest(pageIdx, pageX, pageY);
    } catch {
      return null;
    }
  }

  /** 텍스트 선택 드래그를 시작한다 */
  private startTextSelectionDrag(e: MouseEvent): void {
    this.isDragging = true;
    this.dragLastClientX = e.clientX;
    this.dragLastClientY = e.clientY;
    document.addEventListener('mousemove', this.onMouseMoveBound);
  }

  /** 텍스트 선택 드래그 포인터 좌표를 갱신한다 */
  private updateTextSelectionDragPointer(e: MouseEvent): void {
    this.dragLastClientX = e.clientX;
    this.dragLastClientY = e.clientY;
    this.updateTextSelectionDragAutoScroll();
  }

  /** 마지막 포인터 좌표 기준으로 드래그 선택 focus를 갱신한다 */
  private updateTextSelectionDragFromPointer(): void {
    if (!this.isDragging) return;

    const hit = this.hitTestFromClientPoint(this.dragLastClientX, this.dragLastClientY);
    if (hit && hit.paragraphIndex < 0xFFFFFF00) {
      this.cursor.moveTo(hit);
      this.updateCaretDuringDrag();
    }
  }

  /** 텍스트 선택 드래그를 종료한다 */
  private stopTextSelectionDrag(): void {
    this.isDragging = false;
    document.removeEventListener('mousemove', this.onMouseMoveBound);
    this.stopTextSelectionDragAutoScroll();
  }

  private getTextSelectionDragScrollDeltaY(): number {
    const rect = this.container.getBoundingClientRect();
    const topEdge = rect.top + DRAG_SCROLL_EDGE_PX;
    const bottomEdge = rect.top + this.container.clientHeight - DRAG_SCROLL_EDGE_PX;
    const clientY = this.dragLastClientY;

    if (clientY < topEdge) {
      return -this.scaleTextSelectionDragScrollStep(topEdge - clientY);
    }
    if (clientY > bottomEdge) {
      return this.scaleTextSelectionDragScrollStep(clientY - bottomEdge);
    }
    return 0;
  }

  private scaleTextSelectionDragScrollStep(distance: number): number {
    const ratio = Math.min(1, Math.max(0, distance / DRAG_SCROLL_EDGE_PX));
    return Math.round(DRAG_SCROLL_MIN_STEP_PX + (DRAG_SCROLL_MAX_STEP_PX - DRAG_SCROLL_MIN_STEP_PX) * ratio);
  }

  private updateTextSelectionDragAutoScroll(): void {
    if (!this.isDragging) {
      this.stopTextSelectionDragAutoScroll();
      return;
    }
    if (this.getTextSelectionDragScrollDeltaY() === 0) {
      this.stopTextSelectionDragAutoScroll();
      return;
    }
    if (!this.dragAutoScrollRafId) {
      this.dragAutoScrollRafId = requestAnimationFrame(() => this.runTextSelectionDragAutoScroll());
    }
  }

  private runTextSelectionDragAutoScroll(): void {
    this.dragAutoScrollRafId = 0;
    if (!this.isDragging) return;

    const deltaY = this.getTextSelectionDragScrollDeltaY();
    if (deltaY === 0) return;

    const before = this.container.scrollTop;
    const maxScrollTop = Math.max(0, this.container.scrollHeight - this.container.clientHeight);
    this.container.scrollTop = Math.max(0, Math.min(maxScrollTop, before + deltaY));

    if (this.container.scrollTop === before) return;

    this.updateTextSelectionDragFromPointer();
    this.dragAutoScrollRafId = requestAnimationFrame(() => this.runTextSelectionDragAutoScroll());
  }

  private stopTextSelectionDragAutoScroll(): void {
    if (this.dragAutoScrollRafId) {
      cancelAnimationFrame(this.dragAutoScrollRafId);
      this.dragAutoScrollRafId = 0;
    }
  }

  /** 클릭 좌표가 표 외곽 경계선 위인지 판별한다 (페이지 좌표 기준) */
  private isTableBorderClick(
    pageX: number, pageY: number,
    sec: number, ppi: number, ci: number,
  ): boolean {
    try {
      const bbox = this.wasm.getTableBBox(sec, ppi, ci);
      const tolerance = 5; // 페이지 좌표 기준 px
      const nearLeft = Math.abs(pageX - bbox.x) <= tolerance;
      const nearRight = Math.abs(pageX - (bbox.x + bbox.width)) <= tolerance;
      const nearTop = Math.abs(pageY - bbox.y) <= tolerance;
      const nearBottom = Math.abs(pageY - (bbox.y + bbox.height)) <= tolerance;
      // 세로 범위 내 좌/우 경계, 가로 범위 내 상/하 경계
      const inVertRange = pageY >= bbox.y - tolerance && pageY <= bbox.y + bbox.height + tolerance;
      const inHorzRange = pageX >= bbox.x - tolerance && pageX <= bbox.x + bbox.width + tolerance;
      return (nearLeft && inVertRange) || (nearRight && inVertRange) ||
             (nearTop && inHorzRange) || (nearBottom && inHorzRange);
    } catch {
      return false;
    }
  }

  /**
   * 클릭 좌표 근처에 표가 있는지 확인한다 (표 바깥에서 클릭한 경우).
   * hitTest 결과의 문단과 인접 문단을 검사하여 표 외곽 근처인지 판별한다.
   */
  private findTableByOuterClick(
    pageX: number, pageY: number,
    sec: number, paragraphIndex: number,
  ): { sec: number; ppi: number; ci: number } | null {
    // 현재 문단 및 인접 문단 (±2) 검사
    for (let offset = 0; offset <= 2; offset++) {
      const candidates = offset === 0
        ? [paragraphIndex]
        : [paragraphIndex - offset, paragraphIndex + offset];
      for (const ppi of candidates) {
        if (ppi < 0) continue;
        if (this.isTableBorderClick(pageX, pageY, sec, ppi, 0)) {
          return { sec, ppi, ci: 0 };
        }
      }
    }
    return null;
  }

  /** 표 객체 선택 상태 컨텍스트 메뉴 항목 */
  private getTableObjectContextMenuItems(): ContextMenuItem[] {
    return [
      { type: 'command', commandId: 'edit:cut' },
      { type: 'command', commandId: 'edit:copy' },
      { type: 'command', commandId: 'edit:paste' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:caption-toggle', label: '캡션 넣기(A)' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:cell-props', label: '표 속성...' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:delete' },
    ];
  }

  /** 그림 객체 선택 컨텍스트 메뉴 항목 */
  private getPictureObjectContextMenuItems(): ContextMenuItem[] {
    const ref = this.cursor.getSelectedPictureRef();

    // 다중 선택: 개체 묶기 메뉴
    if (this.cursor.isMultiPictureSelection()) {
      return [
        { type: 'command', commandId: 'insert:group-shapes', label: '개체 묶기(G)' },
        { type: 'separator' },
        { type: 'command', commandId: 'insert:picture-delete', label: '지우기(D)' },
      ];
    }

    const items: ContextMenuItem[] = [
      { type: 'command', commandId: 'edit:cut' },
      { type: 'command', commandId: 'edit:copy' },
      { type: 'command', commandId: 'edit:paste' },
      { type: 'separator' },
    ];
    // 수식 객체: "수식 편집..." 항목 추가
    if (ref?.type === 'equation') {
      items.push(
        { type: 'command', commandId: 'insert:equation-edit', label: '수식 편집...' },
        { type: 'separator' },
      );
    }
    items.push(
      { type: 'command', commandId: 'insert:arrange-front', label: '맨 앞으로' },
      { type: 'command', commandId: 'insert:arrange-forward', label: '앞으로' },
      { type: 'command', commandId: 'insert:arrange-backward', label: '뒤로' },
      { type: 'command', commandId: 'insert:arrange-back', label: '맨 뒤로' },
      { type: 'separator' },
    );
    // 그룹 개체: 개체 풀기
    if (ref?.type === 'group') {
      items.push(
        { type: 'command', commandId: 'insert:ungroup-shapes', label: '개체 풀기(U)' },
        { type: 'separator' },
      );
    }
    // 그림/도형 객체: 캡션 넣기
    if (ref?.type === 'image' || ref?.type === 'shape') {
      items.push(
        { type: 'command', commandId: 'insert:caption-toggle', label: '캡션 넣기(A)' },
      );
    }
    items.push(
      { type: 'command', commandId: 'insert:picture-props', label: '개체 속성(P)...' },
      { type: 'separator' },
      { type: 'command', commandId: 'insert:picture-delete', label: '지우기(D)' },
    );
    return items;
  }

  /** 표 셀 내부 컨텍스트 메뉴 항목 */
  private getTableContextMenuItems(): ContextMenuItem[] {
    return [
      { type: 'command', commandId: 'edit:cut' },
      { type: 'command', commandId: 'edit:copy' },
      { type: 'command', commandId: 'edit:paste' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:cell-props', label: '셀 속성...' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:insert-row-above' },
      { type: 'command', commandId: 'table:insert-row-below' },
      { type: 'command', commandId: 'table:insert-col-left' },
      { type: 'command', commandId: 'table:insert-col-right' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:delete-row' },
      { type: 'command', commandId: 'table:delete-col' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:cell-merge' },
      { type: 'command', commandId: 'table:cell-split' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:border-each', label: '셀 테두리/배경 - 각 셀마다 적용(E)...' },
      { type: 'command', commandId: 'table:border-one', label: '셀 테두리/배경 - 하나의 셀처럼 적용(Z)...' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:caption-toggle', label: '캡션 넣기(A)' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:formula', label: '계산식(F)...' },
      { type: 'separator' },
      { type: 'command', commandId: 'table:delete' },
    ];
  }

  /** 일반 컨텍스트 메뉴 항목 */
  private getDefaultContextMenuItems(): ContextMenuItem[] {
    return [
      { type: 'command', commandId: 'edit:cut' },
      { type: 'command', commandId: 'edit:copy' },
      { type: 'command', commandId: 'edit:paste' },
      { type: 'separator' },
      { type: 'command', commandId: 'format:char-shape', label: '글자 모양' },
      { type: 'command', commandId: 'format:para-shape', label: '문단 모양' },
      { type: 'separator' },
      { type: 'command', commandId: 'format:para-num-shape', label: '문단 번호 모양(N)...' },
    ];
  }

  /** 특수 키 처리 (Backspace, Enter, 화살표, Ctrl+Z/Y) */
  private onKeyDown(e: KeyboardEvent): void {
    _keyboard.onKeyDown.call(this, e);
  }

  /** Ctrl/Meta 단축키 처리 */
  private handleCtrlKey(e: KeyboardEvent): void {
    _keyboard.handleCtrlKey.call(this, e);
  }

  /** Ctrl+A: 전체 선택 */
  private handleSelectAll(): void {
    _keyboard.handleSelectAll.call(this);
  }

  // ─── 클립보드 이벤트 처리 ─────────────────────────────

  /** 복사 이벤트 처리 */
  private onCopy(e: ClipboardEvent): void {
    _keyboard.onCopy.call(this, e);
  }

  /** 잘라내기 이벤트 처리 */
  private onCut(e: ClipboardEvent): void {
    _keyboard.onCut.call(this, e);
  }

  /** 붙여넣기 이벤트 처리 */
  private onPaste(e: ClipboardEvent): void {
    _keyboard.onPaste.call(this, e);
  }

  // ─── 서식 적용 ─────────────────────────────────────────

  /** 선택 범위에 글자 서식을 적용한다 */
  private applyCharFormat(props: Partial<CharProperties>): void {
    const sel = this.cursor.getSelectionOrdered();
    if (!sel) return;
    const cmd = new ApplyCharFormatCommand(sel.start, sel.end, props);
    this.executeOperation({ kind: 'command', command: cmd });
  }

  /** 토글 서식 적용 (상호 배타 처리 포함) */
  private applyToggleFormat(prop: 'bold' | 'italic' | 'underline' | 'strikethrough' | 'emboss' | 'engrave' | 'outline' | 'superscript' | 'subscript'): void {
    if (!this.cursor.hasSelection()) return;
    const current = this.getCharPropertiesAtCursor();

    if (prop === 'emboss') {
      const newVal = !current.emboss;
      const mods: Partial<CharProperties> = { emboss: newVal };
      if (newVal) mods.engrave = false;
      this.applyCharFormat(mods);
    } else if (prop === 'engrave') {
      const newVal = !current.engrave;
      const mods: Partial<CharProperties> = { engrave: newVal };
      if (newVal) mods.emboss = false;
      this.applyCharFormat(mods);
    } else if (prop === 'outline') {
      const curOutline = current.outlineType ?? 0;
      this.applyCharFormat({ outlineType: curOutline ? 0 : 1 });
    } else if (prop === 'superscript') {
      const newVal = !current.superscript;
      const mods: Partial<CharProperties> = { superscript: newVal };
      if (newVal) mods.subscript = false;
      this.applyCharFormat(mods);
    } else if (prop === 'subscript') {
      const newVal = !current.subscript;
      const mods: Partial<CharProperties> = { subscript: newVal };
      if (newVal) mods.superscript = false;
      this.applyCharFormat(mods);
    } else {
      this.applyCharFormat({ [prop]: !current[prop] });
    }
  }

  /** 커서 위치의 글자 서식을 조회한다 */
  private getCharPropertiesAtCursor(): CharProperties {
    const pos = this.cursor.getPosition();
    // offset이 0이면 해당 위치, 아니면 offset-1 위치의 서식 반환 (커서 앞 글자 기준)
    const queryOffset = pos.charOffset > 0 ? pos.charOffset - 1 : 0;
    if (pos.parentParaIndex !== undefined) {
      return this.wasm.getCellCharPropertiesAt(
        pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
        pos.cellIndex!, pos.cellParaIndex!, queryOffset,
      );
    }
    return this.wasm.getCharPropertiesAt(pos.sectionIndex, pos.paragraphIndex, queryOffset);
  }

  /** 커서 위치 문단에 문단 서식을 적용한다 */
  private applyParaFormat(props: Record<string, unknown>): void {
    const pos = this.cursor.getPosition();
    const propsJson = JSON.stringify(props);
    try {
      if (pos.parentParaIndex !== undefined) {
        // 셀 내 선택이 있으면 선택 범위 내 모든 셀 문단에 적용
        const sel = this.cursor.getSelectionOrdered();
        if (sel && sel.start.cellParaIndex !== undefined && sel.end.cellParaIndex !== undefined) {
          for (let cp = sel.start.cellParaIndex; cp <= sel.end.cellParaIndex; cp++) {
            this.wasm.applyParaFormatInCell(
              pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
              pos.cellIndex!, cp, propsJson,
            );
          }
        } else {
          this.wasm.applyParaFormatInCell(
            pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
            pos.cellIndex!, pos.cellParaIndex!, propsJson,
          );
        }
      } else {
        // 선택이 있으면 선택 범위 내 모든 문단에 적용
        const sel = this.cursor.getSelectionOrdered();
        if (sel) {
          for (let p = sel.start.paragraphIndex; p <= sel.end.paragraphIndex; p++) {
            this.wasm.applyParaFormat(pos.sectionIndex, p, propsJson);
          }
        } else {
          this.wasm.applyParaFormat(pos.sectionIndex, pos.paragraphIndex, propsJson);
        }
      }
      this.afterEdit();
    } catch (err) {
      console.warn('[InputHandler] applyParaFormat 실패:', err);
    }
  }

  /** 커서 위치 서식 상태를 Toolbar에 알린다 */
  private emitCursorFormatState(): void {
    if (!this.active) return;
    try {
      const props = this.getCharPropertiesAtCursor();
      this.eventBus.emit('cursor-format-changed', props);
    } catch {
      // 문서 없거나 위치 초과 시 무시
    }
    // 문단 속성 (눈금자 마커용) + 스타일
    try {
      const pos = this.cursor.getPosition();
      const inCell = pos.parentParaIndex !== undefined;
      const paraProps = inCell
        ? this.wasm.getCellParaPropertiesAt(
            pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
            pos.cellIndex!, pos.cellParaIndex!,
          )
        : this.wasm.getParaPropertiesAt(pos.sectionIndex, pos.paragraphIndex);
      this.eventBus.emit('cursor-para-changed', paraProps);

      // 스타일 드롭다운 갱신용
      try {
        const styleInfo = inCell
          ? this.wasm.getCellStyleAt(
              pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
              pos.cellIndex!, pos.cellParaIndex!,
            )
          : this.wasm.getStyleAt(pos.sectionIndex, pos.paragraphIndex);
        this.eventBus.emit('cursor-style-changed', styleInfo);
      } catch { /* 스타일 조회 실패 시 무시 */ }

      // 셀 영역 정보 (눈금자 셀 너비 표시용)
      // getTableCellBboxes는 렌더 트리 전 페이지 순회 비용이 크므로:
      // 1) 같은 셀이면 재조회 생략  2) 새 셀이면 rAF로 지연하여 클릭 응답 블로킹 방지
      if (inCell) {
        const cellKey = `${pos.sectionIndex}:${pos.parentParaIndex}:${pos.controlIndex}:${pos.cellIndex}`;
        if (cellKey !== this.lastCellKey) {
          this.lastCellKey = cellKey;
          const sec = pos.sectionIndex;
          const ppi = pos.parentParaIndex!;
          const ci = pos.controlIndex!;
          const cellIdx = pos.cellIndex!;
          const pageHint = this.cursor.getRect()?.pageIndex;
          requestAnimationFrame(() => {
            try {
              const bboxes = this.wasm.getTableCellBboxes(sec, ppi, ci, pageHint);
              const bbox = bboxes.find(b => b.cellIdx === cellIdx);
              if (bbox) {
                this.eventBus.emit('cursor-cell-changed', {
                  inCell: true, cellX: bbox.x, cellWidth: bbox.w,
                });
              }
            } catch { /* 무시 */ }
          });
        }
      } else if (this.lastCellKey !== null) {
        this.lastCellKey = null;
        this.eventBus.emit('cursor-cell-changed', { inCell: false });
      }
    } catch {
      // 무시
    }
  }

  /** 선택 영역을 삭제한다 */
  private deleteSelection(): void {
    const sel = this.cursor.getSelectionOrdered();
    if (!sel) return;

    const cmd = new DeleteSelectionCommand(sel.start, sel.end);
    this.cursor.clearSelection();
    this.executeOperation({ kind: 'command', command: cmd });
  }

  /** Undo 처리 */
  private handleUndo(): void {
    const newPos = this.history.undo(this.wasm);
    if (newPos) {
      this.cursor.moveTo(newPos);
      this.afterEdit();
    }
  }

  /** Redo 처리 */
  private handleRedo(): void {
    const newPos = this.history.redo(this.wasm);
    if (newPos) {
      this.cursor.moveTo(newPos);
      this.afterEdit();
    }
  }

  /**
   * 편집 작업 통합 라우터.
   * 호출부는 OperationDescriptor로 "무엇을 하려는가"만 서술하고,
   * 라우터가 적절한 Undo 전략을 자동 선택한다.
   */
  executeOperation(desc: OperationDescriptor): void {
    switch (desc.kind) {
      case 'command': {
        const newPos = this.history.execute(desc.command, this.wasm);
        // 글자 서식 변경은 문서 구조 불변 → 선택 영역 유지
        if (desc.command.type !== 'applyCharFormat') {
          this.cursor.moveTo(newPos);
          this.cursor.resetPreferredX();
        }
        this.afterEdit();
        break;
      }
      case 'snapshot': {
        const cursorBefore = this.cursor.getPosition();
        const cmd = new SnapshotCommand(desc.operationType, cursorBefore, cursorBefore, desc.operation);
        const newPos = this.history.execute(cmd, this.wasm);
        this.cursor.moveTo(newPos);
        this.cursor.resetPreferredX();
        this.afterEdit();
        break;
      }
      case 'record': {
        this.history.recordWithoutExecute(desc.command);
        break;
      }
    }
  }

  /** Backspace 처리 */
  private handleBackspace(pos: DocumentPosition, inCell: boolean): void {
    _text.handleBackspace.call(this, pos, inCell);
  }

  /** Delete 처리 */
  private handleDelete(pos: DocumentPosition, inCell: boolean): void {
    _text.handleDelete.call(this, pos, inCell);
  }

  /** IME 조합 시작 */
  private onCompositionStart(): void {
    _text.onCompositionStart.call(this);
  }

  /** IME 조합 완료 — 조합 텍스트를 Command로 기록 */
  private onCompositionEnd(): void {
    _text.onCompositionEnd.call(this);
  }

  /** 위치에서 텍스트를 읽는다 (본문/셀 자동 분기) */
  private getTextAt(pos: DocumentPosition, count: number): string {
    return _text.getTextAt.call(this, pos, count);
  }

  /** 텍스트 입력 처리 (textarea input 이벤트) */
  private onInput(e?: Event): void {
    _text.onInput.call(this, e as InputEvent);
  }

  /** 위치에 텍스트를 삽입한다 (WASM 직접 호출, IME 조합용) */
  private insertTextAtRaw(pos: DocumentPosition, text: string): void {
    _text.insertTextAtRaw.call(this, pos, text);
  }

  /** 위치에서 텍스트를 삭제한다 (WASM 직접 호출, IME 조합용) */
  private deleteTextAt(pos: DocumentPosition, count: number): void {
    _text.deleteTextAt.call(this, pos, count);
  }

  /** textarea에 포커스를 설정한다 (iOS 호환) */
  private focusTextarea(): void {
    this.textarea.focus();
  }

  /** 편집 후 처리: 재렌더링 + 캐럿 갱신 */
  private afterEdit(): void {
    this.lastCellKey = null; // 편집 후 셀 bbox 캐시 무효화
    this.eventBus.emit('document-changed');
    this.updateCaret();
  }

  /**
   * 캐럿 위치를 갱신한다.
   *
   * @param skipScroll true 시 `scrollCaretIntoView` 호출 skip — cursor 변경 trigger 가 동반되지 않은
   *                   onMouseUp (예: drag-during-scroll 영역, scrollbar release 영역) 의 자동 scroll back
   *                   결함 차단 영역. (Task #779)
   */
  private updateCaret(skipScroll: boolean = false): void {
    const rect = this.cursor.getRect();
    if (rect) {
      const zoom = this.viewportManager.getZoom();

      // IME 조합 중: 블랙박스 캐럿 표시
      if (this.isComposing && this.compositionAnchor && this.compositionLength > 0) {
        try {
          const anchor = this.compositionAnchor;
          let startRect: CursorRect;
          if (this.cursor.isInHeaderFooter()) {
            const isHeader = this.cursor.headerFooterMode === 'header';
            startRect = this.wasm.getCursorRectInHeaderFooter(
              this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo,
              this.cursor.hfParaIdx, anchor.charOffset, this.cursor.getRect()?.pageIndex ?? 0,
            )!;
          } else if (this.cursor.isInFootnote()) {
            startRect = this.wasm.getCursorRectInFootnote(
              this.cursor.fnPageNum, this.cursor.fnFootnoteIndex,
              this.cursor.fnInnerParaIdx, anchor.charOffset,
            )!;
          } else if ((anchor.cellPath?.length ?? 0) > 1 && anchor.parentParaIndex !== undefined) {
            startRect = this.wasm.getCursorRectByPath(
              anchor.sectionIndex, anchor.parentParaIndex,
              JSON.stringify(anchor.cellPath), anchor.charOffset,
            );
          } else if (anchor.parentParaIndex !== undefined) {
            startRect = this.wasm.getCursorRectInCell(
              anchor.sectionIndex, anchor.parentParaIndex,
              anchor.controlIndex!, anchor.cellIndex!,
              anchor.cellParaIndex!, anchor.charOffset,
            );
          } else {
            startRect = this.wasm.getCursorRect(
              anchor.sectionIndex, anchor.paragraphIndex, anchor.charOffset,
            );
          }
          const charWidth = rect.x - startRect.x;
          const text = this.textarea.value || '';
          // 현재 커서 위치의 글꼴 정보
          let fontFamily = 'sans-serif';
          try {
            const props = this.getCharPropertiesAtCursor();
            if (props.fontFamily) fontFamily = props.fontFamily;
          } catch { /* fallback */ }
          this.caret.showComposition(startRect, charWidth, zoom, text, fontFamily);
        } catch {
          // getCursorRect 실패 시 일반 캐럿
          this.caret.hideComposition();
          this.caret.update(rect, zoom);
        }
      } else {
        this.caret.hideComposition();
        this.caret.update(rect, zoom);
      }
      if (!skipScroll) {
        this.scrollCaretIntoView(rect);
      }
    }
    this.updateSelection();
    this.emitCursorFormatState();
    // [Task #394] 셀 진입 자동 ON 로직 비활성화 — 한컴 출력 정합성을 위해 OFF 기본값 유지.
    // 되돌리려면 아래 호출 + line ~1520 의 동일 호출 + 메서드 본체 / 상태 변수 / 이벤트 핸들러
    // 의 주석을 동시에 풀면 이전 동작 복원.
    // this.checkTransparentBordersTransition();
    this.updateFieldMarkers();
    // 눈금자 다단 영역 표시용 커서 좌표 전달
    const cursorRect = this.cursor.getRect();
    if (cursorRect) {
      this.eventBus.emit('cursor-rect-updated', { x: cursorRect.x, y: cursorRect.y });
    }
  }

  /** 캐럿 위치를 갱신하되 스크롤하지 않는다 (머리말/꼬리말 닫기 등) */
  private updateCaretNoScroll(): void {
    const rect = this.cursor.getRect();
    if (rect) {
      this.caret.update(rect, this.viewportManager.getZoom());
    }
    this.updateSelection();
    this.emitCursorFormatState();
    // [Task #394] 셀 진입 자동 ON 로직 비활성화 — 위 updateCaretAndScroll 의 코멘트 참고.
    // this.checkTransparentBordersTransition();
  }

  /** 드래그 중 캐럿/선택만 가볍게 갱신한다 */
  private updateCaretDuringDrag(): void {
    if (this.isComposing) {
      this.updateCaret();
      return;
    }

    const rect = this.cursor.getRect();
    if (rect) {
      const zoom = this.viewportManager.getZoom();
      this.caret.hideComposition();
      this.caret.updateLive(rect, zoom);
      // [Task #661] 드래그 중 스크롤은 caret rect 가 아니라 포인터 edge 기준 경로에서만 처리한다.
      // 메인테이너 통합 정정: devel 의 updateLive (PR #664 깜박임 타이머 유지 본질) 보존 +
      // PR #718 의 scrollCaretIntoView 부재 본질 적용.
    }
    this.updateSelection();

    const cursorRect = this.cursor.getRect();
    if (cursorRect) {
      this.eventBus.emit('cursor-rect-updated', { x: cursorRect.x, y: cursorRect.y });
    }
  }

  /** 클릭 좌표에서 같은 표 내 셀의 row/col을 반환한다. 다른 표이거나 셀이 아니면 null. */
  private hitTestCellRowCol(e: MouseEvent): { row: number; col: number } | null {
    const ctx = this.cursor.getCellTableContext();
    if (!ctx) return null;
    const zoom = this.viewportManager.getZoom();
    const scrollContent = this.container.querySelector('#scroll-content')!;
    const contentRect = scrollContent.getBoundingClientRect();
    const contentX = e.clientX - contentRect.left;
    const contentY = e.clientY - contentRect.top;
    const pageIdx = this.virtualScroll.getPageAtPoint(contentX, contentY);
    const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
    const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
    const pageX = (contentX - pageLeft) / zoom;
    const pageY = (contentY - pageOffset) / zoom;
    try {
      const hit = this.wasm.hitTest(pageIdx, pageX, pageY);
      // 같은 표인지 확인
      if (hit.parentParaIndex !== ctx.ppi || hit.controlIndex !== ctx.ci) return null;
      if (hit.cellIndex === undefined) return null;
      if (ctx.cellPath && ctx.cellPath.length > 1 && hit.cellPath) {
        // 중첩 표: 경로 기반으로 셀 정보 조회
        const pathJson = JSON.stringify(hit.cellPath);
        const info = this.wasm.getCellInfoByPath(ctx.sec, ctx.ppi, pathJson);
        return { row: info.row, col: info.col };
      }
      const info = this.wasm.getCellInfo(ctx.sec, ctx.ppi, ctx.ci, hit.cellIndex);
      return { row: info.row, col: info.col };
    } catch {
      return null;
    }
  }

  /** F5 셀 선택 하이라이트를 갱신한다 */
  private updateCellSelection(): void {
    if (!this.cellSelectionRenderer) return;
    const range = this.cursor.getSelectedCellRange();
    const ctx = this.cursor.getCellTableContext();
    if (!range || !ctx) {
      this.cellSelectionRenderer.clear();
      return;
    }
    try {
      let bboxes;
      if (ctx.cellPath && ctx.cellPath.length > 1) {
        // 중첩 표: 경로 기반 API 사용
        const pathJson = JSON.stringify(ctx.cellPath);
        bboxes = this.wasm.getTableCellBboxesByPath(ctx.sec, ctx.ppi, pathJson);
      } else {
        bboxes = this.wasm.getTableCellBboxes(ctx.sec, ctx.ppi, ctx.ci);
      }
      const zoom = this.viewportManager.getZoom();
      const excluded = this.cursor.getExcludedCells();
      this.cellSelectionRenderer.render(bboxes, range, zoom, excluded.size > 0 ? excluded : undefined);
    } catch (e) {
      console.warn('[InputHandler] updateCellSelection 실패:', e);
      this.cellSelectionRenderer.clear();
    }
  }

  /** 선택 영역 하이라이트를 갱신한다 */
  private updateSelection(): void {
    const sel = this.cursor.getSelectionOrdered();
    if (!sel) {
      this.selectionRenderer.clear();
      return;
    }

    const { start, end } = sel;
    const zoom = this.viewportManager.getZoom();

    try {
      let rects;
      const startInCell = start.parentParaIndex !== undefined;
      const endInCell = end.parentParaIndex !== undefined;

      if (startInCell && endInCell &&
          start.parentParaIndex === end.parentParaIndex &&
          start.controlIndex === end.controlIndex &&
          start.cellIndex === end.cellIndex) {
        // 같은 셀 내부 선택
        rects = this.wasm.getSelectionRectsInCell(
          start.sectionIndex, start.parentParaIndex!, start.controlIndex!, start.cellIndex!,
          start.cellParaIndex!, start.charOffset,
          end.cellParaIndex!, end.charOffset,
        );
      } else if (!startInCell && !endInCell) {
        // 본문 선택
        rects = this.wasm.getSelectionRects(
          start.sectionIndex,
          start.paragraphIndex, start.charOffset,
          end.paragraphIndex, end.charOffset,
        );
      } else {
        // 셀↔본문 또는 셀↔다른 셀 혼합 선택: 렌더링 생략
        this.selectionRenderer.clear();
        return;
      }
      this.selectionRenderer.render(rects, zoom);
    } catch (e) {
      console.warn('[InputHandler] getSelectionRects 실패:', e);
      this.selectionRenderer.clear();
    }
  }

  /** 표 객체 선택 시 외곽선 + 핸들을 렌더링한다 */
  private renderTableObjectSelection(): void {
    if (!this.tableObjectRenderer) return;
    const ref = this.cursor.getSelectedTableRef();
    if (!ref) {
      this.tableObjectRenderer.clear();
      return;
    }
    try {
      const zoom = this.viewportManager.getZoom();
      const pageHint = this.cursor.getRect()?.pageIndex;
      // 셀 bbox를 페이지별로 그룹화하여 합집합 계산 (다중 페이지 표 지원)
      let cellBboxes: { cellIdx: number; row: number; col: number; rowSpan: number; colSpan: number; pageIndex: number; x: number; y: number; w: number; h: number }[];
      if (ref.cellPath && ref.cellPath.length > 1) {
        // 중첩 표: 경로 기반 API
        const pathJson = JSON.stringify(ref.cellPath);
        cellBboxes = this.wasm.getTableCellBboxesByPath(ref.sec, ref.ppi, pathJson);
      } else {
        // 외부 표: flat API
        cellBboxes = this.wasm.getTableCellBboxes(ref.sec, ref.ppi, ref.ci, pageHint);
      }
      if (cellBboxes.length === 0) {
        this.tableObjectRenderer.clear();
        return;
      }
      // 페이지별 그룹화
      const byPage = new Map<number, typeof cellBboxes>();
      for (const b of cellBboxes) {
        let arr = byPage.get(b.pageIndex);
        if (!arr) { arr = []; byPage.set(b.pageIndex, arr); }
        arr.push(b);
      }
      const pageBboxes: { pageIndex: number; x: number; y: number; width: number; height: number }[] = [];
      for (const [pageIndex, cells] of byPage) {
        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        for (const c of cells) {
          minX = Math.min(minX, c.x);
          minY = Math.min(minY, c.y);
          maxX = Math.max(maxX, c.x + c.w);
          maxY = Math.max(maxY, c.y + c.h);
        }
        pageBboxes.push({ pageIndex, x: minX, y: minY, width: maxX - minX, height: maxY - minY });
      }
      this.tableObjectRenderer.renderMultiPage(pageBboxes, zoom);
    } catch (e) {
      console.warn('[InputHandler] renderTableObjectSelection 실패:', e);
      this.tableObjectRenderer.clear();
    }
  }

  /** 그림/글상자 클릭 감지 — getPageControlLayout으로 개체 bbox 겹침 확인 */
  private findPictureAtClick(
    pageIdx: number, pageX: number, pageY: number,
  ): { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' | 'line'; x1?: number; y1?: number; x2?: number; y2?: number } | null {
    return _picture.findPictureAtClick.call(this, pageIdx, pageX, pageY);
  }

  /** 선택된 그림/글상자의 bbox를 페이지 레이아웃에서 찾는다 */
  private findPictureBbox(
    ref: { sec: number; ppi: number; ci: number; type?: 'image' | 'shape' | 'equation' },
  ): { pageIndex: number; x: number; y: number; w: number; h: number } | null {
    return _picture.findPictureBbox.call(this, ref);
  }

  /** 개체 속성을 타입에 따라 조회한다 (그림/글상자 분기) */
  private getObjectProperties(ref: { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' | 'line' }): any {
    return _picture.getObjectProperties.call(this, ref);
  }

  /** 개체 속성을 타입에 따라 변경한다 (그림/글상자 분기) */
  private setObjectProperties(ref: { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' | 'line' }, props: Record<string, unknown>): void {
    _picture.setObjectProperties.call(this, ref, props);
  }

  /** 개체를 타입에 따라 삭제한다 (그림/글상자 분기) */
  private deleteObjectControl(ref: { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' | 'line' }): void {
    _picture.deleteObjectControl.call(this, ref);
  }

  /** 그림 객체 선택 시 외곽선 + 핸들을 렌더링한다 */
  private renderPictureObjectSelection(): void {
    _picture.renderPictureObjectSelection.call(this);
  }

  /** 그림 객체 선택을 해제한다 (있으면) */
  private exitPictureObjectSelectionIfNeeded(): void {
    _picture.exitPictureObjectSelectionIfNeeded.call(this);
  }

  /** 클릭 좌표가 글상자의 경계선 위인지 판정한다 */
  private isShapeBorderClick(
    pageX: number, pageY: number,
    shape: { sec: number; ppi: number; ci: number },
  ): boolean {
    return _picture.isShapeBorderClick.call(this, pageX, pageY, shape);
  }

  // ─── 그림 핸들 드래그 리사이즈 ─────────────────────────


  /** 드래그 중 실시간 피드백: 핸들 위치를 새 bbox에 맞춰 재렌더 */
  private updatePictureResizeDrag(e: MouseEvent): void {
    _picture.updatePictureResizeDrag.call(this, e);
  }

  /** 드래그 완료: 새 크기를 WASM에 반영 */
  private finishPictureResizeDrag(e: MouseEvent): void {
    _picture.finishPictureResizeDrag.call(this, e);
  }

  /** 드래그 delta로 새 bbox 계산 (page coords) */
  private calcResizedBbox(e: MouseEvent, zoom: number): { x: number; y: number; width: number; height: number } {
    return _picture.calcResizedBbox.call(this, e, zoom);
  }

  private cleanupPictureResizeDrag(): void {
    _picture.cleanupPictureResizeDrag.call(this);
  }

  // ─── 그림 이동 드래그 ──────────────────────────────

  /** 마우스 드래그로 그림 이동 — 드래그 중 갱신 */
  private updatePictureMoveDrag(e: MouseEvent): void {
    _picture.updatePictureMoveDrag.call(this, e);
  }

  /** 마우스 드래그로 그림 이동 — 드래그 종료 */
  private finishPictureMoveDrag(): void {
    _picture.finishPictureMoveDrag.call(this);
  }

  /** 마우스 드래그로 그림 회전 — 드래그 업데이트 */
  private updatePictureRotateDrag(e: MouseEvent): void {
    _picture.updatePictureRotateDrag.call(this, e);
  }

  /** 마우스 드래그로 그림 회전 — 드래그 종료 */
  private finishPictureRotateDrag(e: MouseEvent): void {
    _picture.finishPictureRotateDrag.call(this, e);
  }

  /* [Task #394] 셀 진입 자동 ON 로직 비활성화 — 호출 지점 (updateCaretAndScroll, updateCaretNoScroll)
     의 호출도 같이 주석 처리됨. 되돌리려면 본 블록 주석 + 호출 지점 주석 + 상태 변수 / 이벤트 핸들러
     주석을 동시에 풀면 이전 동작 복원.

  // 셀 진입/탈출 시 투명선 자동 ON/OFF
  private checkTransparentBordersTransition(): void {
    const nowInCell = this.cursor.isInCell() && !this.cursor.isInTextBox();
    if (nowInCell && !this.wasInCell) {
      // 셀 밖 → 셀 진입: 자동 ON
      if (!this.manualTransparentBorders) {
        this.autoTransparentBorders = true;
        this.wasm.setShowTransparentBorders(true);
        document.querySelectorAll('[data-cmd="view:border-transparent"]').forEach(el => {
          el.classList.add('active');
        });
        this.eventBus.emit('document-changed');
      }
    } else if (!nowInCell && this.wasInCell) {
      // 셀 안 → 셀 탈출: 자동으로 켜진 경우에만 OFF
      if (this.autoTransparentBorders && !this.manualTransparentBorders) {
        this.autoTransparentBorders = false;
        this.wasm.setShowTransparentBorders(false);
        document.querySelectorAll('[data-cmd="view:border-transparent"]').forEach(el => {
          el.classList.remove('active');
        });
        this.eventBus.emit('document-changed');
      }
    }
    this.wasInCell = nowInCell;
  }
  */

  /** 캐럿이 화면 밖이면 스크롤을 조정한다 */
  private scrollCaretIntoView(rect: import('@/core/types').CursorRect): void {
    const zoom = this.viewportManager.getZoom();
    const pageOffset = this.virtualScroll.getPageOffset(rect.pageIndex);
    const caretDocY = pageOffset + rect.y * zoom;
    const caretHeight = rect.height * zoom;

    const scrollTop = this.container.scrollTop;
    const viewHeight = this.container.clientHeight;
    const margin = 20; // 여백 px

    if (caretDocY < scrollTop + margin) {
      // 캐럿이 화면 위쪽 밖
      this.container.scrollTop = Math.max(0, caretDocY - margin);
    } else if (caretDocY + caretHeight > scrollTop + viewHeight - margin) {
      // 캐럿이 화면 아래쪽 밖
      this.container.scrollTop = caretDocY + caretHeight - viewHeight + margin;
    }
  }

  /** 문서 로딩 후 저장된 캐럿 위치에 캐럿을 배치한다 */
  activateWithCaretPosition(): void {
    try {
      const savedPos = this.wasm.getCaretPosition();
      if (savedPos) {
        this.cursor.moveTo(savedPos);
      } else {
        this.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 });
      }
      this.cursor.resetPreferredX();
      this.active = true;

      const rect = this.cursor.getRect();
      if (rect) {
        this.caret.show(rect, this.viewportManager.getZoom());
      }
      this.emitCursorFormatState();
      this.focusTextarea();
    } catch (e) {
      console.warn('[InputHandler] 캐럿 자동 배치 실패:', e);
      // 실패 시 문서 시작에 배치
      this.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 });
      this.active = true;
      const rect = this.cursor.getRect();
      if (rect) {
        this.caret.show(rect, this.viewportManager.getZoom());
      }
      this.focusTextarea();
    }
  }

  /** 캐럿을 숨기고 히스토리를 초기화한다 */
  /** textarea에 포커스를 복원한다 (대화상자 닫힌 후 등) */
  focus(): void {
    this.focusTextarea();
  }

  deactivate(): void {
    this.active = false;
    this.caret.hide();
    this.fieldMarker.hide();
    this.cursor.clearSelection();
    this.selectionRenderer.clear();
    this.history.clear(this.wasm);
  }

  dispose(): void {
    if (this.isResizeDragging) {
      this.cleanupResizeDrag();
    }
    if (this.dragRafId) {
      cancelAnimationFrame(this.dragRafId);
      this.dragRafId = 0;
    }
    this.stopTextSelectionDragAutoScroll();
    if (this.resizeHoverRafId) {
      cancelAnimationFrame(this.resizeHoverRafId);
      this.resizeHoverRafId = 0;
    }
    document.removeEventListener('keydown', this.onF11InterceptBound, true);
    this.container.removeEventListener('mousedown', this.onClickBound);
    this.container.removeEventListener('dblclick', this.onDblClickBound);
    this.container.removeEventListener('contextmenu', this.onContextMenuBound);
    this.container.removeEventListener('mousemove', this.onMouseMoveBound);
    document.removeEventListener('mousemove', this.onMouseMoveBound);
    document.removeEventListener('mouseup', this.onMouseUpBound);
    this.textarea.removeEventListener('keydown', this.onKeyDownBound);
    this.textarea.removeEventListener('input', this.onInputBound);
    this.textarea.removeEventListener('compositionstart', this.onCompositionStartBound);
    this.textarea.removeEventListener('compositionend', this.onCompositionEndBound);
    this.textarea.removeEventListener('copy', this.onCopyBound);
    this.textarea.removeEventListener('cut', this.onCutBound);
    this.textarea.removeEventListener('paste', this.onPasteBound);
    this.textarea.remove();
    this.caret.dispose();
    this.fieldMarker.dispose();
    this.selectionRenderer.dispose();
    this.cellSelectionRenderer?.dispose();
    this.tableObjectRenderer?.dispose();
    this.tableResizeRenderer?.dispose();
    this.contextMenu?.dispose();
  }

  // ─── 커맨드 시스템용 public 접근자 ─────────────────────────

  /** 커맨드 디스패처를 주입한다 (main.ts에서 호출) */
  setDispatcher(d: CommandDispatcher): void { this.dispatcher = d; }

  /** 편집 영역이 활성 상태인지 (문서 로드 + 편집 영역 포커스) */
  isActive(): boolean { return this.active; }

  /** 컨텍스트 메뉴를 주입한다 (main.ts에서 호출) */
  setContextMenu(cm: ContextMenu): void { this.contextMenu = cm; }

  /** 커맨드 팔레트를 주입한다 (main.ts에서 호출) */
  setCommandPalette(cp: CommandPalette): void { this.commandPalette = cp; }

  /** 셀 선택 렌더러를 주입한다 (main.ts에서 호출) */
  setCellSelectionRenderer(r: CellSelectionRenderer): void { this.cellSelectionRenderer = r; }

  /** 표 객체 선택 렌더러를 주입한다 (main.ts에서 호출) */
  setTableObjectRenderer(r: TableObjectRenderer): void { this.tableObjectRenderer = r; }

  /** 그림 객체 선택 렌더러를 주입한다 (main.ts에서 호출) */
  setPictureObjectRenderer(r: TableObjectRenderer): void { this.pictureObjectRenderer = r; }

  /** 그림 객체 선택 모드인가? */
  isInPictureObjectSelection(): boolean { return this.cursor.isInPictureObjectSelection(); }

  /** 선택된 그림/글상자 참조 반환 */
  getSelectedPictureRef(): { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' | 'line'; cellIdx?: number; cellParaIdx?: number } | null { return this.cursor.getSelectedPictureRef(); }

  /** 다중 선택된 개체 목록 */
  getSelectedPictureRefs(): { sec: number; ppi: number; ci: number; type: string }[] { return this.cursor.getSelectedPictureRefs(); }

  /** 다중 선택 상태인가? */
  isMultiPictureSelection(): boolean { return this.cursor.isMultiPictureSelection(); }

  /** 지정 개체를 선택 상태로 진입 */
  selectPictureObject(sec: number, ppi: number, ci: number, type: 'image' | 'shape' | 'equation' | 'group' | 'line'): void {
    this.cursor.enterPictureObjectSelectionDirect(sec, ppi, ci, type);
    this.renderPictureObjectSelection();
    this.eventBus.emit('picture-object-selection-changed', true);
  }

  /** 그림 삭제 후: 선택 해제 + afterEdit */
  /** 커서 위치 반환 */
  getPosition(): { sectionIndex: number; paragraphIndex: number; charOffset: number } {
    return this.cursor.getPosition();
  }

  /** 편집 완료 후 렌더링 갱신 */
  triggerAfterEdit(): void {
    this.afterEdit();
  }

  exitPictureObjectSelectionAndAfterEdit(): void {
    this.exitPictureObjectSelectionIfNeeded();
    this.afterEdit();
  }

  /** 글상자 내부 텍스트 편집 모드 진입 */
  private enterTextboxEditing(sec: number, ppi: number, ci: number): void {
    this.enterInlineEditing(sec, ppi, ci, 0);
  }

  /** 캡션/글상자 내부 텍스트 편집 모드 진입 (charOffset 지정 가능) */
  enterInlineEditing(sec: number, ppi: number, ci: number, charOffset = 0): void {
    this.cursor.clearSelection();
    this.cursor.moveTo({
      sectionIndex: sec,
      paragraphIndex: 0,
      charOffset,
      parentParaIndex: ppi,
      controlIndex: ci,
      cellIndex: 0,
      cellParaIndex: 0,
      isTextBox: true,
    });
    this.cursor.resetPreferredX();
    this.updateCaret();
    this.focusTextarea();
  }

  /** 표 캡션 텍스트 편집 모드 진입 (cellIndex=65534로 캡션 구분) */
  enterTableCaptionEditing(sec: number, ppi: number, ci: number, charOffset = 0): void {
    this.cursor.clearSelection();
    this.cursor.moveTo({
      sectionIndex: sec,
      paragraphIndex: 0,
      charOffset,
      parentParaIndex: ppi,
      controlIndex: ci,
      cellIndex: 65534,
      cellParaIndex: 0,
    });
    this.cursor.resetPreferredX();
    this.updateCaret();
    this.focusTextarea();
  }

  /** 표 경계선 리사이즈 렌더러를 주입한다 (main.ts에서 호출) */
  setTableResizeRenderer(r: TableResizeRenderer): void { this.tableResizeRenderer = r; }

  /** 선택 영역이 있는가? */
  hasSelection(): boolean { return this.cursor.hasSelection(); }

  /** 현재 커서 위치를 반환한다 */
  getCursorPosition(): DocumentPosition { return this.cursor.getPosition(); }

  /** 커서를 지정 위치로 이동하고 캐럿을 표시한다. 성공하면 true 반환. */
  moveCursorTo(pos: DocumentPosition): boolean {
    // 이동 전 위치가 유효한지 사전 검증 (경고 로그 방지)
    try {
      const testRect = this.wasm.getCursorRect(pos.sectionIndex, pos.paragraphIndex, pos.charOffset);
      if (!testRect || testRect.pageIndex === undefined) return false;
    } catch {
      return false;
    }

    this.cursor.clearSelection();
    this.cursor.moveTo(pos);
    this.cursor.resetPreferredX();
    this.active = true;
    const rect = this.cursor.getRect();
    if (rect) {
      this.caret.show(rect, this.viewportManager.getZoom());
      this.updateCaret();
      this.focusTextarea();
      return true;
    }
    this.focusTextarea();
    return false;
  }

  /** 현재 커서 위치의 누름틀 필드를 제거한다 (텍스트 유지). */
  removeCurrentField(): void {
    const pos = this.cursor.getPosition();
    try {
      const result = this.wasm.removeFieldAt(pos);
      if (result.ok) {
        this.afterEdit();
        this.eventBus.emit('field-info-changed', null);
      }
    } catch (err) {
      console.warn('[InputHandler] 누름틀 제거 실패:', err);
    }
  }

  /** 커서 위치의 필드 상태에 따라 낫표 마커를 표시/숨김한다 */
  private updateFieldMarkers(): void {
    const wasVisible = this.fieldMarker.isVisible;
    try {
      const pos = this.cursor.getPosition();
      const fi = this.wasm.getFieldInfoAt(pos);
      if (fi.inField && fi.startCharIdx !== undefined && fi.endCharIdx !== undefined) {
        // 활성 필드 설정 → 안내문 숨김 + 페이지 캐시 무효화
        const fieldChanged = this.wasm.setActiveField(pos);
        const zoom = this.viewportManager.getZoom();
        // 필드 시작/끝 위치의 커서 좌표를 얻어 마커 표시
        let startRect: CursorRect, endRect: CursorRect;
        if ((pos.cellPath?.length ?? 0) > 1 && pos.parentParaIndex !== undefined) {
          // 중첩 표: path 기반 커서 좌표
          const pathJson = JSON.stringify(pos.cellPath);
          startRect = this.wasm.getCursorRectByPath(
            pos.sectionIndex, pos.parentParaIndex, pathJson, fi.startCharIdx,
          );
          endRect = this.wasm.getCursorRectByPath(
            pos.sectionIndex, pos.parentParaIndex, pathJson, fi.endCharIdx,
          );
        } else if (pos.parentParaIndex !== undefined) {
          startRect = this.wasm.getCursorRectInCell(
            pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
            pos.cellIndex!, pos.cellParaIndex!, fi.startCharIdx,
          );
          endRect = this.wasm.getCursorRectInCell(
            pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
            pos.cellIndex!, pos.cellParaIndex!, fi.endCharIdx,
          );
        } else {
          startRect = this.wasm.getCursorRect(pos.sectionIndex, pos.paragraphIndex, fi.startCharIdx);
          endRect = this.wasm.getCursorRect(pos.sectionIndex, pos.paragraphIndex, fi.endCharIdx);
        }
        this.fieldMarker.show(startRect, endRect, zoom);
        // 필드 진입 또는 다른 필드로 전환 시 재렌더링 (안내문 표시/숨김 반영)
        if (!wasVisible || fieldChanged) {
          this.eventBus.emit('document-changed');
          // 재렌더링 후 캐럿 위치 재계산 (가이드 텍스트 제거로 좌표 변경됨)
          this.cursor.updateRect();
          this.updateCaret();
        }
        // 상태 표시줄에 필드 정보 표시
        this.eventBus.emit('field-info-changed', {
          fieldId: fi.fieldId, fieldType: fi.fieldType, guideName: fi.guideName,
        });
        return;
      }
    } catch (err) { console.warn('[updateFieldMarkers] 필드 마커 갱신 실패:', err); }
    // 필드 밖이면 마커 숨김 + 활성 필드 해제
    if (wasVisible) {
      this.fieldMarker.hide();
      this.wasm.clearActiveField();
      this.eventBus.emit('document-changed');
      this.eventBus.emit('field-info-changed', null);
    }
  }

  /** 커서가 누름틀 필드 내부인가? */
  isInField(): boolean {
    try {
      const fi = this.wasm.getFieldInfoAt(this.cursor.getPosition());
      return fi.inField;
    } catch { return false; }
  }

  /** 현재 커서 위치의 필드 정보를 반환한다. */
  getFieldInfo(): { fieldId: number; fieldType: string; guideName: string } | null {
    try {
      const fi = this.wasm.getFieldInfoAt(this.cursor.getPosition());
      if (fi.inField && fi.fieldId !== undefined) {
        return { fieldId: fi.fieldId, fieldType: fi.fieldType ?? '', guideName: fi.guideName ?? '' };
      }
    } catch { /* 무시 */ }
    return null;
  }

  /** 커서가 표 셀 내부인가? */
  isInTable(): boolean { return this.cursor.isInCell(); }

  /** 셀 선택 모드인가? */
  isInCellSelectionMode(): boolean { return this.cursor.isInCellSelectionMode(); }

  /** 표 객체 선택 모드인가? */
  isInTableObjectSelection(): boolean { return this.cursor.isInTableObjectSelection(); }

  /** 선택된 표의 참조 정보 반환 */
  getSelectedTableRef() { return this.cursor.getSelectedTableRef(); }

  /** 표 객체 선택 해제 + 재렌더링 */
  exitTableObjectSelection(): void {
    this.cursor.exitTableObjectSelection();
    this.afterEdit();
  }

  /** 셀 선택 범위 반환 (셀 선택 모드가 아니면 null) */
  getSelectedCellRange() { return this.cursor.getSelectedCellRange(); }

  /** 셀 선택 중인 표의 컨텍스트 반환 */
  getCellTableContext() { return this.cursor.getCellTableContext(); }

  /** 셀 선택 모드 종료 */
  exitCellSelectionMode(): void {
    this.cursor.exitCellSelectionMode();
    this.cellSelectionRenderer?.clear();
    this.updateCaret();
  }

  /** Undo 가능한가? */
  canUndo(): boolean { return this.history.canUndo(); }

  /** Redo 가능한가? */
  canRedo(): boolean { return this.history.canRedo(); }

  /** Undo 실행 (커맨드 시스템용) */
  performUndo(): void { this.handleUndo(); }

  /** Redo 실행 (커맨드 시스템용) */
  performRedo(): void { this.handleRedo(); }

  /** 복사 (커맨드 시스템용 — 컨텍스트 메뉴/도구 상자에서 호출) */
  performCopy(): void {
    // 개체 선택 모드 → 직접 클립보드 기록 (textarea 포커스 불필요)
    if (this.cursor.isInPictureObjectSelection()) {
      const ref = this.cursor.getSelectedPictureRef();
      if (ref) {
        try {
          this.wasm.copyControl(ref.sec, ref.ppi, ref.ci);
          const text = this.wasm.getClipboardText() || '[그림]';
          let html = '';
          try { html = this.wasm.exportControlHtml(ref.sec, ref.ppi, ref.ci) || ''; } catch { /* 무시 */ }
          if (ref.type === 'image') {
            _keyboard.writeImageToClipboard(this.wasm, ref.sec, ref.ppi, ref.ci, text, html)
              .catch(() => navigator.clipboard.writeText(text).catch(() => {}));
          } else {
            navigator.clipboard.writeText(text).catch(() => {});
          }
        } catch (err) {
          console.warn('[InputHandler] 개체 복사 실패:', err);
        }
      }
      return;
    }
    if (this.cursor.isInTableObjectSelection()) {
      const ref = this.cursor.getSelectedTableRef();
      if (ref) {
        try {
          this.wasm.copyControl(ref.sec, ref.ppi, ref.ci);
          const text = this.wasm.getClipboardText() || '[표]';
          navigator.clipboard.writeText(text).catch(() => {});
        } catch (err) {
          console.warn('[InputHandler] 표 복사 실패:', err);
        }
      }
      return;
    }
    // 텍스트 선택 → textarea 포커스 후 execCommand
    this.focusTextarea();
    document.execCommand('copy');
  }

  /** 잘라내기 (커맨드 시스템용 — 컨텍스트 메뉴/도구 상자에서 호출) */
  performCut(): void {
    // 개체 선택 모드 → 복사 + 삭제
    if (this.cursor.isInPictureObjectSelection()) {
      const ref = this.cursor.getSelectedPictureRef();
      if (ref) {
        // 클립보드에 복사
        this.performCopy();
        // 삭제
        this.cursor.moveOutOfSelectedPicture();
        this.pictureObjectRenderer?.clear();
        this.eventBus.emit('picture-object-selection-changed', false);
        this.executeOperation({ kind: 'snapshot', operationType: 'cutObject', operation: (wasm: WasmBridge) => {
          if (ref.type === 'image') {
            wasm.deletePictureControl(ref.sec, ref.ppi, ref.ci);
          } else {
            wasm.deleteShapeControl(ref.sec, ref.ppi, ref.ci);
          }
          return this.cursor.getPosition();
        }});
      }
      return;
    }
    if (this.cursor.isInTableObjectSelection()) {
      const ref = this.cursor.getSelectedTableRef();
      if (ref) {
        this.performCopy();
        this.cursor.moveOutOfSelectedTable();
        this.eventBus.emit('table-object-selection-changed', false);
        this.executeOperation({ kind: 'snapshot', operationType: 'cutTable', operation: (wasm: WasmBridge) => {
          wasm.deleteTableControl(ref.sec, ref.ppi, ref.ci);
          return this.cursor.getPosition();
        }});
      }
      return;
    }
    // 텍스트 선택 → textarea 포커스 후 execCommand
    this.focusTextarea();
    document.execCommand('cut');
  }

  /** 선택 영역 삭제 (커맨드 시스템용 — 편집 > 지우기) */
  performDelete(): void {
    if (this.cursor.isInPictureObjectSelection()) {
      const ref = this.cursor.getSelectedPictureRef();
      if (ref) {
        this.cursor.moveOutOfSelectedPicture();
        this.pictureObjectRenderer?.clear();
        this.eventBus.emit('picture-object-selection-changed', false);
        this.executeOperation({ kind: 'snapshot', operationType: 'deleteObject', operation: (wasm: WasmBridge) => {
          this.deleteObjectControl(ref);
          return this.cursor.getPosition();
        }});
      }
      return;
    }
    if (this.cursor.isInTableObjectSelection()) {
      const ref = this.cursor.getSelectedTableRef();
      if (!ref) return;
      if (ref.cellPath && ref.cellPath.length > 1) {
        this.cursor.moveOutOfSelectedTable();
        this.eventBus.emit('table-object-selection-changed', false);
        return;
      }
      this.cursor.moveOutOfSelectedTable();
      this.eventBus.emit('table-object-selection-changed', false);
      this.executeOperation({ kind: 'snapshot', operationType: 'deleteTable', operation: (wasm: WasmBridge) => {
        wasm.deleteTableControl(ref.sec, ref.ppi, ref.ci);
        return this.cursor.getPosition();
      }});
      return;
    }
    if (this.cursor.hasSelection()) {
      this.deleteSelection();
    }
  }

  /** 전체 선택 (커맨드 시스템용) */
  performSelectAll(): void { this.handleSelectAll(); }

  /** 서식 토글 (커맨드 시스템용) */
  toggleFormat(prop: 'bold' | 'italic' | 'underline' | 'strikethrough' | 'emboss' | 'engrave' | 'outline' | 'superscript' | 'subscript'): void {
    this.applyToggleFormat(prop);
  }

  /** 문단 정렬 적용 (커맨드 시스템용) */
  applyParaAlign(align: string): void {
    this.applyParaFormat({ alignment: align });
  }

  /** 줄 간격 적용 (커맨드 시스템용, Percent 타입) */
  setLineSpacing(value: number): void {
    this.applyParaFormat({ lineSpacing: value, lineSpacingType: 'Percent' });
  }

  /** 글꼴 크기 증감 (커맨드 시스템용, delta: HWPUNIT, 1pt=100) */
  adjustFontSize(delta: number): void {
    if (!this.cursor.hasSelection()) return;
    const current = this.getCharPropertiesAtCursor();
    const newSize = Math.max(100, (current.fontSize ?? 1000) + delta); // 최소 1pt
    this.applyCharFormat({ fontSize: newSize });
  }

  /** 스타일 적용 (커맨드 시스템용) */
  applyStyle(styleId: number): void {
    const pos = this.cursor.getPosition();
    try {
      if (pos.parentParaIndex !== undefined) {
        this.wasm.applyCellStyle(
          pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
          pos.cellIndex!, pos.cellParaIndex!, styleId,
        );
      } else {
        this.wasm.applyStyle(pos.sectionIndex, pos.paragraphIndex, styleId);
      }
      this.afterEdit();
    } catch (err) {
      console.warn('[InputHandler] applyStyle 실패:', err);
    }
  }

  /** 개요 수준 변경 (delta: +1=한 수준 증가, -1=한 수준 감소) */
  changeOutlineLevel(delta: number): void {
    const pos = this.cursor.getPosition();
    try {
      const inCell = pos.parentParaIndex !== undefined;
      const currentStyle = inCell
        ? this.wasm.getCellStyleAt(
            pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
            pos.cellIndex!, pos.cellParaIndex!,
          )
        : this.wasm.getStyleAt(pos.sectionIndex, pos.paragraphIndex);

      // 현재 개요 수준 파싱 (개요 1~7)
      const match = currentStyle.name.match(/^개요\s*(\d)$/);
      if (!match) return; // 개요 스타일이 아니면 무시

      const currentLevel = parseInt(match[1], 10);
      const targetLevel = currentLevel + delta;
      if (targetLevel < 1 || targetLevel > 7) return;

      // 스타일 목록에서 대상 개요 스타일 찾기
      const styles = this.wasm.getStyleList();
      const targetStyle = styles.find(s => {
        const m = s.name.match(/^개요\s*(\d)$/);
        return m && parseInt(m[1], 10) === targetLevel;
      });
      if (!targetStyle) return;

      this.applyStyle(targetStyle.id);
    } catch (err) {
      console.warn('[InputHandler] changeOutlineLevel 실패:', err);
    }
  }

  /** 문단 번호 토글: None→Number, Number/Outline→None */
  toggleNumbering(): void {
    try {
      const props = this.getParaProperties();
      if (props.headType && props.headType !== 'None') {
        // 번호 해제
        this.applyParaFormat({ headType: 'None' } as Partial<import('@/core/types').ParaProperties>);
      } else {
        // 번호 적용
        const nid = this.wasm.ensureDefaultNumbering();
        this.applyParaFormat({
          headType: 'Number',
          numberingId: nid,
          paraLevel: 0,
        } as Partial<import('@/core/types').ParaProperties>);
      }
      this.focusTextarea();
    } catch (err) {
      console.warn('[InputHandler] toggleNumbering 실패:', err);
    }
  }

  /** 글머리표 토글: None→Bullet, Bullet→None */
  toggleBullet(bulletChar = '●'): void {
    try {
      const props = this.getParaProperties();
      if (props.headType === 'Bullet') {
        // 글머리표 해제
        this.applyParaFormat({ headType: 'None' } as Partial<import('@/core/types').ParaProperties>);
      } else {
        // 글머리표 적용
        const bid = this.wasm.ensureDefaultBullet(bulletChar);
        this.applyParaFormat({
          headType: 'Bullet',
          numberingId: bid,
          paraLevel: 0,
        } as Partial<import('@/core/types').ParaProperties>);
      }
      this.focusTextarea();
    } catch (err) {
      console.warn('[InputHandler] toggleBullet 실패:', err);
    }
  }

  /** 글머리표 적용 (팝업에서 선택한 문자, 토글 없이 항상 적용) */
  applyBullet(bulletChar: string): void {
    try {
      const bid = this.wasm.ensureDefaultBullet(bulletChar);
      this.applyParaFormat({
        headType: 'Bullet',
        numberingId: bid,
        paraLevel: 0,
      } as Partial<import('@/core/types').ParaProperties>);
      this.focusTextarea();
    } catch (err) {
      console.warn('[InputHandler] applyBullet 실패:', err);
    }
  }

  /** 문단 번호 모양 적용 (대화상자에서 선택한 numberingId) */
  applyNumbering(numberingId: number): void {
    try {
      this.applyParaFormat({
        headType: 'Number',
        numberingId,
        paraLevel: 0,
      } as Partial<import('@/core/types').ParaProperties>);
      this.focusTextarea();
    } catch (err) {
      console.warn('[InputHandler] applyNumbering 실패:', err);
    }
  }

  /** 글자 모양 대화상자용: 커서 위치의 글자 서식 조회 (커맨드 시스템용) */
  getCharProperties(): CharProperties {
    return this.getCharPropertiesAtCursor();
  }

  /** 문단 모양 대화상자용: 커서 위치의 문단 서식 조회 (커맨드 시스템용) */
  getParaProperties(): ParaProperties {
    // 머리말/꼬리말 모드
    if (this.cursor.isInHeaderFooter()) {
      const isHeader = this.cursor.headerFooterMode === 'header';
      return this.wasm.getParaPropertiesInHf(
        this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo, this.cursor.hfParaIdx,
      );
    }
    const pos = this.cursor.getPosition();
    if (pos.parentParaIndex !== undefined) {
      return this.wasm.getCellParaPropertiesAt(
        pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
        pos.cellIndex!, pos.cellParaIndex!,
      );
    }
    return this.wasm.getParaPropertiesAt(pos.sectionIndex, pos.paragraphIndex);
  }

  /** 커서 위치의 문단 스타일 ID를 반환한다 (스타일 대화상자용) */
  getCurrentStyleId(): number {
    try {
      const pos = this.cursor.getPosition();
      const info = pos.parentParaIndex !== undefined
        ? this.wasm.getCellStyleAt(
            pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!,
            pos.cellIndex!, pos.cellParaIndex!,
          )
        : this.wasm.getStyleAt(pos.sectionIndex, pos.paragraphIndex);
      return info.id;
    } catch {
      return 0;
    }
  }

  /** 현재 선택 범위를 반환한다 (커맨드 시스템용) */
  getSelection(): { start: DocumentPosition; end: DocumentPosition } | null {
    return this.cursor.getSelectionOrdered();
  }

  /** 지정된 선택 범위에 글자 서식을 적용한다 (커맨드 시스템용) */
  applyCharPropsToRange(
    start: DocumentPosition,
    end: DocumentPosition,
    props: Partial<CharProperties>,
  ): void {
    const cmd = new ApplyCharFormatCommand(start, end, props);
    this.executeOperation({ kind: 'command', command: cmd });
  }

  /** 지정된 선택 범위에 문단 서식을 적용한다 (커맨드 시스템용) */
  applyParaPropsToRange(
    start: DocumentPosition,
    end: DocumentPosition,
    props: Partial<ParaProperties>,
  ): void {
    const propsJson = JSON.stringify(props);
    try {
      // 머리말/꼬리말 모드
      if (this.cursor.isInHeaderFooter()) {
        const isHeader = this.cursor.headerFooterMode === 'header';
        this.wasm.applyParaFormatInHf(
          this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo,
          this.cursor.hfParaIdx, propsJson,
        );
        this.afterEdit();
        return;
      }
      if (start.parentParaIndex !== undefined) {
        this.wasm.applyParaFormatInCell(
          start.sectionIndex, start.parentParaIndex, start.controlIndex!,
          start.cellIndex!, start.cellParaIndex!, propsJson,
        );
      } else {
        for (let p = start.paragraphIndex; p <= end.paragraphIndex; p++) {
          this.wasm.applyParaFormat(start.sectionIndex, p, propsJson);
        }
      }
      this.afterEdit();
    } catch (err) {
      console.warn('[InputHandler] applyParaPropsToRange 실패:', err);
    }
  }

  /** 양식 개체 클릭 처리 */
  handleFormObjectClick(formHit: FormObjectHitResult, pageIdx: number, _zoom: number): void {
    if (!formHit.found || formHit.sec === undefined || formHit.para === undefined || formHit.ci === undefined) return;

    const { sec, para, ci, formType } = formHit;

    // 셀 내부 폼 값 설정 헬퍼
    const setFormVal = (valueJson: string) => {
      if (formHit.inCell && formHit.tablePara !== undefined && formHit.tableCi !== undefined
          && formHit.cellIdx !== undefined && formHit.cellPara !== undefined) {
        this.wasm.setFormValueInCell(sec, formHit.tablePara, formHit.tableCi,
          formHit.cellIdx, formHit.cellPara, ci, valueJson);
      } else {
        this.wasm.setFormValue(sec, para, ci, valueJson);
      }
    };

    switch (formType) {
      case 'CheckBox': {
        // 체크박스 토글: value 0↔1
        const newValue = (formHit.value ?? 0) === 0 ? 1 : 0;
        setFormVal(JSON.stringify({ value: newValue }));
        this.afterEdit();
        break;
      }
      case 'RadioButton': {
        // 라디오 버튼: 같은 그룹 내 다른 라디오 버튼 해제 후 선택
        this.handleRadioButtonClick(sec, para, ci);
        break;
      }
      case 'PushButton': {
        // 명령 단추: 웹 환경에서는 보안상 비활성 (클릭 무시)
        break;
      }
      case 'ComboBox': {
        this.showComboBoxOverlay(sec, para, ci, formHit, pageIdx);
        break;
      }
      case 'Edit': {
        this.showEditOverlay(sec, para, ci, formHit, pageIdx);
        break;
      }
    }
  }

  /** 라디오 버튼 클릭: 같은 그룹 내 다른 라디오 버튼 해제 */
  private handleRadioButtonClick(sec: number, para: number, ci: number): void {
    // 현재 클릭된 라디오 버튼의 그룹 이름 조회
    const info = this.wasm.getFormObjectInfo(sec, para, ci);
    if (!info.ok) return;

    const groupName = info.properties?.['GroupName'] ?? '';

    // 같은 문단 내 다른 라디오 버튼 찾아서 해제
    // (HWP 양식에서 라디오 버튼은 보통 같은 문단에 배치됨)
    const section = sec;
    // 동일 문단의 모든 컨트롤을 순회하여 같은 그룹의 라디오 버튼 해제
    for (let i = 0; i < 50; i++) { // 최대 50개 컨트롤 검사
      if (i === ci) continue;
      const otherInfo = this.wasm.getFormObjectInfo(section, para, i);
      if (!otherInfo.ok || otherInfo.formType !== 'RadioButton') continue;
      const otherGroup = otherInfo.properties?.['GroupName'] ?? '';
      if (otherGroup === groupName && otherInfo.value !== 0) {
        this.wasm.setFormValue(section, para, i, JSON.stringify({ value: 0 }));
      }
    }

    // 클릭된 라디오 버튼 선택
    this.wasm.setFormValue(sec, para, ci, JSON.stringify({ value: 1 }));
    this.afterEdit();
  }

  /** 양식 개체 bbox를 scroll-content 내 절대 좌표로 변환 */
  private formBboxToOverlayRect(bbox: { x: number; y: number; w: number; h: number }, pageIdx: number): { left: number; top: number; width: number; height: number } {
    const zoom = this.viewportManager.getZoom();
    const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, contentWidth);

    return {
      left: pageLeft + bbox.x * zoom,
      top: pageOffset + bbox.y * zoom,
      width: bbox.w * zoom,
      height: bbox.h * zoom,
    };
  }

  /** 기존 양식 오버레이 제거 */
  private removeFormOverlay(): void {
    if (this.formOverlay) {
      try { this.formOverlay.remove(); } catch { /* 이미 제거됨 */ }
      this.formOverlay = null;
    }
  }

  /** ComboBox 드롭다운 오버레이 */
  private showComboBoxOverlay(sec: number, para: number, ci: number, formHit: FormObjectHitResult, pageIdx: number): void {
    this.removeFormOverlay();
    if (!formHit.bbox) return;

    const info = this.wasm.getFormObjectInfo(sec, para, ci);
    if (!info.ok) return;

    // 항목 목록: 스크립트 InsertString 추출 결과 (WASM에서 제공)
    const items: string[] = info.items ?? [];
    const currentText = formHit.text ?? '';

    if (items.length === 0) {
      // 항목 없으면 Edit 오버레이로 대체
      this.showEditOverlay(sec, para, ci, formHit, pageIdx);
      return;
    }

    const rect = this.formBboxToOverlayRect(formHit.bbox, pageIdx);
    const fontSize = Math.max(rect.height * 0.6, 10);
    const itemHeight = fontSize * 1.6;

    // 컨테이너 (콤보박스 위치에 드롭다운 리스트 표시)
    const dropdown = document.createElement('div');
    dropdown.className = 'form-combo-dropdown';
    dropdown.style.left = `${rect.left}px`;
    dropdown.style.top = `${rect.top + rect.height}px`;
    dropdown.style.width = `${rect.width}px`;

    for (const item of items) {
      const row = document.createElement('div');
      row.className = 'form-combo-item' + (item === currentText ? ' selected' : '');
      row.textContent = item;
      row.style.fontSize = `${fontSize}px`;
      row.style.lineHeight = `${itemHeight}px`;
      row.addEventListener('mousedown', (e) => {
        e.preventDefault();
        this.wasm.setFormValue(sec, para, ci, JSON.stringify({ text: item }));
        this.removeFormOverlay();
        this.afterEdit();
      });
      dropdown.appendChild(row);
    }

    // 외부 클릭 시 닫기
    const onDocClick = (e: MouseEvent) => {
      if (!dropdown.contains(e.target as Node)) {
        this.removeFormOverlay();
        document.removeEventListener('mousedown', onDocClick, true);
      }
    };
    // 다음 프레임에 등록 (현재 클릭 이벤트 무시)
    requestAnimationFrame(() => {
      document.addEventListener('mousedown', onDocClick, true);
    });

    const scrollContent = this.container.querySelector('#scroll-content');
    (scrollContent ?? this.container).appendChild(dropdown);
    this.formOverlay = dropdown;
  }

  /** Edit 입력 오버레이 */
  private showEditOverlay(sec: number, para: number, ci: number, formHit: FormObjectHitResult, pageIdx: number): void {
    this.removeFormOverlay();
    if (!formHit.bbox) return;

    const rect = this.formBboxToOverlayRect(formHit.bbox, pageIdx);

    const input = document.createElement('input');
    input.type = 'text';
    input.value = formHit.text ?? '';
    input.className = 'form-edit-input';
    input.style.left = `${rect.left}px`;
    input.style.top = `${rect.top}px`;
    input.style.width = `${rect.width}px`;
    input.style.height = `${rect.height}px`;
    input.style.fontSize = `${rect.height * 0.6}px`;

    const commit = () => {
      this.wasm.setFormValue(sec, para, ci, JSON.stringify({ text: input.value }));
      this.removeFormOverlay();
      this.afterEdit();
    };

    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        commit();
      } else if (e.key === 'Escape') {
        e.preventDefault();
        this.removeFormOverlay();
      }
    });
    input.addEventListener('blur', () => {
      commit();
    });

    const scrollContent = this.container.querySelector('#scroll-content');
    (scrollContent ?? this.container).appendChild(input);
    this.formOverlay = input;

    requestAnimationFrame(() => {
      input.focus();
      input.select();
    });
  }
}
