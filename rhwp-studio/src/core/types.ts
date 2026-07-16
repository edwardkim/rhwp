/** WASM getDocumentInfo() 반환 타입 */
export interface DocumentInfo {
  version: string;
  sectionCount: number;
  pageCount: number;
  encrypted: boolean;
  hwp3Variant?: boolean;
  fallbackFont: string;
  fontsUsed: string[];  // 문서에서 사용하는 폰트 이름 목록
}

/** WASM getPageInfo() 반환 타입 */
export interface PageInfo {
  pageIndex: number;
  /** 조판 기준으로 계산된 표시용 쪽 번호(구역 설정 반영) */
  pageNumber?: number;
  width: number;
  height: number;
  sectionIndex: number;
  /** 왼쪽 여백 (px) */
  marginLeft: number;
  /** 오른쪽 여백 (px) */
  marginRight: number;
  /** 위 여백 (px) */
  marginTop: number;
  /** 아래 여백 (px) */
  marginBottom: number;
  /** 머리말 여백 (px) */
  marginHeader: number;
  /** 꼬리말 여백 (px) */
  marginFooter: number;
  /** 쪽 테두리/쪽 영역 왼쪽 위치 (px) */
  pageBorderLeft?: number;
  /** 쪽 테두리/쪽 영역 오른쪽 여백 (px) */
  pageBorderRight?: number;
  /** 쪽 테두리/쪽 영역 위쪽 위치 (px) */
  pageBorderTop?: number;
  /** 쪽 테두리/쪽 영역 아래쪽 여백 (px) */
  pageBorderBottom?: number;
  /** 단별 영역 (px, 페이지 좌표) */
  columns?: { x: number; width: number }[];
}

/** WASM getPageDef() 반환 타입 — HWPUNIT 원본값 */
export interface PageDef {
  width: number;
  height: number;
  marginLeft: number;
  marginRight: number;
  marginTop: number;
  marginBottom: number;
  marginHeader: number;
  marginFooter: number;
  marginGutter: number;
  landscape: boolean;
  /** 0=한쪽, 1=맞쪽, 2=위로 */
  binding: number;
}

export interface BorderLineProps {
  type: number;
  width: number;
  color: string;
}

/** WASM getPageBorderFill() 반환 타입 */
export interface PageBorderFillSettings {
  attr: number;
  basis: 'paper' | 'page';
  spacingLeft: number;
  spacingRight: number;
  spacingTop: number;
  spacingBottom: number;
  borderFillId: number;
  headerInside: boolean;
  footerInside: boolean;
  fillArea: 'paper' | 'page' | 'border';
  hideBorder: boolean;
  hideFill: boolean;
  borderLeft: BorderLineProps;
  borderRight: BorderLineProps;
  borderTop: BorderLineProps;
  borderBottom: BorderLineProps;
  fillType: 'none' | 'solid' | string;
  fillColor: string;
  patternColor: string;
  patternType: number;
  applyPage?: 'all' | 'exceptFirst' | 'firstOnly';
}

export type EndnoteNumberFormat =
  | 'digit'
  | 'circledDigit'
  | 'upperRoman'
  | 'lowerRoman'
  | 'upperAlpha'
  | 'lowerAlpha'
  | 'hangulSyllable'
  | 'hangulJamo'
  | 'hangulDigit'
  | 'hanjaDigit';

/** WASM getEndnoteShape() 반환 타입 — HWPUNIT 원본값 */
export interface EndnoteShapeSettings {
  ok?: boolean;
  numberFormat: EndnoteNumberFormat | string;
  userChar: string;
  prefixChar: string;
  suffixChar: string;
  startNumber: number;
  separatorEnabled: boolean;
  separatorLength: number;
  separatorMarginTop: number;
  separatorMarginBottom: number;
  noteSpacing: number;
  separatorLineType: number;
  separatorLineWidth: number;
  separatorColor: string;
  numbering: 'continue' | 'restartSection' | 'restartPage' | string;
  placement: 'documentEnd' | 'sectionEnd' | string;
}

export interface NoteEditInfo {
  ok: boolean;
  kind: 'footnote' | 'endnote' | string;
  pageNum: number;
  footnoteIndex: number;
  fnParaIndex: number;
  charOffset: number;
  virtualParaIndex?: number;
}

/** 구역 정의 (SectionDef) */
export interface SectionDef {
  pageNum: number;
  /** 쪽 번호 종류: 0=이어서, 1=홀수, 2=짝수 (사용자 지정은 pageNum > 0) */
  pageNumType: number;
  pictureNum: number;
  tableNum: number;
  equationNum: number;
  columnSpacing: number;
  defaultTabSpacing: number;
  hideHeader: boolean;
  hideFooter: boolean;
  hideMasterPage: boolean;
  hideBorder: boolean;
  hideFill: boolean;
  hideEmptyLine: boolean;
}

/** 중첩 표 경로 엔트리 (1레벨 = 단일 표, 2레벨 이상 = 중첩 표) */
export interface CellPathEntry {
  controlIndex: number;
  cellIndex: number;
  cellParaIndex: number;
}

/**
 * [Task #1138] 표 셀 by_path WASM API 의 path segment.
 * Rust API 의 JSON key (`controlIdx`/`cellIdx`/`cellParaIdx`) 와 일치하는 짧은 형식.
 * `CellPathEntry` 와 의미는 같으나 직렬화 시 key 형식이 다름.
 */
export interface CellPathSegment {
  controlIdx: number;
  cellIdx: number;
  cellParaIdx: number;
}
export type CellPath = CellPathSegment[];
export type CellPathLike = Array<CellPathEntry | CellPathSegment>;

/** 문서 트리 DFS 순회 컨텍스트 엔트리 */
export interface NavContextEntry {
  parentPara: number;
  ctrlIdx: number;
  ctrlTextPos: number;
  cellIdx: number;
  isTextBox: boolean;
}

