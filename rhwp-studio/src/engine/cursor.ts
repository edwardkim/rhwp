import type { DocumentPosition, CursorRect, LineInfo, CellPathEntry, NavContextEntry, CellBbox } from '@/core/types';
import { WasmBridge } from '@/core/wasm-bridge';

type CellSelectionReason = 'manual' | 'protected';

type PictureSelectionRef = {
  sec: number;
  ppi: number;
  ci: number;
  type: 'image' | 'shape' | 'equation' | 'group' | 'line';
  cellIdx?: number;
  cellParaIdx?: number;
  outerTableControlIdx?: number;
  cellPath?: CellPathEntry[];
  noteRef?: any;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
};

/** 커서 상태를 관리한다 */
export class CursorState {
  private position: DocumentPosition = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
  private rect: CursorRect | null = null;

  /** 수직 이동 시 원래 X 좌표를 기억 (§6.4.4 preferred X) */
  private preferredX: number | null = null;

  /** 줄 끝 이동 후 경계 위치 판별용 — soft-wrap 줄 경계에서 charEnd == 다음 줄 charStart 동일 문제 해결 */
  private atLineEnd = false;

  /** 선택 시작점 (anchor). null이면 선택 없음 */
  private anchor: DocumentPosition | null = null;
  /** 각주/미주 내부 선택 시작점. 본문 anchor와 별도로 관리한다. */
  private fnAnchor: { fnParaIdx: number; charOffset: number } | null = null;

  // ─── 머리말/꼬리말 편집 모드 ──────────────────────────────
  private _headerFooterMode: 'none' | 'header' | 'footer' = 'none';
  private _hfSectionIdx = 0;
  private _hfApplyTo = 0; // 0=Both, 1=Even, 2=Odd
  private _hfParaIdx = 0;
  private _hfCharOffset = 0;
  /** 머리말/꼬리말이 위치한 선호 페이지 (더블클릭한 페이지) */
  private _hfPreferredPage = -1;
  /** 편집 모드 진입 전 본문 커서 위치 (탈출 시 복원용) */
  private _savedBodyPosition: DocumentPosition | null = null;

  // ─── 각주 편집 모드 ──────────────────────────────────────
  private _footnoteMode = false;
  /** 각주가 속한 본문 문단 인덱스 */
  private _fnParaIdx = 0;
  /** 각주 컨트롤 인덱스 */
  private _fnControlIdx = 0;
  /** 각주 내 문단 인덱스 */
  private _fnInnerParaIdx = 0;
  /** 각주 내 문자 오프셋 */
  private _fnCharOffset = 0;
  /** 각주가 표시된 페이지 */
  private _fnPageNum = 0;
  /** footnotes 배열 내 인덱스 */
  private _fnFootnoteIndex = 0;
  /** 각주 구역 인덱스 */
  private _fnSectionIdx = 0;

  // ─── F5 셀 블록 선택 ──────────────────────────────────────
  private _cellSelectionMode = false;
  /** 셀 선택 단계: 1=단일셀, 2=범위선택, 3=전체선택 */
  private _cellSelectionPhase = 1;
  private _cellSelectionReason: CellSelectionReason = 'manual';
  private cellAnchor: { row: number; col: number } | null = null;
  private cellFocus: { row: number; col: number } | null = null;
  /** Ctrl+클릭으로 제외된 셀 ("row,col" 문자열 Set) */
  private excludedCells = new Set<string>();
  /** 셀 선택 모드 시 표의 식별 정보 (sec/ppi/ci/dims) */
  private cellTableCtx: { sec: number; ppi: number; ci: number; rowCount: number; colCount: number; cellPath?: CellPathEntry[] } | null = null;

  // ─── F5 본문 블록 선택 모드 (#220) ────────────────────────
  private _blockSelectionMode = false;
  private _expandPhase = 0; // F3 확장 단계: 0=none, 1=word, 2=sentence, 3=paragraph, 4=section, 5=document

  // ─── 표 객체 선택 ──────────────────────────────────────
  private _tableObjectSelected = false;
  private selectedTableRef: { sec: number; ppi: number; ci: number; cellPath?: CellPathEntry[] } | null = null;

  constructor(private wasm: WasmBridge) {}

  // ─── Selection (선택 영역) ──────────────────────────────

  /** 선택 영역이 있는지 반환한다 */
  hasSelection(): boolean {
    return this.anchor !== null || this.fnAnchor !== null;
  }

  /** 선택 영역 (anchor → focus)을 반환한다 */
  getSelection(): { anchor: DocumentPosition; focus: DocumentPosition } | null {
    if (!this.anchor) return null;
    return { anchor: { ...this.anchor }, focus: { ...this.position } };
  }

  /** 선택 영역을 start < end 순서로 반환한다 */
  getSelectionOrdered(): { start: DocumentPosition; end: DocumentPosition } | null {
    if (!this.anchor) return null;
    const cmp = CursorState.comparePositions(this.anchor, this.position);
    if (cmp <= 0) {
      return { start: { ...this.anchor }, end: { ...this.position } };
    } else {
      return { start: { ...this.position }, end: { ...this.anchor } };
    }
  }

  /** 각주/미주 내부 선택 영역을 반환한다. */
  getFootnoteSelection(): {
    anchor: { fnParaIdx: number; charOffset: number };
    focus: { fnParaIdx: number; charOffset: number };
    pageNum: number;
    footnoteIndex: number;
  } | null {
    if (!this.fnAnchor) return null;
    return {
      anchor: { ...this.fnAnchor },
      focus: { fnParaIdx: this._fnInnerParaIdx, charOffset: this._fnCharOffset },
      pageNum: this._fnPageNum,
      footnoteIndex: this._fnFootnoteIndex,
    };
  }

  /** 각주/미주 내부 선택 영역을 start < end 순서로 반환한다. */
  getFootnoteSelectionOrdered(): {
    start: { fnParaIdx: number; charOffset: number };
    end: { fnParaIdx: number; charOffset: number };
    pageNum: number;
    footnoteIndex: number;
  } | null {
    if (!this.fnAnchor) return null;
    const focus = { fnParaIdx: this._fnInnerParaIdx, charOffset: this._fnCharOffset };
    const cmp = CursorState.compareFootnotePositions(this.fnAnchor, focus);
    if (cmp <= 0) {
      return {
        start: { ...this.fnAnchor },
        end: focus,
        pageNum: this._fnPageNum,
        footnoteIndex: this._fnFootnoteIndex,
      };
    }
    return {
      start: focus,
      end: { ...this.fnAnchor },
      pageNum: this._fnPageNum,
      footnoteIndex: this._fnFootnoteIndex,
    };
  }

  /** 현재 위치를 anchor로 설정 (선택 시작) */
  setAnchor(): void {
    if (!this.anchor) {
      this.anchor = { ...this.position };
    }
  }

  /** 현재 각주/미주 내부 위치를 anchor로 설정한다. */
  setFnAnchor(): void {
    if (!this.fnAnchor) {
      this.fnAnchor = {
        fnParaIdx: this._fnInnerParaIdx,
        charOffset: this._fnCharOffset,
      };
    }
  }

  /** 선택을 해제한다 */
  clearSelection(): void {
    this.anchor = null;
    this.fnAnchor = null;
  }

  static compareFootnotePositions(
    a: { fnParaIdx: number; charOffset: number },
    b: { fnParaIdx: number; charOffset: number },
  ): number {
    if (a.fnParaIdx !== b.fnParaIdx) return a.fnParaIdx < b.fnParaIdx ? -1 : 1;
    if (a.charOffset !== b.charOffset) return a.charOffset < b.charOffset ? -1 : 1;
    return 0;
  }

  /** 두 DocumentPosition을 비교한다 (-1: a<b, 0: a==b, 1: a>b) */
  static comparePositions(a: DocumentPosition, b: DocumentPosition): number {
    // 본문↔셀 혼합 비교: 셀 컨텍스트가 다르면 parentParaIndex 기준으로 비교
    const aInCell = a.parentParaIndex !== undefined;
    const bInCell = b.parentParaIndex !== undefined;

    if (aInCell && bInCell) {
      // 둘 다 셀 내부 — 같은 셀인지 확인
      if (a.parentParaIndex !== b.parentParaIndex ||
          a.controlIndex !== b.controlIndex ||
          a.cellIndex !== b.cellIndex) {
        // 다른 셀이면 셀 인덱스로 비교
        if (a.parentParaIndex !== b.parentParaIndex) return a.parentParaIndex! < b.parentParaIndex! ? -1 : 1;
        if (a.controlIndex !== b.controlIndex) return a.controlIndex! < b.controlIndex! ? -1 : 1;
        return a.cellIndex! < b.cellIndex! ? -1 : 1;
      }
      // 같은 셀 내부: cellParaIndex → charOffset 비교
      if (a.cellParaIndex !== b.cellParaIndex) return a.cellParaIndex! < b.cellParaIndex! ? -1 : 1;
      if (a.charOffset !== b.charOffset) return a.charOffset < b.charOffset ? -1 : 1;
      return 0;
    }

    if (!aInCell && !bInCell) {
      // 둘 다 본문
      if (a.sectionIndex !== b.sectionIndex) return a.sectionIndex < b.sectionIndex ? -1 : 1;
      if (a.paragraphIndex !== b.paragraphIndex) return a.paragraphIndex < b.paragraphIndex ? -1 : 1;
      if (a.charOffset !== b.charOffset) return a.charOffset < b.charOffset ? -1 : 1;
      return 0;
    }

    // 본문↔셀 혼합: parentParaIndex를 본문 paragraphIndex와 비교
    const aParaRef = aInCell ? a.parentParaIndex! : a.paragraphIndex;
    const bParaRef = bInCell ? b.parentParaIndex! : b.paragraphIndex;
    if (aParaRef !== bParaRef) return aParaRef < bParaRef ? -1 : 1;
    // 같은 문단이면 셀이 본문보다 뒤로 간주
    return aInCell ? 1 : -1;
  }

