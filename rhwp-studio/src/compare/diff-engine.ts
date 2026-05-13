/**
 * HWP 문서 비교 엔진 — 문단 정렬(alignment), 문자 단위 diff, 표·도형·그림 등 컨트롤 diff를 한 모듈에서 처리한다.
 *
 * ── [용어] “밀림(shift)”
 * 한쪽에 문단·표가 삽입되면 이후 문단의 `section`/`paragraph`/`globalIndex`가 통째로 밀린다.
 * 텍스트는 내용 기반 정렬로 상쇄할 수 있으나, 컨트롤 키에 물리 위치가 남으면 표/도형이 “삭제+추가”로
 * 쪼개져 보이는 문제가 생긴다. 이 파일은 (A) 문단 쪽 밀림 완화 (B) 정렬 결과를 컨트롤에 전달 두 축으로 대응한다.
 *
 * ── [도메인 분리: identity vs alignment]
 * - **이력(같은 혈통)**: `stable_id`가 양쪽에 공유되면 `identity` 경로로 문단 1:1을 직접 잡는다(O(N) 수준).
 * - **외부 문서(다른 혈통)**: 공유 sid가 없으므로 `alignment`만 사용한다. 유일·긴 문단과 섹션 제목류를
 *   **글로벌 앵커**로 써 문서를 구간으로 쪼개고, 구간 안은 DP/그리디로 채운다.
 *
 * ── [문단 밀림 완화 — alignment 내부 계층]
 * 1. **글로벌 앵커** (`buildAnchorPairs`, `fillSnapshotFromWasm`의 `isAnchorCandidate`)
 *    - 시그니처가 좌·우 각각 한 번만 등장하는 문단만 앵커 후보.
 *    - 길이·엔트로피 등 `isAnchorTextQualityOk`로 짧은 반복 문장을 걸러 오염 앵커를 막는다.
 *    - `[테스트 N: …]` 같은 짧은 제목은 `isStructuralBlockTitleLine` 등으로 후보로 올리되,
 *      `buildBothSidesUniqueTrimNorm`으로 **trim 본문이 양쪽 문서에서 정확히 한 번씩**일 때만 글로벌 앵커로 승인.
 * 2. **구간 정렬** (`matchSegment` 재귀 트리)
 *    - `buildUniqueSigPairsInSlices`: 구간 안에서만 시그니처 빈도를 세 **내부 Patience 핀**으로 쪼갠다.
 *    - `tryMatchByNormHashPins`: 서명 핀이 없을 때 `normalizedText` 해시 토큰으로 거시 핀(DMP line-mode 유사).
 *    - `matchSegmentDp` / `findLongestEqualSignatureRun` / `matchWindowedGreedy`: 셀 수·시간 한도에 따라 선택.
 * 3. **상대 구조 근접** (`SegmentAnchorBoundary` + `runWithSegmentStructBase` + `isNearStructure`)
 *    - 구간마다 직전 글로벌 앵커 쌍의 `globalIndex`를 베이스로 잡고, `|Δleft - Δright| ≤ 2`로 본다.
 *    - 문서 맨 앞 구간은 베이스가 구간 **첫 문단**이다. 절대 인덱스만 보면 큰 밀림 직후 이웃 문단이
 *      구조적으로 가깝다고 인정받지 못하는 문제를 줄인다.
 * 4. **DP 비용** (`matchCost`, `getEffectiveSimilarity`, `softSimilarityThresholdForPair`)
 *    - 유사도는 “전역 라벨”이 아니라 치환 비용·약매칭 방지용 가중치로만 쓴다.
 *
 * ── [컨트롤 밀림 완화]
 * - 1차: `kind::key` 정확 일치(`sid:…` 우선).
 * - 2차: **`buildRightToLeftParaMapFromAligned`** — `buildTextDiffs`가 만든 문단 정렬 `AlignedPair[]`에서
 *   오른쪽 `section:paragraph` → 짝 왼쪽 문단 맵을 구축해 `buildControlDiffs`에 전달한다.
 * - 3차: **`extractTablePatiencePins`** — 요약 키가 양쪽에서 각각 유일한 표를 슬롯보다 먼저 고정한다.
 * - 4차: **`pairAlignmentSlotControls`** — 남은 컨트롤에 대해, 오른쪽 부모 문단의 맵상 짝 왼쪽 문단에 붙은
 *   미매칭 컨트롤만 후보로 `scoreControlFallback + ALIGNMENT_CONTROL_SLOT_BONUS`로 짝을 찾는다.
 * - 5차: `pairControlsFallback`(전역 탐욕, 임계 2.75).
 *
 * ── [단계적 비교(성능)]
 * - 전 문단×문단 유사도 행렬 같은 O(N²) 전역 최적화는 하지 않는다.
 * - 짝이 맞은 문단에만 `myersCharDiffSummary`(접두·접미 제거, Hirschberg, `CHAR_DIFF_*` 상한)를 적용한다.
 *
 * ── [튜닝 상수 — `// ─── 튜닝 상수` 블록]
 * - 앵커·구간: ANCHOR_*, INTRA_UNIQUE_SIG_*, SEGMENT_DP_MAX, NORM_HASH_PIN_MIN_TOTAL_PARAS, HARD_SEGMENT_CELL_LIMIT,
 *   ALIGNMENT_MAX_COMPUTE_MS, MAX_SEGMENT_RECURSION
 * - 구조 근접·비용: NEAR_STRUCTURE_*, MATCH_SOFT_SIM_MIN, MATCH_COST_WEAK, WINDOW_SIZE, GREEDY_AMBIGUOUS_GAP
 * - 컨트롤 슬롯: ALIGNMENT_CONTROL_SLOT_BONUS, ALIGNMENT_CONTROL_MIN_ADJUSTED_SCORE,
 *   TABLE_SUMMARY_MISMATCH_PENALTY, TABLE_PAIR_SIM_NO_PENALTY, TABLE_PAIR_SIM_FULL_PENALTY
 * - 후처리: PARA_SPLIT_JOIN_SIM_MIN, REMOVED_ADDED_*
 * - 문자 요약: CHAR_DIFF_FULL_DP_MAX, CHAR_DIFF_TOTAL_MAX, CHAR_DIFF_CELL_HARD
 *
 * ── [입력 경로]
 * - `buildSnapshotFromBytes`: 외부 파일 비교.
 * - `buildSnapshotFromWasm`: 편집기 IR — 같은 세션 이력에서 sid 보존에 유리.
 *
 * ── [공개 진입점]
 * - `compareSnapshots`: 전략 선택 → 본문 diff → 컨트롤 diff(맵 전달) → 필터·쪽번호·정렬.
 * - `compareDocuments`: bytes 두 개를 스냅샷으로 만든 뒤 위와 동일.
 *
 * ── [유지보수]
 * - 품질 이슈 시 compare-debug 로그 ① stable_id ② 전략 ③ 앵커를 함께 본다.
 * - 본 파일은 `// ───` 섹션 구분으로 점프 검색이 가능하다.
 */
import { WasmBridge } from '@/core/wasm-bridge';
import type { ControlLayoutItem, DocumentInfo, ParaProperties } from '@/core/types';
import { compareDbg, isCompareDebugEnabled } from './compare-debug';
import type {
  CompareAnchorTuning,
  CompareControlSnapshot,
  CompareDocumentSnapshot,
  CompareOptions,
  CompareParaSnapshot,
  ComparePerformanceTuning,
  CompareSession,
  CompareStrategy,
  DiffAnchor,
  DiffItem,
  DiffKind,
} from './types';

// ═══ 튜닝·런타임 가드 ═══════════════════════════════════════════════════════
// alignment·컨트롤 슬롯 매칭의 비용/품질은 아래 상수에 강하게 묶여 있다.
// 튜닝 변경 후에는 통합 테스트용 HWP(앵커·표 삽입·밀림)으로 회귀 확인하는 것을 권장한다.
// 파일 맨 위 블록 주석(용어·파이프라인)과 이 표를 같이 읽으면 의도가 정리된다.

// ─── 튜닝 상수 (값 변경 시 대표 문서로 회귀 확인 권장) ─────────────────
// 상호 의존:
// - NEAR_STRUCTURE_* 는 `isNearStructure`가 true일 때만 DP/그리디에 반영된다.
// - 큰 밀림 직후에도 구간 베이스(`SegmentAnchorBoundary`)가 맞으면 isNear가 true가 되기 쉬워져
//   치환 비용 할인·유사도 완화가 켜진다.
// - 그래도 (L,null)(null,R) 패턴으로 남는 구간은 REMOVED_ADDED_* 로 “한 슬롯 수정”으로 승격할 수 있다.
/** alignment 경로: 오른쪽 문단마다 왼쪽에서 고를 때의 탐색 반경(문단 개수) */
const WINDOW_SIZE = 32;
/** `buildAnchorPairs`: 시그니처가 같아도 너무 짧은 문단은 앵커 후보에서 제외(오탐 앵커 방지) */
const ANCHOR_MIN_TEXT_LEN = 20;
/** `[테스트 N: …]` 등 짧은 블록 제목을 앵커로 쓸 때 최소 글자 수(공백 제거 `trim` 기준) */
const STRUCTURAL_ANCHOR_MIN_LEN = 5;
/** 최상위 구간 `buildUniqueSigPairsInSlices` 최소 문단 길이 */
const INTRA_UNIQUE_SIG_MIN_TOP = 10;
/** 글로벌 앵커로 이미 자른 하위 구간에서 내부 유일 시그니처 핀의 최소 길이 */
const INTRA_UNIQUE_SIG_MIN_NESTED = 4;
/** 동일 서명 문단 쌍의 globalIndex 차가 이 값보다 크면 "문단 이동" 메타 후보로 본다 */
const MOVE_DISTANCE_THRESHOLD = 3;
/** 정렬이 (null,R앞)(L,R뒤)로 나온 쪼개기만: L≈R앞+R뒤일 때 R앞=변경·R뒤=추가로 재라벨 */
const PARA_SPLIT_JOIN_SIM_MIN = 0.86;
/** 구간 내 DP 직접 적용 최대 한 변 길이 (n*m <= MAX^2) */
const SEGMENT_DP_MAX = 150;
/** DP에서 약한 유사도끼리 붙는 것 방지 (치환 비용 하한으로 del+ins보다 불리하게) */
const MATCH_SOFT_SIM_MIN = 0.7;
/** `matchCost`: 유사도가 너무 낮으면 치환 대신 삽입+삭제 쪽으로 유도하는 비용 상한 */
const MATCH_COST_WEAK = 4;
/** `isNearStructure`이고 문단이 충분히 길 때, `textSimilarity` 결과를 DP/그리디 평가용으로만 끌어올리는 하한 */
const NEAR_STRUCTURE_SIM_BOOST = 0.4;
/** 위 부스트를 받기 위한 최소 raw 유사도(무연관 짧은 문단·우연 일치 완화) */
const NEAR_STRUCTURE_MIN_SIM = 0.12;
/** 구조 근접 시 치환 비용(`1-sim`)에서 깎는 할인 — `getEffectiveSimilarity`와 별개 튜닝 */
const NEAR_STRUCTURE_COST_DISCOUNT = 0.4;
/** 그리디 `scorePairGreedy`: `isNearStructure`일 때 `minScore`(의역·서명 불일치) 통과용 가산 */
const NEAR_STRUCTURE_GREEDY_BONUS = 1.5;
/** 그리디: 1·2위 점수 차가 이 값 미만이면 동률로 보고 매칭 포기(1위가 구조 근접이면 검사 생략) */
const GREEDY_AMBIGUOUS_GAP = 0.35;
/** (L,null)(null,R) 등 삭제+추가로만 남았지만 동일 슬롯 수정으로 승격할 최소 textSimilarity */
const REMOVED_ADDED_MERGE_SIM_MIN = 0.28;
/** 위 승격: 같은 구역에서 허용하는 문단 번호·globalIndex 최대 간격(위쪽 삽입으로 밀린 경우) */
const REMOVED_ADDED_MAX_PARA_GAP = 12;
const REMOVED_ADDED_MAX_GLOBAL_GAP = 24;
/** 문자 diff: 이 이하의 n×m만 전역 DP 역추적(Hirschberg 잎) */
const CHAR_DIFF_FULL_DP_MAX = 280_000;
/** 좌+우 길이 합 상한 — 그 이상은 요약 생략(메인 스레드 보호) */
const CHAR_DIFF_TOTAL_MAX = 96_000;
/** n×m 셀 상한 — 평균 케이스에서도 과도한 O(nm) 방지 */
const CHAR_DIFF_CELL_HARD = 14_000_000;
/** 앵커 품질 기본 가드레일: 공백 비율 상한 */
const ANCHOR_MAX_WHITESPACE_RATIO = 0.62;
/** 앵커 품질 기본 가드레일: 최소 고유 문자 수 */
const ANCHOR_MIN_UNIQUE_CHARS = 6;
/** 앵커 품질 기본 가드레일: 최소 엔트로피 */
const ANCHOR_MIN_ENTROPY = 1.9;
/** 브라우저 프리징 방지: 구간 셀 수 하드캡(초과 시 DP 금지) */
const HARD_SEGMENT_CELL_LIMIT = 180_000;
/** 브라우저 프리징 방지: alignment 타임버짓 기본값(ms) */
const ALIGNMENT_MAX_COMPUTE_MS = 2600;
/** `matchSegment` 재귀(내부 앵커·half-match) 최대 깊이 */
const MAX_SEGMENT_RECURSION = 96;
/** half-match 전수 탐색 시 n×m 상한(초과 시 half-match 생략) */
const HALF_MATCH_MAX_PRODUCT = 450_000;
/**
 * `tryMatchByNormHashPins`: 서명 유일 핀이 없을 때 `normalizedText` trim을 해시 토큰으로 바꿔
 * 구간 내 유일 쌍을 찾는 **거시** 단계. 좌·우 문단 수 합이 이 값 미만이면 오버헤드만 커져 생략한다.
 */
const NORM_HASH_PIN_MIN_TOTAL_PARAS = 48;
/**
 * `pairAlignmentSlotControls`: 문단 정렬로 이미 “같은 논리 문단”으로 묶인 표·도형에 대해
 * `scoreControlFallback` 원점수에 더하는 보너스. 위쪽 삽입으로 앵커 y가 크게 달라져도
 * 원점수가 2.75 근처에서 막히는 현상을 완화한다.
 */
const ALIGNMENT_CONTROL_SLOT_BONUS = 2.35;
/**
 * 슬롯 단계에서 채택하는 최소 점수 = `scoreControlFallback(l,r) + ALIGNMENT_CONTROL_SLOT_BONUS`.
 * 너무 낮추면 다른 문단 개체와 오매칭, 너무 높이면 슬롯이 거의 동작하지 않는다.
 */
const ALIGNMENT_CONTROL_MIN_ADJUSTED_SCORE = 4.38;
/**
 * `scoreControlFallback`: 표 요약(`summary`)이 다를 때 부과하는 **최대** 감점.
 * `tableControlPairContentSimilarity`가 높으면(같은 행·열 그리드에서 셀만 수정) 감점 비율을 0에 가깝게 줄여
 * `buildGranularControlDiffs`로 “표 수정”이 나가게 한다. 유사도가 낮으면(다른 표) 전액 감점한다.
 */
const TABLE_SUMMARY_MISMATCH_PENALTY = 4.25;
/** 이 이상이면 표 요약 불일치 감점을 적용하지 않는다(한 셀 수정 등). */
const TABLE_PAIR_SIM_NO_PENALTY = 0.74;
/** 이 이하이면 표 감점을 전액 적용한다(내용이 다른 표). */
const TABLE_PAIR_SIM_FULL_PENALTY = 0.36;

/** `compareSnapshots` 한 번 호출 동안만 유효. `ALIGNMENT_MAX_COMPUTE_MS` 경과 시 greedy 쪽으로 이탈한다. */
type CompareRuntimeGuard = {
  deadline: number;
  bailedOut: boolean;
};

let activeRuntimeGuard: CompareRuntimeGuard | null = null;

/**
 * `isNearStructure`가 쓰는 좌·우 `globalIndex` 베이스(한 쌍의 절대 인덱스).
 * `matchSegment` 재귀 전체에서 동일 베이스를 쓰면, “구간 전체가 위에서 k칸 밀렸다”를
 * `dL = lp.globalIndex - leftBaseGi`, `dR = rp.globalIndex - rightBaseGi`로 보고 `|dL-dR|≤2`만 검사한다.
 */
type SegmentStructBase = { leftBaseGi: number; rightBaseGi: number };
/**
 * 글로벌 앵커 한 쌍의 globalIndex. `buildTextDiffs`가 앵커 사이 슬라이스를 `matchSegment`에 넘길 때
 * 직전 경계 앵커 `(a.li,a.ri)`가 있으면 그 문단의 인덱스를 베이스로 넣고, 문서 맨 앞 구간은 null이라
 * 베이스가 구간의 **첫 문단** 쌍으로 대체된다(`matchSegment` 내부에서 산출).
 */
type SegmentAnchorBoundary = { leftAnchorGi: number; rightAnchorGi: number };
/** `matchSegment`/`matchSegmentDp`/`matchCost` 호출 동안만 세팅. 중첩 재귀 시 스택처럼 복구한다. */
let activeSegmentStructBase: SegmentStructBase | null = null;

/** `fn` 실행 동안 `isNearStructure`가 `base` 기준 상대 오프셋을 보도록 한다. */
function runWithSegmentStructBase<T>(base: SegmentStructBase, fn: () => T): T {
  const prev = activeSegmentStructBase;
  activeSegmentStructBase = base;
  try {
    return fn();
  } finally {
    activeSegmentStructBase = prev;
  }
}

/** `CompareOptions.anchorTuning`이 있으면 그걸 쓰고, 없으면 파일 상단 기본 앵커 가드레일을 쓴다. */
function resolveAnchorTuning(options: CompareOptions): Required<CompareAnchorTuning> {
  return {
    minTextLen: options.anchorTuning?.minTextLen ?? ANCHOR_MIN_TEXT_LEN,
    minUniqueChars: options.anchorTuning?.minUniqueChars ?? ANCHOR_MIN_UNIQUE_CHARS,
    maxWhitespaceRatio: options.anchorTuning?.maxWhitespaceRatio ?? ANCHOR_MAX_WHITESPACE_RATIO,
    minEntropy: options.anchorTuning?.minEntropy ?? ANCHOR_MIN_ENTROPY,
  };
}

/** `hardSegmentCells` / `maxComputeMs` — 큰 문서에서 DP 테이블·이중 루프가 메인 스레드를 막지 않게 상한을 둔다. */
function resolvePerformanceTuning(options: CompareOptions): Required<ComparePerformanceTuning> {
  return {
    maxComputeMs: options.performanceTuning?.maxComputeMs ?? ALIGNMENT_MAX_COMPUTE_MS,
    hardSegmentCells: options.performanceTuning?.hardSegmentCells ?? HARD_SEGMENT_CELL_LIMIT,
  };
}

function shannonEntropy(text: string): number {
  if (!text) return 0;
  const freq = new Map<string, number>();
  for (const ch of text) freq.set(ch, (freq.get(ch) ?? 0) + 1);
  let entropy = 0;
  for (const c of freq.values()) {
    const p = c / text.length;
    entropy -= p * Math.log2(p);
  }
  return entropy;
}

