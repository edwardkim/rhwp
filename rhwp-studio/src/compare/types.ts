export type DiffKind = 'text' | 'table' | 'shape' | 'image' | 'chart' | 'paragraphMeta';
export type DiffSeverity = 'added' | 'removed' | 'modified';

/** 이력 비교(identity) / 문서 비교(alignment) 호출부에서 명시적으로 선택 */
export type CompareStrategy = 'identity' | 'alignment';

export interface CompareAnchorTuning {
  /** 앵커 후보 최소 정규화 길이 */
  minTextLen?: number;
  /** 앵커 후보 최소 고유 문자 수 */
  minUniqueChars?: number;
  /** 공백 비율이 이 값을 넘으면 앵커에서 제외 */
  maxWhitespaceRatio?: number;
  /** Shannon entropy(문자 분포) 하한 */
  minEntropy?: number;
}

export interface ComparePerformanceTuning {
  /** alignment 계산 타임버짓(ms). 초과 시 그리디 폴백 비중을 높인다. */
  maxComputeMs?: number;
  /** 구간 셀 개수(n*m)가 이 값 초과면 DP를 금지한다. */
  hardSegmentCells?: number;
}

export interface CompareOptions {
  caseSensitive: boolean;
  ignoreWhitespace: boolean;
  kinds: DiffKind[];
  /** 호출부 정책에 맞는 비교 전략을 명시적으로 지정 */
  strategy?: CompareStrategy;
  /** 외부 문서 비교용 앵커 튜닝 */
  anchorTuning?: CompareAnchorTuning;
  /** 브라우저 프리징 방지용 계산 가드레일 */
  performanceTuning?: ComparePerformanceTuning;
}

export interface CompareDocMeta {
  name: string;
  sectionCount: number;
  pageCount: number;
  /** pageIndex(0-base) -> 문서 표시 쪽번호(1-base) */
  pageDisplayNumbers?: number[];
}

/** `buildSnapshotFromWasm` / JSON 이력 저장용 — 문단 `stable_id` 포함 (바이너리 .hwp에는 비영속) */
export interface CompareParaSnapshot {
  section: number;
  paragraph: number;
  /** 구역 내 쪽 번호 (1-base) */
  sectionPage: number;
  globalIndex: number;
  stableId: string;
  text: string;
  normalizedText: string;
  controlCount: number;
  /** 정규화 텍스트·컨트롤 개수·(WASM 가능 시)문단 모양 `ParaProperties` 요약을 묶은 digest — 앵커·유일 시그니처 매칭용 */
  signature: string;
  isAnchorCandidate: boolean;
  anchor?: DiffAnchor;
}

export interface CompareControlSnapshot {
  key: string;
  type: string;
  section: number;
  paragraph: number;
  summary: string;
  kind: DiffKind;
  anchor: DiffAnchor;
}

export interface CompareDocumentSnapshot {
  meta: CompareDocMeta;
  paragraphs: CompareParaSnapshot[];
  controls: CompareControlSnapshot[];
}

export interface ComparePath {
  section: number;
  paragraph?: number;
  controlKey?: string;
}

export interface DiffAnchor {
  pageIndex: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DiffItem {
  id: string;
  kind: DiffKind;
  severity: DiffSeverity;
  path: ComparePath;
  title: string;
  leftPreview: string;
  rightPreview: string;
  leftAnchor?: DiffAnchor;
  rightAnchor?: DiffAnchor;
  /**
   * alignment 기준 반대쪽 문단 — 한쪽만 앵커가 있어도 **각 문서의 레이아웃**으로 페이지를 연다.
   * 예: 오른쪽 '문단 추가'면 왼쪽 패널은 삽입 직전에 짝을 맞춘 왼쪽 문단(`getCursorRect`) 기준.
   */
  contextOnLeft?: { section: number; paragraph: number };
  contextOnRight?: { section: number; paragraph: number };
  /** 좌/우 문서 기준 구역 내 쪽 번호 (1-base) */
  leftSectionPage?: number;
  rightSectionPage?: number;
  /** 정체성 매칭 후 문단 내부 문자 단위 요약(`myersCharDiffSummary`: Levenshtein·Hirschberg·`CHAR_DIFF_*` 상한) */
  inlineTextDiff?: string;
}

export interface CompareSession {
  left: CompareDocMeta;
  right: CompareDocMeta;
  options: CompareOptions;
  diffItems: DiffItem[];
  currentDiffIndex: number;
  generatedAt: number;
  /** 실제 적용된 본문 텍스트 비교 방식 */
  textCompareStrategyUsed?: 'identity' | 'alignment';
}
