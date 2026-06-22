import init, { HwpDocument, version } from '@wasm/rhwp.js';
import type { DocumentInfo, PageInfo, PageDef, SectionDef, PageBorderFillSettings, EndnoteShapeSettings, NoteEditInfo, CursorRect, HitTestResult, BodyFootnoteMarkerHit, FootnoteAtCursorResult, DeleteFootnoteResult, LineInfo, TableDimensions, CellInfo, CellBbox, CellProperties, TableProperties, DocumentPosition, MoveVerticalResult, SelectionRect, CharProperties, ParaProperties, CellPathEntry, CellPathLike, NavContextEntry, FieldInfoResult, BookmarkInfo, LayerRenderProfile, PageLayerTree } from './types';

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
import { fontFamilyChainForDisplay } from './font-substitution';
import type { FileSystemFileHandleLike } from '@/command/file-system-access';

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

  async initialize(): Promise<void> {
    if (this.initialized) return;
    installCanvasFontSubstitution();
    this.installMeasureTextWidth();
    await init();
    this.initialized = true;
    console.log(`[WasmBridge] WASM 초기화 완료 (rhwp ${version()})`);
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
  }

  loadDocument(data: Uint8Array, fileName?: string): DocumentInfo {
    this.releaseDocument();
    const nextFileName = fileName ?? 'document.hwp';
    let nextDoc: HwpDocument | null = null;

    try {
      nextDoc = new HwpDocument(data);
      this.doc = nextDoc;
      this._fileName = nextFileName;
      this.doc.convertToEditable();
      this.ensureParagraphStableIds();
      this.doc.setFileName(this._fileName);
      const info: DocumentInfo = JSON.parse(this.doc.getDocumentInfo());
      console.log(`[WasmBridge] 문서 로드: ${info.pageCount}페이지`);

      // [Task #741 후속] 외부 file path 그림 영역 영역 dev 환경 영역 영역 fetch (basename 영역
      // 영역 영역 same dir 영역 image 영역 영역 영역 — 본 환경 dev 영역 영역 samples/ 영역
      // Vite asset). 영역 영역 영역 영역 영역 부재 영역 영역 placeholder 표시.
      void this.populateExternalImagesFromDevServer();

      return info;
    } catch (error) {
      if (this.doc === nextDoc) {
        this.doc = null;
      }
      if (nextDoc) {
        try {
          nextDoc.free();
        } catch {
          /* noop */
        }
      }
      this._fileName = 'document.hwp';
      this._currentFileHandle = null;
      throw error;
    }
  }

  /** [Task #741 후속] 외부 file path 그림 영역 영역 dev 서버 영역 영역 fetch + inject. */
  private async populateExternalImagesFromDevServer(): Promise<void> {
    if (!this.doc) return;
    try {
      const basenamesJson = this.doc.getExternalImageBasenames();
      const basenames: string[] = JSON.parse(basenamesJson);
      if (basenames.length === 0) return;
      console.log(`[WasmBridge] 외부 image 영역 영역 ${basenames.length}개 영역 영역 fetch 시도`);
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
          console.log(`[WasmBridge] 외부 image inject: ${name} → ${displayPath || url} (${buf.byteLength} bytes, ${injected} 영역)`);
        } catch (e) {
          console.warn(`[WasmBridge] 외부 image 영역 영역 영역: ${name}`, e);
        }
      }
      // 갱신된 image 영역 영역 영역 화면 영역 영역 영역 — eventBus 영역 영역 document-changed 영역 영역.
      // (caller 영역 영역 영역 별도 영역 영역 reflow 영역 영역.)
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
    this.doc.setFileName(this._fileName);
    console.log(`[WasmBridge] 새 문서 생성: ${info.pageCount}페이지`);
    return info;
  }

  get fileName(): string {
    return this._fileName;
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

  get isNewDocument(): boolean {
    return this._fileName === '새 문서.hwp';
  }

  exportHwp(): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportHwp();
  }

  exportHwpx(): Uint8Array {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportHwpx();
  }

  /** HWP 직렬화 + 자기 재로드 검증 메타데이터를 JSON 문자열로 반환 (#178). */
  exportHwpVerify(): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.exportHwpVerify();
  }

  getSourceFormat(): string {
    return this.doc?.getSourceFormat?.() ?? 'hwp';
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
   *                  'behind' = BehindText overlay, 'front' = InFrontOfText overlay
   */
  renderPageToCanvasFiltered(
    pageNum: number,
    canvas: HTMLCanvasElement,
    scale: number,
    layerKind: 'all' | 'background' | 'flow' | 'behind' | 'front',
  ): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      renderPageToCanvasFiltered?: (p: number, c: HTMLCanvasElement, s: number, k: string) => void;
    };
    if (typeof d.renderPageToCanvasFiltered === 'function') {
      d.renderPageToCanvasFiltered(pageNum, canvas, scale, layerKind);
      return;
    }
    // 구버전 WASM(public/rhwp.js 등): 레이어 필터 API 없음 → 전체 캔버스 렌더로 폴백
    this.doc.renderPageToCanvas(pageNum, canvas, scale);
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

  getPageLayerTreeObject(pageNum: number, profile: LayerRenderProfile = 'screen'): PageLayerTree {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      getPageLayerTreeWithProfile?: (p: number, profile: string) => string;
      getPageLayerTree?: (p: number) => string;
    };
    const json = typeof d.getPageLayerTreeWithProfile === 'function'
      ? d.getPageLayerTreeWithProfile(pageNum, profile)
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
    if (!tree.profile) {
      tree.profile = profile;
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

  getCanvasKitReplayPlan(pageNum: number, mode: 'default' | 'compat' = 'default'): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as {
      getCanvasKitReplayPlan?: (p: number, mode: string) => string;
    };
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
    });
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

  deleteText(sec: number, para: number, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.deleteText(sec, para, charOffset, count);
  }

  splitParagraph(sec: number, para: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.splitParagraph(sec, para, charOffset);
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

  splitParagraphInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.splitParagraphInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset);
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

  /** 비교/스냅샷 생성 직전에만 stable_id를 보정한다(문서 로드 시 자동 호출 금지). */
  ensureParagraphStableIds(): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    const d = this.doc as unknown as { ensureParagraphStableIds?: () => void };
    if (typeof d.ensureParagraphStableIds === 'function') {
      try {
        d.ensureParagraphStableIds();
      } catch (e) {
        console.warn('[WasmBridge] ensureParagraphStableIds skipped:', e);
      }
    }
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

  deleteTextInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, charOffset: number, count: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.deleteTextInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, count);
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

  splitParagraphInCellByPath(sec: number, parentPara: number, pathJson: string, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).splitParagraphInCellByPath(sec, parentPara, pathJson, charOffset);
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

  setCellProperties(sec: number, parentPara: number, controlIdx: number, cellIdx: number, props: Partial<CellProperties>): { ok: boolean } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.setCellProperties(sec, parentPara, controlIdx, cellIdx, JSON.stringify(props)));
  }

  resizeTableCells(
    sec: number, parentPara: number, controlIdx: number,
    updates: Array<{ cellIdx: number; widthDelta?: number; heightDelta?: number }>,
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

  deleteTextInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number, charOffset: number, count: number): { ok: boolean; charOffset: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).deleteTextInFootnote(sec, para, controlIdx, fnParaIdx, charOffset, count));
  }

  splitParagraphInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number, charOffset: number): { ok: boolean; fnParaIndex: number; charOffset: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).splitParagraphInFootnote(sec, para, controlIdx, fnParaIdx, charOffset));
  }

  mergeParagraphInFootnote(sec: number, para: number, controlIdx: number, fnParaIdx: number): { ok: boolean; fnParaIndex: number; charOffset: number } {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse((this.doc as any).mergeParagraphInFootnote(sec, para, controlIdx, fnParaIdx));
  }

  getPageFootnoteInfo(pageNum: number, footnoteIndex: number): { ok: boolean; sectionIdx: number; paraIdx: number; controlIdx: number; sourceType: string } | null {
    if (!this.doc) return null;
    try {
      return JSON.parse((this.doc as any).getPageFootnoteInfo(pageNum, footnoteIndex));
    } catch { return null; }
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

  getSelectionRectsInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, startCellPara: number, startOffset: number, endCellPara: number, endOffset: number): SelectionRect[] {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return JSON.parse(this.doc.getSelectionRectsInCell(sec, parentPara, controlIdx, cellIdx, startCellPara, startOffset, endCellPara, endOffset));
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

  setCharShapeIdInCell(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, startOffset: number, endOffset: number, charShapeId: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return (this.doc as any).setCharShapeIdInCell(sec, parentPara, controlIdx, cellIdx, cellParaIdx, startOffset, endOffset, charShapeId);
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

  /** 머리말/꼬리말 문단에 필드 마커를 삽입한다 (1=쪽번호, 2=총쪽수, 3=파일이름) */
  insertFieldInHf(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, charOffset: number, fieldType: number): { ok: boolean; charOffset: number } {
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

  splitParagraphInHeaderFooter(sec: number, isHeader: boolean, applyTo: number, hfParaIdx: number, charOffset: number): string {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    return this.doc.splitParagraphInHeaderFooter(sec, isHeader, applyTo, hfParaIdx, charOffset);
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

  searchText(query: string, fromSec: number, fromPara: number, fromChar: number, forward: boolean, caseSensitive: boolean): import('./types').SearchResult {
    if (!this.doc || typeof (this.doc as any).searchText !== 'function') return { found: false };
    return JSON.parse((this.doc as any).searchText(query, fromSec, fromPara, fromChar, forward, caseSensitive));
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
