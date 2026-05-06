/** WASM getDocumentInfo() 반환 타입 */
export interface DocumentInfo {
  version: string;
  sectionCount: number;
  pageCount: number;
  encrypted: boolean;
  fallbackFont: string;
  fontsUsed: string[];  // 문서에서 사용하는 폰트 이름 목록
}

/** WASM getPageInfo() 반환 타입 */
export interface PageInfo {
  pageIndex: number;
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
  /** 0=top, 1=center, 2=bottom */
  verticalAlign: number;
  /** 0=horizontal, 1=vertical */
  textDirection: number;
  isHeader: boolean;
  /** 셀 보호 */
  cellProtect?: boolean;
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
export interface ControlLayoutItem {
  type: 'table' | 'image' | 'shape' | 'equation' | 'group';
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
}

/** 개체 참조 (그림/글상자 공용) */
export interface ObjectRef {
  sec: number;
  ppi: number;
  ci: number;
  type: 'image' | 'shape' | 'equation' | 'group';
  /** 표 셀 내 수식인 경우: 셀 인덱스 */
  cellIdx?: number;
  /** 표 셀 내 수식인 경우: 셀 내 문단 인덱스 */
  cellParaIdx?: number;
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
  brightness: number;
  contrast: number;
  effect: string;
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