/**
 * 글로벌 앵커 후보 문단의 "텍스트 품질" 검사(엔트로피·공백 비율·고유 문자 수).
 *
 * 너무 짧거나 반복적인 줄을 앵커로 쓰면 뒤따르는 모든 구간 정렬이 한 번에 틀어질 수 있어 기본은 배제한다.
 * 다만 `[테스트 N: …]`·`1. …` 같이 **구조적 한 줄 제목**으로 보이는 패턴(`isStructuralBlockTitleLine`)은
 * 길이가 `STRUCTURAL_ANCHOR_MIN_LEN` 이상·`minTextLen` 미만이면 엔트로피 검사 없이 후보로 통과시킨다.
 * 짧은 줄의 **전역 유일성**(양쪽 문서에서 trim 본문이 각각 한 번)은 `buildAnchorPairs`에서 별도로 걸러진다.
 */
function isAnchorTextQualityOk(text: string, tuning: Required<CompareAnchorTuning>): boolean {
  const trimmed = text.trim();
  if (
    trimmed.length >= STRUCTURAL_ANCHOR_MIN_LEN &&
    trimmed.length < tuning.minTextLen &&
    isStructuralBlockTitleLine(trimmed)
  ) {
    return true;
  }
  if (text.length < tuning.minTextLen) return false;
  const whitespaceCount = (text.match(/\s/g) ?? []).length;
  if (whitespaceCount / Math.max(1, text.length) > tuning.maxWhitespaceRatio) return false;
  if (new Set(text).size < tuning.minUniqueChars) return false;
  return shannonEntropy(text) >= tuning.minEntropy;
}

/**
 * `[테스트 N: …]`·`[제목]`·`1. 소제목` 같이 짧아도 블록 경계로 쓸 만한 한 줄 제목.
 * 글로벌 앵커 후보는 `signature` 중복 여부로 별도 차단한다.
 */
function isStructuralBlockTitleLine(trimmedOneLine: string): boolean {
  if (!trimmedOneLine) return false;
  if (/^\[[^\]]+\]$/.test(trimmedOneLine)) return true;
  if (/^\[[^\]]+:[^\]]+\]$/.test(trimmedOneLine)) return true;
  if (/^\d+\.\s+/.test(trimmedOneLine)) return true;
  return false;
}

/**
 * alignment 중 시간이 `maxComputeMs`를 넘기면 true로 고정되며, 이후 DP 진입을 막고 그리디로만 처리한다.
 * DP 이중 루프 안에서는 `i % 12` 등으로 가끔만 호출해 오버헤드를 제한한다.
 */
function shouldBailToGreedy(): boolean {
  if (!activeRuntimeGuard) return false;
  if (activeRuntimeGuard.bailedOut) return true;
  if (Date.now() <= activeRuntimeGuard.deadline) return false;
  activeRuntimeGuard.bailedOut = true;
  return true;
}

// ─── 문자 diff 요약 (`myersCharDiffSummary`): 접두·접미, 전역 DP 잎, Hirschberg ─

/** 공통 접두·접미를 제거해 Levenshtein/Hirschberg 입력을 줄인다. */
function stripCommonAffixChars(left: string, right: string): { a: string; b: string } {
  let lo = 0;
  const minLen = Math.min(left.length, right.length);
  while (lo < minLen && left.charCodeAt(lo) === right.charCodeAt(lo)) lo += 1;
  let suf = 0;
  while (
    suf < left.length - lo &&
    suf < right.length - lo &&
    left.charCodeAt(left.length - 1 - suf) === right.charCodeAt(right.length - 1 - suf)
  )
    suf += 1;
  return { a: left.slice(lo, left.length - suf), b: right.slice(lo, right.length - suf) };
}

function levenshteinDistanceTwoRow(a: string, b: string): number {
  const n = a.length;
  const m = b.length;
  if (n === 0) return m;
  if (m === 0) return n;
  let prev = new Array<number>(m + 1);
  let cur = new Array<number>(m + 1);
  for (let j = 0; j <= m; j += 1) prev[j] = j;
  for (let i = 1; i <= n; i += 1) {
    cur[0] = i;
    const cAi = a.charCodeAt(i - 1);
    for (let j = 1; j <= m; j += 1) {
      const eq = cAi === b.charCodeAt(j - 1) ? 0 : 1;
      cur[j] = Math.min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + eq);
    }
    const t = prev;
    prev = cur;
    cur = t;
  }
  return prev[m];
}

/** `a[0..nrow)` vs `b` — 마지막 행 비용만 (Hirschberg 전반). `nrow===0`이면 삽입만. */
function levenshteinLastRowPrefix(a: string, nrow: number, b: string): number[] {
  const m = b.length;
  if (nrow <= 0) {
    const row = new Array<number>(m + 1);
    for (let j = 0; j <= m; j += 1) row[j] = j;
    return row;
  }
  let prev = new Array<number>(m + 1);
  let cur = new Array<number>(m + 1);
  for (let j = 0; j <= m; j += 1) prev[j] = j;
  for (let i = 1; i <= nrow; i += 1) {
    cur[0] = i;
    const cAi = a.charCodeAt(i - 1);
    for (let j = 1; j <= m; j += 1) {
      const eq = cAi === b.charCodeAt(j - 1) ? 0 : 1;
      cur[j] = Math.min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + eq);
    }
    const t = prev;
    prev = cur;
    cur = t;
  }
  return prev;
}

/** `D[i][j]` = ed(`a[i..n)`, `b[j..m)`). `i === mid`인 행만 필요할 때 아래로 채운다. */
function levenshteinSuffixRowAt(a: string, mid: number, b: string): number[] {
  const n = a.length;
  const m = b.length;
  let cur = new Array<number>(m + 1);
  for (let j = 0; j <= m; j += 1) cur[j] = m - j;
  for (let i = n - 1; i >= mid; i -= 1) {
    const next = new Array<number>(m + 1);
    next[m] = n - i;
    const ca = a.charCodeAt(i);
    for (let j = m - 1; j >= 0; j -= 1) {
      const eq = ca === b.charCodeAt(j) ? 0 : 1;
      next[j] = Math.min(cur[j] + 1, next[j + 1] + 1, cur[j + 1] + eq);
    }
    cur = next;
  }
  return cur;
}

/** 전체 `dp` 역추적 — `n*m`이 작을 때만 호출. */
function charEditOpsFullDp(left: string, right: string): string {
  const n = left.length;
  const m = right.length;
  const dp: number[][] = Array.from({ length: n + 1 }, () => Array(m + 1).fill(0));
  for (let i = 0; i <= n; i += 1) dp[i][0] = i;
  for (let j = 0; j <= m; j += 1) dp[0][j] = j;
  for (let i = 1; i <= n; i += 1) {
    for (let j = 1; j <= m; j += 1) {
      const eq = left.charCodeAt(i - 1) === right.charCodeAt(j - 1) ? 0 : 1;
      dp[i][j] = Math.min(dp[i - 1][j] + 1, dp[i][j - 1] + 1, dp[i - 1][j - 1] + eq);
    }
  }
  let i = n;
  let j = m;
  const ops: string[] = [];
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && left.charCodeAt(i - 1) === right.charCodeAt(j - 1)) {
      ops.push('=');
      i -= 1;
      j -= 1;
    } else if (i > 0 && dp[i][j] === dp[i - 1][j] + 1) {
      ops.push('-');
      i -= 1;
    } else if (j > 0 && dp[i][j] === dp[i][j - 1] + 1) {
      ops.push('+');
      j -= 1;
    } else if (i > 0 && j > 0) {
      ops.push('×');
      i -= 1;
      j -= 1;
    } else if (i > 0) {
      ops.push('-');
      i -= 1;
    } else {
      ops.push('+');
      j -= 1;
    }
  }
  ops.reverse();
  return ops.join('');
}

function charEditOpsHirschberg(a: string, b: string): string {
  const n = a.length;
  const m = b.length;
  if (n === 0) return '+'.repeat(m);
  if (m === 0) return '-'.repeat(n);
  if (n * m <= CHAR_DIFF_FULL_DP_MAX) {
    return charEditOpsFullDp(a, b);
  }
  const mid = Math.max(1, Math.floor(n / 2));
  const f = levenshteinLastRowPrefix(a, mid, b);
  const suf = levenshteinSuffixRowAt(a, mid, b);
  let bestJ = 0;
  let best = Infinity;
  for (let j = 0; j <= m; j += 1) {
    const s = f[j] + suf[j];
    if (s < best || (s === best && j < bestJ)) {
      best = s;
      bestJ = j;
    }
  }
  return (
    charEditOpsHirschberg(a.slice(0, mid), b.slice(0, bestJ)) +
    charEditOpsHirschberg(a.slice(mid), b.slice(bestJ))
  );
}

/**
 * 동일 stable_id 문단의 문자 단위 편집 거리·간단 패턴 요약 (2-depth diff).
 * 공통 접두·접미 제거 후, 작은 구간은 전역 DP, 큰 구간은 Hirschberg로 선형 메모리에 가깝게 처리한다.
 */
function myersCharDiffSummary(left: string, right: string): string {
  if (left.length === 0 && right.length === 0) return '';
  if (left.length + right.length > CHAR_DIFF_TOTAL_MAX) {
    return `문자 diff 요약 생략(길이: ${left.length}+${right.length})`;
  }
  const { a, b } = stripCommonAffixChars(left, right);
  const n = a.length;
  const m = b.length;
  if (n === 0 && m === 0) return '';
  if (n * m > CHAR_DIFF_CELL_HARD) {
    return `문자 diff 요약 생략(과대: ${n}×${m})`;
  }
  const dist = levenshteinDistanceTwoRow(a, b);
  const opStr = charEditOpsHirschberg(a, b);
  const pat = opStr.replace(/=+/g, '·').slice(0, 100);
  return `편집거리 ${dist} · ${pat}`;
}

// ─── 정규화·해시·Diff ID (문단 시그니처·컨트롤 요약에 공통 사용) ───────────────

/** 비교 옵션에 따른 문단 텍스트 정규화. `ignoreWhitespace`/`caseSensitive`는 여기서만 일괄 적용된다. */
function normalizeText(text: string, options: CompareOptions): string {
  const base = options.ignoreWhitespace ? text.replace(/\s+/g, ' ').trim() : text;
  return options.caseSensitive ? base : base.toLowerCase();
}

/** 짧은 FNV-1a digest — 문단 시그니처·표 텍스트 digest 등 충돌 가능성은 있으나 비교용으론 충분 */
function simpleHash(input: string): string {
  let h = 2166136261;
  for (let i = 0; i < input.length; i += 1) {
    h ^= input.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0).toString(16);
}

/** 바이너리(이미지 픽셀 등)용 FNV-1a 계열 해시 — 짧은 digest 문자열로 요약 비교에 사용 */
function simpleHashBytes(bytes: Uint8Array): string {
  let h = 2166136261;
  for (let i = 0; i < bytes.length; i += 1) {
    h ^= bytes[i];
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0).toString(16);
}

/** DiffItem.id — kind와 고유 키를 합쳐 UI 목록·로그·reflow 필터(`id-moved:` 등)에서 항목을 식별 */
function mkDiffId(kind: DiffKind, key: string): string {
  return `${kind}:${key}`;
}

/**
 * WASM `item.type` + 요약 문자열을 DiffKind로 정규화.
 * chart는 summary 키워드로 shape/group과 구분한다(도형 안 차트 등).
 */
function mapControlKind(type: string, summary: string): DiffKind {
  if (type === 'table') return 'table';
  if (type === 'image') return 'image';
  if (type === 'shape' || type === 'group') {
    if (summary.toLowerCase().includes('chart')) return 'chart';
    return 'shape';
  }
  if (summary.toLowerCase().includes('chart')) return 'chart';
  return 'paragraphMeta';
}

/**
 * 동일 실개체가 `group`↔`shape`처럼 타입만 바뀌며 중복 수집되는 경우를 한 키로 묶는다.
 * 매칭/중복 제거(`uniqueControls`)의 기준이 되므로 stem(`sid:…` 또는 `loc:…`) 추출 규칙을 바꿀 때는
 * 레이아웃 경로·문단 직접 경로 양쪽의 key 포맷을 함께 점검해야 한다.
 */
function canonicalControlKey(c: CompareControlSnapshot): string {
  const m = c.key.match(/^(sid:[^:]+:\d+|loc:-?\d+:-?\d+:\d+):[^:]+$/);
  if (!m) return `${c.kind}::${c.key}`;
  const stem = m[1];
  // 요소별 변경 추적 안정화:
  // - 동일 anchor(control index)인데 type만 group/shape로 달라지는 중복을 하나로 합친다.
  // - 매칭 단계에서는 canonical key를 기준으로 "같은 개체"를 판정한다.
  if (c.kind === 'shape' || c.kind === 'chart') return `${c.kind}::${stem}:shape`;
  if (c.kind === 'table') return `${c.kind}::${stem}:table`;
  return `${c.kind}::${stem}:image`;
}

/** 동일 canonical 키로 여러 스냅샷이 들어왔을 때, UI에 더 유의미한 요약을 남기기 위한 휴리스틱 점수 */
function controlSnapshotQuality(c: CompareControlSnapshot): number {
  let score = 0;
  // 요소별 변경 추적 품질 점수:
  // - 텍스트/픽셀해시/유효 bbox가 있는 스냅샷을 우선 채택해
  //   동일 key 중 더 "정보가 풍부한" 항목을 남긴다.  
  if (c.summary.includes('text="') && !c.summary.includes('text="(없음)"')) score += 4;
  if (!c.summary.includes('pix=nopix')) score += 2;
  if (!c.summary.includes('nobox')) score += 1;
  if (c.type === 'shape') score += 1; // group보다 shape 상세값이 많은 편
  return score;
}

/** DiffKind → 짧은 한글 제목(컨트롤 추가/삭제 카드 등). UI `kindLabel`과 문구를 맞출 때 동기화 */
function kindLabel(kind: DiffKind): string {
  if (kind === 'table') return '표';
  if (kind === 'shape') return '도형';
  if (kind === 'image') return '이미지';
  if (kind === 'chart') return '그래프';
  if (kind === 'text') return '텍스트';
  return '메타';
}

/**
 * 표 셀 텍스트·속성을 요약한 짧은 문자열 — 컨트롤 diff에서 "같은 표인지" 판별용.
 * WASM에 `getTableSignature`가 있으면 그걸 우선(정확·빠름), 없으면 셀 순회 폴백.
 */
function buildTableSummary(
  wasm: WasmBridge,
  options: CompareOptions,
  sec: number,
  para: number,
  ci: number,
): string {
  let sigDigest = 'nosig';
  try {
    const sigJson = wasm.getTableSignature(sec, para, ci);
    sigDigest = simpleHash(sigJson);
  } catch {
    // 신/구 WASM 호환: 시그니처 API가 없으면 기존 JS 조합 경로 사용
  }

  const dim = wasm.getTableDimensions(sec, para, ci);
  const cellCount = Math.max(0, dim.rowCount * dim.colCount);
  const cellSnippets: string[] = [];
  const cellPreviewPairs: string[] = [];
  const cellHashPairs: string[] = [];
  for (let cellIdx = 0; cellIdx < cellCount; cellIdx += 1) {
    let paraCount = 0;
    try {
      paraCount = wasm.getCellParagraphCount(sec, para, ci, cellIdx);
    } catch {
      paraCount = 0;
    }
    const paraTexts: string[] = [];
    for (let cpi = 0; cpi < paraCount; cpi += 1) {
      try {
        const plen = wasm.getCellParagraphLength(sec, para, ci, cellIdx, cpi);
        const t = plen > 0 ? wasm.getTextInCell(sec, para, ci, cellIdx, cpi, 0, plen) : '';
        if (t) paraTexts.push(normalizeText(t, options));
      } catch {
        // 일부 셀 접근 실패 시 해당 문단만 스킵
      }
    }
    const joined = paraTexts.join('|');
    cellSnippets.push(joined);
    if (joined && cellPreviewPairs.length < 240) {
      // 요소별 변경 추적(표 텍스트):
      // - cprev: UI 표시용(사람이 읽는 셀 미리보기)
      // - csha : 긴 문장/여러 줄에서 잘림이 있어도 변경 감지를 유지하기 위한 셀 단위 해시
      const compact = joined
        .replaceAll('"', "'")
        .replaceAll('\r\n', '\n')
        .replaceAll('\n', ' ↵ ')
        .replace(/\s{2,}/g, ' ')
        .trim()
        .slice(0, 180);
      const row = dim.colCount > 0 ? Math.floor(cellIdx / dim.colCount) + 1 : 1;
      const col = dim.colCount > 0 ? (cellIdx % dim.colCount) + 1 : (cellIdx + 1);
      const key = `r${row}c${col}`;
      cellPreviewPairs.push(`${key}=${encodeURIComponent(compact)}`);
      cellHashPairs.push(`${key}=${simpleHash(joined)}`);
    }
  }
  const textDigest = simpleHash(cellSnippets.join('||'));
  const textPreview = cellSnippets
    .filter(Boolean)
    .slice(0, 2)
    .map((s) => s.replaceAll('"', "'").replace(/\s+/g, ' ').trim())
    .join(' | ')
    .slice(0, 80);
  let propsDigest = '';
  try {
    const props = wasm.getTableProperties(sec, para, ci);
    propsDigest = simpleHash(JSON.stringify(props));
  } catch {
    propsDigest = 'noprops';
  }
  let bboxDigest = 'nobox';
  try {
    const bbox = wasm.getTableBBox(sec, para, ci);
    bboxDigest = `${Math.round(bbox.width)}x${Math.round(bbox.height)}`;
  } catch {
    bboxDigest = 'nobox';
  }
  const cellPreview = cellPreviewPairs.join('&');
  const cellHash = cellHashPairs.join('&');
  return `table r=${dim.rowCount} c=${dim.colCount} tprev="${textPreview || '(없음)'}" cprev="${cellPreview || '(없음)'}" csha="${cellHash || '(없음)'}" txt=${textDigest} props=${propsDigest} box=${bboxDigest} sig=${sigDigest}`;
}

// ─── 표 요약 파싱·셀 단위 변경 집계 (buildGranularControlDiffs에서 사용) ───────

/** `sid:…:ci:type` / `loc:…:ci:type` 형태에서 컨트롤 인덱스 `ci`만 뽑아 폴백 점수에 사용 */
function extractControlIndexFromKey(key: string): number | null {
  const m = key.match(/:(\d+):[^:]+$/);
  if (!m) return null;
  return Number.isFinite(Number(m[1])) ? Number(m[1]) : null;
}

/** 스냅샷 문단·컨트롤의 `section`+`paragraph`를 하나의 맵 키로 묶을 때 사용한다. */
function paraPosKey(p: { section: number; paragraph: number }): string {
  return `${p.section}:${p.paragraph}`;
}

/**
 * 문단 정렬 결과(`AlignedPair[]`, cleanup 이전)에서 오른쪽 문단 위치 → 짝 왼쪽 문단을 추출한다.
 * - 키: `paraPosKey(right)` (오른쪽 HWP 좌표계).
 * - 값: DP/그리디가 같은 논리 슬롯으로 판단한 `left` 문단.
 * `(null, right)`·`(left, null)` 삽입/삭제 축은 맵에 넣지 않는다 → 해당 문단의 컨트롤은 슬롯 매칭 대상에서 제외되고
 * 이후 `extractTablePatiencePins`(선행) → `pairAlignmentSlotControls` → `pairControlsFallback` / added·removed로 처리된다.
 */
function buildRightToLeftParaMapFromAligned(aligned: AlignedPair[]): Map<string, CompareParaSnapshot> {
  const m = new Map<string, CompareParaSnapshot>();
  for (const { left, right } of aligned) {
    if (!left || !right) continue;
    m.set(paraPosKey(right), left);
  }
  return m;
}

