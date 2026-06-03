# 최종 결과보고서 — #1022: 측정 정합 (HeightMeasurer ↔ cell_units, VPOS_CORR, rowspan-split)

- 타스크: #1022 (M100)
- 브랜치: `local/task1022` (base `0c9d69c1`)
- 기간: 2026-05-19 ~ 2026-05-20
- 관련: task #993 (split-table cut 모델) 후속

## 1. 배경 / 목표

task #993 에서 split-table 페이지네이션을 px `content_offset` → 이산 line-range
cut(`RowCut`) 모델로 전환했으나, 비공개 184페이지 문서(인공지능 재정통합시스템
제안요청서)에서 **본문 영역을 넘어 렌더링되는(LAYOUT_OVERFLOW) 잔여**가 남았다.
특히 사용자 보고 사안인 **페이지 22 하단 초과**가 핵심이었다.

목표: 페이지네이터와 렌더러의 측정 시스템 정합으로 LAYOUT_OVERFLOW 제거.

## 2. 성과 (LAYOUT_OVERFLOW 추이)

| 시점 | events | 핵심 |
|------|--------|------|
| 베이스라인 (task993 머지) | **42** | — |
| Stage 3 (cell_units ↔ HeightMeasurer) | 38 | 측정 함수 정합 |
| Stage 5-3 (VPOS_CORR over-correction 제거) | 23 | **페이지 22 해소** |
| Stage 5-5 (다중 머리행 overhead 정합) | **12** | **rowspan-split(pi=111/550) 해소** |

**42 → 12 (~71% 감소)**. page-larger 3건(사전존재) 제외한 **주소 가능분 39 → 9 (~77%)**.

## 3. 수정 내역 (3건의 systematic 정합)

### 3-1. cell_units ↔ HeightMeasurer 정합 (Stage 3)

- `src/renderer/layout/table_layout.rs`: `cell_units`(셀 콘텐츠 단위 평탄화),
  `row_cut_content_height`(행 cut 높이 = 셀별 `max(cell.height, content+pad)` 의
  행 최댓값), `advance_row_cut`, `cell_line_ranges_from_cut` 를 단일 권위로 통합.
  trailing-ls 규칙(`!is_cell_last_line || para_count > 1`), 비인라인 컨트롤 filler,
  corrected_line_height 를 HeightMeasurer 와 일치시킴.
- `src/renderer/layout/table_partial.rs`: `layout_partial_table` 가 동일
  `row_cut_content_height`/`cell_line_ranges_from_cut` 사용. 2b 분할행 override
  중복 padding 제거.
- `src/renderer/typeset.rs`: `typeset_block_table` 가 `advance_row_cut`/cut 높이로
  walk.

### 3-2. VPOS_CORR over-correction 제거 (Stage 5-3) — 페이지 22 해소

- `src/renderer/layout.rs`: `y_delta_hu` 계산에서 `+ trailing_ls_hu` 제거.
- 원인: Task #537 의 `+trailing_ls_hu` 는 Task #479 의 trailing-제외 정책 하에
  작성된 stale 보정. Task #452 가 trailing 포함을 복원한 뒤로 **과보정**으로 작동.
- 효과: 페이지 22 하단 18.3px 초과 해소. 공개 골든 issue-677(복학원서)이 5.3px
  상향 → 한컴 2022 PDF(`pdf/복학원서-2022.pdf`) 대조상 **vpos 정합 개선**(본래
  "LineSeg.vpos 정합" 의도에 부합), issue-617 은 부동소수 말단 노이즈. 갱신.

### 3-3. 다중 머리행 overhead 정합 (Stage 5-5) — rowspan-split 해소

- `src/renderer/typeset.rs`: `header_overhead` 를 `start_row` 이전 is_header **전체
  행** 높이 합 + 행당 cs 로 정정 (기존: 행 0만).
- 원인: 렌더러는 연속분에서 rs=2 머리행을 행 0·1 둘 다 반복하나, 페이지네이터는
  행 0만 overhead 계산 → 둘째 머리행(≈40px) 누락 → 연속분마다 초과.
- 효과: pi=111/pi=550(75×10, rs=2 머리행 + rs=49 본문 셀)의 분할 오버플로 전부 해소.

## 4. 검증

- `cargo build/clippy --release` 무경고.
- `cargo test --release` **1302 passed**, 0 failed.
- `svg_snapshot` 8 passed (form-002·issue-617·issue-677 골든 갱신·유지, 한컴 PDF 대조).
- 페이지 수 184→185 (다중 머리행 정확 반영으로 1페이지 증가 — 정상).

## 5. 잔여 12건 (후속 이슈)

공통 원인을 가진 systematic 버그는 더 이상 없으며, 잔여는 marginal 개별 케이스다.

### 5-1. page-larger (4건) — 페이지보다 큰 중첩 콘텐츠
- pi=272 854.9px / pi=567 856.7px / pi=324 143.9px / pi=323 (cell[6] rs=2 내부표 942px)
- 페이지보다 큰 nested 표/문단의 **내부 행 분할**(`calc_nested_split_rows`) 필요.
  task993 §4 scope-out. 별도 후속 타스크 권장.

### 5-2. marginal tight-fit (8건, ≤19.7px)
- pi=354/357: 877·842px 대형 TAC 표 + 헤더 — 헤더+표+spacing 이 본문 marginal 초과
  (분할 불가 TAC 표).
- pi=642: 인라인 TAC 누적 + 마지막 줄 trailing_ls (paginator height_for_fit 의도 허용).
- pi=406/781: 마지막 줄 trailing_ls (≤3px, 의도 허용).
- pi=268 PartialParagraph 12.3px / pi=600·218 분할 표 잔여(≤5.5px).
- 각각 다른 micro-cause, 대부분 ≤12px. over-fit 위험 커 개별 후속 시 신중 권장.

## 6. 결론

사용자 보고 핵심 사안(**페이지 22**)과 확장 목표(**rowspan-split**)를 해소하고,
3건의 systematic 측정 정합으로 LAYOUT_OVERFLOW 를 **42→12(~71%)** 감소시켰다.
회귀 없음(1302 tests pass, 골든 한컴 PDF 대조 유지). 잔여 12건은 page-larger 4 +
marginal tight-fit 8 로, 공통 원인 없는 후속 이슈로 분리한다.

### 후속 권장
- (후속 #) page-larger nested 콘텐츠 내부 분할 (`calc_nested_split_rows`).
- WASM 재빌드 — 릴리즈 시 `docker compose ... run --rm wasm`.
