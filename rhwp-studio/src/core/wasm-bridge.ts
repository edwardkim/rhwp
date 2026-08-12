import init, { HwpDocument, version } from '@wasm/rhwp.js';
import * as wasmExports from '@wasm/rhwp.js';
import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import type { DocumentInfo, PageInfo, PageDef, SectionDef, PageBorderFillSettings, EndnoteShapeSettings, NoteEditInfo, CursorRect, HitTestResult, BodyFootnoteMarkerHit, FootnoteAtCursorResult, DeleteFootnoteResult, LineInfo, TableDimensions, CellInfo, CellBbox, CellProperties, TableProperties, DocumentPosition, MoveVerticalResult, SelectionRect, CharProperties, ParaProperties, CellPathEntry, CellPathLike, NavContextEntry, FieldInfoResult, BookmarkInfo, LayerRenderProfile, PageLayerTree, CanvasKitDocumentPreflight } from './types';
import { parseCanvasKitDocumentPreflight } from './canvaskit-document-preflight';
import {
  normalizeHmlSaveState,
  parseHmlSaveState,
  type HmlSaveBlocker,
  type HmlSaveState,
} from './hml-save-capability';
import {
  getSelectionRectsInCellByPathWithPageHints,
  getSelectionRectsInCellWithPageHints,
  type CellSelectionRectDocument,
  type PathCellSelectionRectDocument,
  type SelectionPageHints,
} from './selection-page-hints';
import {
  parseLocalBodyTextReplaceResult,
  type LocalBodyTextReplaceResult,
} from './local-text-replace-result';
import {
  runReportedExport,
  type DocumentExportArtifact,
  type WasmDocumentExport,
} from './export-content-loss';

/** fresh WASM binding의 reported export 표면. 구버전 모듈은 런타임 가드에서 거부한다. */
interface ReportedWasmDocument {
  exportHwpWithReport(): WasmDocumentExport;
  exportHwpWithPasswordAndReport(password: string): WasmDocumentExport;
  exportHwpxWithReport(): WasmDocumentExport;
  exportHwpxWithPasswordAndReport(password: string): WasmDocumentExport;
}

/**
 * 문단 병합으로 사라진 문단의 스코프 메타데이터 (Task #2342).
 *
 * 병합 결과에 실려 오고 undo 분할에 그대로 되돌려주는 불투명 값이다 — 스튜디오는
 * 내용을 해석하지 않는다.
 */
export type RemovedParaMeta = Record<string, unknown>;

function serializeParaMeta(meta: RemovedParaMeta | undefined): string | undefined {
  return meta && JSON.stringify(meta);
}

/** HWPX 비표준 감지 경고 리포트 (#177). */
export interface ValidationReport {
  /** 경고 총 개수 */
  count: number;
  /** 경고 종류별 요약 (key: 한국어 설명, value: 개수) */
  summary: Record<string, number>;
  /** 개별 경고 목록 */
  warnings: Array<{
    section: number;
    paragraph: number;
    kind: 'LinesegArrayEmpty' | 'LinesegUncomputed' | 'LinesegTextRunReflow';
    cell: { ctrl: number; row: number; col: number; innerPara: number } | null;
  }>;
}

export type HmlWarningCode =
  | 'UnsupportedElement'
  | 'UnsupportedAttribute'
  | 'UnsupportedEquationSemantics'
  | 'MissingResource'
  | 'ExternalResourceBlocked'
  | 'InvalidReference'
  | 'LossyConversion';

export interface HmlOpenMetadata {
  format: 'hml';
  hwpmlVersion?: string;
  encoding: 'utf-8' | 'utf-16le' | 'utf-16be';
  resourceCount: number;
  /** HML로 다시 저장 가능한지 여부 (보존 불가 요소가 있으면 false). */
  hmlSavable: boolean;
  /** hmlSavable이 false일 때, 보존할 수 없는 요소의 경로 목록. */
  saveBlockers: HmlSaveBlocker[];
  warnings: Array<{
    code: HmlWarningCode;
    xmlPath: string;
    message: string;
    preserved: boolean;
  }>;
}

export interface TableCellResizeUpdate {
  cellIdx: number;
  widthDelta?: number;
  heightDelta?: number;
  localResize?: boolean;
  renderWidth?: number;
  renderHeight?: number;
}

export interface TableTransposeResult {
  ok: boolean;
  paraIdx?: number;
  controlIdx?: number;
  sourceRows: number;
  sourceCols: number;
  targetRows: number;
  targetCols: number;
}

/** deferred cell text mutation의 pagination 경계 결과 (#2214/#2424). */
export interface DeferredFocusedCellCursorGeometry {
  baseRevision: number;
  revision: number;
  sourceCharOffset: number;
  targetCharOffset: number;
  deltaX: number;
}

export interface DeferredFocusedPagePatch {
  pageIndex: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DeferredCellTextMutationResult {
  ok: boolean;
  charOffset: number;
  paginationDeferred: boolean;
  cellFlowChanged: boolean;
  /** stable tail edit가 focused page tree의 TextLine 캐시를 직접 갱신했는지. */
  focusedPageTreePatched: boolean;
  focusedCursorGeometry?: DeferredFocusedCellCursorGeometry;
  focusedPagePatch?: DeferredFocusedPagePatch;
}

function parseDeferredFocusedCellCursorGeometry(
  value: unknown,
): DeferredFocusedCellCursorGeometry | undefined {
  if (!value || typeof value !== 'object') return undefined;
  const candidate = value as Partial<DeferredFocusedCellCursorGeometry>;
  const integers = [
    candidate.baseRevision,
    candidate.revision,
    candidate.sourceCharOffset,
    candidate.targetCharOffset,
  ];
  if (
    !integers.every((item) => Number.isSafeInteger(item) && (item as number) >= 0)
    || (candidate.revision as number) <= (candidate.baseRevision as number)
    || typeof candidate.deltaX !== 'number'
    || !Number.isFinite(candidate.deltaX)
  ) {
    return undefined;
  }
  return {
    baseRevision: candidate.baseRevision as number,
    revision: candidate.revision as number,
    sourceCharOffset: candidate.sourceCharOffset as number,
    targetCharOffset: candidate.targetCharOffset as number,
    deltaX: candidate.deltaX,
  };
}

function parseDeferredFocusedPagePatch(value: unknown): DeferredFocusedPagePatch | undefined {
  if (!value || typeof value !== 'object') return undefined;
  const candidate = value as Partial<DeferredFocusedPagePatch>;
  const numbers = [
    candidate.x,
    candidate.y,
    candidate.width,
    candidate.height,
  ];
  if (
    !Number.isSafeInteger(candidate.pageIndex)
    || (candidate.pageIndex as number) < 0
    || !numbers.every((item) => typeof item === 'number' && Number.isFinite(item))
    || (candidate.width as number) <= 0
    || (candidate.height as number) <= 0
  ) {
    return undefined;
  }
  return {
    pageIndex: candidate.pageIndex as number,
    x: candidate.x as number,
    y: candidate.y as number,
    width: candidate.width as number,
    height: candidate.height as number,
  };
}

export type DeferredPaginationStatus = 'none' | 'pending' | 'complete' | 'fallback' | 'stale';

export interface DeferredPaginationResult {
  ok: boolean;
  status: DeferredPaginationStatus;
  revision: number;
  fragmentsProcessed: number;
  pageCount: number;
}

import { fontFamilyChainForDisplay } from './font-substitution';
import type { FileSystemFileHandleLike } from '@/command/file-system-access';
import {
  connectSubsecondDevtools,
  SubsecondPatchAccumulation,
  type SubsecondWasmExports,
} from './subsecond-runtime';

/**
 * devtools 소켓의 해제 함수 — realm 하나에 소켓 하나이므로 중복 연결 guard 로도 쓴다.
 * 스튜디오에는 realm 종료 이전의 해제 시점이 없어 실제로 호출되지는 않는다.
 */
let disconnectSubsecondDevtools: (() => void) | null = null;

/**
 * CSS font 문자열에서 font-family를 추출하여 폰트 치환을 적용한다.
 *
 * 입력: 'bold 14.5px "안상수2006가는", sans-serif'
 * 출력: 'bold 14.5px "돋움", sans-serif'
 */
function substituteCssFontFamily(cssFont: string): string {
  const pxIdx = cssFont.indexOf('px ');
  if (pxIdx < 0) return cssFont;

  const prefix = cssFont.substring(0, pxIdx + 3);
  const familyPart = cssFont.substring(pxIdx + 3);

  const match = familyPart.match(/^"([^"]+)"/);
  if (!match) return cssFont;

  const fontName = match[1];
  return prefix + fontFamilyChainForDisplay(fontName, 0, 0);
}

let canvasFontSubstitutionInstalled = false;

function installCanvasFontSubstitution(): void {
  if (canvasFontSubstitutionInstalled) return;
  if (typeof CanvasRenderingContext2D === 'undefined') return;

  const proto = CanvasRenderingContext2D.prototype;
  const descriptor = Object.getOwnPropertyDescriptor(proto, 'font');
  if (!descriptor?.get || !descriptor.set || descriptor.configurable === false) return;

  Object.defineProperty(proto, 'font', {
    configurable: true,
    enumerable: descriptor.enumerable,
    get() {
      return descriptor.get!.call(this);
    },
    set(value: string) {
      descriptor.set!.call(this, substituteCssFontFamily(String(value)));
    },
  });
  canvasFontSubstitutionInstalled = true;
}

export class WasmBridge {
  private doc: HwpDocument | null = null;
  private initialized = false;
  private _fileName = 'document.hwp';
  private _currentFileHandle: FileSystemFileHandleLike | null = null;
  /** 마지막 저장본이 출력 암호로 보호됐는지 여부만 보관한다. 암호 문자열은 보관하지 않는다. */
  private _requiresPasswordForSave = false;
  private _documentDigest: string | null = null;
  /** 같은 바이트를 다시 열어도 구분되는 문서 인스턴스 세대. */
  private _documentGeneration = 0;
  /** [#3313] 외부 연결 그림 비동기 주입 완료 훅 — 주입 성공(>0)시에만 호출된다.
   * 첫 렌더 이후에 fetch 가 끝나면 뷰가 재갱신 없이는 이미지를 표시하지 못하므로,
   * main 쪽에서 뷰 갱신을 배선한다 (dirty 마킹 없는 뷰 전용 경로여야 함). */
  onExternalImagesInjected?: (injected: number) => void;

  async initialize(): Promise<void> {
    if (this.initialized) return;
    installCanvasFontSubstitution();
    this.installMeasureTextWidth();
    const wasmModule = await init();
    if (!disconnectSubsecondDevtools) {
      disconnectSubsecondDevtools = connectSubsecondDevtools(
        wasmExports as unknown as SubsecondWasmExports,
        {
          patchAccumulation: new SubsecondPatchAccumulation({
            // subsecond 세션에서는 이 모듈이 dx 가 만든 glue(`target/rhwp-subsecond-vite/`)로
            // 바뀐다. 타입은 언제나 `pkg/rhwp.d.ts` 를 보므로 memory 부재는 타입으로 못 걸러진다.
            measureHeapBytes: () => wasmModule.memory?.buffer.byteLength ?? null,
          }),
        },
      );
    }
    this.initialized = true;
    console.log(`[WasmBridge] WASM 초기화 완료 (rhwp ${version()})`);
  }

  isSubsecondHotpatchEnabled(): boolean {
    return typeof Reflect.get(wasmExports, 'subsecondProbe') === 'function';
  }

  getSubsecondProbeValue(): number | null {
    const probe = Reflect.get(wasmExports, 'subsecondProbe');
    return typeof probe === 'function' ? probe() : null;
  }

  getSubsecondPatchRevision(): string | null {
    if (!this.doc) return null;

    const doc = this.doc as unknown as {
      getSubsecondPatchRevision?: () => string;
    };
    return typeof doc.getSubsecondPatchRevision === 'function'
      ? doc.getSubsecondPatchRevision()
      : null;
  }

  invalidateSubsecondRenderCaches(): boolean {
    if (!this.doc) return false;

    const doc = this.doc as unknown as {
      invalidateSubsecondRenderCaches?: () => void;
    };
    if (typeof doc.invalidateSubsecondRenderCaches !== 'function') return false;

    doc.invalidateSubsecondRenderCaches();
    return true;
  }

  /** WASM 렌더러가 호출하는 텍스트 폭 측정 함수를 등록한다 */
  private installMeasureTextWidth(): void {
    if ((globalThis as Record<string, unknown>).measureTextWidth) return;
    let ctx: CanvasRenderingContext2D | null = null;
    let lastFont = '';
    (globalThis as Record<string, unknown>).measureTextWidth = (font: string, text: string): number => {
      if (!ctx) {
        ctx = document.createElement('canvas').getContext('2d');
      }
      const resolved = canvasFontSubstitutionInstalled ? font : substituteCssFontFamily(font);
      if (resolved !== lastFont) {
        ctx!.font = resolved;
        lastFont = resolved;
      }
      return ctx!.measureText(text).width;
    };
  }

  /**
   * 문서 IR만 해제한다. WASM 모듈 초기화 상태는 유지한다.
   * 비교 상세 창 등 보조 WasmBridge 인스턴스에서 반복 로드 시 메모리 누수를 줄이기 위해 사용한다.
   */
  releaseDocument(): void {
    if (this.doc) {
      try {
        this.doc.free();
      } catch {
        /* noop */
      }
      this.doc = null;
    }
    this._currentFileHandle = null;
    this._requiresPasswordForSave = false;
    this._documentDigest = null;
  }