/** `buildTableSummary` 등이 만든 `key=value` 나열을 Record로 파싱. 값에 따옴표가 있으면 제거한다. */
function parseSummaryKV(summary: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const m of summary.matchAll(/([a-z]+)=("([^"]*)"|[^\s]+)/g)) {
    const raw = m[2] ?? '';
    out[m[1]] = raw.startsWith('"') && raw.endsWith('"') ? raw.slice(1, -1) : raw;
  }
  return out;
}

/**
 * `csha="r1c1=…&r2c2=…"` 형태(셀별 해시)를 키 단위로 비교해 변경된 셀 개수를 센다.
 * 행/열 구조가 바뀌어 키 집합이 달라져도 union 키 기준으로 added/removed 셀을 모두 잡는다.
 */
function countChangedCellsByHash(leftHash: string, rightHash: string): number {
  const parse = (v: string): Map<string, string> => {
    const out = new Map<string, string>();
    if (!v || v === '(없음)') return out;
    for (const pair of v.split('&')) {
      const i = pair.indexOf('=');
      if (i <= 0) continue;
      const key = pair.slice(0, i);
      const value = pair.slice(i + 1);
      out.set(key, value);
    }
    return out;
  };
  const l = parse(leftHash);
  const r = parse(rightHash);
  const keys = new Set<string>([...l.keys(), ...r.keys()]);
  let changed = 0;
  for (const k of keys) {
    if ((l.get(k) ?? '') !== (r.get(k) ?? '')) changed += 1;
  }
  return changed;
}

/**
 * 동일 컨트롤 쌍(l,r)에 대해 가능한 한 잘게 쪼갠 DiffItem[]을 만든다.
 * - 표: 행/열·크기·셀 텍스트(cprev/csha)·속성(props)을 분리해 카드가 비지 않게 한다.
 * - 이미지/도형: 크기·텍스트·자르기·효과 등 필드별 push.
 * - `push` 내부에서 좌우 문자열이 같으면(강제 제외 아닌 이상) 항목을 생략한다.
 */
function buildGranularControlDiffs(
  l: CompareControlSnapshot,
  r: CompareControlSnapshot,
  idStem: string,
): DiffItem[] {
  const lk = parseSummaryKV(l.summary);
  const rk = parseSummaryKV(r.summary);
  const items: DiffItem[] = [];
  const label = kindLabel(l.kind);
  const push = (suffix: string, title: string, leftPreview: string, rightPreview: string, force = false) => {
    if (!force && leftPreview === rightPreview) return;
    items.push({
      id: mkDiffId(l.kind, `${idStem}:${suffix}`),
      kind: l.kind,
      severity: 'modified',
      path: { section: r.section, paragraph: r.paragraph, controlKey: r.key },
      title,
      leftPreview,
      rightPreview,
      leftAnchor: l.anchor,
      rightAnchor: r.anchor,
    });
  };

  if (l.type === 'table' && r.type === 'table') {
    const hasCellText =
      (lk.cprev && lk.cprev !== '(없음)') ||
      (rk.cprev && rk.cprev !== '(없음)') ||
      (lk.tprev && lk.tprev !== '(없음)') ||
      (rk.tprev && rk.tprev !== '(없음)');
    const tableLabel = hasCellText ? '표' : '테이블';
    const rowsColsChanged = (lk.r ?? '') !== (rk.r ?? '') || (lk.c ?? '') !== (rk.c ?? '');
    push('rows-cols', `${tableLabel} 행/열 변경`, `r=${lk.r ?? '(없음)'} c=${lk.c ?? '(없음)'}`, `r=${rk.r ?? '(없음)'} c=${rk.c ?? '(없음)'}`);
    push('size', `${tableLabel} 크기 변경`, `box=${lk.box ?? '(없음)'}`, `box=${rk.box ?? '(없음)'}`);
    // UI의 행/열별 셀 비교는 cprev(r1c1=...&r1c2=...) 포맷을 기준으로 동작한다.
    // cprev가 없을 때만 tprev로 폴백한다.
    const lText = `cprev="${lk.cprev ?? lk.tprev ?? '(없음)'}"`;
    const rText = `cprev="${rk.cprev ?? rk.tprev ?? '(없음)'}"`;
    const tableTextChanged = (lk.txt ?? '') !== (rk.txt ?? '') || lText !== rText;
    const changedCells = countChangedCellsByHash(lk.csha ?? '', rk.csha ?? '');
    const textTitle = rowsColsChanged
      ? `${tableLabel} 텍스트 변경(구조변경 동반${changedCells > 0 ? `, ${changedCells}셀` : ''})`
      : `${tableLabel} 텍스트 변경${changedCells > 0 ? `(${changedCells}셀)` : ''}`;
    push('text', textTitle, lText, rText, tableTextChanged);
    // props= 는 `getTableProperties` 전체 JSON 해시라 조판·저장 경로만 달라도 달라져 노이즈가 크다. UI에는 내리지 않는다.
    return items;
  }

  if (l.type === 'image' && r.type === 'image') {
    push('size', '그림 크기 변경', `box=${lk.box ?? '(없음)'}`, `box=${rk.box ?? '(없음)'}`);
    const imageTextChanged = (lk.text ?? '') !== (rk.text ?? '') || ((lk.pix ?? '') !== (rk.pix ?? '') && (lk.text ?? '') === (rk.text ?? ''));
    push('text', '그림 텍스트 변경', `text="${lk.text ?? '(없음)'}" pix=${lk.pix ?? '(없음)'}`, `text="${rk.text ?? '(없음)'}" pix=${rk.pix ?? '(없음)'}`, imageTextChanged);
    push('crop', '그림 자르기 변경', `crop=${lk.crop ?? '(없음)'}`, `crop=${rk.crop ?? '(없음)'}`);
    push('effect', '그림 효과 변경', `effect=${lk.effect ?? '(없음)'} bc=${lk.bc ?? '(없음)'}`, `effect=${rk.effect ?? '(없음)'} bc=${rk.bc ?? '(없음)'}`);
    return items;
  }

  if ((l.type === 'shape' || l.type === 'group') && (r.type === 'shape' || r.type === 'group')) {
    push('size', `${label} 크기 변경`, `box=${lk.box ?? '(없음)'}`, `box=${rk.box ?? '(없음)'}`);
    const shapeTextChanged = (lk.text ?? '') !== (rk.text ?? '') || ((lk.pix ?? '') !== (rk.pix ?? '') && (lk.text ?? '') === (rk.text ?? ''));
    push('text', `${label} 텍스트 변경`, `text="${lk.text ?? '(없음)'}" pix=${lk.pix ?? '(없음)'}`, `text="${rk.text ?? '(없음)'}" pix=${rk.pix ?? '(없음)'}`, shapeTextChanged);
    push('rotate', `${label} 회전/대칭 변경`, `rot=${lk.rot ?? '(없음)'} flip=${lk.flip ?? '(없음)'}`, `rot=${rk.rot ?? '(없음)'} flip=${rk.flip ?? '(없음)'}`);
    push('layout', `${label} 배치 변경`, `wrap=${lk.wrap ?? '(없음)'} rel=${lk.rel ?? '(없음)'}`, `wrap=${rk.wrap ?? '(없음)'} rel=${rk.rel ?? '(없음)'}`);
    return items;
  }

  push('generic', `${label} 속성 변경`, l.summary, r.summary);
  return items;
}

// ─── 스냅샷 수집: WASM 단일 문서 → CompareDocumentSnapshot ─────────────────────

/** 앵커용 문단 모양 요약 — 텍스트만 같고 번호/정렬이 다른 문단을 구분한다. */
function compactParaShapeForAnchor(pp: ParaProperties): string {
  const q = (v: number | undefined) => (v == null || Number.isNaN(v) ? 0 : Math.round(v / 2) * 2);
  return JSON.stringify({
    a: pp.alignment ?? '',
    h: pp.headType ?? '',
    lv: pp.paraLevel ?? 0,
    n: pp.numberingId ?? 0,
    i: q(pp.indent),
    ml: q(pp.marginLeft),
  });
}

/**
 * `getCursorRect(sec,para,0)`이 실패하는 문단(표 셀 내부·빈 줄 등)에 대해 오프셋을 바꿔 재시도한다.
 */
function tryResolveCompareParaAnchorFromCursor(
  wasm: WasmBridge,
  sec: number,
  para: number,
  textLength: number,
): DiffAnchor | undefined {
  const offsets = new Set<number>([0]);
  if (textLength > 0) {
    offsets.add(1);
    offsets.add(Math.max(0, textLength - 1));
    if (textLength > 2) offsets.add(Math.floor(textLength / 2));
  }
  for (const off of offsets) {
    try {
      const rect = wasm.getCursorRect(sec, para, off);
      if (rect && typeof rect.pageIndex === 'number' && Number.isFinite(rect.x) && Number.isFinite(rect.y)) {
        return {
          pageIndex: rect.pageIndex,
          x: rect.x,
          y: rect.y,
          width: 320,
          height: Math.max(18, rect.height || 18),
        };
      }
    } catch {
      /* 다음 오프셋 */
    }
  }
  return undefined;
}

/**
 * 커서 rect를 못 얻은 문단에 대해, 해당 문단에 붙은 첫 레이아웃 개체 박스로 앵커를 채운다(비교 상세 캔버스용).
 */
function fillMissingParaAnchorsFromPageLayout(
  wasm: WasmBridge,
  info: DocumentInfo,
  paragraphs: CompareParaSnapshot[],
  displayedPageByGlobalPage: Map<number, number>,
): void {
  const firstBoxByPara = new Map<string, DiffAnchor>();
  for (let page = 0; page < info.pageCount; page += 1) {
    let controls: ControlLayoutItem[];
    try {
      controls = wasm.getPageControlLayout(page).controls;
    } catch {
      continue;
    }
    for (const item of controls) {
      const sec = item.secIdx;
      const pIdx = item.paraIdx;
      if (sec == null || pIdx == null || sec < 0 || pIdx < 0) continue;
      const key = `${sec}:${pIdx}`;
      if (firstBoxByPara.has(key)) continue;
      firstBoxByPara.set(key, {
        pageIndex: page,
        x: item.x,
        y: item.y,
        width: Math.max(48, Math.round(item.w)),
        height: Math.max(18, Math.round(item.h)),
      });
    }
  }
  for (const p of paragraphs) {
    if (p.anchor) continue;
    const fb = firstBoxByPara.get(`${p.section}:${p.paragraph}`);
    if (!fb) continue;
    p.anchor = fb;
    p.sectionPage = displayedPageByGlobalPage.get(fb.pageIndex) ?? (fb.pageIndex + 1);
  }
}

/**
 * 레이아웃에도 없으면 같은 구역에서 가장 가까운 앵커가 있는 문단 좌표를 복사한다.
 * 이전 문단 앵커를 **그대로** 쓰면(표가 있는 문단 등) 비교 상세 마커가 표 위에 겹쳐 “문단 추가 = 표”로 보이므로 세로로 한 칸 밀어 구분한다.
 */
function fillMissingParaAnchorsFromNeighbors(
  paragraphs: CompareParaSnapshot[],
  displayedPageByGlobalPage: Map<number, number>,
): void {
  for (const p of paragraphs) {
    if (p.anchor) continue;
    let nbr: DiffAnchor | undefined;
    let fromPreviousPara = false;
    for (let gi = p.globalIndex - 1; gi >= 0; gi -= 1) {
      const q = paragraphs[gi];
      if (q.section !== p.section) break;
      if (q.anchor) {
        nbr = q.anchor;
        fromPreviousPara = true;
        break;
      }
    }
    if (!nbr) {
      for (let gi = p.globalIndex + 1; gi < paragraphs.length; gi += 1) {
        const q = paragraphs[gi];
        if (q.section !== p.section) break;
        if (q.anchor) {
          nbr = q.anchor;
          fromPreviousPara = false;
          break;
        }
      }
    }
    if (!nbr) continue;
    if (fromPreviousPara) {
      const dy = Math.min(200, Math.max(28, Math.round(nbr.height) + 10));
      p.anchor = {
        ...nbr,
        y: nbr.y + dy,
        width: nbr.width,
        height: Math.max(18, nbr.height),
      };
    } else {
      p.anchor = {
        ...nbr,
        y: Math.max(0, nbr.y - 28),
        width: nbr.width,
        height: Math.max(18, nbr.height),
      };
    }
    p.sectionPage = displayedPageByGlobalPage.get(nbr.pageIndex) ?? (nbr.pageIndex + 1);
  }
}

/**
 * WASM이 열린 단일 문서에서 비교용 스냅샷을 만든다.
 * - 문단: 텍스트, 정규화 텍스트, stable_id, 레이아웃 앵커(커서 rect·다중 오프셋 → 페이지 개체 박스 → 이웃 문단), 구역 내 쪽번호,
 *   `signature`(정규화 텍스트·컨트롤 개수 + `getParaPropertiesAt` 기반 문단모양 요약 `ps:`)
 * - 개체: 페이지 레이아웃 + 문단별 table 순회를 합쳐 키를 `sid:` 우선으로 통일
 * - 마지막에 `canonicalControlKey`로 중복을 합치고 `controlSnapshotQuality`로 더 나은 요약을 남긴다.
 */