  /** 현재 커서 위치를 반환한다 */
  getPosition(): DocumentPosition {
    return { ...this.position };
  }

  /** 현재 커서의 픽셀 좌표를 반환한다 */
  getRect(): CursorRect | null {
    return this.rect ? { ...this.rect } : null;
  }

  /** 커서가 셀 내부에 있는지 반환한다 */
  isInCell(): boolean {
    return (this.position.cellPath?.length ?? 0) > 0
        || this.position.parentParaIndex !== undefined;
  }

  /** 중첩 표 깊이를 반환한다 (0=본문, 1=단일 표, 2+=중첩 표) */
  nestingDepth(): number {
    return this.position.cellPath?.length ?? (this.isInCell() ? 1 : 0);
  }

  /** 커서가 글상자 내부에 있는지 반환한다 */
  isInTextBox(): boolean {
    return this.position.isTextBox === true;
  }

  /** 커서가 세로쓰기 셀 내부에 있는지 반환한다 */
  isInVerticalCell(): boolean {
    if (!this.isInCell() || this.isInTextBox()) return false;
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellIndex: cei } = this.position;
    if (ppi === undefined || ci === undefined || cei === undefined) return false;
    try {
      return this.wasm.getCellTextDirection(sec, ppi, ci, cei) !== 0;
    } catch {
      return false;
    }
  }

  /** 커서를 문서 위치로 이동한다 */
  moveTo(pos: DocumentPosition): void {
    this.position = { ...pos };
    this.atLineEnd = false;
    this.updateRect();
  }

  /** preferredX 초기화 (수평 이동/클릭/편집 시) */
  resetPreferredX(): void {
    this.preferredX = null;
    this.atLineEnd = false;
  }

  // ─── 수평 이동 ──────────────────────────────────────────

  /** 커서를 좌/우로 이동한다 — 문서 트리 DFS 기반 통합 이동 */
  moveHorizontal(delta: number): void {
    this.preferredX = null;
    this.atLineEnd = false;

    // 표 셀 내부는 기존 로직 유지 (Tab 셀 이동과 연동)
    if (this.isInCell() && !this.isInTextBox()) {
      this.moveHorizontalInCell(delta);
      return;
    }

    const savedPos = { ...this.position };

    // 렌더 불가능한 위치(오버플로우 텍스트 등)를 자동으로 건너뛰기 위해
    // updateRect 실패 시 같은 방향으로 재시도한다 (최대 maxRetries회).
    const maxRetries = 20;
    for (let attempt = 0; attempt < maxRetries; attempt++) {
      const pos = this.position;
      const sec = pos.sectionIndex;
      const context = this.buildNavContext();
      const effectivePara = this.getEffectivePara(context);

      const contextJson = JSON.stringify(context);

      const result = this.wasm.navigateNextEditable(
        sec, effectivePara, pos.charOffset, delta,
        contextJson,
      );

      if (result.type !== 'text') {
        // boundary → 원래 위치로 복원
        this.position = savedPos;
        return;
      }

      this.applyNavResult(sec, result);

      if (this.rect) {
        // 렌더 가능한 위치에 도착 → 성공
        return;
      }
      // rect가 null → 렌더 불가능한 위치, 같은 방향으로 재시도
    }

    // 재시도 한도 초과 → 원래 위치로 복원
    this.position = savedPos;
    this.updateRect();
  }

  /** 현재 DocumentPosition에서 NavContextEntry[] 구성 */
  private buildNavContext(): NavContextEntry[] {
    const pos = this.position;
    if (pos.isTextBox && pos.parentParaIndex !== undefined && pos.controlIndex !== undefined) {
      return [{
        parentPara: pos.parentParaIndex,
        ctrlIdx: pos.controlIndex,
        ctrlTextPos: 0, // Rust가 재계산
        cellIdx: 0,
        isTextBox: true,
      }];
    }
    // body (표 셀은 moveHorizontalInCell에서 별도 처리)
    return [];
  }

  /** context 기반 현재 문단 인덱스 반환 */
  private getEffectivePara(context: NavContextEntry[]): number {
    if (context.length > 0) {
      // 컨테이너 내부에서는 cellParaIndex가 실제 문단 인덱스
      return this.position.cellParaIndex ?? this.position.paragraphIndex;
    }
    return this.position.paragraphIndex;
  }

  /** NavResult를 DocumentPosition으로 변환하여 적용 */
  private applyNavResult(
    currentSec: number,
    result: { type: string; sec: number; para: number; charOffset: number; context: NavContextEntry[] },
  ): void {
    const ctx = result.context;

    if (ctx.length === 0) {
      // Body 본문 텍스트
      this.position = {
        sectionIndex: result.sec,
        paragraphIndex: result.para,
        charOffset: result.charOffset,
      };
    } else {
      const last = ctx[ctx.length - 1];
      if (last.isTextBox) {
        // TextBox 내부
        this.position = {
          sectionIndex: result.sec,
          paragraphIndex: result.para,
          charOffset: result.charOffset,
          parentParaIndex: last.parentPara,
          controlIndex: last.ctrlIdx,
          cellIndex: 0,
          cellParaIndex: result.para,
          isTextBox: true,
        };
      } else {
        // Table Cell 내부
        this.position = {
          sectionIndex: result.sec,
          paragraphIndex: result.para,
          charOffset: result.charOffset,
          parentParaIndex: last.parentPara,
          controlIndex: last.ctrlIdx,
          cellIndex: last.cellIdx,
          cellParaIndex: result.para,
        };
      }
    }
    this.updateRect();
  }

  /** 셀 내부 좌/우 이동 (한컴 방식: 셀 경계에서 다음/이전 셀로 이동) */
  private moveHorizontalInCell(delta: number): void {
    const pos = this.position;
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellIndex: cei, charOffset, cellPath } = pos;
    const useCellPath = (cellPath?.length ?? 0) > 0 && ppi !== undefined;
    const cpi = useCellPath ? cellPath![cellPath!.length - 1].cellParaIndex : pos.cellParaIndex!;
    let newOffset = charOffset + delta;

    // hit-test가 cellPath를 제공하면 1-depth 표라도 경로 기반 API가 더 정확하다.
    const getParaLen = (paraIdx: number): number => {
      if (useCellPath) {
        const path = cellPath!;
        const p = path.map((e, i) => i < path.length - 1 ? e : { ...e, cellParaIndex: paraIdx });
        return this.wasm.getCellParagraphLengthByPath(sec, ppi!, JSON.stringify(p));
      }
      return this.wasm.getCellParagraphLength(sec, ppi!, ci!, cei!, paraIdx);
    };
    const getParaCount = (): number => {
      if (useCellPath) {
        return this.wasm.getCellParagraphCountByPath(sec, ppi!, JSON.stringify(cellPath));
      }
      return this.wasm.getCellParagraphCount(sec, ppi!, ci!, cei!);
    };
    const updateCellParaInPath = (newCpi: number): DocumentPosition => {
      const newPos = { ...pos, paragraphIndex: newCpi, cellParaIndex: newCpi };
      if (cellPath && cellPath.length > 0) {
        newPos.cellPath = cellPath.map((e, i) => i < cellPath.length - 1 ? e : { ...e, cellParaIndex: newCpi });
      }
      return newPos;
    };

    if (newOffset < 0) {
      if (cpi > 0) {
        // 이전 셀 문단 끝으로 이동
        const prevLen = getParaLen(cpi - 1);
        this.position = { ...updateCellParaInPath(cpi - 1), charOffset: prevLen };
      } else {
        // 셀 첫 문단 시작에서 ArrowLeft → 이전 셀로 이동
        this.moveToCellPrev();
        return; // updateRect는 moveToCellPrev 내부에서 호출
      }
    } else {
      const paraLen = getParaLen(cpi);
      if (newOffset > paraLen) {
        const paraCount = getParaCount();
        if (cpi + 1 < paraCount) {
          // 다음 셀 문단 시작으로 이동
          this.position = { ...updateCellParaInPath(cpi + 1), charOffset: 0 };
        } else {
          // 셀 마지막 문단 끝에서 ArrowRight → 다음 셀로 이동
          this.moveToCellNext();
          return;
        }
      } else {
        this.position = { ...pos, charOffset: newOffset };
      }
    }

    this.updateRect();
  }

  // ─── 수직 이동 (ArrowUp/Down) ──────────────────────────

  /** 커서를 위/아래로 이동한다 (delta: -1=위, +1=아래) — WASM 단일 호출 */
  moveVertical(delta: number): void {
    const wasAtLineEnd = this.atLineEnd;
    this.atLineEnd = false;
    let px = this.preferredX ?? this.rect?.x ?? -1.0;
    const pos = this.position;
    let queryPos = pos;
    let preserveLineEndAffinity = false;

    if (wasAtLineEnd && pos.charOffset > 0) {
      try {
        const prevLineInfo = this.getLineInfoForOffset(pos, pos.charOffset - 1);
        if (prevLineInfo.charEnd === pos.charOffset) {
          queryPos = { ...pos, charOffset: pos.charOffset - 1 };
          // soft-wrap 경계의 줄 끝은 다음 줄 시작 offset과 같다.
          // X를 줄 끝 쪽으로 보낸 뒤 WASM이 대상 줄에서 가장 오른쪽 후보를 고르게 한다.
          px = Number.MAX_SAFE_INTEGER;
          preserveLineEndAffinity = true;
        }
      } catch {
        queryPos = pos;
      }
    }

    try {
      let result;
      if ((queryPos.cellPath?.length ?? 0) > 0 && queryPos.parentParaIndex !== undefined) {
        // cellPath가 있으면 1-depth 표/글상자도 경로 기반 API 사용
        const pathJson = JSON.stringify(queryPos.cellPath);
        result = this.wasm.moveVerticalByPath(
          queryPos.sectionIndex, queryPos.parentParaIndex, pathJson,
          queryPos.charOffset, delta, px,
        );
      } else {
        const ppi = queryPos.parentParaIndex ?? 0xFFFFFFFF;
        const ci = queryPos.controlIndex ?? 0xFFFFFFFF;
        const cei = queryPos.cellIndex ?? 0xFFFFFFFF;
        const cpi = queryPos.cellParaIndex ?? 0xFFFFFFFF;
        result = this.wasm.moveVertical(
          queryPos.sectionIndex, queryPos.paragraphIndex, queryPos.charOffset,
          delta, px,
          ppi, ci, cei, cpi,
        );
      }

      // 위치 갱신
      this.position = {
        sectionIndex: result.sectionIndex,
        paragraphIndex: result.paragraphIndex,
        charOffset: result.charOffset,
        parentParaIndex: result.parentParaIndex,
        controlIndex: result.controlIndex,
        cellIndex: result.cellIndex,
        cellParaIndex: result.cellParaIndex,
        cellPath: result.cellPath,
        isTextBox: result.isTextBox,
      };

      // 캐럿 좌표 갱신
      if (result.rectValid === false) {
        // 좌표 조회 실패 → updateRect()로 별도 조회 시도
        this.updateRect();
      } else {
        this.rect = {
          pageIndex: result.pageIndex,
          x: result.x,
          y: result.y,
          height: result.height,
        };
      }

      this.atLineEnd = preserveLineEndAffinity && this.isPreviousLineEnd(this.position);
      // preferredX 저장 (다음 연속 이동에 재사용)
      this.preferredX = this.atLineEnd ? (this.rect?.x ?? result.x) : result.preferredX;
    } catch (e) {
      console.warn('[CursorState] moveVertical 실패:', e);
    }
  }

  // ─── Home/End (줄 시작/끝) ──────────────────────────────

  /** 줄 시작으로 이동 (Home) */
  moveToLineStart(): void {
    this.preferredX = null;
    try {
      const pos = this.position;
      let lineInfo = this.getLineInfoAtCursor();
      // #785: soft-wrap 줄 경계 — charEnd(line N) == charStart(line N+1) 동일 위치.
      // End 키 후 Home 키 시 getLineInfo 가 다음 줄로 판정하여 charStart = 현재 위치 → 미이동.
      // atLineEnd 플래그가 설정된 상태에서 현재 위치가 줄 시작과 동일하면 이전 줄로 판정.
      if (this.atLineEnd && pos.charOffset === lineInfo.charStart && pos.charOffset > 0) {
        const prevLineInfo = this.isInCell()
          ? this.wasm.getLineInfoInCell(
              pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
              pos.cellIndex!, pos.cellParaIndex!, pos.charOffset - 1)
          : this.wasm.getLineInfo(pos.sectionIndex, pos.paragraphIndex, pos.charOffset - 1);
        if (prevLineInfo.charEnd === pos.charOffset) {
          lineInfo = prevLineInfo;
        }
      }
      this.atLineEnd = false;
      this.position = { ...this.position, charOffset: lineInfo.charStart };
      const visualRect = this.getCursorRectOnVisualLine(lineInfo.lineIndex, false);
      if (visualRect) this.rect = visualRect;
      else this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToLineStart 실패:', e);
    }
  }

  /** 줄 끝으로 이동 (End) */
  moveToLineEnd(): void {
    this.preferredX = null;
    try {
      const pos = this.position;
      let lineInfo = this.getLineInfoAtCursor();
      // soft-wrap 줄 끝 offset은 다음 줄 시작 offset과 같다.
      // End를 반복 입력할 때 현재 줄 끝 affinity를 잃고 다음 줄 끝으로 이동하지 않도록 한다.
      if (this.atLineEnd && pos.charOffset === lineInfo.charStart && pos.charOffset > 0) {
        const prevLineInfo = this.getLineInfoForOffset(pos, pos.charOffset - 1);
        if (prevLineInfo.charEnd === pos.charOffset) {
          lineInfo = prevLineInfo;
        }
      }
      this.position = { ...this.position, charOffset: lineInfo.charEnd };
      this.atLineEnd = true;
      const visualRect = this.getCursorRectOnVisualLine(lineInfo.lineIndex, true);
      if (visualRect) this.rect = visualRect;
      else this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToLineEnd 실패:', e);
    }
  }

  /** 현재 커서 위치의 줄 정보를 얻는다 (본문/셀 자동 분기) */
  private getLineInfoAtCursor(): LineInfo {
    return this.getLineInfoForOffset(this.position, this.position.charOffset);
  }

  private getLineInfoForOffset(pos: DocumentPosition, charOffset: number): LineInfo {
    const inCell = (pos.cellPath?.length ?? 0) > 0 || pos.parentParaIndex !== undefined;
    if (inCell) {
      return this.wasm.getLineInfoInCell(
        pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
        pos.cellIndex!, pos.cellParaIndex!, charOffset,
      );
    } else {
      return this.wasm.getLineInfo(pos.sectionIndex, pos.paragraphIndex, charOffset);
    }
  }

  private getCursorRectOnVisualLine(lineIndex: number, atEnd: boolean): CursorRect | null {
    const pos = this.position;
    try {
      return this.wasm.getCursorRectOnLine(
        pos.sectionIndex,
        pos.paragraphIndex,
        lineIndex,
        atEnd,
        pos.parentParaIndex ?? 0xFFFFFFFF,
        pos.controlIndex ?? 0xFFFFFFFF,
        pos.cellIndex ?? 0xFFFFFFFF,
        pos.cellParaIndex ?? 0xFFFFFFFF,
      );
    } catch {
      return null;
    }
  }

  private isPreviousLineEnd(pos: DocumentPosition): boolean {
    if (pos.charOffset <= 0) return false;
    try {
      const prevLineInfo = this.getLineInfoForOffset(pos, pos.charOffset - 1);
      return prevLineInfo.charEnd === pos.charOffset;
    } catch {
      return false;
    }
  }

  // ─── 문서 시작/끝 (Ctrl+Home/End) ──────────────────────

  /** 문서 시작으로 이동 (Ctrl+Home) */
  moveToDocumentStart(): void {
    this.preferredX = null;
    this.atLineEnd = false;
    this.position = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
    this.updateRect();
  }

  /** 문서 끝으로 이동 (Ctrl+End) */
  moveToDocumentEnd(): void {
    this.preferredX = null;
    this.atLineEnd = false;
    try {
      const secCount = this.wasm.getSectionCount();
      const lastSec = secCount > 0 ? secCount - 1 : 0;
      const paraCount = this.wasm.getParagraphCount(lastSec);
      if (paraCount > 0) {
        const lastPara = paraCount - 1;
        const paraLen = this.wasm.getParagraphLength(lastSec, lastPara);
        this.position = { sectionIndex: lastSec, paragraphIndex: lastPara, charOffset: paraLen };
      } else {
        this.position = { sectionIndex: lastSec, paragraphIndex: 0, charOffset: 0 };
      }
      this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToDocumentEnd 실패:', e);
    }
  }

  // ─── 문단 단위 이동 (Ctrl+↑/↓) ────────────────────────

  /** 이전/다음 문단 시작으로 이동 (direction: -1=이전, +1=다음).
   *  한컴 표준 정합 — 본문은 현재 구역 내 문단 이동 (구역 경계 영역 영역 인접 구역 이동).
   *  표 셀 내부는 cellParaIndex 이동 (셀 내부 문단 경계). */
  moveToParagraphBoundary(direction: -1 | 1): void {
    this.preferredX = null;
    this.atLineEnd = false;
    const pos = this.position;

    if (this.isInCell() && !this.isInTextBox()) {
      try {
        const sec = pos.sectionIndex;
        const ppi = pos.parentParaIndex!;
        const ci = pos.controlIndex!;
        const cei = pos.cellIndex!;
        const cellPath = pos.cellPath;
        const useCellPath = (cellPath?.length ?? 0) > 0;
        const cpi = useCellPath ? cellPath![cellPath!.length - 1].cellParaIndex : pos.cellParaIndex!;
        const cellParaCount = useCellPath
          ? this.wasm.getCellParagraphCountByPath(sec, ppi, JSON.stringify(cellPath))
          : this.wasm.getCellParagraphCount(sec, ppi, ci, cei);
        const target = cpi + direction;
        if (target >= 0 && target < cellParaCount) {
          this.position = {
            ...pos,
            paragraphIndex: target,
            cellParaIndex: target,
            cellPath: useCellPath
              ? cellPath!.map((e, i) => i < cellPath!.length - 1 ? e : { ...e, cellParaIndex: target })
              : cellPath,
            charOffset: 0,
          };
          this.updateRect();
        }
      } catch (e) {
        console.warn('[CursorState] moveToParagraphBoundary (cell) 실패:', e);
      }
      return;
    }

    try {
      const sec = pos.sectionIndex;
      const paraCount = this.wasm.getParagraphCount(sec);
      if (direction === 1) {
        const target = pos.paragraphIndex + 1;
        if (target < paraCount) {
          this.position = { ...pos, paragraphIndex: target, charOffset: 0 };
        } else {
          // 구역 경계 — 다음 구역 첫 문단으로
          const secCount = this.wasm.getSectionCount();
          if (sec + 1 < secCount) {
            this.position = { sectionIndex: sec + 1, paragraphIndex: 0, charOffset: 0 };
          } else {
            // 문서 마지막 문단 끝
            const lastPara = paraCount - 1;
            const paraLen = this.wasm.getParagraphLength(sec, lastPara);
            this.position = { ...pos, paragraphIndex: lastPara, charOffset: paraLen };
          }
        }
      } else {
        if (pos.charOffset > 0) {
          // 현재 문단 시작으로 (한컴 표준)
          this.position = { ...pos, charOffset: 0 };
        } else if (pos.paragraphIndex > 0) {
          this.position = { ...pos, paragraphIndex: pos.paragraphIndex - 1, charOffset: 0 };
        } else if (sec > 0) {
          // 구역 경계 — 이전 구역 마지막 문단 시작으로
          const prevParaCount = this.wasm.getParagraphCount(sec - 1);
          this.position = {
            sectionIndex: sec - 1,
            paragraphIndex: prevParaCount > 0 ? prevParaCount - 1 : 0,
            charOffset: 0,
          };
        }
      }
      this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToParagraphBoundary 실패:', e);
    }
  }

  // ─── 단어 단위 이동 (Alt/Option+Arrow) ────────────────

  /** 단어 경계로 이동 (direction: -1=왼쪽, +1=오른쪽) */
  moveToWordBoundary(direction: -1 | 1): void {
    this.preferredX = null;
    const pos = this.position;

    if (this.isInCell() && !this.isInTextBox()) {
      this.moveToWordBoundaryInCell(direction);
      return;
    }

    try {
      const sec = pos.sectionIndex;
      const para = pos.paragraphIndex;
      const paraLen = this.wasm.getParagraphLength(sec, para);

      if (direction === 1) {
        if (pos.charOffset >= paraLen) {
          // 문단 끝 → 다음 문단 시작으로 이동
          this.moveHorizontal(1);
          return;
        }
        const remaining = paraLen - pos.charOffset;
        const text = this.wasm.getTextRange(sec, para, pos.charOffset, Math.min(remaining, 50));
        const offset = findWordBoundaryForward(text);
        this.position = { ...pos, charOffset: pos.charOffset + offset };
      } else {
        if (pos.charOffset <= 0) {
          // 문단 시작 → 이전 문단 끝으로 이동
          this.moveHorizontal(-1);
          return;
        }
        const start = Math.max(0, pos.charOffset - 50);
        const count = pos.charOffset - start;
        const text = this.wasm.getTextRange(sec, para, start, count);
        const offset = findWordBoundaryBackward(text);
        this.position = { ...pos, charOffset: start + offset };
      }
      this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToWordBoundary 실패:', e);
    }
  }

  private moveToWordBoundaryInCell(direction: -1 | 1): void {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const ci = pos.controlIndex!;
    const cei = pos.cellIndex!;
    const cellPath = pos.cellPath;
    const useCellPath = (cellPath?.length ?? 0) > 0;
    const cpi = useCellPath ? cellPath![cellPath!.length - 1].cellParaIndex : pos.cellParaIndex!;

    try {
      const pathJson = useCellPath ? JSON.stringify(cellPath) : '';
      const paraLen = useCellPath
        ? this.wasm.getCellParagraphLengthByPath(sec, ppi, pathJson)
        : this.wasm.getCellParagraphLength(sec, ppi, ci, cei, cpi);

      if (direction === 1) {
        if (pos.charOffset >= paraLen) {
          this.moveHorizontal(1);
          return;
        }
        const remaining = paraLen - pos.charOffset;
        const text = useCellPath
          ? this.wasm.getTextInCellByPath(sec, ppi, pathJson, pos.charOffset, Math.min(remaining, 50))
          : this.wasm.getTextInCell(sec, ppi, ci, cei, cpi, pos.charOffset, Math.min(remaining, 50));
        const offset = findWordBoundaryForward(text);
        this.position = { ...pos, charOffset: pos.charOffset + offset };
      } else {
        if (pos.charOffset <= 0) {
          this.moveHorizontal(-1);
          return;
        }
        const start = Math.max(0, pos.charOffset - 50);
        const count = pos.charOffset - start;
        const text = useCellPath
          ? this.wasm.getTextInCellByPath(sec, ppi, pathJson, start, count)
          : this.wasm.getTextInCell(sec, ppi, ci, cei, cpi, start, count);
        const offset = findWordBoundaryBackward(text);
        this.position = { ...pos, charOffset: start + offset };
      }
      this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToWordBoundaryInCell 실패:', e);
      this.moveHorizontal(direction);
    }
  }

  // ─── 셀 탐색 (Tab/Shift+Tab) ──────────────────────────

  /** 읽기 순서(row → col)로 정렬된 cellIdx 배열을 반환한다 */
  private getCellReadingOrder(): number[] {
    const pos = this.position;
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellPath } = pos;
    const useCellPath = (cellPath?.length ?? 0) > 1 || ((cellPath?.length ?? 0) > 0 && !this.isInTextBox());

    let bboxes: { cellIdx: number; row: number; col: number }[];
    if (useCellPath && cellPath) {
      bboxes = this.wasm.getTableCellBboxesByPath(sec, ppi!, JSON.stringify(cellPath));
    } else {
      bboxes = this.wasm.getTableCellBboxes(sec, ppi!, ci!);
    }

    // cellIdx 기준으로 중복 제거 (다중 페이지 표에서 동일 셀이 여러 페이지에 나타남)
    const seen = new Set<number>();
    const unique: { cellIdx: number; row: number; col: number }[] = [];
    for (const b of bboxes) {
      if (!seen.has(b.cellIdx)) {
        seen.add(b.cellIdx);
        unique.push(b);
      }
    }

    // 읽기 순서: row 우선, col 다음
    unique.sort((a, b) => a.row !== b.row ? a.row - b.row : a.col - b.col);
    return unique.map(u => u.cellIdx);
  }

  /** 다음 셀로 이동 (Tab) — 셀 내부에서만 호출 */
  moveToCellNext(): void {
    if (!this.isInCell()) return;
    this.preferredX = null;
    this.atLineEnd = false;

    const pos = this.position;
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellPath } = pos;
    const useCellPath = (cellPath?.length ?? 0) > 1 || ((cellPath?.length ?? 0) > 0 && !this.isInTextBox());
    const cei = (useCellPath && cellPath) ? cellPath[cellPath.length - 1].cellIndex : pos.cellIndex!;

    try {
      const order = this.getCellReadingOrder();
      const curPos = order.indexOf(cei);
      if (curPos < 0 || curPos + 1 >= order.length) {
        // 마지막 셀 → 표 밖 다음 위치로 이동
        this.exitTable(1);
        return;
      }

      const nextCellIdx = order[curPos + 1];
      this.moveToCellByIndex(sec, ppi!, ci, cellPath, nextCellIdx, 'start');
      this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToCellNext 실패:', e);
    }
  }

  /** 이전 셀로 이동 (Shift+Tab) — 셀 내부에서만 호출 */
  moveToCellPrev(): void {
    if (!this.isInCell()) return;
    this.preferredX = null;
    this.atLineEnd = false;

    const pos = this.position;
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellPath } = pos;
    const useCellPath = (cellPath?.length ?? 0) > 1 || ((cellPath?.length ?? 0) > 0 && !this.isInTextBox());
    const cei = (useCellPath && cellPath) ? cellPath[cellPath.length - 1].cellIndex : pos.cellIndex!;

    try {
      const order = this.getCellReadingOrder();
      const curPos = order.indexOf(cei);
      if (curPos <= 0) {
        // 첫 셀 → 표 밖 이전 위치로 이동
        this.exitTable(-1);
        return;
      }

      const prevCellIdx = order[curPos - 1];
      this.moveToCellByIndex(sec, ppi!, ci, cellPath, prevCellIdx, 'end');
      this.updateRect();
    } catch (e) {
      console.warn('[CursorState] moveToCellPrev 실패:', e);
    }
  }

  /** cellIndex 기준으로 셀 위치를 설정한다 (start=셀 첫 위치, end=셀 마지막 위치) */
  private moveToCellByIndex(
    sec: number, ppi: number, ci: number | undefined,
    cellPath: CellPathEntry[] | undefined,
    targetCellIdx: number, where: 'start' | 'end',
  ): void {
    const useCellPath = (cellPath?.length ?? 0) > 1 || ((cellPath?.length ?? 0) > 0 && !this.isInTextBox());
    if (useCellPath && cellPath) {
      const basePath = cellPath.map((e, i) => i < cellPath.length - 1 ? e : { ...e, cellIndex: targetCellIdx, cellParaIndex: 0 });
      if (where === 'end') {
        const paraCount = this.wasm.getCellParagraphCountByPath(sec, ppi, JSON.stringify(basePath));
        const lastCpi = paraCount > 0 ? paraCount - 1 : 0;
        const lastPath = cellPath.map((e, i) => i < cellPath.length - 1 ? e : { ...e, cellIndex: targetCellIdx, cellParaIndex: lastCpi });
        const lastParaLen = this.wasm.getCellParagraphLengthByPath(sec, ppi, JSON.stringify(lastPath));
        this.position = {
          sectionIndex: sec, paragraphIndex: lastCpi, charOffset: lastParaLen,
          parentParaIndex: ppi, controlIndex: cellPath[0].controlIndex,
          cellIndex: targetCellIdx, cellParaIndex: lastCpi, cellPath: lastPath,
        };
      } else {
        this.position = {
          sectionIndex: sec, paragraphIndex: 0, charOffset: 0,
          parentParaIndex: ppi, controlIndex: cellPath[0].controlIndex,
          cellIndex: targetCellIdx, cellParaIndex: 0, cellPath: basePath,
        };
      }
    } else {
      if (where === 'end') {
        const paraCount = this.wasm.getCellParagraphCount(sec, ppi, ci!, targetCellIdx);
        const lastCpi = paraCount > 0 ? paraCount - 1 : 0;
        const lastParaLen = this.wasm.getCellParagraphLength(sec, ppi, ci!, targetCellIdx, lastCpi);
        this.position = {
          sectionIndex: sec, paragraphIndex: lastCpi, charOffset: lastParaLen,
          parentParaIndex: ppi, controlIndex: ci, cellIndex: targetCellIdx, cellParaIndex: lastCpi,
        };
      } else {
        this.position = {
          sectionIndex: sec, paragraphIndex: 0, charOffset: 0,
          parentParaIndex: ppi, controlIndex: ci, cellIndex: targetCellIdx, cellParaIndex: 0,
        };
      }
    }
  }

  /** 표 밖으로 나가기 (delta: +1=다음 위치, -1=이전 위치) — Tab/Shift+Tab 전용 */
  private exitTable(delta: number): void {
    const { sectionIndex: sec, parentParaIndex: ppi } = this.position;
    if (delta > 0) {
      const paraCount = this.wasm.getParagraphCount(sec);
      if (ppi! + 1 < paraCount) {
        this.position = { sectionIndex: sec, paragraphIndex: ppi! + 1, charOffset: 0 };
      } else {
        return;
      }
    } else {
      this.position = { sectionIndex: sec, paragraphIndex: ppi!, charOffset: 0 };
    }
    this.updateRect();
  }

  // ─── 캐럿 좌표 갱신 ───────────────────────────────────

  /** WASM에서 캐럿 좌표를 갱신한다 */
  updateRect(): void {
    try {
      // 머리말/꼬리말 편집 모드
      if (this._headerFooterMode !== 'none') {
        const isHeader = this._headerFooterMode === 'header';
        this.rect = this.wasm.getCursorRectInHeaderFooter(
          this._hfSectionIdx, isHeader, this._hfApplyTo,
          this._hfParaIdx, this._hfCharOffset, this._hfPreferredPage,
        );
        return;
      }

      // 각주 편집 모드
      if (this._footnoteMode) {
        const noteRect = this.wasm.getCursorRectInNote?.(
          this._fnSectionIdx,
          this._fnParaIdx,
          this._fnControlIdx,
          this._fnInnerParaIdx,
          this._fnCharOffset,
        );
        if (noteRect) {
          this.rect = noteRect;
          return;
        }
        this.rect = this.wasm.getCursorRectInFootnote(
          this._fnPageNum, this._fnFootnoteIndex, this._fnInnerParaIdx, this._fnCharOffset,
        );
        return;
      }

      if ((this.position.cellPath?.length ?? 0) > 0) {
        // cellPath가 있으면 1-depth 표/글상자도 경로 기반 API 사용
        const { sectionIndex: sec, parentParaIndex: ppi, cellPath, charOffset } = this.position;
        const pathJson = JSON.stringify(cellPath);
        this.rect = this.wasm.getCursorRectByPath(sec, ppi!, pathJson, charOffset);
      } else if (this.isInCell()) {
        const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellIndex: cei, cellParaIndex: cpi, charOffset } = this.position;
        this.rect = this.wasm.getCursorRectInCell(sec, ppi!, ci!, cei!, cpi!, charOffset);
      } else {
        this.rect = this.wasm.getCursorRect(
          this.position.sectionIndex,
          this.position.paragraphIndex,
          this.position.charOffset,
        );
      }
      // WASM 결과가 hitTest의 cursorRect와 페이지가 다르면 잘못된 좌표 → 폴백
      if (this.rect && this.position.cursorRect &&
          this.rect.pageIndex !== this.position.cursorRect.pageIndex) {
        console.warn('[CursorState] 캐럿 페이지 불일치 (WASM=%d, hitTest=%d) → hitTest 폴백',
          this.rect.pageIndex, this.position.cursorRect.pageIndex);
        this.rect = { ...this.position.cursorRect };
      }
    } catch (e) {
      // getCursorRect 실패 시 hitTest에서 전달된 cursorRect 폴백
      const pos = this.position;
      if (pos.cursorRect) {
        console.warn('[CursorState] updateRect 실패 → hitTest 폴백 pos=(%d,%d,%d) cell=(%s,%s,%s,%s):',
          pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
          pos.parentParaIndex, pos.controlIndex, pos.cellIndex, pos.cellParaIndex, e);
        this.rect = { ...pos.cursorRect };
      } else {
        // 인라인 컨트롤 위치 건너뛰기는 정상 동작 (조판부호 감추기 모드)
        const msg = String(e);
        if (!msg.includes('인라인 컨트롤 위치')) {
          console.warn('[CursorState] updateRect 실패 → rect=null pos=(%d,%d,%d) cell=(%s,%s,%s,%s):',
            pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
            pos.parentParaIndex, pos.controlIndex, pos.cellIndex, pos.cellParaIndex, e);
        }
        this.rect = null;
      }
    }
  }

  // ─── F5 셀 블록 선택 모드 ─────────────────────────────────

  /** 셀 선택 모드에 진입한다. 현재 셀의 row/col이 anchor/focus가 된다. */
  enterCellSelectionMode(reason: CellSelectionReason = 'manual'): boolean {
    if (!this.isInCell() || this.isInTextBox()) return false;
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellIndex: cei, cellPath } = this.position;
    if (ppi === undefined || ci === undefined || cei === undefined) return false;

    try {
      let info, dims;
      if ((cellPath?.length ?? 0) > 0) {
        // cellPath가 있으면 1-depth 표도 경로 기반 API 사용
        const pathJson = JSON.stringify(cellPath);
        info = this.wasm.getCellInfoByPath(sec, ppi, pathJson);
        dims = this.wasm.getTableDimensionsByPath(sec, ppi, pathJson);
      } else {
        info = this.wasm.getCellInfo(sec, ppi, ci, cei);
        dims = this.wasm.getTableDimensions(sec, ppi, ci);
      }
      this.cellAnchor = { row: info.row, col: info.col };
      this.cellFocus = { row: info.row, col: info.col };
      this.cellTableCtx = { sec, ppi, ci, rowCount: dims.rowCount, colCount: dims.colCount, cellPath };
      this._cellSelectionMode = true;
      this._cellSelectionPhase = 1;
      this._cellSelectionReason = reason;
      return true;
    } catch (e) {
      console.warn('[CursorState] enterCellSelectionMode 실패:', e);
      return false;
    }
  }

  /** 셀 선택 모드를 종료한다. */
  exitCellSelectionMode(): void {
    this._cellSelectionMode = false;
    this._cellSelectionPhase = 1;
    this._cellSelectionReason = 'manual';
    this.cellAnchor = null;
    this.cellFocus = null;
    this.excludedCells.clear();
    this.cellTableCtx = null;
  }

  /** 셀 선택 단계를 반환한다. */
  getCellSelectionPhase(): number { return this._cellSelectionPhase; }

  /** F5 반복: 셀 선택 단계를 다음으로 진행한다. */
  advanceCellSelectionPhase(): void {
    if (!this._cellSelectionMode || !this.cellTableCtx) return;
    if (this._cellSelectionPhase === 1) {
      // phase 1→2: 범위 선택 모드 (anchor 고정, 방향키로 focus 확장)
      this._cellSelectionPhase = 2;
    } else if (this._cellSelectionPhase === 2) {
      // phase 2→3: 전체 셀 선택
      this._cellSelectionPhase = 3;
      this.selectAllCells();
    }
  }

  /** 전체 셀 선택 (phase 3) */
  private selectAllCells(): void {
    if (!this.cellTableCtx) return;
    this.cellAnchor = { row: 0, col: 0 };
    this.cellFocus = { row: this.cellTableCtx.rowCount - 1, col: this.cellTableCtx.colCount - 1 };
    this.excludedCells.clear();
  }

  /** 범위 선택: anchor 고정, focus만 이동 (phase 2) */
  expandCellSelection(deltaRow: number, deltaCol: number): void {
    if (!this._cellSelectionMode || !this.cellFocus || !this.cellTableCtx) return;
    const { rowCount, colCount } = this.cellTableCtx;
    this.cellFocus = {
      row: Math.max(0, Math.min(rowCount - 1, this.cellFocus.row + deltaRow)),
      col: Math.max(0, Math.min(colCount - 1, this.cellFocus.col + deltaCol)),
    };
  }

  /** 셀 선택 모드인가? */
  isInCellSelectionMode(): boolean {
    return this._cellSelectionMode;
  }

  /** 보호 셀 클릭으로 진입한 셀 선택 모드인가? */
  isProtectedCellSelectionMode(): boolean {
    return this._cellSelectionMode && this._cellSelectionReason === 'protected';
  }

  // ─── F5 본문 블록 선택 (#220) ──────────────────────
  isInBlockSelectionMode(): boolean { return this._blockSelectionMode; }

  enterBlockSelectionMode(): void {
    this._blockSelectionMode = true;
    this._expandPhase = 0;
    this.anchor = { ...this.position };
  }

  exitBlockSelectionMode(): void {
    this._blockSelectionMode = false;
    this._expandPhase = 0;
    this.clearSelection();
  }

  expandSelection(): void {
    this._expandPhase++;
    const pos = this.position;
    const sec = pos.sectionIndex;
    const para = pos.paragraphIndex;
    try {
      if (this._expandPhase === 1) {
        // 단어 선택 — 현재 위치의 단어 범위
        const paraLen = this.wasm.getParagraphLength(sec, para);
        const text = this.wasm.getTextRange(sec, para, 0, paraLen);
        const { start, end } = findWordAt(text, pos.charOffset);
        this.anchor = { ...pos, charOffset: start };
        this.position = { ...pos, charOffset: end };
      } else if (this._expandPhase === 2) {
        // 문장 선택 (#839, 한컴 F3 5단계 정합)
        const paraLen = this.wasm.getParagraphLength(sec, para);
        const text = this.wasm.getTextRange(sec, para, 0, paraLen);
        const { start, end } = findSentenceAt(text, pos.charOffset);
        this.anchor = { ...pos, charOffset: start };
        this.position = { ...pos, charOffset: end };
      } else if (this._expandPhase === 3) {
        // 문단 전체 선택
        const paraLen = this.wasm.getParagraphLength(sec, para);
        this.anchor = { ...pos, charOffset: 0 };
        this.position = { ...pos, charOffset: paraLen };
      } else if (this._expandPhase === 4) {
        // 구역 전체 선택
        const paraCount = this.wasm.getParagraphCount(sec);
        const lastParaLen = this.wasm.getParagraphLength(sec, paraCount - 1);
        this.anchor = { sectionIndex: sec, paragraphIndex: 0, charOffset: 0 };
        this.position = { sectionIndex: sec, paragraphIndex: paraCount - 1, charOffset: lastParaLen };
      } else {
        // 문서 전체 선택
        const secCount = this.wasm.getSectionCount();
        const lastSec = secCount - 1;
        const lastParaCount = this.wasm.getParagraphCount(lastSec);
        const lastParaLen = this.wasm.getParagraphLength(lastSec, lastParaCount - 1);
        this.anchor = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
        this.position = { sectionIndex: lastSec, paragraphIndex: lastParaCount - 1, charOffset: lastParaLen };
        this._expandPhase = 5; // cap
      }
      this.updateRect();
    } catch (e) {
      console.warn('[CursorState] expandSelection 실패:', e);
    }
  }

  /** 셀 선택을 화살표 방향으로 이동한다 (anchor/focus 함께 이동, 단일 셀 선택). */
  moveCellSelection(deltaRow: number, deltaCol: number): void {
    if (!this._cellSelectionMode || !this.cellFocus || !this.cellTableCtx) return;
    const { rowCount, colCount } = this.cellTableCtx;
    const { sec, ppi, ci, cellPath } = this.cellTableCtx;

    // 현재 셀의 병합 정보 조회
    let bboxes: CellBbox[];
    try {
      bboxes = cellPath
        ? this.wasm.getTableCellBboxesByPath(sec, ppi, JSON.stringify(cellPath))
        : this.wasm.getTableCellBboxes(sec, ppi, ci!);
    } catch { bboxes = []; }

    const curCell = bboxes.find(b =>
      this.cellFocus!.row >= b.row && this.cellFocus!.row < b.row + b.rowSpan &&
      this.cellFocus!.col >= b.col && this.cellFocus!.col < b.col + b.colSpan
    );

    let newRow = this.cellFocus.row;
    let newCol = this.cellFocus.col;

    if (curCell) {
      if (deltaCol > 0) {
        // 오른쪽: 현재 셀의 오른쪽 끝 다음 열로 이동
        newCol = curCell.col + curCell.colSpan;
        if (newCol >= colCount) {
          // 오른쪽에 셀 없음 → 다음 행 첫 열
          newRow = curCell.row + curCell.rowSpan;
          newCol = 0;
        }
      } else if (deltaCol < 0) {
        // 왼쪽: 현재 셀 왼쪽 열로 이동
        newCol = curCell.col - 1;
        if (newCol < 0) {
          // 왼쪽에 셀 없음 → 이전 행 마지막 열
          newRow = curCell.row - 1;
          newCol = colCount - 1;
        }
      } else if (deltaRow > 0) {
        // 아래: 현재 셀 하단 다음 행으로 이동
        newRow = curCell.row + curCell.rowSpan;
      } else if (deltaRow < 0) {
        // 위: 현재 셀 위 행으로 이동
        newRow = curCell.row - 1;
      }
    } else {
      newRow += deltaRow;
      newCol += deltaCol;
    }

    // 범위 체크: 표 경계를 벗어나면 이동하지 않음
    if (newRow < 0 || newRow >= rowCount || newCol < 0 || newCol >= colCount) {
      return; // 표 끝 → 멈춤
    }

    this.cellAnchor = { row: newRow, col: newCol };
    this.cellFocus = { row: newRow, col: newCol };
    this.excludedCells.clear();
  }

  /** Shift+클릭: anchor 고정, focus를 클릭 셀로 이동 (범위 선택). */
  shiftSelectCell(row: number, col: number): void {
    this.setCellSelectionFocus(row, col);
  }

  /** 마우스 드래그 시작 셀을 anchor/focus로 지정한다. */
  setCellSelectionAnchor(row: number, col: number): void {
    if (!this._cellSelectionMode || !this.cellTableCtx) return;
    const clamped = this.clampCellSelectionPoint(row, col);
    this.cellAnchor = clamped;
    this.cellFocus = clamped;
    this.excludedCells.clear();
    this._cellSelectionPhase = 1;
  }

  /** 셀 선택 anchor를 유지하고 focus만 갱신한다. */
  setCellSelectionFocus(row: number, col: number): void {
    if (!this._cellSelectionMode || !this.cellTableCtx) return;
    this.cellFocus = this.clampCellSelectionPoint(row, col);
    this.excludedCells.clear();
  }

  private clampCellSelectionPoint(row: number, col: number): { row: number; col: number } {
    if (!this.cellTableCtx) return { row, col };
    const { rowCount, colCount } = this.cellTableCtx;
    return {
      row: Math.max(0, Math.min(rowCount - 1, row)),
      col: Math.max(0, Math.min(colCount - 1, col)),
    };
  }

  /** Ctrl+클릭: 해당 셀을 선택에서 제외/복원 토글. */
  ctrlToggleCell(row: number, col: number): void {
    if (!this._cellSelectionMode) return;
    const key = `${row},${col}`;
    if (this.excludedCells.has(key)) {
      this.excludedCells.delete(key);
    } else {
      this.excludedCells.add(key);
    }
  }

  /** 선택된 셀 범위를 반환한다 (정렬된 start/end). */
  getSelectedCellRange(): { startRow: number; startCol: number; endRow: number; endCol: number } | null {
    if (!this._cellSelectionMode || !this.cellAnchor || !this.cellFocus) return null;
    return {
      startRow: Math.min(this.cellAnchor.row, this.cellFocus.row),
      startCol: Math.min(this.cellAnchor.col, this.cellFocus.col),
      endRow: Math.max(this.cellAnchor.row, this.cellFocus.row),
      endCol: Math.max(this.cellAnchor.col, this.cellFocus.col),
    };
  }

  /** 제외된 셀 목록을 반환한다. */
  getExcludedCells(): Set<string> {
    return this.excludedCells;
  }

  /** 현재 셀 선택의 표 컨텍스트를 반환한다 (셀 bbox 조회용). */
  getCellTableContext(): { sec: number; ppi: number; ci: number; cellPath?: CellPathEntry[] } | null {
    if (this.cellTableCtx) return this.cellTableCtx;
    if (!this.isInCell()) return null;

    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellPath } = this.position;
    if (ppi === undefined || ci === undefined) return null;
    return { sec, ppi, ci, cellPath };
  }

  // ─── 표 객체 선택 모드 ─────────────────────────────────

  /** 현재 셀 위치의 표를 객체 선택한다. 셀 내부가 아니면 false.
   *
   *  [Task #919] 글상자 안 표 셀 (isInTextBox + cellPath.length >= 2) 도
   *  허용 — 가장 안쪽 표를 객체 선택한다. 한컴 UX 정합 (글상자 안 표 Esc).
   *  글상자 직접 셀 (cellPath.length === 1, 글상자 자체) 은 표가 아니므로 제외.
   */
  enterTableObjectSelection(): boolean {
    if (!this.isInCell()) return false;
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellPath } = this.position;
    if (ppi === undefined || ci === undefined) return false;
    // 글상자 안 본문 (cellPath.length === 1, 글상자 자체) → 표 객체 선택 대상 아님
    if (this.isInTextBox() && (cellPath?.length ?? 0) < 2) return false;
    this._tableObjectSelected = true;
    if (cellPath && cellPath.length > 1) {
      // 중첩 표: 내부 표를 선택 (cellPath 포함)
      this.selectedTableRef = { sec, ppi, ci, cellPath };
    } else {
      this.selectedTableRef = { sec, ppi, ci };
    }
    return true;
  }

  /** 지정한 표를 객체 선택한다 (커서 위치와 무관). */
  enterTableObjectSelectionDirect(sec: number, ppi: number, ci: number): void {
    this._tableObjectSelected = true;
    this.selectedTableRef = { sec, ppi, ci };
  }

  /** 표 객체 선택을 해제한다. */
  exitTableObjectSelection(): void {
    this._tableObjectSelected = false;
    this.selectedTableRef = null;
  }

  /** 표 객체 선택 모드인가? */
  isInTableObjectSelection(): boolean {
    return this._tableObjectSelected;
  }

  /** 선택된 표의 참조 정보를 반환한다. */
  getSelectedTableRef(): { sec: number; ppi: number; ci: number; cellPath?: CellPathEntry[] } | null {
    return this.selectedTableRef;
  }

  /** 표 이동 후 selectedTableRef의 ppi/ci를 갱신한다. */
  updateSelectedTableRef(sec: number, ppi: number, ci: number): void {
    if (!this.selectedTableRef) return;
    this.selectedTableRef = { ...this.selectedTableRef, sec, ppi, ci };
  }

  /** 표 객체 선택 상태에서 표 밖으로 커서를 이동한다. */
  moveOutOfSelectedTable(): void {
    if (!this.selectedTableRef) return;
    const { sec, ppi, cellPath } = this.selectedTableRef;

    if (cellPath && cellPath.length > 1) {
      // 중첩 표 객체 선택 → 외부 셀로 이동 (한 단계 위)
      // [Task #919] 글상자 안 표였으면 (가장 바깥이 글상자 = isTextBox) 유지.
      const wasInTextBox = this.position.isTextBox === true;
      const outerPath = cellPath.slice(0, -1);
      const lastOuter = outerPath[outerPath.length - 1];
      // outerPath.length === 1 이고 글상자였으면 isTextBox 유지 → 다음 Esc 시
      // 글상자 객체 선택으로 전이 가능
      const stillInTextBox = wasInTextBox && outerPath.length === 1;
      this.position = {
        sectionIndex: sec,
        paragraphIndex: lastOuter.cellParaIndex,
        charOffset: 0,
        parentParaIndex: ppi,
        controlIndex: outerPath[0].controlIndex,
        cellIndex: lastOuter.cellIndex,
        cellParaIndex: lastOuter.cellParaIndex,
        cellPath: outerPath,
        isTextBox: stillInTextBox ? true : undefined,
      };
    } else {
      // 단일 표 객체 선택 → 표 밖으로 이동
      const paraCount = this.wasm.getParagraphCount(sec);
      if (ppi + 1 < paraCount) {
        this.position = { sectionIndex: sec, paragraphIndex: ppi + 1, charOffset: 0 };
      } else if (ppi > 0) {
        const prevLen = this.wasm.getParagraphLength(sec, ppi - 1);
        this.position = { sectionIndex: sec, paragraphIndex: ppi - 1, charOffset: prevLen };
      }
    }
    this.exitTableObjectSelection();
    this.updateRect();
  }

  // ── 그림/글상자 객체 선택 모드 ─────────────────────────────────
  private _pictureObjectSelected = false;
  private selectedPictureRef: PictureSelectionRef | null = null;
  /** 다중 선택된 개체 목록 */
  private selectedPictureRefs: PictureSelectionRef[] = [];

  /** 지정한 개체(그림/글상자/묶음)를 객체 선택한다.
   * [Task #825] `headerFooter` — 머리말/꼬리말 안 그림일 때 outer 위치 marker 보존. */
  enterPictureObjectSelectionDirect(
    sec: number, ppi: number, ci: number,
    type: 'image' | 'shape' | 'equation' | 'group' | 'line' = 'image',
    cellIdx?: number, cellParaIdx?: number,
    headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number },
    outerTableControlIdx?: number,
    cellPath?: CellPathEntry[],
    noteRef?: any,
  ): void {
    this.exitTableObjectSelection();
    this._pictureObjectSelected = true;
    this.selectedPictureRef = { sec, ppi, ci, type, cellIdx, cellParaIdx, outerTableControlIdx, cellPath, noteRef, headerFooter };
    this.selectedPictureRefs = [{ ...this.selectedPictureRef }];
  }

  /** Shift+클릭: 개체를 다중 선택에 추가/제거 (토글) */
  togglePictureObjectSelection(ref: PictureSelectionRef): void;
  togglePictureObjectSelection(sec: number, ppi: number, ci: number, type: 'image' | 'shape' | 'equation' | 'group' | 'line'): void;
  togglePictureObjectSelection(
    refOrSec: PictureSelectionRef | number,
    ppi?: number,
    ci?: number,
    type?: 'image' | 'shape' | 'equation' | 'group' | 'line',
  ): void {
    this.exitTableObjectSelection();
    this._pictureObjectSelected = true;
    const ref: PictureSelectionRef =
      typeof refOrSec === 'number'
        ? { sec: refOrSec, ppi: ppi!, ci: ci!, type: type! }
        : refOrSec;
    const idx = this.selectedPictureRefs.findIndex(r =>
      r.sec === ref.sec &&
      r.ppi === ref.ppi &&
      r.ci === ref.ci &&
      JSON.stringify(r.cellPath ?? []) === JSON.stringify(ref.cellPath ?? []),
    );
    if (idx >= 0) {
      this.selectedPictureRefs.splice(idx, 1);
      if (this.selectedPictureRefs.length === 0) {
        this.exitPictureObjectSelection();
        return;
      }
    } else {
      this.selectedPictureRefs.push({ ...ref });
    }
    // 기본 ref는 마지막 선택된 개체
    const last = this.selectedPictureRefs[this.selectedPictureRefs.length - 1];
    this.selectedPictureRef = { ...last };
  }

  /** 개체 객체 선택을 해제한다. */
  exitPictureObjectSelection(): void {
    this._pictureObjectSelected = false;
    this.selectedPictureRef = null;
    this.selectedPictureRefs = [];
  }

  /** 개체 객체 선택 모드인가? */
  isInPictureObjectSelection(): boolean {
    return this._pictureObjectSelected;
  }

  /** 선택된 개체의 참조 정보를 반환한다. */
  getSelectedPictureRef(): PictureSelectionRef | null {
    return this.selectedPictureRef;
  }

  /** 다중 선택된 개체 목록 반환 */
  getSelectedPictureRefs(): PictureSelectionRef[] {
    return this.selectedPictureRefs;
  }

  /** 다중 선택 상태인가? */
  isMultiPictureSelection(): boolean {
    return this.selectedPictureRefs.length > 1;
  }

  /** 개체 객체 선택 상태에서 개체 밖으로 커서를 이동한다. */
  moveOutOfSelectedPicture(): void {
    if (!this.selectedPictureRef) return;
    const { sec, ppi } = this.selectedPictureRef;
    const paraCount = this.wasm.getParagraphCount(sec);
    if (ppi + 1 < paraCount) {
      this.position = { sectionIndex: sec, paragraphIndex: ppi + 1, charOffset: 0 };
    } else if (ppi > 0) {
      const prevLen = this.wasm.getParagraphLength(sec, ppi - 1);
      this.position = { sectionIndex: sec, paragraphIndex: ppi - 1, charOffset: prevLen };
    }
    this.exitPictureObjectSelection();
    this.updateRect();
  }

  // ─── 머리말/꼬리말 편집 모드 API ────────────────────────────

  /** 머리말/꼬리말 편집 모드인지 반환 */
  get headerFooterMode(): 'none' | 'header' | 'footer' { return this._headerFooterMode; }
  /** 머리말/꼬리말 편집 중인 구역 인덱스 */
  get hfSectionIdx(): number { return this._hfSectionIdx; }
  /** 머리말/꼬리말 applyTo (0=Both, 1=Even, 2=Odd) */
  get hfApplyTo(): number { return this._hfApplyTo; }
  /** 머리말/꼬리말 내 문단 인덱스 */
  get hfParaIdx(): number { return this._hfParaIdx; }
  /** 머리말/꼬리말 내 문자 오프셋 */
  get hfCharOffset(): number { return this._hfCharOffset; }

  /** 머리말/꼬리말 편집 모드인지 반환 */
  isInHeaderFooter(): boolean { return this._headerFooterMode !== 'none'; }

  /** 머리말/꼬리말 편집 모드에 진입한다. */
  enterHeaderFooterMode(isHeader: boolean, sectionIdx: number, applyTo: number, preferredPage = -1): void {
    // 현재 본문 커서 위치 저장
    this._savedBodyPosition = { ...this.position };

    this._headerFooterMode = isHeader ? 'header' : 'footer';
    this._hfSectionIdx = sectionIdx;
    this._hfApplyTo = applyTo;
    this._hfParaIdx = 0;
    this._hfCharOffset = 0;
    this._hfPreferredPage = preferredPage;

    // 선택 해제
    this.clearSelection();

    // 커서 좌표 갱신
    this.updateRect();
  }

  /** 머리말/꼬리말 편집 모드에서 탈출한다. */
  exitHeaderFooterMode(): void {
    if (this._headerFooterMode === 'none') return;

    this._headerFooterMode = 'none';

    // 본문 커서 위치 복원
    if (this._savedBodyPosition) {
      // 머리말/꼬리말 마커 para_index(usize::MAX 계열)가 저장된 경우 → 문서 시작으로 초기화
      if (this._savedBodyPosition.paragraphIndex >= 0xFFFFFF00) {
        this._savedBodyPosition.paragraphIndex = 0;
        this._savedBodyPosition.charOffset = 0;
      }
      this.position = { ...this._savedBodyPosition };
      this._savedBodyPosition = null;
    }

    this.clearSelection();
    this.updateRect();
  }

  /** 다른 머리말/꼬리말로 직접 전환한다 (exit→enter 사이의 updateRect 호출을 피함). */
  switchHeaderFooterTarget(isHeader: boolean, sectionIdx: number, applyTo: number, targetPage = -1): void {
    if (this._headerFooterMode === 'none') return;
    this._headerFooterMode = isHeader ? 'header' : 'footer';
    this._hfSectionIdx = sectionIdx;
    this._hfApplyTo = applyTo;
    this._hfParaIdx = 0;
    this._hfCharOffset = 0;
    this._hfPreferredPage = targetPage >= 0 ? targetPage : (this.rect?.pageIndex ?? this._hfPreferredPage);
    this.clearSelection();
    this.updateRect();
  }

  /** 머리말/꼬리말 내 커서 위치를 설정한다. */
  setHfCursorPosition(paraIdx: number, charOffset: number): void {
    this._hfParaIdx = paraIdx;
    this._hfCharOffset = charOffset;
    this.updateRect();
  }

  /** 머리말/꼬리말 내 수평 이동 */
  moveHorizontalInHf(delta: number): void {
    if (this._headerFooterMode === 'none') return;
    const isHeader = this._headerFooterMode === 'header';

    try {
      const info = JSON.parse(this.wasm.getHeaderFooterParaInfo(
        this._hfSectionIdx, isHeader, this._hfApplyTo, this._hfParaIdx
      ));
      const paraCount = info.paraCount as number;
      const charCount = info.charCount as number;

      const newOffset = this._hfCharOffset + delta;

      if (newOffset >= 0 && newOffset <= charCount) {
        // 같은 문단 내 이동
        this._hfCharOffset = newOffset;
      } else if (delta > 0 && this._hfParaIdx + 1 < paraCount) {
        // 다음 문단 시작으로
        this._hfParaIdx++;
        this._hfCharOffset = 0;
      } else if (delta < 0 && this._hfParaIdx > 0) {
        // 이전 문단 끝으로
        this._hfParaIdx--;
        const prevInfo = JSON.parse(this.wasm.getHeaderFooterParaInfo(
          this._hfSectionIdx, isHeader, this._hfApplyTo, this._hfParaIdx
        ));
        this._hfCharOffset = prevInfo.charCount as number;
      }
      // else: 문서 경계 — 이동 불가
    } catch {
      // WASM 호출 실패 시 무시
    }

    this.updateRect();
  }

  // ─── 각주 편집 모드 ──────────────────────────────────────

  isInFootnote(): boolean { return this._footnoteMode; }

  get fnSectionIdx(): number { return this._fnSectionIdx; }
  get fnParaIdx(): number { return this._fnParaIdx; }
  get fnControlIdx(): number { return this._fnControlIdx; }
  get fnInnerParaIdx(): number { return this._fnInnerParaIdx; }
  get fnCharOffset(): number { return this._fnCharOffset; }
  get fnFootnoteIndex(): number { return this._fnFootnoteIndex; }
  get fnPageNum(): number { return this._fnPageNum; }

  /** 각주 편집 모드에 진입한다.
   *
   * [Task #1058 reopen Round 5] 신규/기존 각주 inner_para 의 한컴 contract 는
   * 두 placeholder space + AutoNumber 8 cu 차지 (text="  ", char_offsets=[0, 8]).
   * caret 초기 위치를 char_offset=2 로 설정하여 사용자 입력이 placeholder 뒤
   * (실제 본문 작성 영역) 부터 시작하도록 한다. char_offset=0/1 위치는 placeholder
   * 자리이므로 사용자 입력 시 AutoNumber jump 8 byte contract 깨짐 (한컴 거부).
   */
  enterFootnoteMode(
    sectionIdx: number, paraIdx: number, controlIdx: number,
    footnoteIndex: number, pageNum: number,
  ): void {
    this._savedBodyPosition = { ...this.position };
    this._footnoteMode = true;
    this._fnSectionIdx = sectionIdx;
    this._fnParaIdx = paraIdx;
    this._fnControlIdx = controlIdx;
    this._fnFootnoteIndex = footnoteIndex;
    this._fnInnerParaIdx = 0;
    this._fnCharOffset = 2;
    this._fnPageNum = pageNum;
    this.clearSelection();
    this.updateRect();
  }

  /** 각주 편집 모드에서 탈출한다. */
  exitFootnoteMode(): void {
    if (!this._footnoteMode) return;
    this._footnoteMode = false;
    if (this._savedBodyPosition) {
      if (this._savedBodyPosition.paragraphIndex >= 0xFFFFFF00) {
        this._savedBodyPosition.paragraphIndex = 0;
        this._savedBodyPosition.charOffset = 0;
      }
      this.position = { ...this._savedBodyPosition };
      this._savedBodyPosition = null;
    }
    this.clearSelection();
    this.updateRect();
  }

  /** 각주 내 커서 위치를 설정한다. */
  setFnCursorPosition(fnParaIdx: number, charOffset: number): void {
    this._fnInnerParaIdx = fnParaIdx;
    this._fnCharOffset = charOffset;
    this.updateRect();
  }

  /** 각주 내 수평 이동 */
  moveHorizontalInFn(delta: number): void {
    if (!this._footnoteMode) return;

    try {
      const info = this.wasm.getFootnoteInfo(this._fnSectionIdx, this._fnParaIdx, this._fnControlIdx);
      const paraCount = info.paraCount;
      // 현재 문단의 텍스트 길이
      const currentText = info.texts[this._fnInnerParaIdx] ?? '';
      const charCount = currentText.length;

      const newOffset = this._fnCharOffset + delta;

      if (newOffset >= 0 && newOffset <= charCount) {
        this._fnCharOffset = newOffset;
      } else if (delta > 0 && this._fnInnerParaIdx + 1 < paraCount) {
        this._fnInnerParaIdx++;
        this._fnCharOffset = 0;
      } else if (delta < 0 && this._fnInnerParaIdx > 0) {
        this._fnInnerParaIdx--;
        const prevText = info.texts[this._fnInnerParaIdx] ?? '';
        this._fnCharOffset = prevText.length;
      }
    } catch {
      // WASM 호출 실패 시 무시
    }

    this.updateRect();
  }
}

