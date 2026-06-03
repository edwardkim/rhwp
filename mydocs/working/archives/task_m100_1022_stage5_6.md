# Stage 5-6 완료보고서 — #1022: rowspan-split 해소 + 잔여 정리

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 5-6 — 다중 머리행 정합(Stage 5-5) 후 검증 + 잔여 분석

## 1. 누적 성과 (LAYOUT_OVERFLOW 추이)

| 시점 | events | 비고 |
|------|--------|------|
| 베이스라인(8be5e0c2) | 42 | task993 머지 상태 |
| Stage 3 (cell_units↔HeightMeasurer) | 38 | -4 |
| Stage 5-3 (VPOS_CORR over-correction 제거) | 23 | -15, **페이지 22 해소** |
| Stage 5-5 (다중 머리행 overhead 정합) | **12** | -11, **rowspan-split(pi=111/550) 해소** |

**42 → 12 (~71% 감소)**. 주소 가능 오버플로(page-larger 3 제외): **39 → 9 (~77% 감소)**.

## 2. rowspan-split 해소 (사용자 지정 목표)

pi=111/pi=550 (75×10, rs=2 머리행 + rs=49 본문 셀)이 4페이지에 분할될 때
각 연속분에서 본문 초과(10~38px)했던 원인:

- 렌더러 `layout_partial_table` 는 연속분에서 `start_row` 이전 is_header 행을
  **전부** 반복 (rs=2 머리행 → 행 0,1 둘 다).
- 페이지네이터 `header_overhead` 는 **행 0만** 계산 (`header_row_height + cs`).
- 둘째 머리행(row_heights[1] ≈ 40px) 누락 → 연속분마다 본문 초과.

수정(Stage 5-5): `header_overhead` 를 `start_row` 이전 is_header 행 전체
높이 합 + 행당 cs 로 정정. pi=111/550 의 rowspan-split 오버플로 전부 해소.

## 3. 검증

- `cargo build/clippy --release` 무경고.
- `cargo test --release` 1302 passed, 0 failed.
- `svg_snapshot` 8 passed (form-002·issue-617·issue-677 유지).
- 페이지 수 184→185 (다중 머리행 정확 반영으로 1페이지 증가 — 정상).

## 4. 잔여 12건

### 4-1. page-larger (3건, 사전 존재·내 변경 무관)

- pi=272 PartialTable 854.9px (page 40)
- pi=567 PartialParagraph 856.7px (page 93)
- pi=324 PartialTable 143.9px (page 63)

페이지보다 큰 중첩 표/문단 — 내부 분할(`calc_nested_split_rows`) 필요. task993 §4 scope-out. 베이스라인과 동일.

### 4-2. small/med (9건, 이종 micro-case)

| pi | 타입 | px | 추정 원인 |
|----|------|-----|----------|
| 642 | FullParagraph | 19.7 | 인라인 TAC 3개 페이지(pi=630/632/640) 누적 + 마지막줄 trailing_ls 허용(paginator used=949.6>941) |
| 323 | PartialTable | 29.1 | nested-table 셀(cell[6] rs=2 내부표 942px) 보유 7×3 표 분할 |
| 268 | PartialParagraph | 12.3 | 분할 단락 측정 |
| 357 | Table | 10.0 | TAC 인라인 표 측정 |
| 354 | Table | 8.3 | TAC 인라인 표 측정 |
| 600 | PartialTable | 5.5 | 분할 표 잔여 |
| 406 | FullParagraph | 3.1 | 누적/trailing_ls |
| 781 | FullParagraph | 3.0 | 누적/trailing_ls |
| 218 | PartialTable | 2.2 | 분할 표 잔여 |

각각 다른 micro-cause (nested-table-in-split / inline-TAC 측정 / trailing_ls
허용). 공통 단일 원인 없음 — 각 개별 조사 필요. 대부분 ≤12px.

## 5. 평가

- 사용자 지정 목표(rowspan-split) **달성** — pi=111/550 해소.
- 두 systematic 수정(VPOS_CORR over-correction, 다중 머리행)으로 26건 해소.
- 잔여 9건은 ≤29px 의 이종 micro-case + page-larger 3건(사전존재).

본 타스크는 명시 범위(HeightMeasurer↔cell_units) + 확장 범위(VPOS_CORR,
rowspan-split) 를 달성. 잔여 micro-case 는 각 개별 후속 가능.