function fillSnapshotFromWasm(
  wasm: WasmBridge,
  info: DocumentInfo,
  displayName: string,
  options: CompareOptions,
): CompareDocumentSnapshot {
  // 비교 스냅샷 직전에 강제 재조판하여 폰트/도형 반영 지연으로 인한 페이지 밀림을 줄인다.
  wasm.refreshLayout();

  const displayedPageByGlobalPage = new Map<number, number>();
  for (let page = 0; page < info.pageCount; page += 1) {
    try {
      const pi = wasm.getPageInfo(page);
      displayedPageByGlobalPage.set(page, pi.pageNumber ?? (page + 1));
    } catch {
      // page info 조회 실패 시 후속 fallback 사용
    }
  }

  const paragraphs: CompareParaSnapshot[] = [];
  /** 구역·문단별: 글머리/번호/개요 등 HWP `headType`이 Outline·Number인지(짧은 제목 앵커 후보용) */
  const paraHeadOutlineOrNumber = new Map<string, boolean>();
  let globalIndex = 0;
  for (let sec = 0; sec < info.sectionCount; sec += 1) {
    const paraCount = wasm.getParagraphCount(sec);
    for (let para = 0; para < paraCount; para += 1) {
      const length = wasm.getParagraphLength(sec, para);
      const text = length > 0 ? wasm.getTextRange(sec, para, 0, length) : '';
      const controls = wasm.getControlTextPositions(sec, para);
      const normalizedText = normalizeText(text, options);
      let shapeDigest = '';
      try {
        const pp = wasm.getParaPropertiesAt(sec, para);
        shapeDigest = simpleHash(compactParaShapeForAnchor(pp));
        const ht = pp.headType ?? 'None';
        paraHeadOutlineOrNumber.set(`${sec}:${para}`, ht === 'Outline' || ht === 'Number');
      } catch {
        shapeDigest = '';
        paraHeadOutlineOrNumber.set(`${sec}:${para}`, false);
      }
      // 문단 "내용+컨트롤 개수+문단모양" 지문 — alignment에서 앵커/유일쌍 찾기에 사용
      const signature = simpleHash(
        `${normalizedText}|cc:${controls.length}${shapeDigest ? `|ps:${shapeDigest}` : ''}`,
      );
      const stableId = wasm.getParagraphStableId(sec, para);
      const anchor = tryResolveCompareParaAnchorFromCursor(wasm, sec, para, length);
      const sectionPage = (() => {
        if (!anchor) return 1;
        return displayedPageByGlobalPage.get(anchor.pageIndex) ?? (anchor.pageIndex + 1);
      })();
      paragraphs.push({
        section: sec,
        paragraph: para,
        sectionPage,
        globalIndex,
        stableId,
        text,
        normalizedText,
        controlCount: controls.length,
        signature,
        isAnchorCandidate: false,
        anchor,
      });
      globalIndex += 1;
    }
  }

  fillMissingParaAnchorsFromPageLayout(wasm, info, paragraphs, displayedPageByGlobalPage);
  fillMissingParaAnchorsFromNeighbors(paragraphs, displayedPageByGlobalPage);

  // 글로벌 앵커 후보(`isAnchorCandidate`): 시그니처 중복은 즉시 탈락. 그 다음
  // - 길이·엔트로피 등으로 “긴” 문단은 `isAnchorTextQualityOk`
  // - 짧은 제목 줄은 `[…]` / 번호 목록 패턴(`isStructuralBlockTitleLine`) 또는 HWP Outline/Number 머리
  // `buildAnchorPairs` 단계에서 짧은 후보는 `buildBothSidesUniqueTrimNorm`으로 양쪽 각 1회만 추가 검증.
  const anchorTuning = resolveAnchorTuning(options);
  const sigCount = new Map<string, number>();
  for (const p of paragraphs) {
    sigCount.set(p.signature, (sigCount.get(p.signature) ?? 0) + 1);
  }
  for (const p of paragraphs) {
    const isDuplicate = (sigCount.get(p.signature) ?? 0) > 1;
    if (isDuplicate) {
      p.isAnchorCandidate = false;
      continue;
    }
    const t = p.normalizedText.trim();
    const headOutlineOrNumber = paraHeadOutlineOrNumber.get(`${p.section}:${p.paragraph}`) ?? false;
    const structBracket =
      t.length >= STRUCTURAL_ANCHOR_MIN_LEN &&
      t.length < anchorTuning.minTextLen &&
      isStructuralBlockTitleLine(t);
    const structHeading =
      headOutlineOrNumber && t.length >= 3 && t.length < anchorTuning.minTextLen;
    const okNormal = isAnchorTextQualityOk(p.normalizedText, anchorTuning);
    p.isAnchorCandidate = okNormal || structBracket || structHeading;
  }
  const paraStableByPos = new Map<string, string>();
  const paraTextByPos = new Map<string, string>();
  for (const p of paragraphs) {
    paraStableByPos.set(`${p.section}:${p.paragraph}`, p.stableId);
    paraTextByPos.set(`${p.section}:${p.paragraph}`, p.normalizedText.slice(0, 48));
  }

  const controls: CompareControlSnapshot[] = [];
  const shapeDebugRows: Array<{
    source: 'layout' | 'direct';
    sec: number;
    para: number;
    ci: number;
    type: string;
    shapeTextLen: number;
    usedDescription: boolean;
    usedParaFallback: boolean;
    hasPix: boolean;
  }> = [];
  for (let page = 0; page < info.pageCount; page += 1) {
    const layout = wasm.getPageControlLayout(page);
    for (const item of layout.controls) {
      const sec = item.secIdx ?? -1;
      const para = item.paraIdx ?? -1;
      const ci = item.controlIdx ?? -1;
      const paraStableId = sec >= 0 && para >= 0 ? paraStableByPos.get(`${sec}:${para}`) : undefined;
      const key = paraStableId
        ? `sid:${paraStableId}:${ci}:${item.type}`
        : `loc:${sec}:${para}:${ci}:${item.type}`;
      let summary = `${item.type} ${Math.round(item.w)}x${Math.round(item.h)}`;

      try {
        if (item.type === 'table' && sec >= 0 && para >= 0 && ci >= 0) {
          summary = buildTableSummary(wasm, options, sec, para, ci);
        } else if (item.type === 'image' && sec >= 0 && para >= 0 && ci >= 0) {
          const pic = wasm.getPictureProperties(sec, para, ci);
          const w = Math.round(pic.width);
          const h = Math.round(pic.height);
          const crop = [pic.cropLeft, pic.cropTop, pic.cropRight, pic.cropBottom].map((v) => Math.round(v)).join(',');
          const effect = `${pic.effect}:${pic.rotationAngle ?? 0}`;
          const bc = `${pic.brightness ?? 0}/${pic.contrast ?? 0}`;
          const paraText = paraTextByPos.get(`${sec}:${para}`) ?? '';
          const desc = ((pic.description ?? '').trim() || paraText).replaceAll('"', "'").slice(0, 48);
          let pix = 'nopix';
          try {
            const raw = wasm.getControlImageData(sec, para, ci);
            pix = simpleHashBytes(raw);
          } catch {
            pix = 'nopix';
          }
          summary = `image box=${w}x${h} crop=${crop} effect=${effect} bc=${bc} text="${desc || '(없음)'}" pix=${pix}`;
        } else if ((item.type === 'shape' || item.type === 'group') && sec >= 0 && para >= 0 && ci >= 0) {
          const props = wasm.getShapeProperties(sec, para, ci);
          const box = `${Math.round(props.width)}x${Math.round(props.height)}`;
          const rot = Math.round(props.rotationAngle ?? 0);
          const flip = `${props.horzFlip ? 1 : 0}${props.vertFlip ? 1 : 0}`;
          const wrap = props.textWrap ?? 'none';
          const rel = `${props.horzRelTo ?? '-'}:${props.vertRelTo ?? '-'}`;
          const paraText = paraTextByPos.get(`${sec}:${para}`) ?? '';
          let shapeText = '';
          try {
            const st = wasm.getShapeText(sec, para, ci);
            if (st.ok && st.text) shapeText = st.text;
          } catch {
            shapeText = '';
          }
          const desc = (shapeText.trim() || (props.description ?? '').trim() || paraText).replaceAll('"', "'").slice(0, 120);
          const usedDescription = !shapeText.trim() && Boolean((props.description ?? '').trim());
          const usedParaFallback = !shapeText.trim() && !(props.description ?? '').trim() && Boolean(paraText.trim());
          let pix = 'nopix';
          try {
            const raw = wasm.getControlImageData(sec, para, ci);
            pix = simpleHashBytes(raw);
          } catch {
            pix = 'nopix';
          }
          summary = `shape box=${box} rot=${rot} flip=${flip} wrap=${wrap} rel=${rel} text="${desc || '(없음)'}" pix=${pix}`;
          shapeDebugRows.push({
            source: 'layout',
            sec,
            para,
            ci,
            type: item.type,
            shapeTextLen: shapeText.trim().length,
            usedDescription,
            usedParaFallback,
            hasPix: pix !== 'nopix',
          });
        }
      } catch {
        // 일부 개체 타입은 속성 조회 API가 제한될 수 있다.
      }

      controls.push({
        key,
        type: item.type,
        section: sec,
        paragraph: para,
        summary,
        kind: mapControlKind(item.type, summary),
        anchor: {
          pageIndex: page,
          x: item.x,
          y: item.y,
          width: Math.max(12, item.w),
          height: Math.max(12, item.h),
        },
      });
    }
  }

  // RTMS 계열에서 유효했던 방식과 동일한 취지:
  // 페이지 레이아웃 매핑(item.secIdx/paraIdx)에만 의존하지 말고,
  // 문단의 컨트롤 인덱스를 직접 순회하며 table/shape/image를 식별/요약한다.
  for (const p of paragraphs) {
    const controlCount = wasm.getControlTextPositions(p.section, p.paragraph).length;
    for (let ci = 0; ci < controlCount; ci += 1) {
      try {
        const summary = buildTableSummary(wasm, options, p.section, p.paragraph, ci);
        const key = p.stableId
          ? `sid:${p.stableId}:${ci}:table`
          : `loc:${p.section}:${p.paragraph}:${ci}:table`;
        let anchor = p.anchor ?? { pageIndex: 0, x: 0, y: 0, width: 12, height: 12 };
        try {
          const bbox = wasm.getTableBBox(p.section, p.paragraph, ci);
          anchor = {
            pageIndex: bbox.pageIndex,
            x: bbox.x,
            y: bbox.y,
            width: Math.max(12, bbox.width),
            height: Math.max(12, bbox.height),
          };
        } catch {
          // bbox 조회 실패 시 문단 anchor 유지
        }
        controls.push({
          key,
          type: 'table',
          section: p.section,
          paragraph: p.paragraph,
          summary,
          kind: 'table',
          anchor,
        });
      } catch {
        // getTableDimensions 실패 => table이 아닌 컨트롤
      }

      // 요소별 변경 추적 정책(도형/그림):
      // - shape/image는 layout 경로만 사용해 오매핑을 최소화한다.
      // - direct 경로는 table 전용으로 제한한다.
    }
  }

  const uniqueControls = new Map<string, CompareControlSnapshot>();
  for (const c of controls) {
    const ck = canonicalControlKey(c);
    const prev = uniqueControls.get(ck);
    if (!prev) {
      uniqueControls.set(ck, c);
      continue;
    }
    // 같은 개체로 판정되면 더 품질 높은 스냅샷으로 교체
    if (controlSnapshotQuality(c) > controlSnapshotQuality(prev)) {
      uniqueControls.set(ck, c);
    }
  }

  if (isCompareDebugEnabled()) {
    const rows = shapeDebugRows;
    const bySource = {
      layout: rows.filter((r) => r.source === 'layout').length,
      direct: rows.filter((r) => r.source === 'direct').length,
    };
    const withShapeText = rows.filter((r) => r.shapeTextLen > 0).length;
    const withDescription = rows.filter((r) => r.usedDescription).length;
    const withParaFallback = rows.filter((r) => r.usedParaFallback).length;
    const withPix = rows.filter((r) => r.hasPix).length;
    compareDbg('[shape-text-debug] 수집 요약', {
      total: rows.length,
      bySource,
      withShapeText,
      withDescription,
      withParaFallback,
      withPix,
    });
    compareDbg(
      '[shape-text-debug] 샘플(최대 20)',
      rows.slice(0, 20).map((r) => ({
        src: r.source,
        sec: r.sec,
        para: r.para,
        ci: r.ci,
        type: r.type,
        shapeTextLen: r.shapeTextLen,
        desc: r.usedDescription,
        paraFallback: r.usedParaFallback,
        pix: r.hasPix,
      })),
    );
    compareDbg(
      '[shape-text-debug] 미추출 대상(shapeTextLen=0)',
      rows
        .filter((r) => r.shapeTextLen === 0)
        .map((r) => ({
          src: r.source,
          sec: r.sec,
          para: r.para,
          ci: r.ci,
          type: r.type,
          desc: r.usedDescription,
          paraFallback: r.usedParaFallback,
          pix: r.hasPix,
        })),
    );
  }

  return {
    meta: {
      name: displayName,
      sectionCount: info.sectionCount,
      pageCount: info.pageCount,
      pageDisplayNumbers: Array.from({ length: info.pageCount }, (_, pageIndex) =>
        displayedPageByGlobalPage.get(pageIndex) ?? (pageIndex + 1),
      ),
    },
    paragraphs,
    controls: [...uniqueControls.values()],
  };
}

// ─── 스냅샷 빌더 export (bytes vs 편집기 WASM) ─────────────────────────────────

/** 디스크/외부 바이트 → 별도 WASM 인스턴스로 파싱 (stable_id는 이 인스턴스 세션 기준) */
export async function buildSnapshotFromBytes(
  bytes: Uint8Array,
  fileName: string,
  options: CompareOptions,
): Promise<CompareDocumentSnapshot> {
  const wasm = new WasmBridge();
  await wasm.initialize();
  const info = wasm.loadDocument(bytes, fileName);
  return fillSnapshotFromWasm(wasm, info, fileName, options);
}

/** 편집기에 올라온 문서 그대로 스냅샷 — 이력 비교 시 stable_id 유지 */
export function buildSnapshotFromWasm(
  wasm: WasmBridge,
  displayName: string,
  options: CompareOptions,
): CompareDocumentSnapshot {
  const info = wasm.getDocumentInfo();
  return fillSnapshotFromWasm(wasm, info, displayName, options);
}

/**
 * IR `stable_id` → 문단 스냅샷. 값이 비어 있으면 identity 경로를 쓸 수 없어 null.
 * 동일 stable_id가 여러 번 나오면 `#0,#1,…` occurrence suffix로 키를 유일화한다(빈 문단 다수 문서).
 */
function buildStableIdMap(snap: CompareDocumentSnapshot): Map<string, CompareParaSnapshot> | null {
  const m = new Map<string, CompareParaSnapshot>();
  const seen = new Map<string, number>();
  for (const p of snap.paragraphs) {
    if (!p.stableId) return null;
    // fallback stable_id가 중복되는 문서(빈 문단 다수 등)도 identity를 사용하기 위해
    // 등장 순서 기반 suffix로 키를 정규화한다.
    const occ = seen.get(p.stableId) ?? 0;
    seen.set(p.stableId, occ + 1);
    const key = occ === 0 ? p.stableId : `${p.stableId}#${occ}`;
    m.set(key, p);
  }
  return m;
}