/** WASM getCursorRect() 반환 타입 */
export interface CursorRect {
  pageIndex: number;
  x: number;
  y: number;
  height: number;
  /** 표 셀 내부 커서일 때만 제공되는 가시 셀 bbox */
  cellBounds?: { x: number; y: number; w: number; h: number };
  /** 원래 TextRun 좌표가 셀 bbox를 벗어나 보정됐는지 여부 */
  cellOverflowed?: boolean;
}

/** WASM hitTest() 반환 타입 */
export interface HitTestResult {
  sectionIndex: number;
  paragraphIndex: number;
  charOffset: number;
  /** 셀/글상자 컨텍스트 (셀 또는 글상자 내부 클릭 시에만 존재) */
  parentParaIndex?: number;
  controlIndex?: number;
  cellIndex?: number;
  cellParaIndex?: number;
  /** 중첩 표 전체 경로 (depth 1=단일 표, depth 2+=중첩 표) */
  cellPath?: CellPathEntry[];
  /** 글상자 내부 여부 */
  isTextBox?: boolean;
  /** 필드 내부 여부 (ClickHere 등) */
  isField?: boolean;
  /** 필드 ID (isField=true일 때) */
  fieldId?: number;
  /** 필드 타입 ("clickhere" 등) */
  fieldType?: string;
}

/** WASM hitTestBodyFootnoteMarker() 반환 타입 */
export interface BodyFootnoteMarkerHit {
  hit: boolean;
  sectionIndex?: number;
  paragraphIndex?: number;
  controlIndex?: number;
  footnoteNumber?: number;
  footnoteIndex?: number;
  bbox?: { x: number; y: number; w: number; h: number };
  cursorRect?: CursorRect;
}

/** WASM getFootnoteAtCursor() 반환 타입 */
export interface FootnoteAtCursorResult {
  hit: boolean;
  sectionIndex?: number;
  paragraphIndex?: number;
  controlIndex?: number;
  charOffset?: number;
  footnoteNumber?: number;
}

/** WASM deleteFootnote() 반환 타입 */
export interface DeleteFootnoteResult {
  ok: boolean;
  sectionIndex: number;
  paragraphIndex: number;
  controlIndex: number;
  charOffset: number;
  deletedNumber: number;
}

/** 커서 위치의 필드 범위 정보 */
export interface FieldInfoResult {
  inField: boolean;
  fieldId?: number;
  fieldType?: string;
  startCharIdx?: number;
  endCharIdx?: number;
  isGuide?: boolean;
  guideName?: string;
  editableInForm?: boolean;
}

/** WASM getLineInfo() 반환 타입 */
export interface LineInfo {
  lineIndex: number;
  lineCount: number;
  charStart: number;
  charEnd: number;
}

/** WASM getTableDimensions() 반환 타입 */
export interface TableDimensions {
  rowCount: number;
  colCount: number;
  cellCount: number;
}

/** WASM getCellInfo() 반환 타입 */
export interface CellInfo {
  row: number;
  col: number;
  rowSpan: number;
  colSpan: number;
}

