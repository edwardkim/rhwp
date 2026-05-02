# Layout 리팩터링 로드맵 — 분석 및 단계 정의

**작성일**: 2026-05-02
**범위**: `src/renderer/layout/` + 관련 모듈 (12,509 LOC)
**동기**: #467 / #491 / #496 보류 사유 "layout 리팩터링 시 종합 해결" 의 구체화

## 1. 모듈 현황

| 파일 | LOC | 주요 책임 |
|------|-----|-----------|
| `layout.rs` | 3,471 | LayoutEngine 진입점, 페이지/단/문단 라우팅 |
| `layout/paragraph_layout.rs` | 2,987 | 일반 문단 + 인라인 표 + 각주 marker |
| `layout/table_layout.rs` | 2,412 | 표 셀 paragraph + 셀 내 인라인 컨트롤 |
| `typeset.rs` | 2,586 | paragraph height/page-fit 계산 |
| `composer.rs` | 1,053 | LINE_SEG composition |

총 **12,509 LOC** — 단일 세션 내 안전한 전면 리팩터링은 비현실적.

## 2. 보류 결함 분류 — 본질별 분리

### 2-A. master 컨트롤 다른 apply_to 조합 처리 (#467)

- **위치**: `src/document_core/queries/rendering.rs` (master 매핑 로직)
- **본질**: 확장 바탕쪽의 active master vs ext master 의 apply_to 조합별 동작 미검증
- **layout 모듈과 별개** — master 매핑 코드 영역
- **리스크**: master 처리 변경은 모든 페이지 헤더/푸터 영향, 회귀 위험 큼
- **해결 조건**: HWP 스펙 + 한컴 환경 직접 검증

### 2-B. PDF 좌표 정규화 (#491)

- **위치**: 새 인프라 필요 (현재 미존재)
- **본질**: PDF (A4 595×841pts) ↔ SVG (272×394mm 1028×1488px) 좌표계 차이로 직접 비교 불가
- **layout 모듈과 별개** — 검증 도구 영역
- **해결 조건**: 좌표 정규화 라이브러리 또는 변환 공식 도입

### 2-C. `layout_inline_table_paragraph` multi-row + multi-line 한계 (#496)

- **위치**: `src/renderer/layout/paragraph_layout.rs:81~574` (494 LOC 함수)
- **본질**: 2행+ 인라인 표 + 다중 줄 본문 텍스트 처리 한계
- **범위**: layout 모듈 내 한 함수
- **분석 결과** (아래 3절) 단일 본질이 아닌 복합 결함

## 3. `layout_inline_table_paragraph` 결함 본질 분석

### 3-1. 재현 데이터 (`samples/exam_science.hwp` p2 pi=61)

HWP IR:
```
ls[0]: vpos=74118, lh=2864 (전체 표 = 2행, 2864 HU = 38.19 px)
ls[1]: vpos=77442, text_start=13   (표 다음 본문 첫 줄)
ls[2]: vpos=79052, text_start=60   (본문 둘째 줄)
표:    2행 1열, tac=true, 셀=[의 에 들어 있는 중성자수 | 의 에 들어 있는 중성자수]
       (분수형 인라인 표 — 분자 / 분모)
```

현재 SVG 렌더 baseline (column 1):
| baseline y | 내용 |
|-----------|------|
| 1191.68 | 표 셀 row 0 ("의 에 들어 있는 중성자수") |
| **1195.85** | **본문 "는? (단, ..." (표의 두 행 사이에 끼임)** |
| 1210.77 | 표 셀 row 1 (분모) |
| 1227.21 | 다음 본문 줄 |

→ 본문 baseline 1195.85 가 표 row 0 (1191.68) 과 row 1 (1210.77) **사이**에 끼어 시각적으로 표와 겹침.

### 3-2. 결함 본질 — 단일 본질이 아닌 3가지 복합

#### (A) 본문 baseline 의 수직 정렬 정책 부재