/** ZWSP·NBSP 등만 남은 문단도 “빈 문단”으로 본다. */
function isEffectivelyEmptyParaNormalized(normalizedText: string): boolean {
  const t = normalizedText
    .replace(/[\u200b-\u200d\ufeff]/g, '')
    .replace(/\u00a0/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  return t.length === 0;
}

/** 텍스트·개체 없는 문단: 정렬/스냅샷 노이즈로 추가·삭제만 남는 경우 diff에서 생략한다. */
function shouldSuppressNoiseParagraphOnly(p: CompareParaSnapshot): boolean {
  return p.controlCount === 0 && isEffectivelyEmptyParaNormalized(p.normalizedText);
}

/** 정렬 스텝에서 빈 고아 문단만 제거해 cleanup 2글자 패턴이 깨지지 않게 한다. */
function stripNoiseOnlyParagraphAlignSteps(steps: ParagraphAlignStep[]): ParagraphAlignStep[] {
  return steps.filter((s) => {
    if (s.kind === 'r') return !shouldSuppressNoiseParagraphOnly(s.r);
    if (s.kind === 'l') return !shouldSuppressNoiseParagraphOnly(s.l);
    return true;
  });
}

function formatParaLocTitle(p: { section: number; paragraph: number }): string {
  return `구역 ${p.section}, 문단 ${p.paragraph}`;
}

// ─── identity 경로: 이력(동일 혈통)에서 stable_id 기준 O(N) 근사 텍스트 diff ───

/**
 * identity 텍스트 비교: stable_id 정규화 키로 좌/우 문단을 1:1 매칭.
 * - 양쪽 중 한쪽에만 있으면 added/removed
 * - 둘 다 있으면 normalizedText 불일치 시 modified + 문자 요약
 * - kinds에 paragraphMeta가 있으면 이동/컨트롤 수 변화도 별도 항목으로 낸다
 */
function buildIdentityTextDiffs(left: CompareDocumentSnapshot, right: CompareDocumentSnapshot, kinds: DiffKind[]): DiffItem[] {
  const lmap = buildStableIdMap(left);
  const rmap = buildStableIdMap(right);
  if (!lmap || !rmap) return [];

  // 양쪽 Map의 key(stableId)를 합집합으로 만든다.
  // -> 어떤 id든 최소 1회는 순회되므로 "한쪽만 존재(추가/삭제)"도 놓치지 않는다.
  const keys = [...new Set([...lmap.keys(), ...rmap.keys()])];
  keys.sort((a, b) => {
    const la = lmap.get(a);
    const lb = lmap.get(b);
    const ra = rmap.get(a);
    const rb = rmap.get(b);
    const sa = la?.section ?? ra?.section ?? 0;
    const sb = lb?.section ?? rb?.section ?? 0;
    if (sa !== sb) return sa - sb;
    const pa = la?.paragraph ?? ra?.paragraph ?? 0;
    const pb = lb?.paragraph ?? rb?.paragraph ?? 0;
    return pa - pb;
  });

  const diffs: DiffItem[] = [];
  for (const id of keys) {
    // 동일 id를 좌/우 Map에서 각각 조회:
    // - l만 있으면: 기준 문서에는 있었는데 비교 문서에는 없음 => removed
    // - r만 있으면: 비교 문서에 새로 생김 => added
    // - 둘 다 있으면: 내용(normalizedText) 비교로 modified 여부 판단
    // 조회 자체는 Map#get 이라 평균 O(1), 전체는 key 개수에 비례해 O(N)으로 동작한다.
    const l = lmap.get(id);
    const r = rmap.get(id);
    if (l && !r) {
      if (!shouldSuppressNoiseParagraphOnly(l)) {
        diffs.push({
          id: mkDiffId('text', `id-removed:${id}`),
          kind: 'text',
          severity: 'removed',
          path: { section: l.section, paragraph: l.paragraph },
          title: '문단 삭제',
          leftPreview: l.text,
          rightPreview: '',
          leftAnchor: l.anchor,
        });
      }
      continue;
    }
    if (!l && r) {
      if (!shouldSuppressNoiseParagraphOnly(r)) {
        diffs.push({
          id: mkDiffId('text', `id-added:${id}`),
          kind: 'text',
          severity: 'added',
          path: { section: r.section, paragraph: r.paragraph },
          title: '문단 추가',
          leftPreview: '',
          rightPreview: r.text,
          rightAnchor: r.anchor,
        });
      }
      continue;
    }
    if (!l || !r) continue;

    if (l.normalizedText !== r.normalizedText) {
      diffs.push({
        id: mkDiffId('text', `id-modified:${id}`),
        kind: 'text',
        severity: 'modified',
        path: preferRightPath(l, r),
        title: '텍스트 변경',
        leftPreview: l.text,
        rightPreview: r.text,
        leftAnchor: l.anchor,
        rightAnchor: r.anchor,
        inlineTextDiff: myersCharDiffSummary(l.text, r.text),
      });
    }

    if (
      kinds.includes('paragraphMeta') &&
      l.signature === r.signature &&
      Math.abs(l.globalIndex - r.globalIndex) > MOVE_DISTANCE_THRESHOLD
    ) {
      diffs.push({
        id: mkDiffId('paragraphMeta', `id-moved:${id}`),
        kind: 'paragraphMeta',
        severity: 'modified',
        path: preferRightPath(l, r),
        title: '문단 순서 이동',
        leftPreview: `A idx=${l.globalIndex}`,
        rightPreview: `B idx=${r.globalIndex}`,
        leftAnchor: l.anchor,
        rightAnchor: r.anchor,
      });
    }

    if (kinds.includes('paragraphMeta') && l.controlCount !== r.controlCount) {
      diffs.push({
        id: mkDiffId('paragraphMeta', `id-ctrlcount:${id}`),
        kind: 'paragraphMeta',
        severity: 'modified',
        path: preferRightPath(l, r),
        title: '문단 개체 수 변경',
        leftPreview: `controls=${l.controlCount}`,
        rightPreview: `controls=${r.controlCount}`,
        leftAnchor: l.anchor,
        rightAnchor: r.anchor,
      });
    }
  }
  return diffs;
}

// ─── 전략 선택·순수 reflow 에 대한 moved 메타 제거 ───────────────────────────

/**
 * 호출부 `options.strategy`와 실제 가능 여부를 교차 검사한다.
 * `identity`를 요청해도 양쪽 `buildStableIdMap`이 null이면 alignment로 내려가 변동성이 커질 수 있다.
 */
function resolveTextCompareStrategy(
  strategy: CompareStrategy,
  left: CompareDocumentSnapshot,
  right: CompareDocumentSnapshot,
): 'identity' | 'alignment' {
  const canId = Boolean(buildStableIdMap(left) && buildStableIdMap(right));
  if (strategy === 'identity') return canId ? 'identity' : 'alignment';
  return 'alignment';
}

/** `suppressPureReflowMoves`가 제거 대상으로 삼는 paragraphMeta 이동 항목만 true */
function isParagraphMoveMeta(diff: DiffItem): boolean {
  return diff.kind === 'paragraphMeta' && (diff.id.includes('moved:') || diff.id.includes('id-moved:'));
}

/**
 * 삽입/삭제로 globalIndex만 밀린 것과 "진짜 순서 바뀜"을 구분해 moved 노이즈를 제거한다.
 * (공유 sid의 상대 순서가 유지되고, 밀림량이 앞쪽 순수 추가/삭제 개수와 일치하면 제외)
 */
function suppressPureReflowMoves(
  diffs: DiffItem[],
  left: CompareDocumentSnapshot,
  right: CompareDocumentSnapshot,
): DiffItem[] {
  const lmap = buildStableIdMap(left);
  const rmap = buildStableIdMap(right);
  if (!lmap || !rmap) return diffs;

  const sharedLeftIds = left.paragraphs.map((p) => p.stableId).filter((id) => id && rmap.has(id));
  const sharedRightIds = right.paragraphs.map((p) => p.stableId).filter((id) => id && lmap.has(id));
  const rankLeft = new Map<string, number>();
  const rankRight = new Map<string, number>();
  sharedLeftIds.forEach((id, i) => rankLeft.set(id, i));
  sharedRightIds.forEach((id, i) => rankRight.set(id, i));

  const rightOnlyPrefixCount: number[] = Array(right.paragraphs.length + 1).fill(0);
  for (let i = 0; i < right.paragraphs.length; i += 1) {
    rightOnlyPrefixCount[i + 1] = rightOnlyPrefixCount[i] + (lmap.has(right.paragraphs[i].stableId) ? 0 : 1);
  }
  const leftOnlyPrefixCount: number[] = Array(left.paragraphs.length + 1).fill(0);
  for (let i = 0; i < left.paragraphs.length; i += 1) {
    leftOnlyPrefixCount[i + 1] = leftOnlyPrefixCount[i] + (rmap.has(left.paragraphs[i].stableId) ? 0 : 1);
  }

  return diffs.filter((d) => {
    if (!isParagraphMoveMeta(d)) return true;
    let l: CompareParaSnapshot | undefined;
    let r: CompareParaSnapshot | undefined;

    // identity 경로: id는 "paragraphMeta:id-moved:<stableId>"
    const sid = d.id.includes('id-moved:') ? d.id.split('id-moved:')[1] : '';
    if (sid) {
      l = lmap.get(sid);
      r = rmap.get(sid);
    } else {
      // alignment 경로: path가 좌측 기준으로 내려오는 moved:<sec>:<para>
      l = left.paragraphs.find((p) => p.section === d.path.section && p.paragraph === d.path.paragraph);
      r = l ? rmap.get(l.stableId) : undefined;
    }
    if (!l || !r) return true;

    const delta = r.globalIndex - l.globalIndex;
    if (delta === 0) return false;

    const sameRelativeOrder = rankLeft.get(l.stableId) === rankRight.get(l.stableId);
    if (!sameRelativeOrder) return true;

    if (delta > 0) {
      const addedBefore = rightOnlyPrefixCount[r.globalIndex];
      if (delta === addedBefore) return false;
    } else {
      const removedBefore = leftOnlyPrefixCount[l.globalIndex];
      if (-delta === removedBefore) return false;
    }
    return true;
  });
}

// ─── alignment 중간 표현: 정렬 결과 한 줄(좌만·우만·양쪽) ─────────────────────
// `AlignedPair[]`는 `buildTextDiffs`가 앵커 경계마다 `matchSegment`를 호출해 쌓은 뒤,
// `buildParagraphAlignStepsFromAligned` → `cleanupParagraphAlignStepsToDiffItems`로 소비된다.
// 컨트롤 슬롯 매칭은 cleanup **이전**의 동일 배열에서 `buildRightToLeftParaMapFromAligned`로 맵만 뽑는다.

/** `matchSegment*`의 출력 원소. null 쪽은 해당 문서에 문단이 없음(삽입/삭제 축). */
type AlignedPair = {
  left: CompareParaSnapshot | null;
  right: CompareParaSnapshot | null;
};

/** 컨트롤 폴백 매칭 단계에서 확정된 (좌, 우) 쌍 */
type ControlPair = {
  left: CompareControlSnapshot;
  right: CompareControlSnapshot;
};

/** diff의 path는 보통 "변경 후" 좌표를 우선하지만, 우측이 없으면 좌측을 쓴다. */
function preferRightPath(
  left: { section: number; paragraph: number } | null,
  right: { section: number; paragraph: number } | null,
): { section: number; paragraph: number } {
  if (right) return { section: right.section, paragraph: right.paragraph };
  if (left) return { section: left.section, paragraph: left.paragraph };
  return { section: 0, paragraph: 0 };
}

/**
 * **구간 한정** 유일 시그니처 앵커(Patience / LCS 스타일).
 *
 * `leftSeg`·`rightSeg` **안에서만** 시그니처 빈도를 세므로, 전역적으로는 중복이어도 구간 안에서만 유일하면 핀이 된다.
 * 양쪽에서 정확히 1번씩만 나오는 시그니처끼리 `ri`를 잡되, `ri`가 단조 증가하도록만 쌍을 채택해 교차 매칭을 막는다.
 * `minTextLen`: 상위 구간은 `INTRA_UNIQUE_SIG_MIN_TOP`, 깊은 재귀는 `INTRA_UNIQUE_SIG_MIN_NESTED`로
 * 짧은 줄의 우연 유일 핀을 줄인다. `tryMatchByNormHashPins`는 `minTextLen=0`으로 같은 로직을 재사용한다.
 */
function buildUniqueSigPairsInSlices(
  leftSeg: CompareParaSnapshot[],
  rightSeg: CompareParaSnapshot[],
  minTextLen: number,
): Array<{ li: number; ri: number }> {
  const countL = new Map<string, number>();
  const countR = new Map<string, number>();
  for (const p of leftSeg) countL.set(p.signature, (countL.get(p.signature) ?? 0) + 1);
  for (const p of rightSeg) countR.set(p.signature, (countR.get(p.signature) ?? 0) + 1);

  const rightBySig = new Map<string, number[]>();
  for (let ri = 0; ri < rightSeg.length; ri += 1) {
    const p = rightSeg[ri];
    if ((countR.get(p.signature) ?? 0) !== 1) continue;
    if (p.normalizedText.length < minTextLen) continue;
    if (!rightBySig.has(p.signature)) rightBySig.set(p.signature, []);
    rightBySig.get(p.signature)!.push(ri);
  }

  const pairs: Array<{ li: number; ri: number }> = [];
  let lastRi = -1;
  for (let li = 0; li < leftSeg.length; li += 1) {
    const lp = leftSeg[li];
    if ((countL.get(lp.signature) ?? 0) !== 1) continue;
    if (lp.normalizedText.length < minTextLen) continue;
    const candidates = rightBySig.get(lp.signature);
    if (!candidates || candidates.length !== 1) continue;
    const ri = candidates[0];
    if (ri <= lastRi) continue;
    pairs.push({ li, ri });
    lastRi = ri;
  }
  return pairs;
}

/** 문서 전체에서 `normalizedText.trim()` 문자열이 몇 번 나오는지 센다(빈 문자열 제외). */
function countTrimNormText(paras: CompareParaSnapshot[]): Map<string, number> {
  const m = new Map<string, number>();
  for (const p of paras) {
    const k = p.normalizedText.trim();
    if (!k) continue;
    m.set(k, (m.get(k) ?? 0) + 1);
  }
  return m;
}

/**
 * trim `normalizedText` 가 왼쪽·오른쪽 문서에서 각각 정확히 한 번만 등장하는 문자열(교집합).
 * 짧은 구조 앵커 후보가 전역적으로 우연 중복이 아닌지 확인할 때 사용한다.
 */
function buildBothSidesUniqueTrimNorm(left: CompareParaSnapshot[], right: CompareParaSnapshot[]): Set<string> {
  const cl = countTrimNormText(left);
  const cr = countTrimNormText(right);
  const out = new Set<string>();
  for (const [k, v] of cl) {
    if (v !== 1) continue;
    if (cr.get(k) === 1) out.add(k);
  }
  return out;
}

/**
 * 전역 문단 배열에서 유일한 동일 시그니처 쌍을 단조 증가하는 `ri`로만 잡아 alignment 구간 경계를 만든다.
 * 후보는 `fillSnapshotFromWasm`에서 `isAnchorCandidate`로 거른다(기본은 길이·엔트로피, 예외는 짧은 블록 제목·Outline/Number).
 * `minTextLen` 미만인 후보는 trim 본문이 양쪽 문서에서 각각 한 번만 나올 때만 앵커로 승인한다.
 */
function buildAnchorPairs(
  left: CompareParaSnapshot[],
  right: CompareParaSnapshot[],
  anchorTuning: Required<CompareAnchorTuning>,
): Array<{ li: number; ri: number }> {
  const shortOkTrim = buildBothSidesUniqueTrimNorm(left, right);
  const rightBySig = new Map<string, number[]>();
  for (let i = 0; i < right.length; i += 1) {
    const p = right[i];
    if (!p.isAnchorCandidate) continue;
    if (!rightBySig.has(p.signature)) rightBySig.set(p.signature, []);
    rightBySig.get(p.signature)!.push(i);
  }

  const pairs: Array<{ li: number; ri: number }> = [];
  let lastRi = -1;
  for (let li = 0; li < left.length; li += 1) {
    const lp = left[li];
    if (!lp.isAnchorCandidate) continue;
    const t = lp.normalizedText.trim();
    if (t.length < anchorTuning.minTextLen && !shortOkTrim.has(t)) continue;
    const candidates = rightBySig.get(lp.signature);
    if (!candidates || candidates.length !== 1) continue; // 앵커 오염 방지: 단일 매치만 앵커로 인정
    const ri = candidates[0];
    if (ri <= lastRi) continue;
    pairs.push({ li, ri });
    lastRi = ri;
  }
  return pairs;
}

// ─── 문단 정렬: 유사도·구조·비용 (DP / 그리디 공통) ─────────────────────────────
// `textSimilarity`: 스냅샷의 normalizedText 기준. 정렬 외 일부 휴리스틱에서도 호출된다.
// `isNearStructure` / `getEffectiveSimilarity` / `matchCost` / `scorePairGreedy` 는 문단 쌍 정렬 전용.

/** 문자 2-gram Dice 계수. 공백 제거 문자열에 사용해 형태소 단위 토큰이 없을 때도 신호를 남긴다. */
function charBigramSimilarity(a: string, b: string): number {
  const aa = Array.from(a);
  const bb = Array.from(b);
  if (aa.length < 2 || bb.length < 2) return 0;
  const gramsA = new Set<string>();
  const gramsB = new Set<string>();
  for (let i = 0; i < aa.length - 1; i += 1) gramsA.add(`${aa[i]}${aa[i + 1]}`);
  for (let i = 0; i < bb.length - 1; i += 1) gramsB.add(`${bb[i]}${bb[i + 1]}`);
  if (gramsA.size === 0 || gramsB.size === 0) return 0;
  let inter = 0;
  for (const g of gramsA) if (gramsB.has(g)) inter += 1;
  return (2 * inter) / (gramsA.size + gramsB.size);
}

/**
 * 토큰(공백 분리) + 문자 바이그램 혼합 유사도.
 * 한국어는 조사/어미만 바뀌어도 토큰 집합이 크게 달라질 수 있어 `charSim`에 0.8 가중.
 * `containsBoost`는 짧은 문자열이 긴 쪽에 거의 그대로 포함되는 경우(번호 접미 등)를 보정.
 */
function textSimilarity(a: string, b: string): number {
  if (!a && !b) return 1;
  if (!a || !b) return 0;
  if (a === b) return 1;

  const sa = new Set(a.split(/\s+/).filter(Boolean));
  const sb = new Set(b.split(/\s+/).filter(Boolean));
  const tokenSim =
    sa.size === 0 || sb.size === 0
      ? 0
      : (() => {
          let inter = 0;
          for (const t of sa) if (sb.has(t)) inter += 1;
          return (2 * inter) / (sa.size + sb.size);
        })();

  // 문자 유사도는 공백 변형 영향을 줄이기 위해 공백 제거본으로 계산한다.
  const aNoWs = a.replace(/\s+/g, '');
  const bNoWs = b.replace(/\s+/g, '');
  const charSim = charBigramSimilarity(aNoWs, bNoWs);

  // 한쪽이 다른 쪽을 거의 포함하면(예: "맛있다" -> "맛있다2") 동일 문단 가능성이 높다.
  const shorter = a.length <= b.length ? a : b;
  const longer = a.length <= b.length ? b : a;
  const containsBoost = shorter.length >= 4 && longer.includes(shorter) ? 0.96 : 0;

  // 문단 정렬: 한국어는 조사·어미만 바뀌어도 공백 토큰이 크게 달라지므로 바이그램 쪽 비중을 둔다.
  const mixed = tokenSim * 0.2 + charSim * 0.8;
  return Math.max(mixed, containsBoost);
}

/**
 * 구역·문단 번호·(상대)globalIndex·컨트롤 수가 "같은 슬롯의 이웃"으로 볼 만한 쌍.
 * - `matchSegment` 안에서는 `activeSegmentStructBase` 기준으로 베이스 대비 오프셋 차만 본다(베이스는 직전
 *   글로벌 앵커 쌍이 있으면 그 `globalIndex`, 없으면 구간 첫 문단).
 * - 그 밖(후처리 등)에서는 전역 `globalIndex` 차 ≤2를 그대로 사용한다.
 */
function isNearStructure(lp: CompareParaSnapshot, rp: CompareParaSnapshot): boolean {
  if (lp.section !== rp.section) return false;
  if (Math.abs(lp.paragraph - rp.paragraph) > 1) return false;
  if (lp.controlCount !== rp.controlCount) return false;
  if (activeSegmentStructBase) {
    const dL = lp.globalIndex - activeSegmentStructBase.leftBaseGi;
    const dR = rp.globalIndex - activeSegmentStructBase.rightBaseGi;
    return Math.abs(dL - dR) <= 2;
  }
  return Math.abs(lp.globalIndex - rp.globalIndex) <= 2;
}

/** 공백 제거 기준 양쪽 문단이 충분히 길 때만 임계/부스트 완화(짧은 문단 노이즈 억제) */
function isNearStructureLongPair(lp: CompareParaSnapshot, rp: CompareParaSnapshot): boolean {
  const lenL = lp.normalizedText.replace(/\s+/g, '').length;
  const lenR = rp.normalizedText.replace(/\s+/g, '').length;
  return lenL > 15 && lenR > 15;
}

/**
 * textSimilarity 통과 하한: 전역은 엄격, 구조 근접 시만 완화(짧은 문단은 우연 일치 억제).
 * `textSimilarity` 자체는 수정하지 않고, DP/그리디의 `matchCost`·`scorePairGreedy`에서만 사용한다.
 */
function softSimilarityThresholdForPair(lp: CompareParaSnapshot, rp: CompareParaSnapshot): number {
  if (!isNearStructure(lp, rp)) return MATCH_SOFT_SIM_MIN;
  return isNearStructureLongPair(lp, rp) ? 0.25 : 0.45;
}

/**
 * DP/그리디 평가 전용 유사도. `textSimilarity`는 순수 값으로 두고, 구조 근접·긴 문단일 때만 보정한다.
 */
function getEffectiveSimilarity(lp: CompareParaSnapshot, rp: CompareParaSnapshot): number {
  const rawSim = textSimilarity(lp.normalizedText, rp.normalizedText);
  if (!isNearStructure(lp, rp)) return rawSim;
  if (!isNearStructureLongPair(lp, rp)) return rawSim;
  if (rawSim > NEAR_STRUCTURE_MIN_SIM) {
    return Math.max(rawSim, NEAR_STRUCTURE_SIM_BOOST);
  }
  return rawSim;
}

/** 매칭 비용: 낮을수록 좋음 (0 = 완전 일치). 서명 불일치 + 낮은 유사도는 치환 경로를 막아 이웃 문단 오매칭을 줄인다. */
function matchCost(lp: CompareParaSnapshot, rp: CompareParaSnapshot): number {
  if (lp.signature === rp.signature) return 0;
  const sim = getEffectiveSimilarity(lp, rp);
  const threshold = softSimilarityThresholdForPair(lp, rp);
  if (sim < threshold) return MATCH_COST_WEAK;
  let c = 1 - sim;
  if (lp.controlCount !== rp.controlCount) c += 0.35;
  if (isNearStructure(lp, rp)) c = Math.max(0, c - NEAR_STRUCTURE_COST_DISCOUNT);
  return c;
}

/** 원본 문단을 복제하되 `signature`만 `nh:<hash(trim)>`로 바꿔, 유일 시그니처 핀 로직을 재사용한다. */
function paraWithNormHashSig(p: CompareParaSnapshot): CompareParaSnapshot {
  const key = p.normalizedText.trim();
  const h = key ? simpleHash(key) : 'empty';
  return { ...p, signature: `nh:${h}` };
}

/**
 * DMP `diff_lineMode_`와 비슷한 **2단계 중 거시**: 구간 문단을 줄 단위 해시 토큰으로 본 뒤
 * `buildUniqueSigPairsInSlices(..., minTextLen=0)`로 구간 내 양쪽 유일 쌍을 핀으로 세운다.
 * 원문 시그니처만으로는 유일 핀이 없을 때(대량 삽입으로 서명이 겹칠 때) 큰 덩어리를 가른다.
 * `NORM_HASH_PIN_MIN_TOTAL_PARAS` 미만이면 호출하지 않는다.
 */
function tryMatchByNormHashPins(
  leftSeg: CompareParaSnapshot[],
  rightSeg: CompareParaSnapshot[],
  depth: number,
  anchorBoundary: SegmentAnchorBoundary | null,
): AlignedPair[] | null {
  const n = leftSeg.length;
  const m = rightSeg.length;
  if (n + m < NORM_HASH_PIN_MIN_TOTAL_PARAS) return null;
  const lh = leftSeg.map(paraWithNormHashSig);
  const rh = rightSeg.map(paraWithNormHashSig);
  const intra0 = buildUniqueSigPairsInSlices(lh, rh, 0);
  if (intra0.length === 0) return null;
  return runInternalAnchorBoundaries(leftSeg, rightSeg, intra0, depth, anchorBoundary);
}

/**
 * 내부 핀(`intra`)을 경계로 삼아 슬라이스를 나누고, 각 조각에 `matchSegment`를 재귀한다.
 * `intra`는 시그니처 기반(`matchSegmentWithInternalAnchors`)이든 해시 기반(`tryMatchByNormHashPins`)이든 동일.
 */
function runInternalAnchorBoundaries(
  leftSeg: CompareParaSnapshot[],
  rightSeg: CompareParaSnapshot[],
  intra: Array<{ li: number; ri: number }>,
  depth: number,
  anchorBoundary: SegmentAnchorBoundary | null,
): AlignedPair[] {
  const boundaries = [{ li: -1, ri: -1 }, ...intra, { li: leftSeg.length, ri: rightSeg.length }];
  const out: AlignedPair[] = [];
  for (let i = 0; i < boundaries.length - 1; i += 1) {
    const a = boundaries[i];
    const b = boundaries[i + 1];
    if (a.li >= 0 && a.ri >= 0) {
      out.push({ left: leftSeg[a.li], right: rightSeg[a.ri] });
    }
    const ls = leftSeg.slice(a.li + 1, b.li);
    const rs = rightSeg.slice(a.ri + 1, b.ri);
    if (ls.length === 0 && rs.length === 0) continue;
    out.push(...matchSegment(ls, rs, depth + 1, anchorBoundary));
  }
  return out;
}

/**
 * 구간 내부 유일 시그니처 앵커로 쪼개 각 조각에 `matchSegment` 재귀.
 * Git patience와 유사: DP에 넘기기 전 구간을 최대한 잘게 나눈다.
 */
function matchSegmentWithInternalAnchors(
  leftSeg: CompareParaSnapshot[],
  rightSeg: CompareParaSnapshot[],
  depth: number,
  anchorBoundary: SegmentAnchorBoundary | null,
): AlignedPair[] {
  const intraMin = depth >= 1 ? INTRA_UNIQUE_SIG_MIN_NESTED : INTRA_UNIQUE_SIG_MIN_TOP;
  const intra = buildUniqueSigPairsInSlices(leftSeg, rightSeg, intraMin);
  if (intra.length === 0) return [];
  return runInternalAnchorBoundaries(leftSeg, rightSeg, intra, depth, anchorBoundary);
}

/**
 * 문단 시그니처가 **연속으로** 동일한 최장 구간(시작 인덱스 포함).
 * DMP half-match 유사: 그리디 직전 큰 구간을 한 번 가른다.
 */
function findLongestEqualSignatureRun(
  leftSeg: CompareParaSnapshot[],
  rightSeg: CompareParaSnapshot[],
): { li0: number; ri0: number; len: number } | null {
  const n = leftSeg.length;
  const m = rightSeg.length;
  if (n === 0 || m === 0) return null;
  if (n * m > HALF_MATCH_MAX_PRODUCT) return null;
  let bestLen = 0;
  let bestI = 0;
  let bestJ = 0;
  for (let i = 0; i < n; i += 1) {
    for (let j = 0; j < m; j += 1) {
      if (leftSeg[i].signature !== rightSeg[j].signature) continue;
      let k = 0;
      while (i + k < n && j + k < m && leftSeg[i + k].signature === rightSeg[j + k].signature) k += 1;
      if (k > bestLen) {
        bestLen = k;
        bestI = i;
        bestJ = j;
      }
    }
  }
  if (bestLen < 1) return null;
  const touchesAll = bestI === 0 && bestJ === 0 && bestLen === n && bestLen === m;
  if (touchesAll) return null;
  const hasLeftover = bestI > 0 || bestJ > 0 || bestI + bestLen < n || bestJ + bestLen < m;
  if (!hasLeftover) return null;
  if (bestLen < 2 && n + m < 12) return null;
  return { li0: bestI, ri0: bestJ, len: bestLen };
}

/**
 * 앵커 사이(또는 재귀 하위) 한 구간의 좌·우 문단 슬라이스를 `AlignedPair[]`로 정렬한다.
 *
 * 순서(대략):
 * 1. `runWithSegmentStructBase`: `anchorBoundary`가 있으면 그 앵커의 globalIndex를 구조 베이스로,
 *    없으면 슬라이스 첫 문단 쌍을 베이스로 `isNearStructure`가 동작하게 한다.
 * 2. 시간 가드(`shouldBailToGreedy`) → 그리디 조기 종료.
 * 3. `buildUniqueSigPairsInSlices`로 내부 유일 시그니처 핀이 있으면 `matchSegmentWithInternalAnchors`.
 * 4. 없으면 `tryMatchByNormHashPins`(큰 구간만).
 * 5. 셀 수 한도 내면 `matchSegmentDp`, 아니면 `findLongestEqualSignatureRun`으로 한 번 가르고 재귀,
 *    그것도 어렵면 `matchWindowedGreedy`.
 */
function matchSegment(
  leftSeg: CompareParaSnapshot[],
  rightSeg: CompareParaSnapshot[],
  depth = 0,
  anchorBoundary: SegmentAnchorBoundary | null = null,
): AlignedPair[] {
  const n = leftSeg.length;
  const m = rightSeg.length;
  const perf = resolvePerformanceTuning(activeCompareOptions ?? DEFAULT_COMPARE_OPTIONS);
  if (n === 0 && m === 0) return [];
  if (n === 0) return rightSeg.map((rp) => ({ left: null, right: rp }));
  if (m === 0) return leftSeg.map((lp) => ({ left: lp, right: null }));

  const structBase: SegmentStructBase = anchorBoundary
    ? { leftBaseGi: anchorBoundary.leftAnchorGi, rightBaseGi: anchorBoundary.rightAnchorGi }
    : { leftBaseGi: leftSeg[0].globalIndex, rightBaseGi: rightSeg[0].globalIndex };

  return runWithSegmentStructBase(structBase, () => {
    if (shouldBailToGreedy()) return matchWindowedGreedy(leftSeg, rightSeg);

    if (depth >= MAX_SEGMENT_RECURSION) {
      if (n * m <= SEGMENT_DP_MAX * SEGMENT_DP_MAX && n * m <= perf.hardSegmentCells) {
        return matchSegmentDp(leftSeg, rightSeg);
      }
      return matchWindowedGreedy(leftSeg, rightSeg);
    }

    const intraMin = depth >= 1 ? INTRA_UNIQUE_SIG_MIN_NESTED : INTRA_UNIQUE_SIG_MIN_TOP;
    const intra = buildUniqueSigPairsInSlices(leftSeg, rightSeg, intraMin);
    if (intra.length > 0) {
      return matchSegmentWithInternalAnchors(leftSeg, rightSeg, depth, anchorBoundary);
    }

    const hashPinned = tryMatchByNormHashPins(leftSeg, rightSeg, depth, anchorBoundary);
    if (hashPinned) return hashPinned;

    if (n * m <= SEGMENT_DP_MAX * SEGMENT_DP_MAX && n * m <= perf.hardSegmentCells) {
      return matchSegmentDp(leftSeg, rightSeg);
    }

    const run = findLongestEqualSignatureRun(leftSeg, rightSeg);
    if (run && depth + 1 < MAX_SEGMENT_RECURSION) {
      const { li0, ri0, len } = run;
      const out: AlignedPair[] = [];
      out.push(...matchSegment(leftSeg.slice(0, li0), rightSeg.slice(0, ri0), depth + 1, anchorBoundary));
      for (let k = 0; k < len; k += 1) {
        out.push({ left: leftSeg[li0 + k], right: rightSeg[ri0 + k] });
      }
      out.push(...matchSegment(leftSeg.slice(li0 + len), rightSeg.slice(ri0 + len), depth + 1, anchorBoundary));
      return out;
    }

    return matchWindowedGreedy(leftSeg, rightSeg);
  });
}

/**
 * 표준 2문자열 편집 DP를 "문단 시퀀스"에 적용한 것. `dp[i][j]` = 왼쪽 i개·오른쪽 j개까지 최소 비용.
 * - 치환 비용은 `matchCost`(0~수렴)이고 삽입/삭제는 고정 `del`/`ins`(1.05)로 스무딩한다.
 * - 백트래킹 동률 시 `match > delete > insert` 순으로 분기해, 삽입으로만 밀린 것처럼 보이는 경향을 줄인다.
 * - 연속한 `matchCost===0`(시그니처 일치 등) 대각선은 한 번에 소비(snake)해 백트래킹 루프 비용을 줄인다.
 */
function matchSegmentDp(leftSeg: CompareParaSnapshot[], rightSeg: CompareParaSnapshot[]): AlignedPair[] {
  if (shouldBailToGreedy()) return matchWindowedGreedy(leftSeg, rightSeg);
  const n = leftSeg.length;
  const m = rightSeg.length;
  const inf = 1e9;
  /** 문단 한 줄 삽입/삭제 비용. 1.0보다 약간 크게 두어 무의미한 치환 남발을 억제 */
  const del = 1.05;
  const ins = 1.05;
  const dp: number[][] = Array.from({ length: n + 1 }, () => Array(m + 1).fill(inf));
  dp[0][0] = 0;
  for (let i = 1; i <= n; i += 1) dp[i][0] = dp[i - 1][0] + del;
  for (let j = 1; j <= m; j += 1) dp[0][j] = dp[0][j - 1] + ins;

  for (let i = 1; i <= n; i += 1) {
    if (i % 12 === 0 && shouldBailToGreedy()) return matchWindowedGreedy(leftSeg, rightSeg);
    for (let j = 1; j <= m; j += 1) {
      const mc = matchCost(leftSeg[i - 1], rightSeg[j - 1]);
      dp[i][j] = Math.min(dp[i - 1][j - 1] + mc, dp[i - 1][j] + del, dp[i][j - 1] + ins);
    }
  }

  const eps = 1e-5;
  const out: AlignedPair[] = [];
  let i = n;
  let j = m;
  while (i > 0 || j > 0) {
    const candMatch =
      i > 0 && j > 0
        ? dp[i - 1][j - 1] + matchCost(leftSeg[i - 1], rightSeg[j - 1])
        : inf;
    const candDel = i > 0 ? dp[i - 1][j] + del : inf;
    const candIns = j > 0 ? dp[i][j - 1] + ins : inf;
    const target = dp[i][j];
    if (i > 0 && j > 0 && Math.abs(candMatch - target) < eps) {
      out.push({ left: leftSeg[i - 1], right: rightSeg[j - 1] });
      i -= 1;
      j -= 1;
      // Snake: 이후 `matchCost===0`이고 대각선이 최적을 유지하는 동안 연속 소비.
      while (i > 0 && j > 0) {
        const lp = leftSeg[i - 1];
        const rp = rightSeg[j - 1];
        if (matchCost(lp, rp) !== 0) break;
        if (Math.abs(dp[i][j] - dp[i - 1][j - 1]) >= eps) break;
        const delC = dp[i - 1][j] + del;
        const insC = dp[i][j - 1] + ins;
        if (delC < dp[i][j] - eps || insC < dp[i][j] - eps) break;
        out.push({ left: lp, right: rp });
        i -= 1;
        j -= 1;
      }
      continue;
    } else if (i > 0 && Math.abs(candDel - target) < eps) {
      out.push({ left: leftSeg[i - 1], right: null });
      i -= 1;
    } else if (j > 0 && Math.abs(candIns - target) < eps) {
      out.push({ left: null, right: rightSeg[j - 1] });
      j -= 1;
    } else {
      if (i > 0 && j > 0) {
        out.push({ left: leftSeg[i - 1], right: rightSeg[j - 1] });
        i -= 1;
        j -= 1;
      } else if (i > 0) {
        out.push({ left: leftSeg[i - 1], right: null });
        i -= 1;
      } else {
        out.push({ left: null, right: rightSeg[j - 1] });
        j -= 1;
      }
    }
  }
  out.reverse();
  return out;
}

/**
 * 윈도 그리디 전용 점수. 음수면 후보에서 제외(`sim`이 쌍별 임계 미만 등).
 * 서명 일치(+5)·컨트롤 수 일치(+1)·구조 근접(`NEAR_STRUCTURE_GREEDY_BONUS`)에 `sim*3`을 더해 스케일을 맞춘다.
 */
function scorePairGreedy(lp: CompareParaSnapshot, rp: CompareParaSnapshot): number {
  const sim = getEffectiveSimilarity(lp, rp);
  if (lp.signature !== rp.signature && sim < softSimilarityThresholdForPair(lp, rp)) return -1;
  let score = 0;
  if (lp.signature === rp.signature) score += 5;
  if (lp.controlCount === rp.controlCount) score += 1;
  if (isNearStructure(lp, rp)) score += NEAR_STRUCTURE_GREEDY_BONUS;
  return score + sim * 3;
}

/**
 * 대구간 폴백: 오른쪽 문단 순서대로 왼쪽 후보를 윈도 안에서 고른 뒤, 점수로 매칭.
 * - `minScore`: 서명이 다른 의역 문단은 sim*3+보너스로도 낮게 나올 수 있어 NEAR_STRUCTURE_GREEDY_BONUS 로 보완.
 * - `ambiguous`: 1·2위 점수 차가 GREEDY_AMBIGUOUS_GAP 미만이면 매칭 포기. 단 1위가 `isNearStructure`이면
 *   동률 검사 생략(의역 쌍이 엉뚱한 2위와 0.2점 차로 동반 탈락하는 것 방지).
 */
function matchWindowedGreedy(leftSeg: CompareParaSnapshot[], rightSeg: CompareParaSnapshot[]): AlignedPair[] {
  const aligned: AlignedPair[] = [];
  const usedLeft = new Set<number>();
  let leftCursor = 0;
  /** 후보 거절 임계. 튜닝 시 `scorePairGreedy` 최솟값(서명 불일치·의역)과 함께 맞출 것 */
  const minScore = 3.45;

  const pickBestInRange = (rp: CompareParaSnapshot, start: number, end: number) => {
    let bestLi = -1;
    let bestScore = -1;
    let secondScore = -1;
    const lo = Math.max(0, start);
    const hi = Math.min(leftSeg.length - 1, end);
    for (let li = lo; li <= hi; li += 1) {
      if (usedLeft.has(li)) continue;
      const s = scorePairGreedy(leftSeg[li], rp);
      if (s < 0) continue;
      if (s > bestScore) {
        secondScore = bestScore;
        bestScore = s;
        bestLi = li;
      } else if (s > secondScore) {
        secondScore = s;
      }
    }
    return { bestLi, bestScore, secondScore };
  };

  for (let ri = 0; ri < rightSeg.length; ri += 1) {
    if (ri % 32 === 0 && shouldBailToGreedy()) {
      for (let rj = ri; rj < rightSeg.length; rj += 1) aligned.push({ left: null, right: rightSeg[rj] });
      for (let li = 0; li < leftSeg.length; li += 1) {
        if (!usedLeft.has(li)) aligned.push({ left: leftSeg[li], right: null });
      }
      return aligned;
    }
    const rp = rightSeg[ri];
    // 이전 매칭 위치 근처를 먼저 본 뒤, 실패 시 전 구간 재탐색(의역으로 인덱스가 크게 어긋난 경우).
    const start = Math.max(leftCursor, 0);
    const end = Math.min(leftSeg.length - 1, leftCursor + WINDOW_SIZE + WINDOW_SIZE);
    let { bestLi, bestScore, secondScore } = pickBestInRange(rp, start, end);

    if (bestLi < 0 || bestScore < minScore) {
      const full = pickBestInRange(rp, 0, leftSeg.length - 1);
      if (full.bestScore > bestScore) {
        bestLi = full.bestLi;
        bestScore = full.bestScore;
        secondScore = full.secondScore;
      }
    }

    const isBestNear = bestLi >= 0 && isNearStructure(leftSeg[bestLi], rp);
    const ambiguous =
      !isBestNear && bestLi >= 0 && secondScore >= 0 && bestScore - secondScore < GREEDY_AMBIGUOUS_GAP;
    if (bestLi >= 0 && bestScore >= minScore && !ambiguous) {
      usedLeft.add(bestLi);
      aligned.push({ left: leftSeg[bestLi], right: rp });
      leftCursor = bestLi;
    } else {
      aligned.push({ left: null, right: rp });
    }
  }

  for (let li = 0; li < leftSeg.length; li += 1) {
    if (!usedLeft.has(li)) aligned.push({ left: leftSeg[li], right: null });
  }
  return aligned;
}

/** 왼쪽 한 문단이 오른쪽 연속 두 문단으로만 쪼개진 경우(순서: 앞·뒤) */
function isLeftParagraphSplitIntoTwoRightParas(
  left: CompareParaSnapshot,
  rightHead: CompareParaSnapshot,
  rightTail: CompareParaSnapshot,
): boolean {
  const joined = `${rightHead.normalizedText} ${rightTail.normalizedText}`.trim();
  if (!joined) return false;
  const leftN = left.normalizedText.replace(/\s+/g, ' ').trim();
  const joinedN = joined.replace(/\s+/g, ' ').trim();
  if (!leftN) return false;
  // 중간 삽입으로 밀린 경우: R앞에 원문에 없던 긴 텍스트가 끼면 이어붙인 길이가 원문보다 커진다.
  // textSimilarity의 contains 부스트만으로는 이 경우도 '분할'로 오인할 수 있어 길이로 한 번 걸러낸다.
  const maxExtra = Math.max(2, Math.ceil(leftN.length * 0.06));
  if (joinedN.length > leftN.length + maxExtra) return false;
  return textSimilarity(left.normalizedText, joined) >= PARA_SPLIT_JOIN_SIM_MIN;
}

/**
 * 유사도만으로는 잡기 어려운 케이스 보정:
 * - 기존 빈 문단에 텍스트를 입력한 경우(sim=0에 가까움)도
 *   구조 신호(구역/문단 위치/컨트롤 수)가 일치하면 "텍스트 변경"으로 본다.
 */
function shouldPromoteEmptyTextEdit(
  left: CompareParaSnapshot,
  right: CompareParaSnapshot,
  leftParas: CompareParaSnapshot[],
  rightParas: CompareParaSnapshot[],
): boolean {
  if (left.section !== right.section) return false;
  // 문서 비교(alignment)에서는 앞쪽 삽입/삭제로 인덱스가 쉽게 밀리므로
  // 빈문단 편집 승격은 근접 허용폭을 약간 넓힌다.
  if (Math.abs(left.paragraph - right.paragraph) > 2) return false;
  if (Math.abs(left.globalIndex - right.globalIndex) > 4) return false;
  if (left.controlCount !== right.controlCount) return false;
  const lEmpty = left.normalizedText.length === 0;
  const rEmpty = right.normalizedText.length === 0;
  if (lEmpty === rEmpty) return false;
  const leftByGlobal = new Map<number, CompareParaSnapshot>(leftParas.map((p) => [p.globalIndex, p] as const));
  const rightByGlobal = new Map<number, CompareParaSnapshot>(rightParas.map((p) => [p.globalIndex, p] as const));

  const leftPrev = leftByGlobal.get(left.globalIndex - 1) ?? null;
  const leftNext = leftByGlobal.get(left.globalIndex + 1) ?? null;
  const rightPrev = rightByGlobal.get(right.globalIndex - 1) ?? null;
  const rightNext = rightByGlobal.get(right.globalIndex + 1) ?? null;

  const isEmptyPara = (p: CompareParaSnapshot | null) => !!p && p.normalizedText.length === 0;

  // 연속 빈 문단 구간은 원래 보수적으로 차단했지만,
  // 실제 문서에서는 "빈 문단 -> 텍스트 입력" 케이스가 여기에 걸려 added/removed로 남는 경우가 잦다.
  // 슬롯 근접(문단/전역 인덱스)일 때는 빈 이웃이 있어도 승격을 허용한다.
  const hasAdjacentEmpty =
    isEmptyPara(leftPrev) || isEmptyPara(leftNext) || isEmptyPara(rightPrev) || isEmptyPara(rightNext);
  if (hasAdjacentEmpty) {
    const nearSlot =
      Math.abs(left.paragraph - right.paragraph) <= 2 &&
      Math.abs(left.globalIndex - right.globalIndex) <= 4;
    if (!nearSlot) return false;
  }

  // 양옆 문맥 정합성: 바로 위/아래 문단 중 하나 이상은 시그니처가 유지되어야 한다.
  const prevStable =
    !!leftPrev &&
    !!rightPrev &&
    leftPrev.section === rightPrev.section &&
    leftPrev.signature === rightPrev.signature;
  const nextStable =
    !!leftNext &&
    !!rightNext &&
    leftNext.section === rightNext.section &&
    leftNext.signature === rightNext.signature;
  if (!prevStable && !nextStable) {
    // 양옆 시그니처가 모두 흔들린 케이스에서도,
    // 같은 슬롯 근처(문단/전역 인덱스가 충분히 가까운 경우)면 빈문단 편집으로 본다.
    const nearSlot =
      Math.abs(left.paragraph - right.paragraph) <= 2 &&
      Math.abs(left.globalIndex - right.globalIndex) <= 4;
    if (!nearSlot) return false;
  }

  return true;
}

/**
 * 정렬이 (삭제, 추가)로 쪼개졌지만 텍스트는 같은 문단의 수정에 가깝다고 볼 때(밀림·globalIndex 한계 보정).
 */
function shouldMergeRemovedAddedAsModify(lp: CompareParaSnapshot, rp: CompareParaSnapshot): boolean {
  if (lp.section !== rp.section) return false;
  if (lp.controlCount !== rp.controlCount) return false;
  if (Math.abs(lp.paragraph - rp.paragraph) > REMOVED_ADDED_MAX_PARA_GAP) return false;
  if (Math.abs(lp.globalIndex - rp.globalIndex) > REMOVED_ADDED_MAX_GLOBAL_GAP) return false;
  return textSimilarity(lp.normalizedText, rp.normalizedText) >= REMOVED_ADDED_MERGE_SIM_MIN;
}

// ─── alignment 본문: 앵커 → 구간 정렬 → 단계 스트림 → cleanup ─────────────────

/**
 * `AlignedPair`를 한 칸씩 전진하는 스트림으로 바꾼다(DMP cleanup 전 단계).
 * lookahead 없이 `cleanupParagraphAlignStepsToDiffItems`에서만 2칸 패턴을 처리한다.
 */
type ParagraphAlignStep =
  | { kind: 'lr'; l: CompareParaSnapshot; r: CompareParaSnapshot }
  | { kind: 'l'; l: CompareParaSnapshot }
  | { kind: 'r'; r: CompareParaSnapshot };

/** 정렬 스트림에서 직전에 등장한 왼쪽 문단(짝 맞춤 기준) — 오른쪽 전용 추가 항목의 왼쪽 패널 프리뷰용 */
function nearestAlignedLeftPeer(steps: ParagraphAlignStep[], fromIndex: number): CompareParaSnapshot | null {
  for (let j = fromIndex - 1; j >= 0; j--) {
    const s = steps[j];
    if (s.kind === 'lr') return s.l;
    if (s.kind === 'l') return s.l;
  }
  return null;
}

/** 정렬 스트림에서 직전에 등장한 오른쪽 문단 — 왼쪽 전용 삭제 항목의 오른쪽 패널 프리뷰용 */
function nearestAlignedRightPeer(steps: ParagraphAlignStep[], fromIndex: number): CompareParaSnapshot | null {
  for (let j = fromIndex - 1; j >= 0; j--) {
    const s = steps[j];
    if (s.kind === 'lr') return s.r;
    if (s.kind === 'r') return s.r;
  }
  return null;
}

function buildParagraphAlignStepsFromAligned(aligned: AlignedPair[]): ParagraphAlignStep[] {
  const steps: ParagraphAlignStep[] = [];
  for (const pair of aligned) {
    const { left: l, right: r } = pair;
    if (!l && !r) continue;
    if (l && !r) steps.push({ kind: 'l', l });
    else if (!l && r) steps.push({ kind: 'r', r });
    else steps.push({ kind: 'lr', l: l!, r: r! });
  }
  return steps;
}

/**
 * 정렬 스트림을 `DiffItem[]`로 바꾼다. DMP `diff_cleanupSemantic`과 같이
 * “기계적 변환 + 고정 순서 cleanup”을 한곳에 모은다.
 */
function cleanupParagraphAlignStepsToDiffItems(
  steps: ParagraphAlignStep[],
  lps: CompareParaSnapshot[],
  rps: CompareParaSnapshot[],
): DiffItem[] {
  const diffs: DiffItem[] = [];
  let i = 0;
  while (i < steps.length) {
    const s0 = steps[i];
    const s1 = i + 1 < steps.length ? steps[i + 1] : null;

    if (s0.kind === 'l' && s1?.kind === 'r' && shouldPromoteEmptyTextEdit(s0.l, s1.r, lps, rps)) {
      diffs.push({
        id: mkDiffId('text', `modified-empty-edit:${s0.l.section}:${s0.l.paragraph}`),
        kind: 'text',
        severity: 'modified',
        path: preferRightPath(s0.l, s1.r),
        title: `텍스트 변경 (${formatParaLocTitle(s0.l)})`,
        leftPreview: s0.l.text,
        rightPreview: s1.r.text,
        leftAnchor: s0.l.anchor,
        rightAnchor: s1.r.anchor,
      });
      i += 2;
      continue;
    }
    if (s0.kind === 'r' && s1?.kind === 'l' && shouldPromoteEmptyTextEdit(s1.l, s0.r, lps, rps)) {
      diffs.push({
        id: mkDiffId('text', `modified-empty-edit:${s1.l.section}:${s1.l.paragraph}`),
        kind: 'text',
        severity: 'modified',
        path: preferRightPath(s1.l, s0.r),
        title: `텍스트 변경 (${formatParaLocTitle(s1.l)})`,
        leftPreview: s1.l.text,
        rightPreview: s0.r.text,
        leftAnchor: s1.l.anchor,
        rightAnchor: s0.r.anchor,
      });
      i += 2;
      continue;
    }

    if (s0.kind === 'l' && s1?.kind === 'r' && shouldMergeRemovedAddedAsModify(s0.l, s1.r)) {
      const r2 = s1.r;
      diffs.push({
        id: mkDiffId('text', `modified-merged:${s0.l.section}:${s0.l.paragraph}`),
        kind: 'text',
        severity: 'modified',
        path: preferRightPath(s0.l, r2),
        title: `텍스트 변경 (${formatParaLocTitle(r2)})`,
        leftPreview: s0.l.text,
        rightPreview: r2.text,
        leftAnchor: s0.l.anchor,
        rightAnchor: r2.anchor,
        leftSectionPage: s0.l.sectionPage,
        rightSectionPage: r2.sectionPage,
      });
      i += 2;
      continue;
    }
    if (s0.kind === 'r' && s1?.kind === 'l' && shouldMergeRemovedAddedAsModify(s1.l, s0.r)) {
      const l2 = s1.l;
      diffs.push({
        id: mkDiffId('text', `modified-merged:${l2.section}:${l2.paragraph}`),
        kind: 'text',
        severity: 'modified',
        path: preferRightPath(l2, s0.r),
        title: `텍스트 변경 (${formatParaLocTitle(s0.r)})`,
        leftPreview: l2.text,
        rightPreview: s0.r.text,
        leftAnchor: l2.anchor,
        rightAnchor: s0.r.anchor,
        leftSectionPage: l2.sectionPage,
        rightSectionPage: s0.r.sectionPage,
      });
      i += 2;
      continue;
    }

    if (s0.kind === 'r' && s1?.kind === 'lr' && isLeftParagraphSplitIntoTwoRightParas(s1.l, s0.r, s1.r)) {
      const L = s1.l;
      const rHead = s0.r;
      const rTail = s1.r;
      diffs.push(
        {
          id: mkDiffId('text', `modified:${L.section}:${L.paragraph}`),
          kind: 'text',
          severity: 'modified',
          path: preferRightPath(L, rHead),
          title: `텍스트 변경 (${formatParaLocTitle(rHead)})`,
          leftPreview: L.text,
          rightPreview: rHead.text,
          leftAnchor: L.anchor,
          rightAnchor: rHead.anchor,
          leftSectionPage: L.sectionPage,
          rightSectionPage: rHead.sectionPage,
        },
        ...(shouldSuppressNoiseParagraphOnly(rTail)
          ? []
          : [
              {
                id: mkDiffId('text', `added:${rTail.section}:${rTail.paragraph}`),
                kind: 'text' as const,
                severity: 'added' as const,
                path: { section: rTail.section, paragraph: rTail.paragraph },
                title: `문단 추가 (${formatParaLocTitle(rTail)})`,
                leftPreview: '',
                rightPreview: rTail.text,
                rightAnchor: rTail.anchor,
                rightSectionPage: rTail.sectionPage,
                contextOnLeft: { section: L.section, paragraph: L.paragraph },
              },
            ]),
      );
      i += 2;
      continue;
    }

    if (s0.kind === 'r') {
      const r = s0.r;
      if (!shouldSuppressNoiseParagraphOnly(r)) {
        const ctxL = nearestAlignedLeftPeer(steps, i);
        diffs.push({
          id: mkDiffId('text', `added:${r.section}:${r.paragraph}`),
          kind: 'text',
          severity: 'added',
          path: { section: r.section, paragraph: r.paragraph },
          title: `문단 추가 (${formatParaLocTitle(r)})`,
          leftPreview: '',
          rightPreview: r.text,
          rightAnchor: r.anchor,
          rightSectionPage: r.sectionPage,
          ...(ctxL ? { contextOnLeft: { section: ctxL.section, paragraph: ctxL.paragraph } } : {}),
        });
      }
      i += 1;
      continue;
    }
    if (s0.kind === 'l') {
      const l = s0.l;
      if (!shouldSuppressNoiseParagraphOnly(l)) {
        const ctxR = nearestAlignedRightPeer(steps, i);
        diffs.push({
          id: mkDiffId('text', `removed:${l.section}:${l.paragraph}`),
          kind: 'text',
          severity: 'removed',
          path: preferRightPath(l, null),
          title: `문단 삭제 (${formatParaLocTitle(l)})`,
          leftPreview: l.text,
          rightPreview: '',
          leftAnchor: l.anchor,
          leftSectionPage: l.sectionPage,
          ...(ctxR ? { contextOnRight: { section: ctxR.section, paragraph: ctxR.paragraph } } : {}),
        });
      }
      i += 1;
      continue;
    }

    const l = s0.l;
    const r = s0.r;
    if (l.normalizedText !== r.normalizedText) {
      diffs.push({
        id: mkDiffId('text', `modified:${l.section}:${l.paragraph}`),
        kind: 'text',
        severity: 'modified',
        path: preferRightPath(l, r),
        title: `텍스트 변경 (${formatParaLocTitle(l)})`,
        leftPreview: l.text,
        rightPreview: r.text,
        leftAnchor: l.anchor,
        rightAnchor: r.anchor,
      });
    }

    if (l.signature === r.signature && Math.abs(l.globalIndex - r.globalIndex) > MOVE_DISTANCE_THRESHOLD) {
      diffs.push({
        id: mkDiffId('paragraphMeta', `moved:${l.section}:${l.paragraph}`),
        kind: 'paragraphMeta',
        severity: 'modified',
        path: { section: l.section, paragraph: l.paragraph },
        title: `문단 이동 감지 (${formatParaLocTitle(l)})`,
        leftPreview: `A idx=${l.globalIndex}`,
        rightPreview: `B idx=${r.globalIndex}`,
        leftAnchor: l.anchor,
        rightAnchor: r.anchor,
      });
    }

    if (l.controlCount !== r.controlCount) {
      diffs.push({
        id: mkDiffId('paragraphMeta', `ctrlcount:${l.section}:${l.paragraph}`),
        kind: 'paragraphMeta',
        severity: 'modified',
        path: { section: l.section, paragraph: l.paragraph },
        title: `문단 개체 수 변경 (${formatParaLocTitle(l)})`,
        leftPreview: `controls=${l.controlCount}`,
        rightPreview: `controls=${r.controlCount}`,
        leftAnchor: l.anchor,
        rightAnchor: r.anchor,
      });
    }

    i += 1;
  }
  return diffs;
}

/**
 * 문서 대 문서(alignment) 텍스트 diff + 컨트롤 매칭용 문단 맵.
 *
 * 1. `buildAnchorPairs`: 글로벌 앵커(단조 `ri`, 시그니처 유일, 짧은 줄은 trim 양쪽 1회).
 * 2. 앵커 경계마다 `matchSegment(..., anchorBoundary)`로 `AlignedPair[]` 누적.
 * 3. `buildRightToLeftParaMapFromAligned(aligned)`: 오른쪽 문단 좌표 → 짝 왼쪽 문단(양쪽 non-null만).
 * 4. `buildParagraphAlignStepsFromAligned` → `cleanupParagraphAlignStepsToDiffItems`로 `DiffItem[]` 생성.
 *
 * 반환의 `rightToLeftPara`는 cleanup 이전 `aligned`와 일치한다. 컨트롤 단계는 이 맵으로
 * 물리 `paragraph`가 달라도 같은 논리 문단의 개체를 다시 붙인다(`buildControlDiffs`).
 */
function buildTextDiffs(left: CompareDocumentSnapshot, right: CompareDocumentSnapshot): {
  diffs: DiffItem[];
  rightToLeftPara: Map<string, CompareParaSnapshot>;
} {
  const lps = left.paragraphs;
  const rps = right.paragraphs;
  const anchorTuning = resolveAnchorTuning(activeCompareOptions ?? DEFAULT_COMPARE_OPTIONS);
  const anchors = buildAnchorPairs(lps, rps, anchorTuning);
  if (isCompareDebugEnabled()) {
    compareDbg(
      '[③ 앵커] 시그니처 유일 + 품질필터(길이/공백비율/엔트로피). 빈 줄·패턴 문장이 많으면 구간이 어긋날 수 있음.',
      `앵커 ${anchors.length}쌍 (앞 20개)`,
      anchors.slice(0, 20).map(({ li, ri }) => ({
        li,
        ri,
        left: lps[li].text.slice(0, 44),
        right: rps[ri].text.slice(0, 44),
        normLenL: lps[li].normalizedText.length,
        normLenR: rps[ri].normalizedText.length,
        anchorMinLen: anchorTuning.minTextLen,
      })),
    );
  }
  // 경계: (문서 시작) — 앵커1 — … — 앵커N — (문서 끝). 각 앵커 쌍 자체는 1:1 고정 매칭.
  const boundaries = [{ li: -1, ri: -1 }, ...anchors, { li: lps.length, ri: rps.length }];

  const aligned: AlignedPair[] = [];
  for (let i = 0; i < boundaries.length - 1; i += 1) {
    const a = boundaries[i];
    const b = boundaries[i + 1];

    if (a.li >= 0 && a.ri >= 0) {
      aligned.push({ left: lps[a.li], right: rps[a.ri] });
    }

    // 앵커 (a)와 (b) "사이"의 문단만 DP/그리디에 넘긴다. 앵커 문단 본인은 위에서 이미 짝을 맞춤.
    const leftSeg = lps.slice(a.li + 1, b.li);
    const rightSeg = rps.slice(a.ri + 1, b.ri);
    if (leftSeg.length === 0 && rightSeg.length === 0) continue;
    const anchorBoundary: SegmentAnchorBoundary | null =
      a.li >= 0 && a.ri >= 0
        ? { leftAnchorGi: lps[a.li].globalIndex, rightAnchorGi: rps[a.ri].globalIndex }
        : null;
    aligned.push(...matchSegment(leftSeg, rightSeg, 0, anchorBoundary));
  }

  const rightToLeftPara = buildRightToLeftParaMapFromAligned(aligned);
  const steps = stripNoiseOnlyParagraphAlignSteps(buildParagraphAlignStepsFromAligned(aligned));
  const diffs = cleanupParagraphAlignStepsToDiffItems(steps, lps, rps);
  return { diffs, rightToLeftPara };
}

/**
 * 키가 달라진 뒤에도 **표(table)** 만 모아, 요약 문자열에서 뽑은 키(`txt` / `sig` / 해시) 기준으로
 * 양쪽 문서에서 각각 **정확히 한 개**뿐인 표끼리 1:1 고정한다(Git Patience 유사).
 *
 * `buildControlDiffs`에서는 **정렬 슬롯보다 먼저** 호출한다. 삽입으로 키가 어긋져도 요약이 유일하게 대응되면
 * 같은 내용의 표가 먼저 짝지어져, 슬롯이 위치만 보고 엉뚱한 표를 가져가는 일을 줄인다.
 * 이 단계는 표 요약이 충분히 구별될 때만 효과가 있다.
 */
function extractTablePatiencePins(
  left: CompareControlSnapshot[],
  right: CompareControlSnapshot[],
): { pins: ControlPair[]; restLeft: CompareControlSnapshot[]; restRight: CompareControlSnapshot[] } {
  const pins: ControlPair[] = [];
  const lTables = left.filter((c) => c.type === 'table');
  const rTables = right.filter((c) => c.type === 'table');
  if (lTables.length === 0 || rTables.length === 0) {
    return { pins: [], restLeft: left, restRight: right };
  }
  const keyOf = (c: CompareControlSnapshot): string => {
    const kv = parseSummaryKV(c.summary);
    const t = (kv.txt ?? '').trim();
    if (t) return `t:${t}`;
    const sg = (kv.sig ?? '').trim();
    if (sg) return `s:${sg}`;
    return `h:${simpleHash(c.summary)}`;
  };
  const lByKey = new Map<string, CompareControlSnapshot[]>();
  const rByKey = new Map<string, CompareControlSnapshot[]>();
  for (const c of lTables) {
    const k = keyOf(c);
    const arr = lByKey.get(k);
    if (arr) arr.push(c);
    else lByKey.set(k, [c]);
  }
  for (const c of rTables) {
    const k = keyOf(c);
    const arr = rByKey.get(k);
    if (arr) arr.push(c);
    else rByKey.set(k, [c]);
  }
  const usedL = new Set<CompareControlSnapshot>();
  const usedR = new Set<CompareControlSnapshot>();
  for (const [k, ls] of lByKey) {
    const rs = rByKey.get(k);
    if (!rs || ls.length !== 1 || rs.length !== 1) continue;
    pins.push({ left: ls[0], right: rs[0] });
    usedL.add(ls[0]);
    usedR.add(rs[0]);
  }
  return {
    pins,
    restLeft: left.filter((c) => !usedL.has(c)),
    restRight: right.filter((c) => !usedR.has(c)),
  };
}

/**
 * 문단 정렬 맵(`rightToLeftPara`)을 이용한 컨트롤 **슬롯** 매칭.
 *
 * 전제: `kind::key` 1차 매칭에서 빠진 표·도형·그림이 있다. 오른쪽 컨트롤 `r`의 부모 문단이
 * 맵에서 왼쪽 문단 `L`로 짝지어져 있으면, 후보는 **같은 문단 좌표(`paraPosKey(L)`)에 남아 있는
 * 왼쪽 미매칭 컨트롤**으로 한정한다. 이렇게 하면 위쪽에 표가 삽입되어 y·para 인덱스만 밀린 경우에도
 * “테스트 7 표 vs 테스트 7 표”처럼 짝을 다시 찾을 수 있다.
 *
 * 알고리즘: 오른쪽을 (section, paragraph, controlIdx)순으로 정렬한 뒤, 각 `r`에 대해 버킷 내에서
 * `scoreControlFallback` 최대 + `ALIGNMENT_CONTROL_SLOT_BONUS`가 `ALIGNMENT_CONTROL_MIN_ADJUSTED_SCORE`
 * 이상인 쌍만 채택(탐욕, 한 왼쪽 개체는 한 번만 사용).
 */
function pairAlignmentSlotControls(
  unmatchedLeft: CompareControlSnapshot[],
  unmatchedRight: CompareControlSnapshot[],
  rightToLeftPara: Map<string, CompareParaSnapshot>,
): { pairs: ControlPair[]; restLeft: CompareControlSnapshot[]; restRight: CompareControlSnapshot[] } {
  if (rightToLeftPara.size === 0) {
    return { pairs: [], restLeft: unmatchedLeft, restRight: unmatchedRight };
  }
  const leftByPara = new Map<string, CompareControlSnapshot[]>();
  for (const c of unmatchedLeft) {
    const k = paraPosKey(c);
    if (!leftByPara.has(k)) leftByPara.set(k, []);
    leftByPara.get(k)!.push(c);
  }
  for (const arr of leftByPara.values()) {
    arr.sort((a, b) => (extractControlIndexFromKey(a.key) ?? 0) - (extractControlIndexFromKey(b.key) ?? 0));
  }

  const sortedR = [...unmatchedRight].sort((a, b) => {
    if (a.section !== b.section) return a.section - b.section;
    if (a.paragraph !== b.paragraph) return a.paragraph - b.paragraph;
    return (extractControlIndexFromKey(a.key) ?? 0) - (extractControlIndexFromKey(b.key) ?? 0);
  });

  const usedL = new Set<CompareControlSnapshot>();
  const usedR = new Set<CompareControlSnapshot>();
  const pairs: ControlPair[] = [];

  for (const r of sortedR) {
    const lp = rightToLeftPara.get(paraPosKey(r));
    if (!lp) continue;
    const bucket = leftByPara.get(paraPosKey(lp));
    if (!bucket?.length) continue;
    let best: CompareControlSnapshot | null = null;
    let bestAdj = -1;
    for (const l of bucket) {
      if (usedL.has(l)) continue;
      const raw = scoreControlFallback(l, r);
      if (raw < 0) continue;
      const adj = raw + ALIGNMENT_CONTROL_SLOT_BONUS;
      if (adj > bestAdj) {
        bestAdj = adj;
        best = l;
      }
    }
    if (best && bestAdj >= ALIGNMENT_CONTROL_MIN_ADJUSTED_SCORE) {
      usedL.add(best);
      usedR.add(r);
      pairs.push({ left: best, right: r });
    }
  }

  return {
    pairs,
    restLeft: unmatchedLeft.filter((c) => !usedL.has(c)),
    restRight: unmatchedRight.filter((c) => !usedR.has(c)),
  };
}

/** compareSnapshots 등에서 options 일부가 빠졌을 때의 기본값. kinds는 UI 노이즈를 줄이기 위해 화이트리스트 형태 */
const DEFAULT_COMPARE_OPTIONS: CompareOptions = {
  caseSensitive: true,
  ignoreWhitespace: true,
  kinds: ['text', 'table', 'shape', 'image', 'chart', 'paragraphMeta'],
};

/** compareSnapshots 실행 중 `matchCost`/`resolveAnchorTuning` 등이 읽는 전역(스레드 안전은 요구하지 않음) */
let activeCompareOptions: CompareOptions | null = null;

// ─── 컨트롤 diff: 키 매칭 → 표 patience 핀 → 정렬 슬롯 → 폴백 → added/removed ───

/**
 * 표·도형·그림 등 컨트롤 diff. 단계:
 *
 * 1. **키 정확 매칭** — `kind::key`(우선 `sid:paraStableId:ci:type`). 좌우 동시 존재하면 내용 비교 후
 *    `buildGranularControlDiffs` 또는 동일 시 생략. 한쪽만 있으면 unmatched 버퍼.
 * 2. **표 Patience(선행)** — `extractTablePatiencePins`(요약 키가 양쪽에서 각각 유일할 때).
 * 3. **정렬 슬롯**(`rightToLeftPara`가 비어 있지 않을 때) — `pairAlignmentSlotControls`.
 *    짝이 맞고 요약·종류까지 같으면 diff 생략(순수 밀림), 다르면 `align-slot:` 스템으로 상세 diff.
 * 4. **전역 폴백** — `pairControlsFallback`(임계 2.75). 동일 요약·타입이면 생략.
 * 5. 남은 항목은 added / removed.
 *
 * `identity` 전략에서는 `rightToLeftPara`가 비어 3단계 슬롯만 no-op이며, 2단계 Patience는 그대로 적용된다.
 */
function buildControlDiffs(
  left: CompareDocumentSnapshot,
  right: CompareDocumentSnapshot,
  rightToLeftPara?: Map<string, CompareParaSnapshot>,
): DiffItem[] {
  const diffs: DiffItem[] = [];
  const lmap = new Map(left.controls.map((c) => [`${c.kind}::${c.key}`, c] as const));
  const rmap = new Map(right.controls.map((c) => [`${c.kind}::${c.key}`, c] as const));
  const unmatchedLeft: CompareControlSnapshot[] = [];
  const unmatchedRight: CompareControlSnapshot[] = [];

  // 1차: key 기반 정확 매칭
  const keys = new Set([...lmap.keys(), ...rmap.keys()]);
  for (const key of keys) {
    const l = lmap.get(key);
    const r = rmap.get(key);
    if (l && r) {
      if (l.summary !== r.summary || l.kind !== r.kind) {
        diffs.push(...buildGranularControlDiffs(l, r, `modified:${key}`));
      }
      continue;
    }
    if (l) unmatchedLeft.push(l);
    if (r) unmatchedRight.push(r);
  }

  const {
    pins: tablePins,
    restLeft: afterPatienceL,
    restRight: afterPatienceR,
  } = extractTablePatiencePins(unmatchedLeft, unmatchedRight);
  for (const { left: l, right: r } of tablePins) {
    if (l.summary !== r.summary || l.kind !== r.kind) {
      diffs.push(...buildGranularControlDiffs(l, r, `table-pin:${l.key}=>${r.key}`));
    }
  }

  let slotRestL = afterPatienceL;
  let slotRestR = afterPatienceR;
  if (rightToLeftPara && rightToLeftPara.size > 0) {
    const slot = pairAlignmentSlotControls(afterPatienceL, afterPatienceR, rightToLeftPara);
    for (const { left: l, right: r } of slot.pairs) {
      if (l.summary !== r.summary || l.kind !== r.kind) {
        diffs.push(...buildGranularControlDiffs(l, r, `align-slot:${l.key}=>${r.key}`));
      }
    }
    slotRestL = slot.restLeft;
    slotRestR = slot.restRight;
  }

  // 2차: key가 달라진 컨트롤(특히 표/이미지)의 폴백 매칭
  const fallbackPairs = pairControlsFallback(slotRestL, slotRestR);
  const pairedL = new Set(fallbackPairs.map((p) => p.left));
  const pairedR = new Set(fallbackPairs.map((p) => p.right));
  for (const { left: l, right: r } of fallbackPairs) {
    // 매칭 키만 달라지고 내용/속성이 동일하면 "밀림 보정" 성격의 재매칭이므로 결과에서 제외한다.
    if (l.summary === r.summary && l.type === r.type && l.kind === r.kind) continue;
    diffs.push(...buildGranularControlDiffs(l, r, `fallback-modified:${l.key}=>${r.key}`));
  }

  for (const r of slotRestR) {
    if (pairedR.has(r)) continue;
    const key = `${r.kind}::${r.key}`;
    diffs.push({
      id: mkDiffId(r.kind, `added:${key}`),
      kind: r.kind,
      severity: 'added',
      path: { section: r.section, paragraph: r.paragraph, controlKey: key },
      title: `${kindLabel(r.kind)} 추가`,
      leftPreview: '',
      rightPreview: r.summary,
      rightAnchor: r.anchor,
    });
  }
  for (const l of slotRestL) {
    if (pairedL.has(l)) continue;
    const key = `${l.kind}::${l.key}`;
    diffs.push({
      id: mkDiffId(l.kind, `removed:${key}`),
      kind: l.kind,
      severity: 'removed',
      path: { section: l.section, paragraph: l.paragraph, controlKey: key },
      title: `${kindLabel(l.kind)} 삭제`,
      leftPreview: l.summary,
      rightPreview: '',
      leftAnchor: l.anchor,
    });
  }
  return diffs;
}

/**
 * 키·표 Patience·슬롯 이후에도 남은 좌·우 컨트롤을 **전역 탐욕**으로 짝지음(오른쪽 각각에 대해 왼쪽 후보 중 최고점).
 *
 * `bestScore >= 2.75` 미만이면 매칭하지 않는다 — 절대 좌표·요약 유사도만으로 오매칭하면 added/removed 노이즈가 커지기 때문.
 * 문단 정렬이 신뢰되는 경우에는 `pairAlignmentSlotControls`가 같은 논리 문단 안에서 점수 보너스를 준다.
 */
function pairControlsFallback(
  left: CompareControlSnapshot[],
  right: CompareControlSnapshot[],
): ControlPair[] {
  const pairs: ControlPair[] = [];
  const usedL = new Set<number>();
  for (let ri = 0; ri < right.length; ri += 1) {
    const r = right[ri];
    let bestLi = -1;
    let bestScore = -1;
    for (let li = 0; li < left.length; li += 1) {
      if (usedL.has(li)) continue;
      const l = left[li];
      const score = scoreControlFallback(l, r);
      if (score > bestScore) {
        bestScore = score;
        bestLi = li;
      }
    }
    if (bestLi >= 0 && bestScore >= 2.75) {
      usedL.add(bestLi);
      pairs.push({ left: left[bestLi], right: r });
    }
  }
  return pairs;
}

/**
 * 표 한 쌍의 “같은 표에서 셀만 바뀜” 정도를 0~1로 본다. 행·열(r/c)이 다르면 낮게 나와 전액 감점된다.
 */
function tableControlPairContentSimilarity(l: CompareControlSnapshot, r: CompareControlSnapshot): number {
  const lk = parseSummaryKV(l.summary);
  const rk = parseSummaryKV(r.summary);
  const sameGrid = (lk.r ?? '') === (rk.r ?? '') && (lk.c ?? '') === (rk.c ?? '');
  if (!sameGrid) return 0.08;

  const pickCells = (kv: Record<string, string>) => {
    const cp = kv.cprev;
    if (cp && cp !== '(없음)') return cp;
    const tp = kv.tprev;
    if (tp && tp !== '(없음)') return tp;
    return kv.txt ?? '';
  };
  const a = pickCells(lk);
  const b = pickCells(rk);
  if (a || b) return textSimilarity(a, b);

  const shaL = lk.csha ?? '';
  const shaR = rk.csha ?? '';
  if (shaL && shaR && shaL === shaR) return 1;

  return textSimilarity(l.summary.replace(/\s+/g, ' '), r.summary.replace(/\s+/g, ' '));
}

function tableSummaryMismatchPenaltyFactor(l: CompareControlSnapshot, r: CompareControlSnapshot): number {
  const sim = tableControlPairContentSimilarity(l, r);
  if (sim >= TABLE_PAIR_SIM_NO_PENALTY) return 0;
  if (sim <= TABLE_PAIR_SIM_FULL_PENALTY) return 1;
  return (TABLE_PAIR_SIM_NO_PENALTY - sim) / (TABLE_PAIR_SIM_NO_PENALTY - TABLE_PAIR_SIM_FULL_PENALTY);
}

/**
 * `pairControlsFallback` / `pairAlignmentSlotControls` 공통 점수 함수.
 *
 * - kind 불일치 → -1 (즉시 탈락).
 * - 동일 `type`, 동일 `summary`에 큰 가중(표·도형은 요약 문자열에 구조·텍스트 요약이 들어 있음).
 * - **표끼리 요약이 다르면** `TABLE_SUMMARY_MISMATCH_PENALTY`를 **내용 유사도에 비례해** 감점(같은 그리드·거의 같은 셀 문자열이면 감점 없음).
 * - 그다음 `sid`/`loc` 키에서 뽑은 **control index** 일치, 같은 section, 페이지 간격, 앵커 박스 x/y/wh 근접.
 *
 * 문단 삽입으로 y가 크게 밀리면 위치 항만으로는 2.75에 못 미칠 수 있어, alignment 맵이 있을 때는
 * 슬롯 단계에서 보너스를 더해 같은 문제를 완화한다.
 */
function scoreControlFallback(l: CompareControlSnapshot, r: CompareControlSnapshot): number {
  if (l.kind !== r.kind) return -1;
  let score = 0;
  if (l.type === r.type) score += 1.2;
  if (l.summary === r.summary) score += 2.6;
  else {
    const ld = l.summary.toLowerCase();
    const rd = r.summary.toLowerCase();
    if (ld.includes('table') && rd.includes('table')) score += 1.0;
    if (ld.includes('shape') && rd.includes('shape')) score += 0.8;
  }
  const lci = extractControlIndexFromKey(l.key);
  const rci = extractControlIndexFromKey(r.key);
  if (lci != null && rci != null && lci === rci) score += 1.15;
  if (l.section === r.section) score += 0.45;
  const pageGap = Math.abs(l.anchor.pageIndex - r.anchor.pageIndex);
  if (pageGap === 0) score += 0.9;
  else if (pageGap === 1) score += 0.45;
  const xGap = Math.abs(l.anchor.x - r.anchor.x);
  const yGap = Math.abs(l.anchor.y - r.anchor.y);
  const wGap = Math.abs(l.anchor.width - r.anchor.width);
  const hGap = Math.abs(l.anchor.height - r.anchor.height);
  if (xGap < 120) score += 0.35;
  else if (xGap < 260) score += 0.18;
  if (yGap < 80) score += 0.6;
  else if (yGap < 180) score += 0.3;
  if (wGap < 45 && hGap < 45) score += 0.45;
  else if (wGap < 120 && hGap < 120) score += 0.22;
  if (l.kind === 'table' && r.kind === 'table' && l.summary !== r.summary) {
    const factor = tableSummaryMismatchPenaltyFactor(l, r);
    score -= TABLE_SUMMARY_MISMATCH_PENALTY * factor;
  }
  return score;
}

/**
 * 각 DiffItem에 구역 내 "사람이 읽는" 쪽번호(`sectionPage`)를 붙인다.
 * identity id 패턴·path·controlKey 내 sid를 역추적해 문단을 찾고, 실패 시 anchor의 전역 pageDisplayNumbers로 보완.
 */
function annotateDiffSectionPages(
  diffs: DiffItem[],
  left: CompareDocumentSnapshot,
  right: CompareDocumentSnapshot,
): void {
  const lByPos = new Map<string, CompareParaSnapshot>();
  const rByPos = new Map<string, CompareParaSnapshot>();
  const lByStable = new Map<string, CompareParaSnapshot>();
  const rByStable = new Map<string, CompareParaSnapshot>();
  for (const p of left.paragraphs) {
    lByPos.set(`${p.section}:${p.paragraph}`, p);
    lByStable.set(p.stableId, p);
  }
  for (const p of right.paragraphs) {
    rByPos.set(`${p.section}:${p.paragraph}`, p);
    rByStable.set(p.stableId, p);
  }

  for (const d of diffs) {
    let lp: CompareParaSnapshot | undefined;
    let rp: CompareParaSnapshot | undefined;

    const sidMatch = d.id.match(/id-(?:added|removed|modified|moved|ctrlcount):(.+)$/);
    if (sidMatch) {
      const sid = sidMatch[1];
      lp = lByStable.get(sid);
      rp = rByStable.get(sid);
    } else {
      const key = `${d.path.section}:${d.path.paragraph ?? -1}`;
      if (d.severity === 'removed') {
        lp = lByPos.get(key);
      } else if (d.severity === 'added') {
        rp = rByPos.get(key);
      } else {
        lp = lByPos.get(key);
        rp = rByPos.get(key);
      }
    }

    // 컨트롤 diff는 controlKey에 부모 문단 stable_id가 포함될 수 있다.
    // path 기반 문단 매핑이 어긋난 경우 sid로 좌/우 문단을 재식별해 쪽번호를 고정한다.
    if ((!lp || !rp) && d.path.controlKey) {
      const sidMatch = d.path.controlKey.match(/sid:([^:]+):\d+:/);
      if (sidMatch) {
        const sid = sidMatch[1];
        if (!lp) lp = lByStable.get(sid);
        if (!rp) rp = rByStable.get(sid);
      }
    }

    if (!lp && d.contextOnLeft) {
      lp = lByPos.get(`${d.contextOnLeft.section}:${d.contextOnLeft.paragraph}`);
    }
    if (!rp && d.contextOnRight) {
      rp = rByPos.get(`${d.contextOnRight.section}:${d.contextOnRight.paragraph}`);
    }

    if (lp) d.leftSectionPage = lp.sectionPage;
    if (rp) d.rightSectionPage = rp.sectionPage;

    // context 짝 문단이 잡혔는데 구역 쪽번호가 비어 있으면(특정 조판에서 생략) 앵커 기준 표시쪽으로 보강한다.
    if (!d.leftSectionPage && lp?.anchor?.pageIndex !== undefined && left.meta.pageDisplayNumbers) {
      const pn = left.meta.pageDisplayNumbers[lp.anchor.pageIndex];
      if (pn && pn > 0) d.leftSectionPage = pn;
    }
    if (!d.rightSectionPage && rp?.anchor?.pageIndex !== undefined && right.meta.pageDisplayNumbers) {
      const pn = right.meta.pageDisplayNumbers[rp.anchor.pageIndex];
      if (pn && pn > 0) d.rightSectionPage = pn;
    }

    // 문단 매핑 실패(특히 컨트롤/일부 메타) 시에도 렌더 엔진이 계산한 표시 쪽번호를 사용.
    if (!d.leftSectionPage && d.leftAnchor) {
      const pn = left.meta.pageDisplayNumbers?.[d.leftAnchor.pageIndex];
      if (pn && pn > 0) d.leftSectionPage = pn;
    }
    if (!d.rightSectionPage && d.rightAnchor) {
      const pn = right.meta.pageDisplayNumbers?.[d.rightAnchor.pageIndex];
      if (pn && pn > 0) d.rightSectionPage = pn;
    }
  }
}

// ─── 공개 진입점: 스냅샷 비교 세션 생성 ───────────────────────────────────────

/**
 * 이미 파싱된 스냅샷 두 개를 비교해 `CompareSession`을 만든다.
 *
 * 흐름:
 * - `resolveTextCompareStrategy`: 공유 stable_id가 신뢰되면 `identity`, 아니면 `alignment`.
 * - **본문**: `buildIdentityTextDiffs` 또는 `buildTextDiffs`(후자는 `{ diffs, rightToLeftPara }`).
 * - **컨트롤**: `buildControlDiffs(left, right, rightToLeftPara)` — 문단 밀림 보정을 위해 alignment 맵 전달.
 * - **후처리**: `options.kinds` 필터 → `suppressPureReflowMoves` → `annotateDiffSectionPages` →
 *   구역·문단 순 정렬.
 *
 * 성능: `activeRuntimeGuard`로 wall-clock 상한. 초과 시 `matchSegment` 쪽이 그리디 위주로 이탈할 수 있다.
 */
export function compareSnapshots(
  left: CompareDocumentSnapshot,
  right: CompareDocumentSnapshot,
  options: CompareOptions,
): CompareSession {
  activeCompareOptions = options;
  const strategy: CompareStrategy = options.strategy ?? 'alignment';
  const perf = resolvePerformanceTuning(options);
  activeRuntimeGuard = {
    deadline: Date.now() + perf.maxComputeMs,
    bailedOut: false,
  };
  const textMode = resolveTextCompareStrategy(strategy, left, right);

  // compare-debug 켜진 빌드에서만: stable_id 품질(①) → 전략(②) → alignment는 ③ 앵커 로그 참고
  if (isCompareDebugEnabled()) {
    const lmap = buildStableIdMap(left);
    const rmap = buildStableIdMap(right);
    let sharedStable = 0;
    if (lmap && rmap) {
      for (const id of lmap.keys()) {
        if (rmap.has(id)) sharedStable += 1;
      }
    }
    compareDbg('[① stable_id] 스냅샷 요약', {
      left: left.meta.name,
      right: right.meta.name,
      leftParas: left.paragraphs.length,
      rightParas: right.paragraphs.length,
      leftHead: left.paragraphs.slice(0, 10).map((p) => ({
        sec: p.section,
        para: p.paragraph,
        id: p.stableId ? `${p.stableId.slice(0, 14)}…` : '(빈)',
        t: p.text.slice(0, 32),
      })),
      rightHead: right.paragraphs.slice(0, 10).map((p) => ({
        sec: p.section,
        para: p.paragraph,
        id: p.stableId ? `${p.stableId.slice(0, 14)}…` : '(빈)',
        t: p.text.slice(0, 32),
      })),
    });
    compareDbg('[② 전략·폴백]', {
      optionsStrategy: strategy,
      textMode,
      mapsBuildOk: Boolean(lmap && rmap),
      sharedStableIdCount: lmap && rmap ? sharedStable : null,
      path:
        textMode === 'identity'
          ? 'buildIdentityTextDiffs (Map<stableId>, 인덱스 1:1 아님)'
          : 'buildTextDiffs (앵커 + 구간 DP/그리디 — ③ 로그 참고)',
    });
  }

  // 1) 본문: identity면 sid 집합 비교, 아니면 앵커+구간 정렬
  const textBundle =
    textMode === 'identity'
      ? { diffs: buildIdentityTextDiffs(left, right, options.kinds), rightToLeftPara: new Map<string, CompareParaSnapshot>() }
      : buildTextDiffs(left, right);
  const textDiffs = textBundle.diffs;

  // 2) 개체(표/도형 등) 병합 — 문단 정렬 맵으로 밀린 문단 좌표의 표·그림을 같은 슬롯에서 재짝짓기
  const all = [...textDiffs, ...buildControlDiffs(left, right, textBundle.rightToLeftPara)];

  // 3) kinds 필터 + 순수 리플로우 이동 노이즈 제거 후, UI용 쪽번호 주석
  const filtered = suppressPureReflowMoves(
    all.filter((d) => options.kinds.includes(d.kind)),
    left,
    right,
  );
  annotateDiffSectionPages(filtered, left, right);
  if (isCompareDebugEnabled() && activeRuntimeGuard?.bailedOut) {
    compareDbg('[성능 가드레일] 타임버짓 초과로 일부 구간을 greedy/fallback으로 처리했습니다.');
  }
  // 목록 정렬: 구역 → 문단 순으로 탐색하기 쉽게
  filtered.sort((a, b) => {
    const sa = a.path.section ?? 0;
    const sb = b.path.section ?? 0;
    if (sa !== sb) return sa - sb;
    const pa = a.path.paragraph ?? 0;
    const pb = b.path.paragraph ?? 0;
    return pa - pb;
  });

  try {
    return {
      left: left.meta,
      right: right.meta,
      options,
      diffItems: filtered,
      currentDiffIndex: filtered.length > 0 ? 0 : -1,
      generatedAt: Date.now(),
      textCompareStrategyUsed: textMode,
    };
  } finally {
    activeRuntimeGuard = null;
    activeCompareOptions = null;
  }
}

/**
 * 외부 두 파일(bytes): 각각 별도 WASM으로 스냅샷을 만든 뒤 `compareSnapshots`와 동일 파이프라인.
 * 좌·우 stable_id 집합이 보통 겹치지 않아 alignment가 기본이 된다.
 */
export async function compareDocuments(
  leftBytes: Uint8Array,
  leftName: string,
  rightBytes: Uint8Array,
  rightName: string,
  options: CompareOptions,
): Promise<CompareSession> {
  const left = await buildSnapshotFromBytes(leftBytes, leftName, options);
  const right = await buildSnapshotFromBytes(rightBytes, rightName, options);
  return compareSnapshots(left, right, options);
}