// ─── 단어 경계 탐색 유틸 (PR #794, Alt+Arrow 단어 이동) ──────────────────────────────────

const enum CharClass { Space, Hangul, Latin, Digit, Punct }

function classifyChar(ch: string): CharClass {
  const c = ch.charCodeAt(0);
  if (c === 0x20 || c === 0x09 || c === 0x0A || c === 0x0D || c === 0xA0) return CharClass.Space;
  if (c >= 0xAC00 && c <= 0xD7AF) return CharClass.Hangul; // 완성형
  if (c >= 0x3131 && c <= 0x318E) return CharClass.Hangul; // 자모
  if (c >= 0x1100 && c <= 0x11FF) return CharClass.Hangul; // 첫가끝
  if ((c >= 0x41 && c <= 0x5A) || (c >= 0x61 && c <= 0x7A)) return CharClass.Latin;
  if (c >= 0x30 && c <= 0x39) return CharClass.Digit;
  return CharClass.Punct;
}

function findWordBoundaryForward(text: string): number {
  if (text.length === 0) return 0;
  const startClass = classifyChar(text[0]);
  let i = 0;
  // Skip current word (same class)
  if (startClass === CharClass.Space) {
    while (i < text.length && classifyChar(text[i]) === CharClass.Space) i++;
  } else {
    while (i < text.length && classifyChar(text[i]) === startClass) i++;
    // Also skip trailing spaces
    while (i < text.length && classifyChar(text[i]) === CharClass.Space) i++;
  }
  return i || 1;
}