  private loadDocumentAtomically(
    data: Uint8Array,
    fileName: string | undefined,
    createDocument: () => HwpDocument,
  ): DocumentInfo {
    const nextFileName = fileName ?? 'document.hwp';
    const nextDocumentDigest = `blake3:${bytesToHex(blake3(data))}`;
    let nextDoc: HwpDocument | null = null;

    try {
      nextDoc = createDocument();
      nextDoc.convertToEditable();
      this.ensureParagraphStableIdsFor(nextDoc);
      nextDoc.setFileName(nextFileName);
      const info: DocumentInfo = JSON.parse(nextDoc.getDocumentInfo());

      // 새 문서를 끝까지 준비한 뒤에만 기존 문서를 교체한다. 암호 필요·오답·손상
      // 오류에서는 현재 문서와 최근 문서 연결을 그대로 유지해야 한다 (#3474).
      const previousDoc = this.doc;
      this.doc = nextDoc;
      this._fileName = nextFileName;
      this._currentFileHandle = null;
      this._requiresPasswordForSave = false;
      this._documentDigest = nextDocumentDigest;
      this._documentGeneration += 1;
      if (previousDoc) {
        try {
          previousDoc.free();
        } catch {
          /* noop */
        }
      }
      console.log(`[WasmBridge] 문서 로드: ${info.pageCount}페이지`);

      // [Task #741 후속] 외부 file path 그림 영역 영역 dev 환경 영역 영역 fetch (basename 영역
      // 영역 영역 same dir 영역 image 영역 영역 영역 — 본 환경 dev 영역 영역 samples/ 영역
      // Vite asset). 영역 영역 영역 영역 영역 부재 영역 영역 placeholder 표시.
      void this.populateExternalImagesFromDevServer();

      return info;
    } catch (error) {
      if (nextDoc) {
        try {
          nextDoc.free();
        } catch {
          /* noop */
        }
      }
      throw error;
    }
  }

  loadDocument(data: Uint8Array, fileName?: string): DocumentInfo {
    return this.loadDocumentAtomically(data, fileName, () => new HwpDocument(data));
  }

  loadDocumentWithPassword(data: Uint8Array, password: string, fileName?: string): DocumentInfo {
    return this.loadDocumentAtomically(data, fileName, () => HwpDocument.openWithPassword(data, password));
  }

  /** [Task #741 후속] 외부 file path 그림 영역 영역 dev 서버 영역 영역 fetch + inject. */
  private async populateExternalImagesFromDevServer(): Promise<void> {
    if (!this.doc) return;
    // [#3348] /samples/ fetch는 vite dev 서버 전용(server.fs.allow). 프로덕션 빌드
    // (Pages·확장)에는 경로가 없어 실패 로그만 쌓이므로 dev 외에는 시도하지 않는다.
    // 프로덕션 사이드카 공급 UX는 #3313 잔여 범위.
    if (!import.meta.env.DEV) return;
    try {
      const basenamesJson = this.doc.getExternalImageBasenames();
      const basenames: string[] = JSON.parse(basenamesJson);
      if (basenames.length === 0) return;
      console.log(`[WasmBridge] 외부 image 영역 영역 ${basenames.length}개 영역 영역 fetch 시도`);
      let totalInjected = 0;
      for (const name of basenames) {
        try {
          const url = `/samples/${name}`;
          const res = await fetch(url);
          if (!res.ok) {
            console.warn(`[WasmBridge] 외부 image 영역 영역 영역 fetch 실패: ${url} (status=${res.status})`);
            continue;
          }
          const buf = await res.arrayBuffer();
          // [Task #741 후속] OS 절대 경로 영역 영역 X-File-Path header 영역 영역 영역 → dialog
          // 영역 영역 한컴 viewer 정합 (resolved local path 영역 영역).
          const filePathHeader = res.headers.get('X-File-Path');
          const displayPath = filePathHeader ? decodeURI(filePathHeader) : '';
          const injected = this.doc.injectExternalImage(name, new Uint8Array(buf), displayPath);
          totalInjected += injected;
          console.log(`[WasmBridge] 외부 image inject: ${name} → ${displayPath || url} (${buf.byteLength} bytes, ${injected} 영역)`);
        } catch (e) {
          console.warn(`[WasmBridge] 외부 image 영역 영역 영역: ${name}`, e);
        }
      }
      // [#3313] 주입은 첫 렌더 이후에 끝나므로, 주입이 있었으면 뷰 갱신 훅을 호출한다.
      // 훅 없이는 페이지 트리 캐시만 무효화되고 화면은 재요청 전까지 이전 프레임을 유지한다.
      if (totalInjected > 0) {
        this.onExternalImagesInjected?.(totalInjected);
      }
    } catch (e) {
      console.warn('[WasmBridge] populateExternalImagesFromDevServer 실패', e);
    }
  }

  /** 메인 뷰에 문서가 올라와 있는지(비교 보조 WasmBridge 등과 구분). */
  hasLoadedDocument(): boolean {
    return this.doc != null;
  }

  createNewDocument(): DocumentInfo {
    if (!this.doc) {
      // 아직 WASM 객체가 없으면 더미로 생성 (createEmpty → 즉시 교체)
      this.doc = HwpDocument.createEmpty();
    }
    const info: DocumentInfo = JSON.parse(this.doc.createBlankDocument());
    this.ensureParagraphStableIds();
    this._fileName = '새 문서.hwp';
    this._currentFileHandle = null;
    this._requiresPasswordForSave = false;
    this.doc.setFileName(this._fileName);
    try {
      this._documentDigest = `blake3:${bytesToHex(blake3(this.doc.exportHwp()))}`;
    } catch {
      this._documentDigest = null;
    }
    this._documentGeneration += 1;
    console.log(`[WasmBridge] 새 문서 생성: ${info.pageCount}페이지`);
    return info;
  }

  get fileName(): string {
    return this._fileName;
  }

  get documentDigest(): string | null {
    return this._documentDigest;
  }

  get documentGeneration(): number {
    return this._documentGeneration;
  }

  set fileName(name: string) {
    this._fileName = name;
    this.doc?.setFileName(name);
  }

  get currentFileHandle(): FileSystemFileHandleLike | null {
    return this._currentFileHandle;
  }

  set currentFileHandle(handle: FileSystemFileHandleLike | null) {
    this._currentFileHandle = handle;
  }

  get requiresPasswordForSave(): boolean {
    return this._requiresPasswordForSave;
  }

  set requiresPasswordForSave(value: boolean) {
    this._requiresPasswordForSave = value;
  }

  get isNewDocument(): boolean {
    return this._fileName === '새 문서.hwp';
  }

  /**
   * [#4180] 바이트 생산 직전 호출되는 훅 — 저장 시점 캐럿 스탬핑용 (main.ts 가 등록).
   * 편집별 스탬핑은 "마지막 본문 편집 위치"를 남겨 열기 캐럿이 엉뚱한 페이지로
   * 복원됐다. 저장/autosave/비교/히스토리 등 모든 export 경로가 이 브리지 메서드를
   * 지나므로 여기가 단일 지점이다.
   */
  onBeforeExport: (() => void) | null = null;