/** WASM getTableCellBboxes() 반환 타입 */
export interface CellBbox {
  cellIdx: number;
  row: number;
  col: number;
  rowSpan: number;
  colSpan: number;
  pageIndex: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

/** WASM moveVertical() 반환 타입 */
export interface MoveVerticalResult {
  sectionIndex: number;
  paragraphIndex: number;
  charOffset: number;
  parentParaIndex?: number;
  controlIndex?: number;
  cellIndex?: number;
  cellParaIndex?: number;
  /** 중첩 표 전체 경로 */
  cellPath?: CellPathEntry[];
  /** 글상자 내부 여부 */
  isTextBox?: boolean;
  pageIndex: number;
  x: number;
  y: number;
  height: number;
  preferredX: number;
  /** 커서 좌표 조회 실패 시 false */
  rectValid?: boolean;
}

/** 선택 영역의 줄별 사각형 (렌더링용) */
export interface SelectionRect {
  pageIndex: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

/** 글자 서식 속성 (CharShape) */
export interface CharProperties {
  fontFamily?: string;
  fontSize?: number;       // HWPUNIT (1pt = 100, base_size)
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  textColor?: string;      // '#RRGGBB'
  shadeColor?: string;     // '#RRGGBB'
  emboss?: boolean;
  engrave?: boolean;
  charShapeId?: number;
  fontId?: number;
  fontIds?: number[];       // 언어별 개별 글꼴 ID (7개)
  // 확장 속성
  underlineType?: string;  // 'None' | 'Bottom' | 'Top'
  underlineColor?: string;
  outlineType?: number;    // 0-6
  shadowType?: number;     // 0=없음, 1=비연속, 2=연속
  shadowColor?: string;
  shadowOffsetX?: number;  // -100~100
  shadowOffsetY?: number;
  strikeColor?: string;
  subscript?: boolean;
  superscript?: boolean;
  // 언어별 배열 (7개: 한글/영문/한자/일어/외국어/기호/사용자)
  fontFamilies?: string[];
  ratios?: number[];       // 장평
  spacings?: number[];     // 자간
  relativeSizes?: number[];// 상대크기
  charOffsets?: number[];  // 글자 위치
  fontName?: string;       // 글꼴 변경 시 (mods 전용)
  // 강조점/밑줄모양/취소선모양/커닝
  emphasisDot?: number;    // 0=없음, 1=● 2=○ 3=ˇ 4=˜ 5=･ 6=:
  underlineShape?: number; // 0=실선, 1=긴점선, 2=점선, ...(표 27 선 종류)
  strikeShape?: number;    // 0=실선, 1=긴점선, 2=점선, ...(표 27 선 종류)
  kerning?: boolean;
  // 테두리/배경
  borderFillId?: number;
  borderLeft?: { type: number; width: number; color: string };
  borderRight?: { type: number; width: number; color: string };
  borderTop?: { type: number; width: number; color: string };
  borderBottom?: { type: number; width: number; color: string };
  fillType?: string;       // 'none' | 'solid'
  fillColor?: string;      // '#RRGGBB'
  patternColor?: string;   // '#RRGGBB'
  patternType?: number;    // 0=없음, 1=가로줄, 2=세로줄, 3=역슬래시, 4=슬래시, 5=십자, 6=X자
}

/** 문단 서식 속성 (ParaShape) — WASM getParaPropertiesAt 반환 타입 */
export interface ParaProperties {
  alignment?: string;        // 'justify'|'left'|'right'|'center'|'distribute'|'split'
  lineSpacing?: number;      // Percent일 때 %, 그 외 HWPUNIT
  lineSpacingType?: string;  // 'Percent'|'Fixed'|'SpaceOnly'|'Minimum'
  marginLeft?: number;       // px (96dpi, zoom=1 기준, ResolvedParaStyle)
  marginRight?: number;      // px (96dpi, zoom=1 기준, ResolvedParaStyle)
  indent?: number;           // px (96dpi, zoom=1 기준, ResolvedParaStyle)
  spacingBefore?: number;    // px (96dpi, zoom=1 기준)
  spacingAfter?: number;     // px (96dpi, zoom=1 기준)
  paraShapeId?: number;
  // 확장 탭 속성
  headType?: string;         // 'None'|'Outline'|'Number'|'Bullet'
  paraLevel?: number;        // 0-6 (=1-7수준)
  numberingId?: number;      // 번호/글머리표 정의 ID (1-based, 0=없음)
  widowOrphan?: boolean;
  keepWithNext?: boolean;
  keepLines?: boolean;
  pageBreakBefore?: boolean;
  fontLineHeight?: boolean;
  singleLine?: boolean;
  autoSpaceKrEn?: boolean;
  autoSpaceKrNum?: boolean;
  verticalAlign?: number;    // 0=글꼴기준, 1=위, 2=가운데, 3=아래
  englishBreakUnit?: number; // 0=단어, 1=하이픈, 2=글자
  koreanBreakUnit?: number;  // 0=어절, 1=글자
  // 탭 설정 탭 속성
  tabAutoLeft?: boolean;
  tabAutoRight?: boolean;
  tabStops?: { position: number; type: number; fill: number }[];
  defaultTabSpacing?: number;    // HWPUNIT (읽기 전용, 구역 기본 탭 간격)
  // 테두리/배경 탭 속성
  borderFillId?: number;
  borderLeft?: { type: number; width: number; color: string };
  borderRight?: { type: number; width: number; color: string };
  borderTop?: { type: number; width: number; color: string };
  borderBottom?: { type: number; width: number; color: string };
  fillType?: string;       // 'none' | 'solid'
  fillColor?: string;      // '#RRGGBB'
  patternColor?: string;   // '#RRGGBB'
  patternType?: number;    // 0=없음, 1~6=무늬
  borderSpacing?: number[];  // [좌, 우, 상, 하] HWPUNIT
  borderConnect?: boolean;   // 문단 테두리 연결
  borderIgnoreMargin?: boolean; // 문단 여백 무시
}

/** 테두리 선 정보 */
export interface BorderLineInfo {
  /** 선 종류 (0=없음, 1=실선, 2=파선, 3=점선, ...) */
  type: number;
  /** 선 굵기 (0-6) */
  width: number;
  /** 선 색상 (#rrggbb) */
  color: string;
}

/** WASM getCellProperties() 반환 타입 — HWPUNIT 원본값 */
export interface CellProperties {
  width: number;
  height: number;
  paddingLeft: number;
  paddingRight: number;
  paddingTop: number;
  paddingBottom: number;
  /** 셀 고유 안 여백 지정 */
  applyInnerMargin: boolean;
  /** 0=top, 1=center, 2=bottom */
  verticalAlign: number;
  /** 0=horizontal, 1=vertical */
  textDirection: number;
  isHeader: boolean;
  /** 셀 보호 */
  cellProtect?: boolean;
  /** 셀 필드 이름 */
  fieldName?: string;
  /** 양식 모드에서 편집 가능 */
  editableInForm?: boolean;
  /** 테두리/배경 */
  borderFillId?: number;
  borderLeft?: BorderLineInfo;
  borderRight?: BorderLineInfo;
  borderTop?: BorderLineInfo;
  borderBottom?: BorderLineInfo;
  fillType?: string;
  fillColor?: string;
  patternColor?: string;
  patternType?: number;
  /** 대각선 선 종류 (0=없음, 1=실선, 2=파선, ...) */
  diagonalLine?: number;
  /** / 대각선 방향 비트 */
  diagonalSlash?: number;
  /** \ 대각선 방향 비트 */
  diagonalBackSlash?: number;
  /** 대각선 굵기 (0-6) */
  diagonalWidth?: number;
  /** 대각선 색상 (#rrggbb) */
  diagonalColor?: string;
  /** 중심선 방향: NONE / VERTICAL / HORIZONTAL / CROSS */
  centerLine?: string;
}

/** WASM getTableProperties() 반환 타입 — HWPUNIT 원본값 */
export interface TableProperties {
  cellSpacing: number;
  paddingLeft: number;
  paddingRight: number;
  paddingTop: number;
  paddingBottom: number;
  /** 0=none(나누지 않음), 1=cellBreak(셀 단위로 나눔) */
  pageBreak: number;
  repeatHeader: boolean;
  /** 표 전체 크기 (HWPUNIT) */
  tableWidth?: number;
  tableHeight?: number;
  /** 바깥 여백 (HWP16) */
  outerLeft?: number;
  outerRight?: number;
  outerTop?: number;
  outerBottom?: number;
  /** 캡션 */
  hasCaption?: boolean;
  captionDirection?: number;  // 0=왼쪽, 1=오른쪽, 2=위쪽, 3=아래쪽
  captionVertAlign?: number;  // 0=위, 1=가운데, 2=아래 (Left/Right 캡션)
  captionWidth?: number;      // HWPUNIT
  captionSpacing?: number;    // HWP16
  /** 글자처럼 취급 (본문배치) */
  treatAsChar?: boolean;
  /** 본문과의 배치 */
  textWrap?: string;
  /** 세로 위치 기준 */
  vertRelTo?: string;
  /** 세로 정렬 */
  vertAlign?: string;
  /** 가로 위치 기준 */
  horzRelTo?: string;
  /** 가로 정렬 */
  horzAlign?: string;
  /** 세로 오프셋 (HWPUNIT) */
  vertOffset?: number;
  /** 가로 오프셋 (HWPUNIT) */
  horzOffset?: number;
  /** 쪽 영역 안으로 제한 */
  restrictInPage?: boolean;
  /** 서로 겹침 허용 */
  allowOverlap?: boolean;
  /** 개체와 조판부호를 항상 같은 쪽에 놓기 */
  keepWithAnchor?: boolean;
  /** 테두리/배경 */
  borderFillId?: number;
  borderLeft?: BorderLineInfo;
  borderRight?: BorderLineInfo;
  borderTop?: BorderLineInfo;
  borderBottom?: BorderLineInfo;
  fillType?: string;
  fillColor?: string;
  patternColor?: string;
  patternType?: number;
}

/** WASM getPageControlLayout() 반환 요소 */
export interface NoteControlRef {
  kind: 'footnote' | 'endnote';
  sectionIdx: number;
  paraIdx: number;
  controlIdx: number;
  noteParaIdx: number;
  innerControlIdx: number;
}

export interface ControlLayoutItem {
  type: 'table' | 'image' | 'shape' | 'equation' | 'group' | 'line' | 'ole';
  x: number;
  y: number;
  w: number;
  h: number;
  secIdx?: number;
  paraIdx?: number;
  controlIdx?: number;
  /** 표 셀 내 수식인 경우: 셀 인덱스 */
  cellIdx?: number;
  /** 표 셀 내 수식인 경우: 셀 내 문단 인덱스 */
  cellParaIdx?: number;
  /** 각주/미주 내부 컨트롤인 경우 원본 위치 */
  noteRef?: NoteControlRef;
  outerTableControlIdx?: number;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
  /**
   * [Task #1280 v2] 렌더 정렬키 — 겹침 클릭 시 "최상단 개체" 판정용.
   * Rust `paper_node_sort_key`(layout.rs)와 단일 진실 원천. 클수록 위.
   * plane: BehindText=1, 어울림/기본=2, InFrontOfText=3.
   */
  plane?: number;
  /** [Task #1280 v2] 개체 z-order (작을수록 먼저 그림 = 아래). */
  zOrder?: number;
  /** [Task #1280 v2] 같은 plane/zOrder 내 안정 정렬 tie-breaker. */
  stableIndex?: number;
  /** [Task #1280 v2] 텍스트 어울림 모드(이미지뿐 아니라 shape/line/group에도 노출). */
  wrap?: string;
  /**
   * [Task #2230] 그림 미지정 placeholder(bin 참조 실패 + 외부 경로 없음).
   * 더블클릭 시 그림 지정(파일 선택) 진입 분기 근거.
   */
  missing?: boolean;
}

/** 개체 참조 (그림/글상자 공용) */
export interface ObjectRef {
  sec: number;
  ppi: number;
  ci: number;
  type: 'image' | 'shape' | 'equation' | 'group' | 'line' | 'ole';
  /** 표 셀 내 수식인 경우: 셀 인덱스 */
  cellIdx?: number;
  /** 표 셀 내 수식인 경우: 셀 내 문단 인덱스 */
  cellParaIdx?: number;
  noteRef?: NoteControlRef;
}

/** WASM getShapeProperties() 반환 타입 */
export interface ShapeProperties {
  width: number;
  height: number;
  treatAsChar: boolean;
  vertRelTo: string;
  vertAlign: string;
  horzRelTo: string;
  horzAlign: string;
  vertOffset: number;
  horzOffset: number;
  textWrap: string;
  /** 크기 고정 */
  sizeProtect?: boolean;
  tbMarginLeft?: number;
  tbMarginRight?: number;
  tbMarginTop?: number;
  tbMarginBottom?: number;
  tbVerticalAlign?: string;
  borderColor?: number;
  borderWidth?: number;
  borderAttr?: number;
  borderOutlineStyle?: number;
  lineType?: number;         // 0=없음, 1=실선, 2=파선, 3=점선, 4=일점쇄선, 5=이점쇄선, ...
  lineEndShape?: number;     // 0=둥근, 1=평면
  arrowStart?: number;       // 0=없음, 1~6=화살표 모양
  arrowEnd?: number;
  arrowStartSize?: number;   // 0~8
  arrowEndSize?: number;
  rotationAngle?: number;
  horzFlip?: boolean;
  vertFlip?: boolean;
  fillType?: string;
  fillBgColor?: number;
  fillPatColor?: number;
  fillPatType?: number;
  fillAlpha?: number;
  gradientType?: number;
  gradientAngle?: number;
  gradientCenterX?: number;
  gradientCenterY?: number;
  gradientBlur?: number;
  roundRate?: number;
  description: string;
}

/** WASM getEquationProperties() 반환 타입 */
export interface EquationProperties {
  width?: number;
  height?: number;
  treatAsChar?: boolean;
  vertRelTo?: string;
  vertAlign?: string;
  horzRelTo?: string;
  horzAlign?: string;
  vertOffset?: number;
  horzOffset?: number;
  textWrap?: string;
  /** 크기 고정 */
  sizeProtect?: boolean;
  zOrder?: number;
  instanceId?: number;
  outerMarginLeft?: number;
  outerMarginTop?: number;
  outerMarginRight?: number;
  outerMarginBottom?: number;
  hasCaption?: boolean;
  captionDirection?: string;
  captionWidth?: number;
  captionSpacing?: number;
  description?: string;
  script: string;
  fontSize: number;
  color: number;
  baseline: number;
  fontName: string;
}

/** WASM getPictureProperties() 반환 타입 */
export interface PictureProperties {
  width: number;
  height: number;
  treatAsChar: boolean;
  vertRelTo: string;
  vertAlign: string;
  horzRelTo: string;
  horzAlign: string;
  vertOffset: number;
  horzOffset: number;
  textWrap: string;
  /** 쪽 영역 안으로 제한 */
  restrictInPage?: boolean;
  /** 서로 겹침 허용 */
  allowOverlap?: boolean;
  /** 크기 고정 */
  sizeProtect?: boolean;
  brightness: number;
  contrast: number;
  effect: string;
  /** 그림 개체 전체 투명도. 한컴 UI 기준 0=불투명, 100=완전 투명. */
  transparency?: number;
  description: string;
  rotationAngle: number;
  horzFlip: boolean;
  vertFlip: boolean;
  originalWidth: number;
  originalHeight: number;
  cropLeft: number;
  cropTop: number;
  cropRight: number;
  cropBottom: number;
  paddingLeft: number;
  paddingTop: number;
  paddingRight: number;
  paddingBottom: number;
  outerMarginLeft: number;
  outerMarginTop: number;
  outerMarginRight: number;
  outerMarginBottom: number;
  borderColor: number;
  borderWidth: number;
  hasCaption: boolean;
  captionDirection: string;
  captionVertAlign: string;
  captionWidth: number;
  captionSpacing: number;
  captionMaxWidth: number;
  captionIncludeMargin: boolean;
  /** [Task #741 후속] 외부 file path (HWP3 외부 그림). 부재 시 문서 포함 그림. */
  externalPath?: string;
}

/** 양식 개체 히트 결과 */
export interface FormObjectHitResult {
  found: boolean;
  sec?: number;
  para?: number;
  ci?: number;
  formType?: 'PushButton' | 'CheckBox' | 'ComboBox' | 'RadioButton' | 'Edit';
  name?: string;
  value?: number;
  caption?: string;
  text?: string;
  bbox?: { x: number; y: number; w: number; h: number };
  // 셀 내부 위치 (표 셀 안에 있는 경우)
  inCell?: boolean;
  tablePara?: number;
  tableCi?: number;
  cellIdx?: number;
  cellPara?: number;
}

/** 양식 개체 값 정보 */
export interface FormValueResult {
  ok: boolean;
  formType?: string;
  name?: string;
  value?: number;
  text?: string;
  caption?: string;
  enabled?: boolean;
}

/** 양식 개체 상세 정보 */
export interface FormObjectInfoResult {
  ok: boolean;
  formType?: string;
  name?: string;
  value?: number;
  text?: string;
  caption?: string;
  enabled?: boolean;
  width?: number;
  height?: number;
  foreColor?: number;
  backColor?: number;
  properties?: Record<string, string>;
  /** ComboBox 항목 목록 (스크립트 InsertString 추출) */
  items?: string[];
}

/** 텍스트 검색 결과 */
export interface SearchResult {
  found: boolean;
  wrapped?: boolean;
  sec?: number;
  para?: number;
  charOffset?: number;
  length?: number;
  cellContext?: {
    parentPara: number;
    ctrlIdx: number;
    cellIdx: number;
    cellPara: number;
  };
}

/** 전체 검색 결과 항목 */
export interface SearchHit {
  sec: number;
  /** 본문 매치: 문단 인덱스. 셀 매치: 부모(호스트) 문단 인덱스 (= cellContext.parentPara) */
  para: number;
  charOffset: number;
  length: number;
  /** 표 셀/글상자 내부 매치 시 컨텍스트. cellPara가 실제 매치 문단 인덱스 */
  cellContext?: {
    parentPara: number;
    ctrlIdx: number;
    cellIdx: number;
    cellPara: number;
  };
}

/** 치환 결과 */
export interface ReplaceResult {
  ok: boolean;
  charOffset?: number;
  newLength?: number;
}

/** 단일 치환 (검색어 기반) 결과 */
export interface ReplaceOneResult {
  ok: boolean;
  sec?: number;
  para?: number;
  charOffset?: number;
  newLength?: number;
}

/** 전체 치환 결과 */
export interface ReplaceAllResult {
  ok: boolean;
  count?: number;
}

/** 쪽 번호 조회 결과 */
export interface PageOfPositionResult {
  ok: boolean;
  page?: number;
}

/** 문서 내 커서 위치 */
export interface DocumentPosition {
  sectionIndex: number;
  paragraphIndex: number;
  charOffset: number;
  /** 셀 컨텍스트 — 레거시 flat 필드 (외부 표 기준) */
  parentParaIndex?: number;
  controlIndex?: number;
  cellIndex?: number;
  cellParaIndex?: number;
  /** 중첩 표 전체 경로 (depth 1=단일 표, depth 2+=중첩 표) */
  cellPath?: CellPathEntry[];
  /** 글상자 내부 여부 */
  isTextBox?: boolean;
  /** hitTest에서 계산된 커서 좌표 (중첩 표 등 getCursorRect 폴백용) */
  cursorRect?: CursorRect;
}

/** 책갈피 정보 */
export interface BookmarkInfo {
  name: string;
  sec: number;
  para: number;
  ctrlIdx: number;
  charPos: number;
}

export type LayerRenderProfile = 'fastPreview' | 'screen' | 'print' | 'highQuality';

export interface LayerBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LayerAffineTransform {
  a: number;
  b: number;
  c: number;
  d: number;
  e: number;
  f: number;
}

export interface PageLayerTree {
  schemaVersion?: number;
  schemaMinorVersion?: number;
  schema?: {
    major: number;
    minor: number;
  };
  unit?: 'px';
  coordinateSystem?: string;
  profile?: LayerRenderProfile;
  buildOptions?: {
    showTransparentBorders?: boolean;
    clipEnabled?: boolean;
  };
  debugOptions?: {
    debugOverlay?: boolean;
  };
  pageWidth: number;
  pageHeight: number;
  outputOptions?: {
    showParagraphMarks?: boolean;
    showControlCodes?: boolean;
    /** Compatibility mirror; prefer buildOptions.showTransparentBorders. */
    showTransparentBorders?: boolean;
    /** Compatibility mirror; prefer buildOptions.clipEnabled. */
    clipEnabled?: boolean;
    /** Compatibility mirror; prefer debugOptions.debugOverlay. */
    debugOverlay?: boolean;
  };
  resources?: LayerResources;
  root: LayerNode;
}

export interface LayerResources {
  tableId?: number;
  images?: Array<Uint8Array | number[] | string | undefined>;
  imageHashes?: string[];
  imageKeys?: string[];
  svgFragments?: Array<string | undefined>;
  svgHashes?: string[];
  svgKeys?: string[];
}

export interface LayerInfo {
  textWrap?: string | null;
  zOrder: number;
  stableIndex: number;
}

export type LayerNode = LayerGroupNode | LayerClipNode | LayerLeafNode;

export interface LayerGroupNode {
  kind: 'group';
  bounds: LayerBounds;
  layer?: LayerInfo;
  groupKind?: { kind: string; [key: string]: unknown };
  cacheHint?: LayerCacheHint;
  children: LayerNode[];
}

export interface LayerClipNode {
  kind: 'clipRect';
  bounds: LayerBounds;
  layer?: LayerInfo;
  clip: LayerBounds;
  clipKind: 'body' | 'tableCell' | 'textBox' | 'generic';
  child: LayerNode;
}

export interface LayerLeafNode {
  kind: 'leaf';
  bounds: LayerBounds;
  layer?: LayerInfo;
  ops: LayerPaintOp[];
}

export type LayerCacheHint =
  | 'none'
  | 'staticSubtree'
  | 'preferRaster'
  | 'preferVectorRecording';

export type LayerPaintOp =
  | LayerPageBackgroundOp
  | LayerTextRunOp
  | LayerFootnoteMarkerOp
  | LayerLineOp
  | LayerRectangleOp
  | LayerEllipseOp
  | LayerPathOp
  | LayerImageOp
  | LayerEquationOp
  | LayerFormObjectOp
  | LayerPlaceholderOp
  | LayerRawSvgOp
  | LayerTextDecorationOp
  | LayerTextControlMarkOp
  | LayerTabLeaderOp
  | LayerCharOverlapOp
  | LayerGlyphRunOp
  | LayerGlyphOutlineOp;

export interface LayerPageBackgroundOp {
  type: 'pageBackground';
  bbox: LayerBounds;
  backgroundColor?: string;
  borderColor?: string;
  borderWidth?: number;
}

export interface LayerTextStyle {
  fontFamily?: string;
  fontSize?: number;
  color?: string;
  bold?: boolean;
  italic?: boolean;
  ratio?: number;
  underline?: string;
  underlineShape?: number;
  strikethrough?: boolean;
  strikeShape?: number;
  outlineType?: number;
  shadowType?: number;
  shadowColor?: string;
  shadowOffsetX?: number;
  shadowOffsetY?: number;
  emboss?: boolean;
  engrave?: boolean;
  superscript?: boolean;
  subscript?: boolean;
  underlineColor?: string;
  strikeColor?: string;
  shadeColor?: string;
  emphasisDot?: number;
}

export interface LayerTextRunOp {
  type: 'textRun';
  bbox: LayerBounds;
  text: string;
  displayText?: string;
  /** Run-local baseline offset from bbox.y when placement is absent. */
  baseline?: number;
  rotation?: number;
  isVertical?: boolean;
  style?: LayerTextStyle;
  placement?: { runToPage?: LayerAffineTransform; baselineY?: number };
  positions?: number[];
  displayPositions?: number[];
  variant?: LayerTextVariantMeta;
}

export interface LayerFootnoteMarkerOp {
  type: 'footnoteMarker';
  bbox: LayerBounds;
  text: string;
  fontFamily?: string;
  fontSize?: number;
  color?: string;
}

export interface LayerLineStyle {
  color?: string;
  width?: number;
  dash?: string;
  lineType?: string;
  startArrow?: string;
  endArrow?: string;
}

export interface LayerShapeStyle {
  fillColor?: string | null;
  strokeColor?: string | null;
  strokeWidth?: number;
  strokeDash?: string;
  opacity?: number;
}

export interface LayerLineOp {
  type: 'line';
  bbox: LayerBounds;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  style?: LayerLineStyle;
}

export interface LayerRectangleOp {
  type: 'rectangle';
  bbox: LayerBounds;
  cornerRadius?: number;
  style?: LayerShapeStyle;
}

export interface LayerEllipseOp {
  type: 'ellipse';
  bbox: LayerBounds;
  style?: LayerShapeStyle;
}

export type LayerPathCommand =
  | { type: 'moveTo'; x: number; y: number }
  | { type: 'lineTo'; x: number; y: number }
  | { type: 'curveTo'; x1: number; y1: number; x2: number; y2: number; x3: number; y3: number }
  | { type: 'arcTo'; rx: number; ry: number; rotation: number; largeArc: boolean; sweep: boolean; x: number; y: number }
  | { type: 'closePath' };

/**
 * [Task #1067] 도형(polygon, rectangle 등) path 의 회전/반전 변환.
 *
 * Rust paint pipeline (`src/paint/json.rs:write_transform`) 이 JSON 으로 다음 형식 emit:
 * `{"rotation": <degrees>, "horzFlip": <bool>, "vertFlip": <bool>}`
 *
 * 누락 시 HWPX/HWP 도형의 회전/flip 정보가 캔버스 렌더링에 반영되지 않아
 * 도형이 회전 없이 출력 (e.g. 두 도형이 거울 대칭이어야 하는데 같은 모양으로 보임).
 */
export interface LayerPathTransform {
  rotation?: number;
  horzFlip?: boolean;
  vertFlip?: boolean;
}

export interface LayerPathOp {
  type: 'path';
  bbox: LayerBounds;
  commands?: LayerPathCommand[];
  style?: LayerShapeStyle;
  lineStyle?: LayerLineStyle;
  transform?: LayerPathTransform;
}

export interface LayerImageOp {
  type: 'image';
  bbox: LayerBounds;
  mime?: string;
  base64?: string;
  imageRef?: number | string;
  fillMode?: string;
  originalSize?: { width: number; height: number };
  crop?: { left: number; top: number; right: number; bottom: number };
  effect?: string;
  brightness?: number;
  contrast?: number;
  opacity?: number;
  bakedWatermark?: boolean;
  wrap?: 'behindText' | 'inFrontOfText' | string;
  transform?: LayerPathTransform;
}

export interface LayerEquationOp {
  type: 'equation';
  bbox: LayerBounds;
  svgContent?: string;
  color?: string;
  fontSize?: number;
  layoutBox?: LayerEquationLayoutBox;
}

export type LayerEquationMatrixStyle = 'plain' | 'paren' | 'bracket' | 'vert';
export type LayerEquationDecoration =
  | 'hat'
  | 'check'
  | 'tilde'
  | 'acute'
  | 'grave'
  | 'dot'
  | 'dDot'
  | 'bar'
  | 'vec'
  | 'dyad'
  | 'under'
  | 'arch'
  | 'underline'
  | 'overline'
  | 'strikeThrough';
export type LayerEquationFontStyle =
  | 'roman'
  | 'italic'
  | 'bold'
  | 'blackboard'
  | 'calligraphy'
  | 'fraktur'
  | 'sansSerif'
  | 'monospace';

export interface LayerEquationLayoutBox {
  x: number;
  y: number;
  width: number;
  height: number;
  baseline: number;
  kind: LayerEquationLayoutKind;
}

export type LayerEquationLayoutKind =
  | { type: 'row'; children: LayerEquationLayoutBox[] }
  | { type: 'text'; text: string }
  | { type: 'number'; text: string }
  | { type: 'symbol'; text: string }
  | { type: 'mathSymbol'; text: string }
  | { type: 'function'; name: string }
  | { type: 'fraction'; numer: LayerEquationLayoutBox; denom: LayerEquationLayoutBox }
  | { type: 'atop'; top: LayerEquationLayoutBox; bottom: LayerEquationLayoutBox }
  | { type: 'sqrt'; body: LayerEquationLayoutBox; index?: LayerEquationLayoutBox }
  | { type: 'superscript'; base: LayerEquationLayoutBox; sup: LayerEquationLayoutBox }
  | { type: 'subscript'; base: LayerEquationLayoutBox; sub: LayerEquationLayoutBox }
  | { type: 'subSup'; base: LayerEquationLayoutBox; sub: LayerEquationLayoutBox; sup: LayerEquationLayoutBox }
  | { type: 'bigOp'; symbol: string; sub?: LayerEquationLayoutBox; sup?: LayerEquationLayoutBox }
  | { type: 'limit'; isUpper: boolean; sub?: LayerEquationLayoutBox }
  | { type: 'matrix'; style: LayerEquationMatrixStyle; cells: LayerEquationLayoutBox[][] }
  | { type: 'rel'; arrow: LayerEquationLayoutBox; over: LayerEquationLayoutBox; under?: LayerEquationLayoutBox }
  | { type: 'eqAlign'; rows: Array<{ left: LayerEquationLayoutBox; right: LayerEquationLayoutBox }> }
  | { type: 'paren'; left: string; right: string; body: LayerEquationLayoutBox }
  | { type: 'decoration'; decoration: LayerEquationDecoration; body: LayerEquationLayoutBox }
  | { type: 'fontStyle'; fontStyle: LayerEquationFontStyle; body: LayerEquationLayoutBox }
  | { type: 'space'; width: number }
  | { type: 'newline' }
  | { type: 'empty' };

export interface LayerFormObjectOp {
  type: 'formObject';
  bbox: LayerBounds;
  formType?: string;
  caption?: string;
  text?: string;
  foreColor?: string;
  backColor?: string;
  value?: boolean;
  enabled?: boolean;
}

export interface LayerPlaceholderOp {
  type: 'placeholder';
  bbox: LayerBounds;
  kind?: 'ole' | 'missingPicture';
  fillColor?: string;
  strokeColor?: string;
  label?: string;
}

export interface LayerRawSvgOp {
  type: 'rawSvg';
  bbox: LayerBounds;
  svg?: string;
}

export interface LayerTextDecorationOp {
  type: 'textDecoration';
  bbox: LayerBounds;
  decoration?: unknown;
}

export interface LayerTextControlMarkOp {
  type: 'textControlMark';
  bbox: LayerBounds;
  fieldMarker?: string | { kind?: string };
}

export interface LayerTabLeaderOp {
  type: 'tabLeader';
  bbox: LayerBounds;
  leaders?: Array<{ startX: number; endX: number; fillType: number }>;
  color?: string;
  fontSize?: number;
  baseline?: number;
}

export interface LayerCharOverlapOp {
  type: 'charOverlap';
  bbox: LayerBounds;
  text?: string;
  baseline?: number;
  style?: LayerTextStyle;
}

export interface LayerGlyphRunOp {
  type: 'glyphRun';
  bbox: LayerBounds;
  variant?: LayerTextVariantMeta;
}

export interface LayerGlyphOutlineOp {
  type: 'glyphOutline';
  bbox: LayerBounds;
  variant?: LayerTextVariantMeta;
  payloadKind?: LayerGlyphOutlinePayloadKind;
  payloadResourceKey?: string;
  paintStyle?: LayerTextStyle;
  placement?: { runToPage?: LayerAffineTransform; baselineY?: number };
  paths?: LayerGlyphOutlinePath[];
  stroke?: LayerGlyphOutlineStroke;
  colorLayers?: LayerColorLayersPayload;
  bitmapGlyph?: LayerBitmapGlyphPayload;
  svgGlyph?: LayerSvgGlyphPayload;
  diagnostics?: { strictVisualEligible?: boolean; [key: string]: unknown };
}

export interface LayerTextVariantMeta {
  equivalenceGroup?: string;
  variantId?: string;
  variantKind?: 'textRun' | 'glyphRun' | 'glyphOutline' | string;
  partIndex?: number;
  partCount?: number;
  isDefaultFallback?: boolean;
  requires?: string[];
  quality?: string;
  anchorOpId?: string;
  localPaintOrder?: number;
}

export type LayerGlyphOutlinePayloadKind =
  | 'monochromeFill'
  | 'monochromeFillStroke'
  | 'colorLayers'
  | 'bitmapGlyph'
  | 'svgGlyph'
  | string;

export interface LayerGlyphOutlinePath {
  glyphId?: number;
  sourceRangeUtf8?: LayerTextRange;
  glyphRange?: LayerTextRange;
  fillRule?: 'nonzero' | 'evenodd' | string;
  commands?: LayerPathCommand[];
}

export interface LayerGlyphOutlineStroke {
  color?: string;
  width?: number;
  join?: 'miter' | 'round' | 'bevel' | string;
  cap?: 'butt' | 'round' | 'square' | string;
  miterLimit?: number;
  paintOrder?: 'fillOnly' | 'strokeOnly' | 'fillThenStroke' | 'strokeThenFill' | string;
  strictSubset?: boolean;
}

export interface LayerTextRange {
  start?: number;
  end?: number;
}

export interface LayerResolvedColor {
  colorSpace?: string;
  rgba?: number[];
}

export interface LayerColorGradientStop {
  offset?: number;
  color?: LayerResolvedColor;
}

export interface LayerColorSolidPathNode {
  commands?: LayerPathCommand[];
  fill?: LayerResolvedColor;
  fillRule?: 'nonzero' | 'evenodd' | string;
  sourceGlyphId?: number;
  paletteIndex?: number;
}

export interface LayerColorLinearGradient {
  x0?: number;
  y0?: number;
  x1?: number;
  y1?: number;
  stops?: LayerColorGradientStop[];
}

export interface LayerColorRadialGradient {
  cx?: number;
  cy?: number;
  radius?: number;
  stops?: LayerColorGradientStop[];
}

export interface LayerColorSweepGradient {
  cx?: number;
  cy?: number;
  startAngleDegrees?: number;
  endAngleDegrees?: number;
  stops?: LayerColorGradientStop[];
}

export interface LayerColorLinearGradientPathNode {
  commands?: LayerPathCommand[];
  gradient?: LayerColorLinearGradient;
  fillRule?: 'nonzero' | 'evenodd' | string;
  sourceGlyphId?: number;
  paletteIndex?: number;
}

export interface LayerColorRadialGradientPathNode {
  commands?: LayerPathCommand[];
  gradient?: LayerColorRadialGradient;
  fillRule?: 'nonzero' | 'evenodd' | string;
  sourceGlyphId?: number;
  paletteIndex?: number;
}

export interface LayerColorSweepGradientPathNode {
  commands?: LayerPathCommand[];
  gradient?: LayerColorSweepGradient;
  fillRule?: 'nonzero' | 'evenodd' | string;
  sourceGlyphId?: number;
  paletteIndex?: number;
}

export interface LayerColorTransformNode {
  childNodeId?: number;
  transform?: LayerAffineTransform;
}

export interface LayerFontColorGlyphRef {
  faceKey?: string;
  glyphId?: number;
  paletteIndex?: number;
  colorFormat?: 'colrV0' | 'colrV1' | 'other' | string;
}

export interface LayerPaletteRef {
  id?: string;
  index?: number;
  cpalDigest?: string;
}

export interface LayerColorPaintGraphNode {
  nodeId?: number;
  kind?: string;
  solidPath?: LayerColorSolidPathNode;
  linearGradientPath?: LayerColorLinearGradientPathNode;
  radialGradientPath?: LayerColorRadialGradientPathNode;
  sweepGradientPath?: LayerColorSweepGradientPathNode;
  transform?: LayerColorTransformNode;
  sourceRangeUtf8?: LayerTextRange;
  glyphRange?: LayerTextRange;
  sourceFontRef?: LayerFontColorGlyphRef;
}

export interface LayerColorPaintGraphPayload {
  rootNodeId?: number;
  nodes?: LayerColorPaintGraphNode[];
}

export interface LayerColorLayersPayload {
  colorFormat?: 'colrV0' | 'colrV1' | 'other' | string;
  sourceFontRef?: LayerFontColorGlyphRef;
  paletteRef?: LayerPaletteRef;
  layers?: Array<{
    layerIndex?: number | null;
    glyphId?: number;
    glyphRange?: LayerTextRange;
    sourceRangeUtf8?: LayerTextRange;
    sourceFontRef?: LayerFontColorGlyphRef;
    commands?: LayerPathCommand[];
    fill?: LayerResolvedColor;
    fillRule?: 'nonzero' | 'evenodd' | string;
    paletteIndex?: number;
    color?: string;
    opacity?: number;
    transformToRun?: LayerAffineTransform;
  }>;
  paintGraph?: LayerColorPaintGraphPayload;
  sourceRangeUtf8?: LayerTextRange;
  glyphRange?: LayerTextRange;
}

export interface LayerBitmapGlyphPayload {
  imageRef?: number;
  imageResourceId?: number | string;
  sourceRangeUtf8?: LayerTextRange;
  glyphRange?: LayerTextRange;
  placement?: LayerBounds;
  alphaPremultiplied?: boolean;
  scalingPolicy?: 'sourceExact' | 'pixelAligned' | 'backendDefault' | string;
  filtering?: 'nearest' | 'linear' | string;
  transformToRun?: LayerAffineTransform;
}

export interface LayerSvgGlyphPayload {
  svgRef?: number;
  vectorResourceId?: number | string;
  sourceRangeUtf8?: LayerTextRange;
  glyphRange?: LayerTextRange;
  viewBox?: LayerBounds;
  intrinsicSize?: { width?: number; height?: number };
  staticSanitized?: boolean;
  scriptAllowed?: boolean;
  animationAllowed?: boolean;
  externalResourcesAllowed?: boolean;
  interactivityAllowed?: boolean;
  transformToRun?: LayerAffineTransform;
}