- 현재: 본문 = `current_y + baseline_dist` (paragraph 시작 y + ls[0] baseline)
- 단일행 인라인 표 (ls[0] = 1행짜리): 본문과 표 row 0 baseline 일치 (정상)
- **다중행 인라인 표 (ls[0] = N행짜리)**: 본문이 row 0 위치에 고정, 분수형/스택형 표와 부정합

#### (B) ls[2..] 의 break 미사용

- 현재: `line_break_char_idx` 는 ls[1].text_start 만 사용, 이후 줄바꿈은 `right_margin` 동적 reflow
- 다중 줄 본문에서 HWP 인코딩 break 위치와 어긋날 수 있음

#### (C) 표 inline rendering 정책 — 인라인 vs 블록

- HWP/PDF 동작: 분수형 다중행 표는 fraction glyph 처럼 본문 옆에 인라인 렌더 (현재와 같은 방향)
- 다른 다중행 표 (예: 2행 데이터 표): 별도 블록으로 처리하는 게 자연스러움
- 어떤 케이스가 어디 가는지 **정책 부재**

## 4. 리팩터링 단계 정의 (제안)

### Phase 1 — 분석 인프라 (선결, 1 세션)

**목표**: 결함 측정·재현·회귀 검증 자동화

- [ ] `RHWP_LAYOUT_DEBUG=1` 환경변수로 paragraph 별 baseline / line_seg vpos 로그 출력 도구 추가
- [ ] 다중 샘플 byte-diff 도구 정형화 (현재 ad-hoc)
- [ ] PDF 좌표 정규화 (#491) 인프라 추가 — A4 좌표 → mm → SVG px 변환

**산출**: 본질 결함 측정 도구 + 문서화

### Phase 2 — `layout_inline_table_paragraph` 협소 정정 (1 세션)

**대상**: 결함 본질 (B) ls[2..] break 사용

- [ ] `line_break_char_idx` 를 `Vec<usize>` 로 일반화 (ls[1..N] 지원)
- [ ] wrap 조건에 `&& !wrapped_below_table` 제약 제거 (다중 break 허용)
- [ ] 7 샘플 광범위 byte-diff 회귀 검증

**리스크**: 중간 — `wrapped_below_table` 의 미묘한 흐름 변경

### Phase 3 — 다중행 인라인 표 baseline 정렬 (1~2 세션)

**대상**: 결함 본질 (A) 본문 수직 정렬

- [ ] 인라인 표가 다중행일 때 본문 baseline 을 표 마지막 행 baseline 또는 vertical center 로 변경
- [ ] HWP 스펙 + 한컴 PDF 직접 비교 필요
- [ ] 회귀 위험: 단일행 인라인 표 케이스 (수식 등)

**리스크**: 큼 — 정렬 정책 변경은 광범위 영향

### Phase 4 — 인라인 vs 블록 정책 (장기, 다세션)

**대상**: 결함 본질 (C) 표 inline 정책

- [ ] HWP 스펙 + 한컴 동작 분석으로 다중행 표의 인라인/블록 결정 룰 정의
- [ ] 분기 코드 추가 + 광범위 회귀

**리스크**: 매우 큼

## 5. 본 세션 결정

**Phase 0** (현재 세션) — **분석 + 로드맵 문서화** 만 수행.

이유:
1. 결함 본질 (A)/(B)/(C) 가 복합. 단일 패치로 해결 어려움
2. 광범위 회귀 검증 자동화 (Phase 1) 가 선결 — 이 없이 Phase 2/3/4 시도는 위험
3. 메모리 가이드 `feedback_essential_fix_regression_risk.md`: 본질 정정은 다단/단일 단/표분할 상호작용으로 회귀 위험 큼
4. 메모리 가이드 `feedback_pdf_not_authoritative.md`: PDF 비교만으로 정답 단정 어려움

후속 세션에서 Phase 1 부터 단계적으로 진행 권고.

## 6. 산출물

- 본 로드맵 문서 (`mydocs/tech/layout_refactor_roadmap.md`)
- 코드 변경: 없음