function findWordBoundaryBackward(text: string): number {
  if (text.length === 0) return 0;
  let i = text.length;
  const endClass = classifyChar(text[i - 1]);
  // Skip trailing spaces
  if (endClass === CharClass.Space) {
    while (i > 0 && classifyChar(text[i - 1]) === CharClass.Space) i--;
  }
  if (i === 0) return 0;
  // Skip the word (same class)
  const wordClass = classifyChar(text[i - 1]);
  while (i > 0 && classifyChar(text[i - 1]) === wordClass) i--;
  return i;
}

// ─── 단어 범위 탐색 유틸 (PR #811, F3 단계 1 단어 선택) ──────────────────────────────────

function isWordChar(c: string): boolean {
  const code = c.charCodeAt(0);
  if (code >= 0x30 && code <= 0x39) return true; // digit
  if (code >= 0x41 && code <= 0x5A) return true; // A-Z
  if (code >= 0x61 && code <= 0x7A) return true; // a-z
  if (code >= 0xAC00 && code <= 0xD7AF) return true; // Hangul
  if (code >= 0x3131 && code <= 0x318E) return true; // Hangul Jamo
  return false;
}

function findWordAt(text: string, offset: number): { start: number; end: number } {
  if (!text || offset >= text.length) return { start: offset, end: offset };
  const atWord = isWordChar(text[offset] ?? '');
  let start = offset;
  let end = offset;
  if (atWord) {
    while (start > 0 && isWordChar(text[start - 1])) start--;
    while (end < text.length && isWordChar(text[end])) end++;
  } else {
    while (start > 0 && !isWordChar(text[start - 1])) start--;
    while (end < text.length && !isWordChar(text[end])) end++;
  }
  return { start, end };
}

// ─── 문장 범위 탐색 유틸 (#839, F3 단계 2 문장 선택) ──────────────────────────────────

const SENTENCE_TERMINATORS = new Set(['.', '?', '!', '。', '？', '！']);

function findSentenceAt(text: string, offset: number): { start: number; end: number } {
  if (!text) return { start: offset, end: offset };
  const len = text.length;
  const clampedOffset = Math.min(offset, len);

  let start = clampedOffset;
  while (start > 0) {
    const prev = text[start - 1];
    if (SENTENCE_TERMINATORS.has(prev)) break;
    start--;
  }
  while (start < clampedOffset && (text[start] === ' ' || text[start] === '\t')) start++;

  let end = clampedOffset;
  while (end < len) {
    if (SENTENCE_TERMINATORS.has(text[end])) { end++; break; }
    end++;
  }

  return { start, end };
}