  exportHwp(): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.onBeforeExport?.();
    return this.doc.exportHwp();
  }

  /**
   * 명시적 저장용 HWP artifact. 바이트와 content-loss 보고서는 같은 WASM 결과에 속한다.
   * byte-only `exportHwp()`는 autosave/embed/history/compare/hwpctl/digest 호환 표면이며
   * 보고서를 전달하지 않는다.
   */
  exportHwpWithReport(): DocumentExportArtifact {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.onBeforeExport?.();
    const exportFn = (this.doc as unknown as Partial<ReportedWasmDocument>).exportHwpWithReport;
    if (typeof exportFn !== 'function') {
      throw new Error('현재 WASM 빌드는 HWP 내용 손실 보고를 지원하지 않습니다');
    }
    return runReportedExport(
      () => exportFn.call(this.doc) as WasmDocumentExport,
    );
  }

  exportHwpWithPassword(password: string): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.onBeforeExport?.();
    return this.doc.exportHwpWithPassword(password);
  }

  exportHwpWithPasswordAndReport(password: string): DocumentExportArtifact {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.onBeforeExport?.();
    const exportFn = (this.doc as unknown as Partial<ReportedWasmDocument>)
      .exportHwpWithPasswordAndReport;
    if (typeof exportFn !== 'function') {
      throw new Error('현재 WASM 빌드는 비밀번호 HWP 내용 손실 보고를 지원하지 않습니다');
    }
    return runReportedExport(
      () => exportFn.call(this.doc, password) as WasmDocumentExport,
    );
  }

  exportHwpx(): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.onBeforeExport?.();
    return this.doc.exportHwpx();
  }

  /** 명시적 저장용 HWPX artifact. byte-only 보조 소비자와 의도적으로 분리한다. */
  exportHwpxWithReport(): DocumentExportArtifact {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.onBeforeExport?.();
    const exportFn = (this.doc as unknown as Partial<ReportedWasmDocument>).exportHwpxWithReport;
    if (typeof exportFn !== 'function') {
      throw new Error('현재 WASM 빌드는 HWPX 내용 손실 보고를 지원하지 않습니다');
    }
    return runReportedExport(
      () => exportFn.call(this.doc) as WasmDocumentExport,
    );
  }

  exportHwpxWithPassword(password: string): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportHwpxWithPassword(password);
  }

  exportHwpxWithPasswordAndReport(password: string): DocumentExportArtifact {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.onBeforeExport?.();
    const exportFn = (this.doc as unknown as Partial<ReportedWasmDocument>)
      .exportHwpxWithPasswordAndReport;
    if (typeof exportFn !== 'function') {
      throw new Error('현재 WASM 빌드는 비밀번호 HWPX 내용 손실 보고를 지원하지 않습니다');
    }
    return runReportedExport(
      () => exportFn.call(this.doc, password) as WasmDocumentExport,
    );
  }

  /** HML로 저장 (보존 불가 요소가 있으면 던진다). 현재 WASM 빌드가 지원하지 않으면 던진다. */
  exportHml(): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const exportFn = (this.doc as any).exportHml?.bind(this.doc);
    if (!exportFn) throw new Error('현재 WASM 빌드는 HML 저장을 지원하지 않습니다');
    return exportFn();
  }

  hasHmlExportCapability(): boolean {
    return typeof (this.doc as any)?.exportHml === 'function';
  }

  getHmlSaveState(): HmlSaveState {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const raw = (this.doc as any).getHmlSaveState?.();
    if (typeof raw !== 'string') throw new Error('HML 저장 정보를 확인할 수 없습니다');
    const saveState = parseHmlSaveState(JSON.parse(raw));
    if (!saveState) throw new Error('HML 저장 정보를 확인할 수 없습니다');
    return saveState;
  }

  /** HWP 직렬화 + 자기 재로드 검증 메타데이터를 JSON 문자열로 반환 (#178). */
  exportHwpVerify(): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportHwpVerify();
  }

  getSourceFormat(): string {
    return this.doc?.getSourceFormat?.() ?? 'hwp';
  }

  getHmlOpenMetadata(): HmlOpenMetadata | null {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const raw = (this.doc as any).getHmlOpenMetadata?.();
    if (!raw) return null;
    try {
      const parsed = JSON.parse(raw) as unknown;
      const saveState = normalizeHmlSaveState(parsed);
      if (!saveState) return null;
      return {
        ...(parsed as HmlOpenMetadata),
        hmlSavable: saveState.hmlSavable,
        saveBlockers: saveState.saveBlockers,
        warnings: Array.isArray((parsed as HmlOpenMetadata).warnings)
          ? (parsed as HmlOpenMetadata).warnings
          : [],
      };
    } catch {
      return null;
    }
  }

  /** HWPX 비표준 감지 경고 조회 (#177). */
  getValidationWarnings(): ValidationReport {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const raw = (this.doc as any).getValidationWarnings?.();
    if (!raw) return { count: 0, summary: {}, warnings: [] };
    try {
      return JSON.parse(raw);
    } catch {
      return { count: 0, summary: {}, warnings: [] };
    }
  }

  /** 사용자 명시 요청에 의한 lineseg reflow (#177). 반환: reflow된 문단 수. */
  reflowLinesegs(): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).reflowLinesegs?.() ?? 0;
  }

  get pageCount(): number {
    return this.doc?.pageCount() ?? 0;
  }

  beginDeferredPagination(fragmentBudget = 1): DeferredPaginationResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { beginDeferredPagination?: (budget: number) => string };
    if (typeof d.beginDeferredPagination !== 'function') {
      return {
        ok: true,
        status: 'fallback',
        revision: 0,
        fragmentsProcessed: 0,
        pageCount: this.pageCount,
      };
    }
    return JSON.parse(d.beginDeferredPagination(Math.max(1, Math.trunc(fragmentBudget))));
  }

  stepDeferredPagination(fragmentBudget = 1): DeferredPaginationResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { stepDeferredPagination?: (budget: number) => string };
    if (typeof d.stepDeferredPagination !== 'function') {
      return {
        ok: true,
        status: 'fallback',
        revision: 0,
        fragmentsProcessed: 0,
        pageCount: this.pageCount,
      };
    }
    return JSON.parse(d.stepDeferredPagination(Math.max(1, Math.trunc(fragmentBudget))));
  }

  cancelDeferredPagination(): boolean {
    if (!this.doc) return false;
    const d = this.doc as unknown as { cancelDeferredPagination?: () => boolean };
    return typeof d.cancelDeferredPagination === 'function'
      ? d.cancelDeferredPagination()
      : false;
  }

  flushDeferredPagination(): DeferredPaginationResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { flushDeferredPagination?: () => string };
    if (typeof d.flushDeferredPagination !== 'function') {
      return {
        ok: true,
        status: 'fallback',
        revision: 0,
        fragmentsProcessed: 0,
        pageCount: this.pageCount,
      };
    }
    return JSON.parse(d.flushDeferredPagination());
  }

  getSectionCount(): number {
    return this.doc?.getSectionCount() ?? 0;
  }

  getPageInfo(pageNum: number): PageInfo {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getPageInfo(pageNum));
  }

  refreshLayout(): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    try {
      (this.doc as any).refreshLayout?.();
    } catch (e) {
      console.warn('[WasmBridge] refreshLayout failed:', e);
    }
  }

  getDocumentInfo(): DocumentInfo {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getDocumentInfo());
  }

  getPageDef(sectionIdx: number): PageDef {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getPageDef(sectionIdx));
  }

  setPageDef(sectionIdx: number, pageDef: PageDef): { ok: boolean; pageCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setPageDef(sectionIdx, JSON.stringify(pageDef)));
  }

  getSectionDef(sectionIdx: number): SectionDef {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getSectionDef(sectionIdx));
  }

  setSectionDef(sectionIdx: number, sectionDef: SectionDef): { ok: boolean; pageCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setSectionDef(sectionIdx, JSON.stringify(sectionDef)));
  }

  setSectionDefAll(sectionDef: SectionDef): { ok: boolean; pageCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setSectionDefAll(JSON.stringify(sectionDef)));
  }

  getPageBorderFill(sectionIdx: number): PageBorderFillSettings {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getPageBorderFill(sectionIdx));
  }

  setPageBorderFill(sectionIdx: number, settings: PageBorderFillSettings): { ok: boolean; pageCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).setPageBorderFill(sectionIdx, JSON.stringify(settings)));
  }

  renderPageToCanvas(pageNum: number, canvas: HTMLCanvasElement, scale = 1.0): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.renderPageToCanvas(pageNum, canvas, scale);
  }

  /**
   * 다층 레이어 필터를 적용한 Canvas 렌더링 (Task #516, Stage 5.2).
   *
   * @param layerKind 'all' = 모든 PaintOp, 'background' = page background layer,
   *                  'flow' = 본문 layer (BehindText/InFrontOfText 제외),
   *                  'flow-dynamic' = 본문 layer 중 Image/RawSvg 제외,
   *                  'flow-static' = page background + 본문 Image/RawSvg layer,
   *                  'behind' = BehindText overlay, 'front' = InFrontOfText overlay
   */
  renderPageToCanvasFiltered(
    pageNum: number,
    canvas: HTMLCanvasElement,
    scale: number,
    layerKind: 'all' | 'background' | 'flow' | 'flow-dynamic' | 'flow-static' | 'behind' | 'front',
    profile: LayerRenderProfile = 'screen',
  ): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      renderPageToCanvasFiltered?: (p: number, c: HTMLCanvasElement, s: number, k: string) => void;
      renderPageToCanvasFilteredWithProfile?: (
        p: number,
        c: HTMLCanvasElement,
        s: number,
        k: string,
        profile: string,
      ) => void;
    };
    if (typeof d.renderPageToCanvasFilteredWithProfile === 'function') {
      d.renderPageToCanvasFilteredWithProfile(pageNum, canvas, scale, layerKind, profile);
      return;
    }
    if (profile !== 'screen') {
      throw new Error('[WasmBridge] 현재 WASM은 profile별 Canvas2D 렌더링을 지원하지 않습니다');
    }
    if (typeof d.renderPageToCanvasFiltered === 'function') {
      d.renderPageToCanvasFiltered(pageNum, canvas, scale, layerKind);
      return;
    }
    // 구버전 WASM(public/rhwp.js 등): 레이어 필터 API 없음 → 전체 캔버스 렌더로 폴백
    this.doc.renderPageToCanvas(pageNum, canvas, scale);
  }

  /** 기존 Canvas를 유지한 채 page-space 일부만 filtered replay한다 (#3137 Stage 4). */
  renderPagePatchToCanvasFiltered(
    pageNum: number,
    canvas: HTMLCanvasElement,
    scale: number,
    layerKind: 'flow' | 'flow-dynamic',
    patch: DeferredFocusedPagePatch,
    profile: LayerRenderProfile = 'screen',
  ): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      renderPagePatchToCanvasFilteredWithProfile?: (
        p: number,
        c: HTMLCanvasElement,
        s: number,
        k: string,
        profile: string,
        x: number,
        y: number,
        width: number,
        height: number,
      ) => void;
    };
    if (typeof d.renderPagePatchToCanvasFilteredWithProfile !== 'function') {
      throw new Error('[WasmBridge] 현재 WASM은 focused page patch 렌더링을 지원하지 않습니다');
    }
    d.renderPagePatchToCanvasFilteredWithProfile(
      pageNum,
      canvas,
      scale,
      layerKind,
      profile,
      patch.x,
      patch.y,
      patch.width,
      patch.height,
    );
  }

  /**
   * PageLayerTree JSON 가져오기 (Task #516, Stage 5.2).
   * BehindText/InFrontOfText 그림의 메타정보 (bin_id, bbox, transform, effect, brightness, contrast,
   * watermark, wrap) 를 추출하여 overlay 생성에 사용.
   */
  getPageLayerTree(pageNum: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { getPageLayerTree?: (p: number) => string };
    if (typeof d.getPageLayerTree === 'function') {
      return d.getPageLayerTree(pageNum);
    }
    return '{"pageWidth":0,"pageHeight":0,"profile":"screen","buildOptions":{"showTransparentBorders":false,"clipEnabled":true},"debugOptions":{"debugOverlay":false},"outputOptions":{"showParagraphMarks":false,"showControlCodes":false,"showTransparentBorders":false,"clipEnabled":true,"debugOverlay":false},"root":{"kind":"leaf","bounds":{"x":0,"y":0,"width":0,"height":0},"ops":[]}}';
  }

  /**
   * 페이지가 그리는 그림들의 신원 키만 받는다 (Task #3315).
   *
   * "그림이 그대로면 앞서 만든 디코드 결과를 재사용"을 판정하는 서명이다. 같은 판정을
   * PageLayerTree JSON 으로 하면 그림 1장에 수 MB 를 다시 받아 훑어야 한다.
   * 구형 WASM(키 조회 미지원)에서는 `null` — 호출부는 종전대로 매번 다시 계산한다.
   */
  getPageSourceImageKeys(pageNum: number): string | null {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { getPageSourceImageKeys?: (p: number) => string };
    if (typeof d.getPageSourceImageKeys !== 'function') return null;
    try {
      return d.getPageSourceImageKeys(pageNum);
    } catch {
      return null;
    }
  }

  /**
   * 본문(flow) 그림의 배치 정보만 받는다 (Task #3315).
   *
   * 전체 레이어 트리를 받아 flow 그림을 걸러내면 그림 1장에 수 MB 를 편집마다 옮긴다.
   * 이 질의는 바이트를 빼고 bbox·잘림·효과·신원 키만 주므로 수백 바이트다. 바이트는
   * `getSourceImageBytes(key)` 로 그림이 바뀔 때만 따로 받는다.
   *
   * 구형 WASM(미지원)에서는 `null` — 호출부는 종전의 전체 트리 경로로 되돌아간다.
   */
  getPageFlowImageOps(pageNum: number): string | null {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { getPageFlowImageOps?: (p: number) => string };
    if (typeof d.getPageFlowImageOps !== 'function') return null;
    try {
      return d.getPageFlowImageOps(pageNum);
    } catch {
      return null;
    }
  }

  /**
   * 그림 신원 키로 바이트를 받는다 (Task #3315).
   *
   * 키를 풀 수 없으면 `null` — 세대가 바뀐 낡은 키이거나 없는 그림이다. 호출부는 종전
   * 경로로 되돌아가야 한다.
   */
  getSourceImageBytes(key: string): Uint8Array | null {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { getSourceImageBytes?: (k: string) => Uint8Array };
    if (typeof d.getSourceImageBytes !== 'function') return null;
    try {
      return d.getSourceImageBytes(key);
    } catch {
      return null;
    }
  }

  getPageLayerTreeObject(pageNum: number, profile: LayerRenderProfile = 'screen'): PageLayerTree {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      getPageLayerTreeWithProfile?: (p: number, profile: string) => string;
      getPageLayerTree?: (p: number) => string;
    };
    const hasProfileApi = typeof d.getPageLayerTreeWithProfile === 'function';
    if (!hasProfileApi && profile !== 'screen') {
      throw new Error('[WasmBridge] 현재 WASM은 profile별 PageLayerTree를 지원하지 않습니다');
    }
    const json = hasProfileApi
      ? d.getPageLayerTreeWithProfile!(pageNum, profile)
      : this.getPageLayerTree(pageNum);
    let parsed: unknown;
    try {
      parsed = JSON.parse(json);
    } catch (error) {
      throw new Error(`[WasmBridge] PageLayerTree JSON parse 실패 (page=${pageNum}): ${error}`);
    }
    if (!parsed || typeof parsed !== 'object') {
      throw new Error(`[WasmBridge] PageLayerTree JSON shape 오류 (page=${pageNum}): object가 아닙니다`);
    }
    const tree = parsed as Partial<PageLayerTree> & { layers?: unknown };
    if (!tree.root || typeof tree.root !== 'object' || !('kind' in tree.root)) {
      if (Array.isArray(tree.layers)) {
        const pageInfo = this.getPageInfo(pageNum);
        return {
          pageWidth: pageInfo.width,
          pageHeight: pageInfo.height,
          profile,
          buildOptions: {
            showTransparentBorders: false,
            clipEnabled: true,
          },
          debugOptions: {
            debugOverlay: false,
          },
          outputOptions: {
            showParagraphMarks: false,
            showControlCodes: false,
            showTransparentBorders: false,
            clipEnabled: true,
            debugOverlay: false,
          },
          root: {
            kind: 'leaf',
            bounds: { x: 0, y: 0, width: pageInfo.width, height: pageInfo.height },
            ops: [],
          },
        };
      }
      throw new Error(`[WasmBridge] PageLayerTree JSON shape 오류 (page=${pageNum}): root.kind가 없습니다`);
    }
    const rootKind = (tree.root as { kind?: unknown }).kind;
    if (rootKind !== 'group' && rootKind !== 'clipRect' && rootKind !== 'leaf') {
      throw new Error(`[WasmBridge] PageLayerTree JSON shape 오류 (page=${pageNum}): 알 수 없는 root.kind=${String(rootKind)}`);
    }
    if (tree.profile !== profile) {
      throw new Error(
        `[WasmBridge] PageLayerTree profile 불일치 (page=${pageNum}): requested=${profile}, actual=${String(tree.profile)}`,
      );
    }
    const outputOptions = tree.outputOptions ?? {};
    const buildOptions = tree.buildOptions ?? {};
    buildOptions.showTransparentBorders ??= outputOptions.showTransparentBorders ?? false;
    buildOptions.clipEnabled ??= outputOptions.clipEnabled ?? true;
    const debugOptions = tree.debugOptions ?? {};
    debugOptions.debugOverlay ??= outputOptions.debugOverlay ?? false;
    outputOptions.showParagraphMarks ??= false;
    outputOptions.showControlCodes ??= false;
    outputOptions.showTransparentBorders ??= buildOptions.showTransparentBorders;
    outputOptions.clipEnabled ??= buildOptions.clipEnabled;
    outputOptions.debugOverlay ??= debugOptions.debugOverlay;
    tree.outputOptions = outputOptions;
    tree.buildOptions = buildOptions;
    tree.debugOptions = debugOptions;
    return tree as PageLayerTree;
  }

  clearLayerResourceCache(): void {
    /* Reserved for JS-value resource transport builds. JSON export is self-contained. */
  }

  getCanvasKitReplayPlan(
    pageNum: number,
    mode: 'default' | 'compat' = 'default',
    profile: LayerRenderProfile = 'screen',
  ): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      getCanvasKitReplayPlan?: (p: number, mode: string) => string;
      getCanvasKitReplayPlanWithProfile?: (p: number, mode: string, profile: string) => string;
    };
    if (typeof d.getCanvasKitReplayPlanWithProfile === 'function') {
      return d.getCanvasKitReplayPlanWithProfile(pageNum, mode, profile);
    }
    if (profile !== 'screen') {
      throw new Error('[WasmBridge] 현재 WASM은 profile별 CanvasKit replay plan을 지원하지 않습니다');
    }
    if (typeof d.getCanvasKitReplayPlan === 'function') {
      return d.getCanvasKitReplayPlan(pageNum, mode);
    }
    return JSON.stringify({
      mode,
      hiddenCanvas2dOverlayAllowed: false,
      directReplayRequired: true,
      summary: {
        totalItems: 0,
        directItems: 0,
        directRequiredItems: 0,
        compatOverlayItems: 0,
        textFallbackItems: 0,
        unsupportedItems: 0,
        hiddenOverlayViolations: 0,
      },
      items: [],
      textVariants: [],
      requiredFontFamilies: [],
      requiredFontFamiliesComplete: true,
    });
  }

  getCanvasKitDocumentPreflight(
    mode: 'default' | 'compat' = 'default',
    profile: LayerRenderProfile = 'screen',
  ): CanvasKitDocumentPreflight {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      getCanvasKitDocumentPreflight?: (mode: string, profile: string) => string;
    };
    if (typeof d.getCanvasKitDocumentPreflight !== 'function') {
      throw new Error('[WasmBridge] 현재 WASM은 CanvasKit document preflight를 지원하지 않습니다');
    }
    return parseCanvasKitDocumentPreflight(
      d.getCanvasKitDocumentPreflight(mode, profile),
      '[WasmBridge] CanvasKit document preflight',
    );
  }

  getPageOverlayImages(pageNum: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { getPageOverlayImages?: (p: number) => string };
    if (typeof d.getPageOverlayImages === 'function') {
      return d.getPageOverlayImages(pageNum);
    }
    return '';
  }

  renderPageSvg(pageNum: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.renderPageSvg(pageNum);
  }

  renderPageSvgWithProfile(pageNum: number, profile: LayerRenderProfile): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      renderPageSvgWithProfile?: (pageNum: number, profile: string) => string;
    };
    if (typeof d.renderPageSvgWithProfile !== 'function') {
      throw new Error('[WasmBridge] 현재 WASM은 profile별 SVG 렌더링을 지원하지 않습니다');
    }
    return d.renderPageSvgWithProfile(pageNum, profile);
  }

  getCursorRect(sec: number, para: number, charOffset: number): CursorRect {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCursorRect(sec, para, charOffset));
  }

  getCursorRectOnLine(
    sec: number,
    para: number,
    lineIndex: number,
    atEnd: boolean,
    parentPara: number,
    controlIdx: number,
    cellIdx: number,
    cellParaIdx: number,
  ): CursorRect {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const getRectOnLine = (this.doc as any).getCursorRectOnLine;
    if (typeof getRectOnLine !== 'function') {
      throw new Error('getCursorRectOnLine API를 사용할 수 없습니다');
    }
    return JSON.parse(getRectOnLine.call(
      this.doc,
      sec,
      para,
      lineIndex,
      atEnd,
      parentPara,
      controlIdx,
      cellIdx,
      cellParaIdx,
    ));
  }

  hitTest(pageNum: number, x: number, y: number): HitTestResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.hitTest(pageNum, x, y));
  }

  hitTestBodyFootnoteMarker(pageNum: number, x: number, y: number): BodyFootnoteMarkerHit {
    if (!this.doc) return { hit: false };
    const hitTest = (this.doc as any).hitTestBodyFootnoteMarker;
    if (typeof hitTest !== 'function') return { hit: false };
    return JSON.parse(hitTest.call(this.doc, pageNum, x, y));
  }

  insertText(sec: number, para: number, charOffset: number, text: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.insertText(sec, para, charOffset, text);
  }

  replaceBodyTextLocal(
    sec: number,
    para: number,
    charOffset: number,
    deleteCount: number,
    text: string,
  ): LocalBodyTextReplaceResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const doc = this.doc as unknown as {
      replaceBodyTextLocal?: (
        sec: number,
        para: number,
        charOffset: number,
        deleteCount: number,
        text: string,
      ) => string;
    };
    if (typeof doc.replaceBodyTextLocal === 'function') {
      return parseLocalBodyTextReplaceResult(
        doc.replaceBodyTextLocal(sec, para, charOffset, deleteCount, text),
      );
    }
    if (deleteCount > 0) {
      this.doc.deleteText(sec, para, charOffset, deleteCount);
    }
    if (text.length > 0) {
      this.doc.insertText(sec, para, charOffset, text);
    }
    return {
      ok: true,
      charOffset: charOffset + [...text].length,
      documentPaginationPending: false,
      flowChanged: true,
    };
  }

  deleteText(sec: number, para: number, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.deleteText(sec, para, charOffset, count);
  }

  splitParagraph(sec: number, para: number, charOffset: number, removedParaMeta?: RemovedParaMeta): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.splitParagraph(sec, para, charOffset, serializeParaMeta(removedParaMeta));
  }

  insertPageBreak(sec: number, para: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).insertPageBreak(sec, para, charOffset);
  }

  insertColumnBreak(sec: number, para: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).insertColumnBreak(sec, para, charOffset);
  }

  getColumnDef(sec: number): { columnCount: number; columnType: number; sameWidth: boolean; spacing: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getColumnDef(sec));
  }

  insertNewNumber(sec: number, para: number, charOffset: number, startNum: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).insertNewNumber(sec, para, charOffset, startNum);
  }

  setColumnDef(sec: number, columnCount: number, columnType: number, sameWidth: number, spacingHu: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setColumnDef(sec, columnCount, columnType, sameWidth, spacingHu);
  }

  mergeParagraph(sec: number, para: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.mergeParagraph(sec, para);
  }

  splitParagraphInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, removedParaMeta?: RemovedParaMeta): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.splitParagraphInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, serializeParaMeta(removedParaMeta));
  }

  mergeParagraphInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.mergeParagraphInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx);
  }

  getTextRange(sec: number, para: number, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getTextRange(sec, para, charOffset, count);
  }

  getParagraphLength(sec: number, para: number): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getParagraphLength(sec, para);
  }

  getParagraphCount(sec: number): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getParagraphCount(sec);
  }

  getParagraphStableId(sec: number, para: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { getParagraphStableId?: (a: number, b: number) => string };
    if (typeof d.getParagraphStableId !== 'function') return '';
    return d.getParagraphStableId(sec, para) ?? '';
  }

  private ensureParagraphStableIdsFor(document: HwpDocument): void {
    const d = document as unknown as { ensureParagraphStableIds?: () => void };
    if (typeof d.ensureParagraphStableIds === 'function') {
      try {
        d.ensureParagraphStableIds();
      } catch (e) {
        console.warn('[WasmBridge] ensureParagraphStableIds skipped:', e);
      }
    }
  }

  /** 비교·스냅샷 생성 요청 시 현재 문서의 stable_id를 다시 보정한다. */
  ensureParagraphStableIds(): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.ensureParagraphStableIdsFor(this.doc);
  }

  /** 디버그: `JSON.parse(bridge.debugDumpStableIds(0,0,12))` 등 분할 직후 등 stable_id 확인 */
  debugDumpStableIds(sec: number, startPara: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { debugDumpStableIds?: (a: number, b: number, c: number) => string };
    if (typeof d.debugDumpStableIds !== 'function') return '[]';
    return d.debugDumpStableIds(sec, startPara, count) ?? '[]';
  }

  /** 문단에 텍스트박스 Shape 컨트롤이 있으면 control_index, 없으면 -1 */
  getTextBoxControlIndex(sec: number, para: number): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getTextBoxControlIndex(sec, para);
  }

  /** 문서 트리에서 다음 편집 가능한 컨트롤/본문을 찾는다. delta=+1(앞)/-1(뒤) */
  findNextEditableControl(sec: number, para: number, ctrlIdx: number, delta: number): { type: string; sec: number; para: number; ci: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.findNextEditableControl(sec, para, ctrlIdx, delta));
  }

  /** 커서에서 이전 방향으로 가장 가까운 선택 가능 컨트롤을 찾는다 (F11 키) */
  findNearestControlBackward(sec: number, para: number, charOffset: number): { type: string; sec: number; para: number; ci: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.findNearestControlBackward(sec, para, charOffset));
  }

  /** 현재 위치 이후의 가장 가까운 선택 가능 컨트롤 (Shift+F11) */
  findNearestControlForward(sec: number, para: number, charOffset: number): { type: string; sec: number; para: number; ci: number; charPos?: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).findNearestControlForward(sec, para, charOffset));
  }

  /** 문단 내 컨트롤의 텍스트 위치 배열 반환 */
  getControlTextPositions(sec: number, para: number): number[] {
    if (!this.doc) return [];
    try {
      return JSON.parse((this.doc as any).getControlTextPositions(sec, para));
    } catch { return []; }
  }

  /** 문서 트리 DFS 기반 다음/이전 편집 가능 위치 반환 */
  navigateNextEditable(
    sec: number, para: number, charOffset: number, delta: number,
    contextJson: string,
  ): { type: string; sec: number; para: number; charOffset: number; context: NavContextEntry[] } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.navigateNextEditable(sec, para, charOffset, delta, contextJson));
  }

  // ─── 셀 편집 API ─────────────────────────────────────────

  getCursorRectInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number): CursorRect {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCursorRectInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset));
  }

  insertTextInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, text: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.insertTextInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, text);
  }

  insertTextInCellDeferredPagination(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, text: string): DeferredCellTextMutationResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      insertTextInCellDeferredPagination?: (
        sec: number,
        parentPara: number,
        controlIdx: number,
        cellIdx: number,
        cellParaIdx: number,
        charOffset: number,
        text: string,
      ) => string;
    };
    let raw: string;
    let paginationDeferred = false;
    if (typeof d.insertTextInCellDeferredPagination === 'function') {
      raw = d.insertTextInCellDeferredPagination(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, text);
      paginationDeferred = true;
    } else {
      raw = this.doc.insertTextInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, text);
    }
    const parsed = JSON.parse(raw) as Partial<DeferredCellTextMutationResult>;
    const parsedCharOffset = parsed.charOffset;
    if (
      parsed.ok !== true ||
      typeof parsedCharOffset !== 'number' ||
      !Number.isInteger(parsedCharOffset)
    ) {
      throw new Error('잘못된 deferred cell text insert 결과');
    }
    return {
      ok: true,
      charOffset: parsedCharOffset,
      paginationDeferred,
      // Stage 3 이전 deferred API는 신호가 없다. mutation 후 예외로
      // history/cursor를 놓치지 않도록 누락 시 보수적 경계 flush로 복구한다.
      cellFlowChanged: paginationDeferred && parsed.cellFlowChanged !== false,
      focusedPageTreePatched:
        paginationDeferred && parsed.focusedPageTreePatched === true,
      ...(paginationDeferred
        ? {
            focusedCursorGeometry: parseDeferredFocusedCellCursorGeometry(
              parsed.focusedCursorGeometry,
            ),
            focusedPagePatch: parseDeferredFocusedPagePatch(parsed.focusedPagePatch),
          }
        : {}),
    };
  }

  replaceTextInCellDeferredPagination(
    sec: number,
    parentPara: number,
    controlIdx: number,
    cellIdx: number,
    cellParaIdx: number,
    charOffset: number,
    deleteCount: number,
    text: string,
  ): DeferredCellTextMutationResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      replaceTextInCellDeferredPagination?: (
        sec: number,
        parentPara: number,
        controlIdx: number,
        cellIdx: number,
        cellParaIdx: number,
        charOffset: number,
        deleteCount: number,
        text: string,
      ) => string;
    };

    let raw: string;
    let paginationDeferred = false;
    if (typeof d.replaceTextInCellDeferredPagination === 'function') {
      raw = d.replaceTextInCellDeferredPagination(
        sec,
        parentPara,
        controlIdx,
        cellIdx,
        cellParaIdx,
        charOffset,
        deleteCount,
        text,
      );
      paginationDeferred = true;
    } else {
      if (deleteCount > 0) {
        raw = this.doc.deleteTextInCell(
          sec,
          parentPara,
          controlIdx,
          cellIdx,
          cellParaIdx,
          charOffset,
          deleteCount,
        );
      } else {
        raw = JSON.stringify({ ok: true, charOffset });
      }
      if (text.length > 0) {
        raw = this.doc.insertTextInCell(
          sec,
          parentPara,
          controlIdx,
          cellIdx,
          cellParaIdx,
          charOffset,
          text,
        );
      }
    }

    const parsed = JSON.parse(raw) as Partial<DeferredCellTextMutationResult>;
    const parsedCharOffset = parsed.charOffset;
    if (
      parsed.ok !== true ||
      typeof parsedCharOffset !== 'number' ||
      !Number.isInteger(parsedCharOffset)
    ) {
      throw new Error('잘못된 deferred cell text replace 결과');
    }
    return {
      ok: true,
      charOffset: parsedCharOffset,
      paginationDeferred,
      cellFlowChanged: paginationDeferred && parsed.cellFlowChanged !== false,
      focusedPageTreePatched:
        paginationDeferred && parsed.focusedPageTreePatched === true,
      ...(paginationDeferred
        ? {
            focusedCursorGeometry: parseDeferredFocusedCellCursorGeometry(
              parsed.focusedCursorGeometry,
            ),
            focusedPagePatch: parseDeferredFocusedPagePatch(parsed.focusedPagePatch),
          }
        : {}),
    };
  }

  deleteTextInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.deleteTextInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, count);
  }

  deleteTextInCellDeferredPagination(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, count: number): DeferredCellTextMutationResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      deleteTextInCellDeferredPagination?: (
        sec: number,
        parentPara: number,
        controlIdx: number,
        cellIdx: number,
        cellParaIdx: number,
        charOffset: number,
        count: number,
      ) => string;
    };
    let raw: string;
    let paginationDeferred = false;
    if (typeof d.deleteTextInCellDeferredPagination === 'function') {
      raw = d.deleteTextInCellDeferredPagination(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, count);
      paginationDeferred = true;
    } else {
      raw = this.doc.deleteTextInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, count);
    }
    const parsed = JSON.parse(raw) as Partial<DeferredCellTextMutationResult>;
    const parsedCharOffset = parsed.charOffset;
    if (
      parsed.ok !== true ||
      typeof parsedCharOffset !== 'number' ||
      !Number.isInteger(parsedCharOffset)
    ) {
      throw new Error('잘못된 deferred cell text delete 결과');
    }
    return {
      ok: true,
      charOffset: parsedCharOffset,
      paginationDeferred,
      cellFlowChanged: paginationDeferred && parsed.cellFlowChanged !== false,
      focusedPageTreePatched:
        paginationDeferred && parsed.focusedPageTreePatched === true,
      ...(paginationDeferred
        ? {
            focusedCursorGeometry: parseDeferredFocusedCellCursorGeometry(
              parsed.focusedCursorGeometry,
            ),
            focusedPagePatch: parseDeferredFocusedPagePatch(parsed.focusedPagePatch),
          }
        : {}),
    };
  }

  // ─── 중첩 표 path 기반 편집 API ──────────────────────────

  insertTextInCellByPath(sec: number, parentPara: number, pathJson: string, charOffset: number, text: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).insertTextInCellByPath(sec, parentPara, pathJson, charOffset, text);
  }

  deleteTextInCellByPath(sec: number, parentPara: number, pathJson: string, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).deleteTextInCellByPath(sec, parentPara, pathJson, charOffset, count);
  }

  /** deleteRangeInCell 의 cellPath 변형 — 중첩 표 셀의 선택 삭제가 최내곽 셀을 대상으로 한다. */
  deleteRangeInCellByPath(sec: number, parentPara: number, pathJson: string, startPara: number, startOffset: number, endPara: number, endOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).deleteRangeInCellByPath(sec, parentPara, pathJson, startPara, startOffset, endPara, endOffset);
  }

  splitParagraphInCellByPath(sec: number, parentPara: number, pathJson: string, charOffset: number, removedParaMeta?: RemovedParaMeta): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).splitParagraphInCellByPath(sec, parentPara, pathJson, charOffset, serializeParaMeta(removedParaMeta));
  }

  mergeParagraphInCellByPath(sec: number, parentPara: number, pathJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).mergeParagraphInCellByPath(sec, parentPara, pathJson);
  }

  getTextInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getTextInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, count);
  }

  getTextInCellByPath(sec: number, parentPara: number, pathJson: string, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).getTextInCellByPath(sec, parentPara, pathJson, charOffset, count);
  }

  getCellParagraphLength(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getCellParagraphLength(sec, parentPara, controlIdx, cellIdx, cellParaIdx);
  }

  getCellParagraphCount(sec: number, parentPara: number, controlIdx: number, cellIdx: number): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getCellParagraphCount(sec, parentPara, controlIdx, cellIdx);
  }

  getCellParagraphCountByPath(sec: number, parentPara: number, pathJson: string): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getCellParagraphCountByPath(sec, parentPara, pathJson);
  }

  getCellParagraphLengthByPath(sec: number, parentPara: number, pathJson: string): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getCellParagraphLengthByPath(sec, parentPara, pathJson);
  }

  getCellTextDirection(sec: number, parentPara: number, controlIdx: number, cellIdx: number): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getCellTextDirection(sec, parentPara, controlIdx, cellIdx);
  }

  // ─── 커서 이동 API ─────────────────────────────────────────

  getLineInfo(sec: number, para: number, charOffset: number): LineInfo {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getLineInfo(sec, para, charOffset));
  }

  getLineInfoInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number): LineInfo {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getLineInfoInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset));
  }

  getCaretPosition(): DocumentPosition | null {
    if (!this.doc) return null;
    try {
      return JSON.parse(this.doc.getCaretPosition());
    } catch {
      return null;
    }
  }

  /** [#4180] 저장 시점 캐럿 스탬핑 — 범위 밖 위치는 wasm 쪽에서 무시된다. */
  setCaretPosition(sec: number, para: number, charOffset: number): void {
    if (!this.doc) return;
    try {
      this.doc.setCaretPosition(sec, para, charOffset);
    } catch {
      // 저장을 막지 않는다
    }
  }

  getTableDimensions(sec: number, parentPara: number, controlIdx: number): TableDimensions {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getTableDimensions(sec, parentPara, controlIdx));
  }

  getCellInfo(sec: number, parentPara: number, controlIdx: number, cellIdx: number): CellInfo {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCellInfo(sec, parentPara, controlIdx, cellIdx));
  }

  getTableCellBboxes(sec: number, parentPara: number, controlIdx: number, pageHint?: number): CellBbox[] {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getTableCellBboxes(sec, parentPara, controlIdx, pageHint ?? undefined));
  }

  getTableBBox(sec: number, parentPara: number, controlIdx: number): { pageIndex: number; x: number; y: number; width: number; height: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getTableBBox(sec, parentPara, controlIdx));
  }

  /** 지정 page 에 배치된 표 fragment 의 페이지 좌표 bbox (#2400). */
  getTableBBoxAtPage(sec: number, parentPara: number, controlIdx: number, pageIdx: number): { pageIndex: number; x: number; y: number; width: number; height: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getTableBBoxAtPage(sec, parentPara, controlIdx, pageIdx));
  }

  /** [Task #919] 글상자/도형 컨트롤의 페이지 좌표 바운딩박스 */
  getShapeBBox(sec: number, parentPara: number, controlIdx: number): { pageIndex: number; x: number; y: number; width: number; height: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getShapeBBox(sec, parentPara, controlIdx));
  }

  deleteTableControl(sec: number, parentPara: number, controlIdx: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deleteTableControl(sec, parentPara, controlIdx));
  }

  getCellProperties(sec: number, parentPara: number, controlIdx: number, cellIdx: number): CellProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCellProperties(sec, parentPara, controlIdx, cellIdx));
  }

  getCellOwnProperties(sec: number, parentPara: number, controlIdx: number, cellIdx: number): CellProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const doc = this.doc as unknown as {
      getCellOwnProperties(sec: number, parentPara: number, controlIdx: number, cellIdx: number): string;
    };
    return JSON.parse(doc.getCellOwnProperties(sec, parentPara, controlIdx, cellIdx));
  }

  setCellProperties(sec: number, parentPara: number, controlIdx: number, cellIdx: number, props: Partial<CellProperties>): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setCellProperties(sec, parentPara, controlIdx, cellIdx, JSON.stringify(props)));
  }

  setCellZoneProperties(
    sec: number,
    parentPara: number,
    controlIdx: number,
    range: { startRow: number; startCol: number; endRow: number; endCol: number },
    props: Partial<CellProperties>,
  ): { ok: boolean; borderFillId: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const doc = this.doc as unknown as {
      setCellZoneProperties(
        sec: number,
        parentPara: number,
        controlIdx: number,
        startRow: number,
        startCol: number,
        endRow: number,
        endCol: number,
        json: string,
      ): string;
    };
    return JSON.parse(doc.setCellZoneProperties(
      sec,
      parentPara,
      controlIdx,
      range.startRow,
      range.startCol,
      range.endRow,
      range.endCol,
      JSON.stringify(props),
    ));
  }

  resizeTableCells(
    sec: number, parentPara: number, controlIdx: number,
    updates: TableCellResizeUpdate[],
  ): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.resizeTableCells(sec, parentPara, controlIdx, JSON.stringify(updates)));
  }

  moveTableOffset(sec: number, parentPara: number, controlIdx: number, deltaH: number, deltaV: number): { ok: boolean; ppi: number; ci: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.moveTableOffset(sec, parentPara, controlIdx, deltaH, deltaV));
  }

  getTableProperties(sec: number, parentPara: number, controlIdx: number): TableProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getTableProperties(sec, parentPara, controlIdx));
  }

  getTableSignature(sec: number, parentPara: number, controlIdx: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { getTableSignature?: (a: number, b: number, c: number) => string };
    if (typeof d.getTableSignature !== 'function') {
      throw new Error('getTableSignature API unavailable');
    }
    return d.getTableSignature(sec, parentPara, controlIdx);
  }

  setTableProperties(sec: number, parentPara: number, controlIdx: number, props: Partial<TableProperties>): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setTableProperties(sec, parentPara, controlIdx, JSON.stringify(props)));
  }

  mergeTableCells(sec: number, parentPara: number, controlIdx: number, startRow: number, startCol: number, endRow: number, endCol: number): { ok: boolean; cellCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.mergeTableCells(sec, parentPara, controlIdx, startRow, startCol, endRow, endCol));
  }

  splitTableCell(sec: number, parentPara: number, controlIdx: number, row: number, col: number): { ok: boolean; cellCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.splitTableCell(sec, parentPara, controlIdx, row, col));
  }

  splitTableCellInto(
    sec: number, parentPara: number, controlIdx: number,
    row: number, col: number,
    nRows: number, mCols: number,
    equalRowHeight: boolean, mergeFirst: boolean,
  ): { ok: boolean; cellCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).splitTableCellInto(sec, parentPara, controlIdx, row, col, nRows, mCols, equalRowHeight, mergeFirst));
  }

  splitTableCellsInRange(
    sec: number, parentPara: number, controlIdx: number,
    startRow: number, startCol: number, endRow: number, endCol: number,
    nRows: number, mCols: number, equalRowHeight: boolean,
  ): { ok: boolean; cellCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).splitTableCellsInRange(sec, parentPara, controlIdx, startRow, startCol, endRow, endCol, nRows, mCols, equalRowHeight));
  }

  copyTableCellsTransposed(
    sec: number,
    parentPara: number,
    controlIdx: number,
    startRow: number,
    startCol: number,
    endRow: number,
    endCol: number,
  ): TableTransposeResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).copyTableCellsTransposed(
      sec,
      parentPara,
      controlIdx,
      startRow,
      startCol,
      endRow,
      endCol,
    ));
  }

  pasteTableCellsTransposed(
    sec: number,
    parentPara: number,
    controlIdx: number,
    startRow: number,
    startCol: number,
  ): TableTransposeResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).pasteTableCellsTransposed(
      sec,
      parentPara,
      controlIdx,
      startRow,
      startCol,
    ));
  }

  transposeTableCellsInPlace(
    sec: number,
    parentPara: number,
    controlIdx: number,
  ): TableTransposeResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).transposeTableCellsInPlace(sec, parentPara, controlIdx));
  }

  pasteTableCellsTransposedAsTable(
    sec: number,
    para: number,
    charOffset: number,
  ): TableTransposeResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).pasteTableCellsTransposedAsTable(sec, para, charOffset));
  }

  hasTableTransposeClipboard(): boolean {
    if (!this.doc) return false;
    return Boolean((this.doc as any).hasTableTransposeClipboard?.());
  }

  /** 표를 지정 행에서 두 개로 나눈다 (한컴 [표-표 나누기]). */
  splitTable(sec: number, parentPara: number, controlIdx: number, atRow: number): { ok: boolean; frontRows: number; backParaIdx: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.splitTable(sec, parentPara, controlIdx, atRow));
  }

  /** 현재 표에 다음 표를 이어 붙인다 (한컴 [표-표 붙이기]). */
  mergeTableWithNext(sec: number, parentPara: number, controlIdx: number): { ok: boolean; rowCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.mergeTableWithNext(sec, parentPara, controlIdx));
  }

  insertTableRow(sec: number, parentPara: number, controlIdx: number, rowIdx: number, below: boolean): { ok: boolean; rowCount: number; colCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.insertTableRow(sec, parentPara, controlIdx, rowIdx, below));
  }

  insertTableColumn(sec: number, parentPara: number, controlIdx: number, colIdx: number, right: boolean): { ok: boolean; rowCount: number; colCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.insertTableColumn(sec, parentPara, controlIdx, colIdx, right));
  }

  deleteTableRow(sec: number, parentPara: number, controlIdx: number, rowIdx: number): { ok: boolean; rowCount: number; colCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deleteTableRow(sec, parentPara, controlIdx, rowIdx));
  }

  deleteTableColumn(sec: number, parentPara: number, controlIdx: number, colIdx: number): { ok: boolean; rowCount: number; colCount: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deleteTableColumn(sec, parentPara, controlIdx, colIdx));
  }

  createTable(sec: number, para: number, charOffset: number, rows: number, cols: number): { ok: boolean; paraIdx: number; controlIdx: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.createTable(sec, para, charOffset, rows, cols));
  }

  createTableEx(options: {
    sectionIdx: number;
    paraIdx: number;
    charOffset: number;
    rowCount: number;
    colCount: number;
    treatAsChar?: boolean;
    colWidths?: number[];
    rowHeights?: number[];
  }): { ok: boolean; paraIdx: number; controlIdx: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).createTableEx(JSON.stringify(options)));
  }

  evaluateTableFormula(sec: number, parentPara: number, controlIdx: number,
    targetRow: number, targetCol: number, formula: string, writeResult: boolean): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.evaluateTableFormula(sec, parentPara, controlIdx, targetRow, targetCol, formula, writeResult);
  }

  /**
   * 커서 위치에 그림을 삽입한다.
   *
   * @param cellPathJson 표 셀 안 삽입 시 cellPath JSON (#1151).
   *   빈 문자열 또는 `'[]'` 면 본문 sibling floating picture 삽입
   *   (한컴 정합, treat_as_char=false). 비어있지 않으면 셀 영역에 floating
   *   picture 삽입 (tac=false, wrap=Square, Page-relative offset). 셀 자체는
   *   비어있는 채로 유지되어 클릭 시 cursor 가 정상 동작한다.
   *   예: `JSON.stringify([{controlIndex:0, cellIndex:2, cellParaIndex:0}])`
   */
  insertPicture(sec: number, paraIdx: number, charOffset: number,
                cellPathJson: string,
                imageData: Uint8Array, width: number, height: number,
                naturalWidthPx: number, naturalHeightPx: number,
                extension: string, description: string = '',
                // [Task #1151 v8 결함 C] 사용자 클릭/드래그 paper-relative 좌표 (HU).
                // 셀 floating 분기에서 사용. undefined 면 셀 좌상단 default (기존 동작).
                paperOffsetXHu?: number, paperOffsetYHu?: number): { ok: boolean; paraIdx: number; controlIdx: number; logicalOffset?: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).insertPicture(
      sec, paraIdx, charOffset, cellPathJson, imageData,
      width, height, naturalWidthPx, naturalHeightPx, extension, description,
      paperOffsetXHu, paperOffsetYHu,
    ));
  }

  /**
   * [Task #2230] 기존 Picture 컨트롤에 이미지를 지정한다 — 그림 미지정
   * placeholder(missing image 컨트롤)의 편집 뷰 그림 삽입.
   * 개체 틀 크기는 유지된다 (한컴 placeholder 는 틀에 그림을 맞춤).
   * cellPathJson 규약은 insertPicture 와 동일 (빈 문자열/"[]" = 본문).
   */
  assignPictureImage(sec: number, parentParaIdx: number, cellPathJson: string,
                     controlIdx: number, imageData: Uint8Array,
                     naturalWidthPx: number, naturalHeightPx: number,
                     extension: string): { ok: boolean; binDataId: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).assignPictureImage(
      sec, parentParaIdx, cellPathJson, controlIdx, imageData,
      naturalWidthPx, naturalHeightPx, extension,
    ));
  }

  // ── 그림 속성 API ─────────────────────────────────────
  getPageControlLayout(pageNum: number): { controls: import('./types').ControlLayoutItem[] } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getPageControlLayout(pageNum));
  }

  getPictureProperties(sec: number, para: number, ci: number): import('./types').PictureProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getPictureProperties(sec, para, ci));
  }

  /** [Task #825] 머리말/꼬리말 안 그림 속성 조회. */
  getHeaderFooterPictureProperties(
    sec: number,
    outerPara: number,
    outerCtrl: number,
    innerPara: number,
    innerCtrl: number,
  ): import('./types').PictureProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(
      (this.doc as any).getHeaderFooterPictureProperties(sec, outerPara, outerCtrl, innerPara, innerCtrl)
    );
  }

  /** [Task #825] 머리말/꼬리말 안 그림 속성 변경. */
  setHeaderFooterPictureProperties(
    sec: number,
    outerPara: number,
    outerCtrl: number,
    innerPara: number,
    innerCtrl: number,
    props: Record<string, unknown>,
  ): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(
      (this.doc as any).setHeaderFooterPictureProperties(
        sec, outerPara, outerCtrl, innerPara, innerCtrl, JSON.stringify(props),
      )
    );
  }

  /** [Task #1138] 표 셀 내 Shape 속성 조회 (by_path). */
  getCellShapePropertiesByPath(
    sec: number,
    parentPara: number,
    cellPath: CellPathLike,
    innerControlIdx: number,
  ): import('./types').ShapeProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(
      (this.doc as any).getCellShapePropertiesByPath(
        sec, parentPara, JSON.stringify(cellPath), innerControlIdx,
      )
    );
  }

  /** [Task #1151 v4] 표 셀 내 Picture 속성 조회 (by_path). Shape 패턴 정합. */
  getCellPicturePropertiesByPath(
    sec: number,
    parentPara: number,
    cellPath: CellPathLike,
    innerControlIdx: number,
  ): import('./types').PictureProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(
      (this.doc as any).getCellPicturePropertiesByPath(
        sec, parentPara, JSON.stringify(cellPath), innerControlIdx,
      )
    );
  }

  /** [Task #1138] 표 셀 내 Shape 속성 변경 (by_path). */
  setCellShapePropertiesByPath(
    sec: number,
    parentPara: number,
    cellPath: CellPathLike,
    innerControlIdx: number,
    props: Record<string, unknown>,
  ): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(
      (this.doc as any).setCellShapePropertiesByPath(
        sec, parentPara, JSON.stringify(cellPath), innerControlIdx, JSON.stringify(props),
      )
    );
  }

  setPictureProperties(sec: number, para: number, ci: number, props: Record<string, unknown>): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setPictureProperties(sec, para, ci, JSON.stringify(props)));
  }

  /** [Task #1151 v4] 표 셀 내 Picture 속성 변경 (by_path). Shape 패턴 정합. */
  setCellPicturePropertiesByPath(
    sec: number,
    parentPara: number,
    cellPath: CellPathLike,
    innerControlIdx: number,
    props: Record<string, unknown>,
  ): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(
      (this.doc as any).setCellPicturePropertiesByPath(
        sec, parentPara, JSON.stringify(cellPath), innerControlIdx, JSON.stringify(props),
      )
    );
  }

  // ── 수식 속성 API ─────────────────────────────────────
  getEquationProperties(sec: number, para: number, ci: number, cellIdx?: number, cellParaIdx?: number): import('./types').EquationProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getEquationProperties(sec, para, ci, cellIdx ?? -1, cellParaIdx ?? -1));
  }

  getNoteEquationProperties(noteRef: import('./types').NoteControlRef): import('./types').EquationProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getNoteEquationProperties(
      noteRef.kind,
      noteRef.sectionIdx,
      noteRef.paraIdx,
      noteRef.controlIdx,
      noteRef.noteParaIdx,
      noteRef.innerControlIdx,
    ));
  }

  setEquationProperties(sec: number, para: number, ci: number, cellIdx: number | undefined, cellParaIdx: number | undefined, props: Record<string, unknown>): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setEquationProperties(sec, para, ci, cellIdx ?? -1, cellParaIdx ?? -1, JSON.stringify(props)));
  }

  setNoteEquationProperties(noteRef: import('./types').NoteControlRef, props: Record<string, unknown>): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).setNoteEquationProperties(
      noteRef.kind,
      noteRef.sectionIdx,
      noteRef.paraIdx,
      noteRef.controlIdx,
      noteRef.noteParaIdx,
      noteRef.innerControlIdx,
      JSON.stringify(props),
    ));
  }

  renderEquationPreview(script: string, fontSizeHwpunit: number, color: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.renderEquationPreview(script, fontSizeHwpunit, color);
  }

  deletePictureControl(sec: number, para: number, ci: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deletePictureControl(sec, para, ci));
  }

  /** [Task #1171 / PR #1254] 표 셀/글상자 내부 Picture 삭제 (by_path). */
  deleteCellPictureControlByPath(
    sec: number,
    parentPara: number,
    cellPath: CellPathLike,
    innerControlIdx: number,
  ): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(
      (this.doc as any).deleteCellPictureControlByPath(
        sec, parentPara, JSON.stringify(cellPath), innerControlIdx,
      )
    );
  }

  createShapeControl(params: Record<string, unknown>): { ok: boolean; paraIdx: number; controlIdx: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.createShapeControl(JSON.stringify(params)));
  }

  getShapeProperties(sec: number, para: number, ci: number): import('./types').ShapeProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getShapeProperties(sec, para, ci));
  }

  getShapeText(sec: number, para: number, ci: number): { ok: boolean; text: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getShapeText(sec, para, ci));
  }

  setShapeProperties(sec: number, para: number, ci: number, props: Record<string, unknown>): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setShapeProperties(sec, para, ci, JSON.stringify(props)));
  }

  deleteShapeControl(sec: number, para: number, ci: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deleteShapeControl(sec, para, ci));
  }

  deleteEquationControl(sec: number, para: number, ci: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deleteEquationControl(sec, para, ci));
  }

  changeShapeZOrder(sec: number, para: number, ci: number, operation: string): { ok: boolean; zOrder?: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.changeShapeZOrder(sec, para, ci, operation));
  }

  groupShapes(sec: number, targets: { paraIdx: number; controlIdx: number }[]): { ok: boolean; paraIdx: number; controlIdx: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const json = JSON.stringify({ sectionIdx: sec, targets });
    return JSON.parse((this.doc as any).groupShapes(json));
  }

  insertEquation(sec: number, para: number, charOffset: number, script: string, fontSizeHwpunit: number, color: number): { ok: boolean; paraIdx: number; controlIdx: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).insertEquation(sec, para, charOffset, script, fontSizeHwpunit, color));
  }

  insertFootnote(sec: number, para: number, charOffset: number): { ok: boolean; paraIdx: number; controlIdx: number; footnoteNumber: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).insertFootnote(sec, para, charOffset));
  }

  insertEndnote(sec: number, para: number, charOffset: number): { ok: boolean; paraIdx: number; controlIdx: number; endnoteNumber: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).insertEndnote(sec, para, charOffset));
  }

  getEndnoteShape(sec: number): EndnoteShapeSettings {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getEndnoteShape(sec));
  }

  applyEndnoteShape(sec: number, settings: EndnoteShapeSettings): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).applyEndnoteShape(sec, JSON.stringify(settings)));
  }

  getFootnoteInfo(sec: number, para: number, controlIdx: number): { ok: boolean; paraCount: number; totalTextLen: number; number: number; texts: string[] } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getFootnoteInfo(sec, para, controlIdx));
  }

  getFootnoteAtCursor(sec: number, para: number, charOffset: number, direction: 'backward' | 'forward'): FootnoteAtCursorResult {
    if (!this.doc) return { hit: false };
    const getter = (this.doc as any).getFootnoteAtCursor;
    if (typeof getter !== 'function') return { hit: false };
    return JSON.parse(getter.call(this.doc, sec, para, charOffset, direction));
  }

  deleteFootnote(sec: number, para: number, controlIdx: number): DeleteFootnoteResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).deleteFootnote(sec, para, controlIdx));
  }

  insertTextInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number, charOffset: number, text: string): { ok: boolean; charOffset: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).insertTextInFootnote(sec, para, controlIdx, fnParaIdx, charOffset, text));
  }

  deleteTextInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number, charOffset: number, count: number): { ok: boolean; charOffset: number; deletedText: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).deleteTextInFootnote(sec, para, controlIdx, fnParaIdx, charOffset, count));
  }

  splitParagraphInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number, charOffset: number, removedParaMeta?: RemovedParaMeta): { ok: boolean; fnParaIndex: number; charOffset: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).splitParagraphInFootnote(sec, para, controlIdx, fnParaIdx, charOffset, serializeParaMeta(removedParaMeta)));
  }

  mergeParagraphInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number): { ok: boolean; fnParaIndex: number; charOffset: number; removedParaMeta: RemovedParaMeta } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).mergeParagraphInFootnote(sec, para, controlIdx, fnParaIdx));
  }

  getPageFootnoteInfo(pageNum: number, footnoteIndex: number): { ok: boolean; sectionIdx: number; paraIdx: number; controlIdx: number; sourceType: string } | null {
    if (!this.doc) return null;
    try {
      return JSON.parse((this.doc as any).getPageFootnoteInfo(pageNum, footnoteIndex));
    } catch { return null; }
  }

  pageHasFootnoteFootholds(pageNum: number): boolean {
    if (!this.doc) return false;
    return (this.doc as any).pageHasFootnoteFootholds(pageNum);
  }

  hitTestFootnote(pageNum: number, x: number, y: number): { hit: boolean; footnoteIndex?: number } {
    if (!this.doc) return { hit: false };
    return JSON.parse((this.doc as any).hitTestFootnote(pageNum, x, y));
  }

  hitTestInFootnote(pageNum: number, x: number, y: number): { hit: boolean; fnParaIndex?: number; charOffset?: number; footnoteIndex?: number; cursorRect?: { pageIndex: number; x: number; y: number; height: number } } {
    if (!this.doc) return { hit: false };
    return JSON.parse((this.doc as any).hitTestInFootnote(pageNum, x, y));
  }

  getCursorRectInFootnote(pageNum: number, footnoteIndex: number, fnParaIdx: number, charOffset: number): { pageIndex: number; x: number; y: number; height: number } | null {
    if (!this.doc) return null;
    try {
      return JSON.parse((this.doc as any).getCursorRectInFootnote(pageNum, footnoteIndex, fnParaIdx, charOffset));
    } catch { return null; }
  }

  getNoteEditInfo(sec: number, para: number, controlIdx: number): NoteEditInfo | null {
    if (!this.doc) return null;
    try {
      return JSON.parse((this.doc as any).getNoteEditInfo(sec, para, controlIdx));
    } catch { return null; }
  }

  getCursorRectInNote(sec: number, para: number, controlIdx: number, noteParaIdx: number, charOffset: number): CursorRect | null {
    if (!this.doc) return null;
    try {
      return JSON.parse((this.doc as any).getCursorRectInNote(sec, para, controlIdx, noteParaIdx, charOffset));
    } catch { return null; }
  }

  getParaPropertiesInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number): ParaProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getParaPropertiesInFootnote(sec, para, controlIdx, fnParaIdx));
  }

  applyParaFormatInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number, propsJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).applyParaFormatInFootnote(sec, para, controlIdx, fnParaIdx, propsJson);
  }

  moveLineEndpoint(sec: number, para: number, ci: number, sx: number, sy: number, ex: number, ey: number): void {
    if (!this.doc) return;
    (this.doc as any).moveLineEndpoint(sec, para, ci, sx, sy, ex, ey);
  }

  updateConnectorsInSection(sec: number): void {
    if (!this.doc) return;
    (this.doc as any).updateConnectorsInSection(sec);
  }

  ungroupShape(sec: number, para: number, ci: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).ungroupShape(sec, para, ci));
  }

  moveVertical(
    sec: number, para: number, charOffset: number,
    delta: number, preferredX: number,
    parentPara: number, controlIdx: number,
    cellIdx: number, cellParaIdx: number,
  ): MoveVerticalResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.moveVertical(
      sec, para, charOffset, delta, preferredX,
      parentPara, controlIdx, cellIdx, cellParaIdx,
    ));
  }

  // ─── 경로 기반 중첩 표 API ─────────────────────────────

  getCursorRectByPath(sec: number, parentPara: number, pathJson: string, charOffset: number): CursorRect {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCursorRectByPath(sec, parentPara, pathJson, charOffset));
  }

  /** [#2021] 경로 기반 커서 좌표 + 페이지 힌트 — 직전 캐럿 페이지를 넘기면 해당
   *  페이지(±1)를 먼저 탐색해 거대 표 문서의 선형 페이지 재빌드를 피한다.
   *  힌트가 틀려도 전체 탐색 fallback으로 좌표는 동일하다. */
  getCursorRectByPathNear(
    sec: number, parentPara: number, pathJson: string, charOffset: number, hintPage: number,
  ): CursorRect {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    if (typeof this.doc.getCursorRectByPathNear !== 'function') {
      // 구버전 wasm 폴백
      return JSON.parse(this.doc.getCursorRectByPath(sec, parentPara, pathJson, charOffset));
    }
    return JSON.parse(
      this.doc.getCursorRectByPathNear(sec, parentPara, pathJson, charOffset, hintPage),
    );
  }

  getCellInfoByPath(sec: number, parentPara: number, pathJson: string): CellInfo {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCellInfoByPath(sec, parentPara, pathJson));
  }

  getTableDimensionsByPath(sec: number, parentPara: number, pathJson: string): TableDimensions {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getTableDimensionsByPath(sec, parentPara, pathJson));
  }

  getTableCellBboxesByPath(sec: number, parentPara: number, pathJson: string): CellBbox[] {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getTableCellBboxesByPath(sec, parentPara, pathJson));
  }

  moveVerticalByPath(
    sec: number, parentPara: number, pathJson: string,
    charOffset: number, delta: number, preferredX: number,
  ): MoveVerticalResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.moveVerticalByPath(
      sec, parentPara, pathJson, charOffset, delta, preferredX,
    ));
  }

  // ─── Selection API ──────────────────────────────────────

  getSelectionRects(sec: number, startPara: number, startOffset: number, endPara: number, endOffset: number): SelectionRect[] {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getSelectionRects(sec, startPara, startOffset, endPara, endOffset));
  }

  getSelectionRectsInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number, pageHints?: SelectionPageHints): SelectionRect[] {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return getSelectionRectsInCellWithPageHints(
      this.doc as unknown as CellSelectionRectDocument,
      {
        sectionIdx: sec,
        parentParaIdx: parentPara,
        controlIdx,
        cellIdx,
        startCellParaIdx: startCellPara,
        startCharOffset: startOffset,
        endCellParaIdx: endCellPara,
        endCharOffset: endOffset,
      },
      pageHints,
    );
  }

  getSelectionRectsInCellByPath(sec: number, parentPara: number, path: string, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number, pageHints?: SelectionPageHints): SelectionRect[] {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return getSelectionRectsInCellByPathWithPageHints(
      this.doc as unknown as PathCellSelectionRectDocument,
      {
        sectionIdx: sec,
        parentParaIdx: parentPara,
        path,
        startCellParaIdx: startCellPara,
        startCharOffset: startOffset,
        endCellParaIdx: endCellPara,
        endCharOffset: endOffset,
      },
      pageHints,
    );
  }

  getSelectionRectsInFootnote(pageNum: number, footnoteIndex: number, startFnPara: number, startOffset: number, endFnPara: number, endOffset: number): SelectionRect[] {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getSelectionRectsInFootnote(pageNum, footnoteIndex, startFnPara, startOffset, endFnPara, endOffset));
  }

  deleteRange(sec: number, startPara: number, startOffset: number, endPara: number, endOffset: number): { ok: boolean; paraIdx: number; charOffset: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deleteRange(sec, startPara, startOffset, endPara, endOffset));
  }

  deleteRangeInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number): { ok: boolean; paraIdx: number; charOffset: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.deleteRangeInCell(sec, parentPara, controlIdx, cellIdx, startCellPara, startOffset, endCellPara, endOffset));
  }

  // ─── 클립보드 API ──────────────────────────────────────

  copySelection(sec: number, startPara: number, startOffset: number, endPara: number, endOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.copySelection(sec, startPara, startOffset, endPara, endOffset);
  }

  copySelectionInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.copySelectionInCell(sec, parentPara, controlIdx, cellIdx, startCellPara, startOffset, endCellPara, endOffset);
  }

  copySelectionInCellByPath(sec: number, parentPara: number, pathJson: string, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).copySelectionInCellByPath(sec, parentPara, pathJson, startCellPara, startOffset, endCellPara, endOffset);
  }

  pasteInternal(sec: number, para: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.pasteInternal(sec, para, charOffset);
  }

  pasteInternalInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.pasteInternalInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset);
  }

  pasteInternalInCellByPath(sec: number, parentPara: number, pathJson: string, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).pasteInternalInCellByPath(sec, parentPara, pathJson, charOffset);
  }

  hasInternalClipboard(): boolean {
    if (!this.doc) return false;
    return this.doc.hasInternalClipboard();
  }

  getClipboardText(): string {
    if (!this.doc) return '';
    return this.doc.getClipboardText();
  }

  // [Task #1161] cellPathJson: 셀/글상자 안 picture 복사 시 다단계 경로
  // (`[{controlIndex,cellIndex,cellParaIndex},...]`). 빈 문자열이면 본문.
  copyControl(sec: number, para: number, ci: number, cellPathJson = ''): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.copyControl(sec, para, cellPathJson, ci);
  }

  exportControlHtml(sec: number, para: number, ci: number, cellPathJson = ''): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportControlHtml(sec, para, cellPathJson, ci);
  }

  getControlImageData(sec: number, para: number, ci: number, cellPathJson = ''): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getControlImageData(sec, para, cellPathJson, ci);
  }

  getControlImageMime(sec: number, para: number, ci: number, cellPathJson = ''): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getControlImageMime(sec, para, cellPathJson, ci);
  }

  clipboardHasControl(): boolean {
    if (!this.doc) return false;
    return this.doc.clipboardHasControl();
  }

  pasteControl(sec: number, para: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.pasteControl(sec, para, charOffset);
  }

  exportSelectionHtml(sec: number, startPara: number, startOffset: number, endPara: number, endOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportSelectionHtml(sec, startPara, startOffset, endPara, endOffset);
  }

  exportSelectionInCellHtml(sec: number, parentPara: number, controlIdx: number, cellIdx: number, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportSelectionInCellHtml(sec, parentPara, controlIdx, cellIdx, startCellPara, startOffset, endCellPara, endOffset);
  }

  exportSelectionInCellHtmlByPath(sec: number, parentPara: number, pathJson: string, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).exportSelectionInCellHtmlByPath(sec, parentPara, pathJson, startCellPara, startOffset, endCellPara, endOffset);
  }

  pasteHtml(sec: number, para: number, charOffset: number, html: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.pasteHtml(sec, para, charOffset, html);
  }

  pasteHtmlInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, html: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.pasteHtmlInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, html);
  }

  pasteHtmlInCellByPath(sec: number, parentPara: number, pathJson: string, charOffset: number, html: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).pasteHtmlInCellByPath(sec, parentPara, pathJson, charOffset, html);
  }

  // ─── CharShape (서식) API ──────────────────────────────

  getCharPropertiesAt(sec: number, para: number, charOffset: number): CharProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCharPropertiesAt(sec, para, charOffset));
  }

  getCellCharPropertiesAt(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number): CharProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCellCharPropertiesAt(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset));
  }

  /** getCellCharPropertiesAt 의 cellPath 변형 — 중첩 셀의 charShapeId 조회. */
  getCellCharPropertiesAtByPath(sec: number, parentPara: number, pathJson: string, charOffset: number): CharProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getCellCharPropertiesAtByPath(sec, parentPara, pathJson, charOffset));
  }

  applyCharFormat(sec: number, para: number, startOffset: number, endOffset: number, propsJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.applyCharFormat(sec, para, startOffset, endOffset, propsJson);
  }

  setCharShapeId(sec: number, para: number, startOffset: number, endOffset: number, charShapeId: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setCharShapeId(sec, para, startOffset, endOffset, charShapeId);
  }

  applyCharFormatInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, startOffset: number, endOffset: number, propsJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.applyCharFormatInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, startOffset, endOffset, propsJson);
  }

  /** applyCharFormatInCell 의 cellPath 변형 — 중첩 셀 선택에 서식을 적용한다. */
  applyCharFormatInCellByPath(sec: number, parentPara: number, pathJson: string, startOffset: number, endOffset: number, propsJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).applyCharFormatInCellByPath(sec, parentPara, pathJson, startOffset, endOffset, propsJson);
  }

  setCharShapeIdInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, startOffset: number, endOffset: number, charShapeId: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setCharShapeIdInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, startOffset, endOffset, charShapeId);
  }

  /** setCharShapeIdInCell 의 cellPath 변형 — 중첩 셀 서식 undo 복원. */
  setCharShapeIdInCellByPath(sec: number, parentPara: number, pathJson: string, startOffset: number, endOffset: number, charShapeId: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setCharShapeIdInCellByPath(sec, parentPara, pathJson, startOffset, endOffset, charShapeId);
  }

  findOrCreateFontId(name: string): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.findOrCreateFontId(name);
  }

  findOrCreateFontIdForLang(lang: number, name: string): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).findOrCreateFontIdForLang(lang, name) as number;
  }

  // ─── 문단 서식 API ──────────────────────────────────────

  getParaPropertiesAt(sec: number, para: number): ParaProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getParaPropertiesAt(sec, para));
  }

  getCellParaPropertiesAt(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number): ParaProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCellParaPropertiesAt(sec, parentPara, controlIdx, cellIdx, cellParaIdx));
  }

  setNumberingRestart(sec: number, para: number, mode: number, startNum: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setNumberingRestart(sec, para, mode, startNum);
  }

  applyParaFormat(sec: number, para: number, propsJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.applyParaFormat(sec, para, propsJson);
  }

  setParaShapeId(sec: number, para: number, paraShapeId: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setParaShapeId(sec, para, paraShapeId);
  }

  applyParaFormatInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, propsJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.applyParaFormatInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, propsJson);
  }

  setCellParaShapeId(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, paraShapeId: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setCellParaShapeId(sec, parentPara, controlIdx, cellIdx, cellParaIdx, paraShapeId);
  }

  /** 머리말/꼬리말 문단의 문단 속성을 조회한다 */
  getParaPropertiesInHf(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number): ParaProperties {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getParaPropertiesInHf(sec, isHeader, applyTo, hfParaIdx));
  }

  /** 머리말/꼬리말 문단에 문단 서식을 적용한다 */
  applyParaFormatInHf(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, propsJson: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.applyParaFormatInHf(sec, isHeader, applyTo, hfParaIdx, propsJson);
  }

  /**
   * 머리말/꼬리말 문단에 필드 마커를 삽입한다 (1=쪽번호, 2=총쪽수, 3=파일이름).
   *
   * `charOffset`은 삽입 뒤 커서 좌표, `insertedAt`/`insertedLength`는 history가
   * 역연산할 실제 모델 텍스트 범위다. inline control 뒤 cursor처럼 둘이 다를 수 있다.
   */
  insertFieldInHf(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, charOffset: number, fieldType: number): { ok: boolean; charOffset: number; insertedAt: number; insertedLength: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.insertFieldInHf(sec, isHeader, applyTo, hfParaIdx, charOffset, fieldType));
  }

  applyHfTemplate(sec: number, isHeader: boolean, applyTo: number, templateId: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.applyHfTemplate(sec, isHeader, applyTo, templateId));
  }

  // ─── 스타일 API ──────────────────────────────────────

  getStyleList(): Array<{ id: number; name: string; englishName: string; type: number; nextStyleId: number; paraShapeId: number; charShapeId: number }> {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).getStyleList());
  }

  getStyleDetail(styleId: number): { charProps: import('./types').CharProperties; paraProps: import('./types').ParaProperties } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).getStyleDetail(styleId));
  }

  updateStyle(styleId: number, json: string): boolean {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (this.doc as any).updateStyle(styleId, json);
  }

  updateStyleShapes(styleId: number, charModsJson: string, paraModsJson: string): boolean {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (this.doc as any).updateStyleShapes(styleId, charModsJson, paraModsJson);
  }

  createStyle(json: string): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (this.doc as any).createStyle(json);
  }

  deleteStyle(styleId: number): boolean {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (this.doc as any).deleteStyle(styleId);
  }

  // ─── 번호/글머리표 API ─────────────────────────────────

  getNumberingList(): Array<{ id: number; levelFormats: string[]; startNumber: number }> {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).getNumberingList());
  }

  getBulletList(): Array<{ id: number; char: string; rawCode: number }> {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).getBulletList());
  }

  ensureDefaultNumbering(): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (this.doc as any).ensureDefaultNumbering();
  }

  createNumbering(json: string): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (this.doc as any).createNumbering(json);
  }

  ensureDefaultBullet(bulletChar: string): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (this.doc as any).ensureDefaultBullet(bulletChar);
  }

  getStyleAt(sec: number, para: number): { id: number; name: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).getStyleAt(sec, para));
  }

  getCellStyleAt(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number): { id: number; name: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).getCellStyleAt(sec, parentPara, controlIdx, cellIdx, cellParaIdx));
  }

  applyStyle(sec: number, para: number, styleId: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).applyStyle(sec, para, styleId));
  }

  applyCellStyle(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, styleId: number): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return JSON.parse((this.doc as any).applyCellStyle(sec, parentPara, controlIdx, cellIdx, cellParaIdx, styleId));
  }

  // ─── 보기 옵션 API ──────────────────────────────────

  setShowParagraphMarks(enabled: boolean): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.setShowParagraphMarks(enabled);
  }

  /** 문단부호 표시 여부 반환 */
  getShowParagraphMarks(): boolean {
    if (!this.doc) return false;
    return (this.doc as any).getShowParagraphMarks();
  }

  /** 조판부호 표시 여부 반환 */
  getShowControlCodes(): boolean {
    if (!this.doc) return false;
    return (this.doc as any).getShowControlCodes();
  }

  setShowControlCodes(enabled: boolean): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.setShowControlCodes(enabled);
  }

  getShowTransparentBorders(): boolean {
    if (!this.doc) return false;
    return this.doc.getShowTransparentBorders();
  }

  setShowTransparentBorders(enabled: boolean): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.setShowTransparentBorders(enabled);
  }

  setClipEnabled(enabled: boolean): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.setClipEnabled(enabled);
  }

  // ─── Undo/Redo 스냅샷 API ──────────────────────────

  saveSnapshot(): number {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.saveSnapshot();
  }

  restoreSnapshot(id: number): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.restoreSnapshot(id);
  }

  discardSnapshot(id: number): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.discardSnapshot(id);
  }

  // ─── 머리말/꼬리말 API ──────────────────────────────────

  getHeaderFooter(sectionIdx: number, isHeader: boolean, applyTo: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getHeaderFooter(sectionIdx, isHeader, applyTo);
  }

  createHeaderFooter(sectionIdx: number, isHeader: boolean, applyTo: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.createHeaderFooter(sectionIdx, isHeader, applyTo);
  }

  insertTextInHeaderFooter(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, charOffset: number, text: string): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.insertTextInHeaderFooter(sec, isHeader, applyTo, hfParaIdx, charOffset, text);
  }

  deleteTextInHeaderFooter(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.deleteTextInHeaderFooter(sec, isHeader, applyTo, hfParaIdx, charOffset, count);
  }

  splitParagraphInHeaderFooter(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, charOffset: number, removedParaMeta?: RemovedParaMeta): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.splitParagraphInHeaderFooter(sec, isHeader, applyTo, hfParaIdx, charOffset, serializeParaMeta(removedParaMeta));
  }

  mergeParagraphInHeaderFooter(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.mergeParagraphInHeaderFooter(sec, isHeader, applyTo, hfParaIdx);
  }

  getHeaderFooterParaInfo(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.getHeaderFooterParaInfo(sec, isHeader, applyTo, hfParaIdx);
  }

  getCursorRectInHeaderFooter(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, charOffset: number, preferredPage = -1): CursorRect {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getCursorRectInHeaderFooter(sec, isHeader, applyTo, hfParaIdx, charOffset, preferredPage));
  }

  hitTestHeaderFooter(pageNum: number, x: number, y: number): { hit: boolean; isHeader?: boolean; sectionIndex?: number; applyTo?: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.hitTestHeaderFooter(pageNum, x, y));
  }

  /**
   * 이 쪽에서 머리말/꼬리말을 편집할 때 대상이 되는 (구역, applyTo).
   *
   * 좌표 없이 쪽만으로 묻는다 — 히트테스트(`hitTestHeaderFooter`)가 영역 판정 뒤에 쓰는
   * 것과 같은 답이다 (Task #3206).
   */
  getHeaderFooterEditTarget(pageNum: number, isHeader: boolean): { sectionIndex: number; applyTo: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getHeaderFooterEditTarget(pageNum, isHeader));
  }

  hitTestInHeaderFooter(pageNum: number, isHeader: boolean, x: number, y: number): { hit: boolean; paraIndex?: number; charOffset?: number; cursorRect?: { pageIndex: number; x: number; y: number; height: number } } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.hitTestInHeaderFooter(pageNum, isHeader, x, y));
  }

  deleteHeaderFooter(sectionIdx: number, isHeader: boolean, applyTo: number): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.deleteHeaderFooter(sectionIdx, isHeader, applyTo);
  }

  getHeaderFooterList(currentSectionIdx: number, currentIsHeader: boolean, currentApplyTo: number): { ok: boolean; items: { sectionIdx: number; isHeader: boolean; applyTo: number; label: string }[]; currentIndex: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getHeaderFooterList(currentSectionIdx, currentIsHeader, currentApplyTo));
  }

  toggleHideHeaderFooter(pageIndex: number, isHeader: boolean): { hidden: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.toggleHideHeaderFooter(pageIndex, isHeader));
  }

  navigateHeaderFooterByPage(currentPage: number, isHeader: boolean, direction: number): { ok: boolean; pageIndex?: number; sectionIdx?: number; isHeader?: boolean; applyTo?: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.navigateHeaderFooterByPage(currentPage, isHeader, direction));
  }

  // ─── 필드 API (Task 230) ─────────────────────────────────

  /** 문서 내 모든 필드 목록을 반환한다. */
  getFieldList(): Array<{
    fieldId: number;
    fieldType: string;
    /** 셀 구역 이름(가상 필드)이면 true. `fieldType` 은 누름틀과 셀 필드를 가르지 못한다. */
    cellField: boolean;
    name: string;
    guide: string;
    command: string;
    value: string;
    location: { sectionIndex: number; paraIndex: number; path?: Array<any> };
    startCharIdx?: number;
    endCharIdx?: number;
    editableInForm?: boolean;
  }> {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getFieldList());
  }

  /** field_id로 필드 값을 조회한다. */
  getFieldValue(fieldId: number): { ok: boolean; value: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getFieldValue(fieldId));
  }

  /** 필드 이름으로 값을 조회한다. */
  getFieldValueByName(name: string): { ok: boolean; fieldId: number; value: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).getFieldValueByName(name));
  }

  /** field_id로 필드 값을 설정한다. */
  setFieldValue(fieldId: number, value: string): { ok: boolean; fieldId: number; oldValue: string; newValue: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).setFieldValue(fieldId, value));
  }

  /** 필드 이름으로 값을 설정한다. */
  setFieldValueByName(name: string, value: string): { ok: boolean; fieldId: number; oldValue: string; newValue: string } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).setFieldValueByName(name, value));
  }

  /** 커서 위치의 필드 범위 정보를 조회한다. */
  getFieldInfoAt(pos: DocumentPosition): FieldInfoResult {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    // 중첩 표 (depth > 1): path 기반 API 사용
    if ((pos.cellPath?.length ?? 0) > 1 && pos.parentParaIndex !== undefined) {
      return JSON.parse((this.doc as any).getFieldInfoAtByPath(
        pos.sectionIndex, pos.parentParaIndex, JSON.stringify(pos.cellPath), pos.charOffset,
      ));
    }
    if (pos.parentParaIndex !== undefined && pos.controlIndex !== undefined) {
      return JSON.parse((this.doc as any).getFieldInfoAtInCell(
        pos.sectionIndex, pos.parentParaIndex, pos.controlIndex,
        pos.cellIndex ?? 0, pos.cellParaIndex ?? 0, pos.charOffset,
        pos.isTextBox ?? false,
      ));
    }
    return JSON.parse((this.doc as any).getFieldInfoAt(
      pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
    ));
  }

  /** 커서 위치의 누름틀 필드와 내용을 제거한다. */
  removeFieldAt(pos: DocumentPosition): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    if (pos.parentParaIndex !== undefined && pos.controlIndex !== undefined) {
      return JSON.parse((this.doc as any).removeFieldAtInCell(
        pos.sectionIndex, pos.parentParaIndex, pos.controlIndex,
        pos.cellIndex ?? 0, pos.cellParaIndex ?? 0, pos.charOffset,
        pos.isTextBox ?? false,
      ));
    }
    return JSON.parse((this.doc as any).removeFieldAt(
      pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
    ));
  }

  /** 활성 필드를 설정한다 (안내문 숨김용). 변경 시 true 반환. */
  setActiveField(pos: DocumentPosition): boolean {
    if (!this.doc) return false;
    // 중첩 표 (depth > 1): path 기반 API 사용
    if ((pos.cellPath?.length ?? 0) > 1 && pos.parentParaIndex !== undefined) {
      return (this.doc as any).setActiveFieldByPath(
        pos.sectionIndex, pos.parentParaIndex, JSON.stringify(pos.cellPath), pos.charOffset,
      );
    }
    if (pos.parentParaIndex !== undefined && pos.controlIndex !== undefined) {
      return (this.doc as any).setActiveFieldInCell(
        pos.sectionIndex, pos.parentParaIndex, pos.controlIndex,
        pos.cellIndex ?? 0, pos.cellParaIndex ?? 0, pos.charOffset,
        pos.isTextBox ?? false,
      );
    } else {
      return (this.doc as any).setActiveField(
        pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
      );
    }
  }

  /** 활성 필드를 해제한다 (안내문 다시 표시). */
  clearActiveField(): void {
    if (!this.doc) return;
    (this.doc as any).clearActiveField();
  }

  /** 누름틀 필드 속성을 조회한다. */
  getClickHereProps(fieldId: number): { ok: boolean; guide?: string; memo?: string; name?: string; editable?: boolean } {
    if (!this.doc) return { ok: false };
    return JSON.parse((this.doc as any).getClickHereProps(fieldId));
  }

  /** 누름틀 필드 속성을 수정한다. */
  updateClickHereProps(fieldId: number, guide: string, memo: string, name: string, editable: boolean): { ok: boolean } {
    if (!this.doc) return { ok: false };
    return JSON.parse((this.doc as any).updateClickHereProps(fieldId, guide, memo, name, editable));
  }

  /** 현재 커서 위치에 누름틀 필드를 삽입한다. */
  insertClickHereField(
    pos: DocumentPosition,
    guide: string,
    memo: string,
    name: string,
    editable: boolean,
  ): { ok: boolean; fieldId?: number; charOffset?: number } {
    if (!this.doc) return { ok: false };
    const doc = this.doc as any;
    if ((pos.cellPath?.length ?? 0) > 1 && pos.parentParaIndex !== undefined) {
      return JSON.parse(doc.insertClickHereFieldByPath(
        pos.sectionIndex,
        pos.parentParaIndex,
        JSON.stringify(pos.cellPath),
        pos.charOffset,
        guide,
        memo,
        name,
        editable,
      ));
    }
    if (pos.parentParaIndex !== undefined && pos.controlIndex !== undefined) {
      return JSON.parse(doc.insertClickHereFieldInCell(
        pos.sectionIndex,
        pos.parentParaIndex,
        pos.controlIndex,
        pos.cellIndex ?? 0,
        pos.cellParaIndex ?? 0,
        pos.charOffset,
        pos.isTextBox ?? false,
        guide,
        memo,
        name,
        editable,
      ));
    }
    return JSON.parse(doc.insertClickHereField(
      pos.sectionIndex,
      pos.paragraphIndex,
      pos.charOffset,
      guide,
      memo,
      name,
      editable,
    ));
  }

  // ─────────────────────────────────────────────
  // 양식 개체(Form Object) API
  // ─────────────────────────────────────────────

  /** 페이지 좌표에서 양식 개체를 찾는다. */
  getFormObjectAt(pageNum: number, x: number, y: number): import('./types').FormObjectHitResult {
    if (!this.doc || typeof (this.doc as any).getFormObjectAt !== 'function') return { found: false };
    return JSON.parse((this.doc as any).getFormObjectAt(pageNum, x, y));
  }

  /** 양식 개체 값을 조회한다. */
  getFormValue(sec: number, para: number, ci: number): import('./types').FormValueResult {
    if (!this.doc || typeof (this.doc as any).getFormValue !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).getFormValue(sec, para, ci));
  }

  /** 양식 개체 값을 설정한다. */
  setFormValue(sec: number, para: number, ci: number, valueJson: string): { ok: boolean } {
    if (!this.doc || typeof (this.doc as any).setFormValue !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).setFormValue(sec, para, ci, valueJson));
  }

  /** 셀 내부 양식 개체 값을 설정한다. */
  setFormValueInCell(sec: number, tablePara: number, tableCi: number, cellIdx: number, cellPara: number, formCi: number, valueJson: string): { ok: boolean } {
    if (!this.doc || typeof (this.doc as any).setFormValueInCell !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).setFormValueInCell(sec, tablePara, tableCi, cellIdx, cellPara, formCi, valueJson));
  }

  /** 양식 개체 상세 정보를 반환한다. */
  getFormObjectInfo(sec: number, para: number, ci: number): import('./types').FormObjectInfoResult {
    if (!this.doc || typeof (this.doc as any).getFormObjectInfo !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).getFormObjectInfo(sec, para, ci));
  }

  // ── 검색/치환 API ──

  /**
   * [#3865] includeCells 를 켜면 표 셀 안의 일반 텍스트 매치도 돌려준다. 그 결과에는
   * cellContext 가 실리므로 호출자는 셀 좌표로 커서를 옮길 수 있어야 한다.
   * 기본값은 종전대로 본문만 — 셀 이동을 못 하는 호출자가 무회귀로 남는다.
   */
  searchText(query: string, fromSec: number, fromPara: number, fromChar: number, forward: boolean, caseSensitive: boolean, includeCells: boolean = false): import('./types').SearchResult {
    if (!this.doc || typeof (this.doc as any).searchText !== 'function') return { found: false };
    return JSON.parse((this.doc as any).searchText(query, fromSec, fromPara, fromChar, forward, caseSensitive, includeCells));
  }

  searchAllText(query: string, caseSensitive: boolean, includeCells: boolean = false): import('./types').SearchHit[] {
    if (!this.doc || typeof (this.doc as any).searchAllText !== 'function') return [];
    return JSON.parse((this.doc as any).searchAllText(query, caseSensitive, includeCells));
  }

  replaceText(sec: number, para: number, charOffset: number, length: number, newText: string): import('./types').ReplaceResult {
    if (!this.doc || typeof (this.doc as any).replaceText !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).replaceText(sec, para, charOffset, length, newText));
  }

  replaceOne(query: string, newText: string, caseSensitive: boolean): import('./types').ReplaceOneResult {
    if (!this.doc || typeof (this.doc as any).replaceOne !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).replaceOne(query, newText, caseSensitive));
  }

  replaceAll(query: string, newText: string, caseSensitive: boolean): import('./types').ReplaceAllResult {
    if (!this.doc || typeof (this.doc as any).replaceAll !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).replaceAll(query, newText, caseSensitive));
  }

  getPositionOfPage(globalPage: number): { ok: boolean; sec?: number; para?: number; charOffset?: number } {
    if (!this.doc || typeof (this.doc as any).getPositionOfPage !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).getPositionOfPage(globalPage));
  }

  getPageOfPosition(sectionIdx: number, paraIdx: number): import('./types').PageOfPositionResult {
    if (!this.doc || typeof (this.doc as any).getPageOfPosition !== 'function') return { ok: false };
    return JSON.parse((this.doc as any).getPageOfPosition(sectionIdx, paraIdx));
  }

  // ── 책갈피 API ──

  getBookmarks(): BookmarkInfo[] {
    if (!this.doc) return [];
    try {
      const json = (this.doc as any).getBookmarks();
      return typeof json === 'string' ? JSON.parse(json) : json;
    } catch { return []; }
  }

  addBookmark(sec: number, para: number, charOffset: number, name: string): { ok: boolean; error?: string } {
    if (!this.doc) return { ok: false, error: '문서가 로드되지 않았습니다' };
    try {
      const json = (this.doc as any).addBookmark(sec, para, charOffset, name);
      return typeof json === 'string' ? JSON.parse(json) : json;
    } catch (e) { return { ok: false, error: String(e) }; }
  }

  deleteBookmark(sec: number, para: number, ctrlIdx: number): { ok: boolean; error?: string } {
    if (!this.doc) return { ok: false, error: '문서가 로드되지 않았습니다' };
    try {
      const json = (this.doc as any).deleteBookmark(sec, para, ctrlIdx);
      return typeof json === 'string' ? JSON.parse(json) : json;
    } catch (e) { return { ok: false, error: String(e) }; }
  }

  renameBookmark(sec: number, para: number, ctrlIdx: number, newName: string): { ok: boolean; error?: string } {
    if (!this.doc) return { ok: false, error: '문서가 로드되지 않았습니다' };
    try {
      const json = (this.doc as any).renameBookmark(sec, para, ctrlIdx, newName);
      return typeof json === 'string' ? JSON.parse(json) : json;
    } catch (e) { return { ok: false, error: String(e) }; }
  }

  dispose(): void {
    if (this.doc) {
      this.doc.free();
      this.doc = null;
    }
  }
}
