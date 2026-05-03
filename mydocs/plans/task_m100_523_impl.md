# Task #523 구현 계획서 — exam_science p1→p2 누적 vpos drift

> **상태**: Stage 1 진단 후 deferral 결정. 본 구현 계획서는 참고용 보존. 최종 보고서: `mydocs/report/task_m100_523_report.md`.

## 1. 핵심 코드 위치

| 역할 | 파일 | 라인 |
|------|------|------|
| 문단 시작 spacing_before/after 적용 | `src/renderer/layout/paragraph_layout.rs` | 110-114, 719-720, 765-766, 2689-2691 |
| `has_tac_shape` 분기 line_height 폰트 보정 | `src/renderer/layout/paragraph_layout.rs` | 841-850, 886-889 |
| `line_step` (line_segs ≥ 2 시) | `src/renderer/layout/paragraph_layout.rs` | 285-... |
| `corrected_line_height` 보정식 | `src/renderer/mod.rs` 또는 `paragraph_layout.rs:836` 근방 | (호출처) |
| pagination column-fit 판정 | `src/renderer/typeset.rs` 또는 paginator 모듈 | (Stage 1 에서 식별) |

## 2. 단계 상세

### Stage 1 — 진단 (drift 발원지 식별)

`paragraph_layout.rs` 의 문단 처리 진입/종료 지점에 `RHWP_DEBUG_T523` 가드 로깅:

- 진입 시: `pi`, `y_start`, `spacing_before`, `vpos_first_seg` (HWP 인코딩 기대값)
- 종료 시: `pi`, `y_end`, `expected_y_end` (HWP `vpos_first + line_segs total height`), `drift = y_end - expected_y_end`

`samples/exam_science.hwp` page 1 단 1 (pi=16..29) 의 pi 별 drift 누적 곡선을 표로 수집. drift 가 "단발 점프" 인지 "균등 누적" 인지 판단:

- 단발 점프 → 특정 pi (수식 다발 또는 빈 문단) 가 발원
- 균등 누적 → `corrected_line_height` 또는 spacing 가산 분기 전반의 정밀도 문제

추가로 page 1 column 1 의 column 잔여 fit 판정 로직 위치를 식별하고, pi=30 (3×3 표) 가 column overflow 로 라우팅되는 정확한 분기를 확인.

산출물: `working/task_m100_523_stage1.md` — drift 표, 발원 가설.

### Stage 2 — 수정

Stage 1 결과에 따라 다음 중 1~2 곳에 한정 수정:

- **(가) line_height 폰트 보정** — `paragraph_layout.rs:841-850`. `has_tac_shape && raw_lh > max_fs * 1.5` 분기에서 폰트 기반 `font_lh = max_fs * 1.2` 가 HWP `raw_lh` 보다 크게 산출될 때 차이만큼 누적. 가능하면 `min(font_lh, raw_lh)` 또는 HWP 값 우선.
- **(나) 빈 문단 line_height** — empty paragraph (runs=0) 의 line_height 가 HWP `line_segs.line_height` 와 다를 경우 강제 일치.
- **(다) spacing 중복 가산** — HWP vpos 가 이미 spacing 을 반영했음에도 `spacing_before`/`spacing_after` 를 추가 적용하는 경우. `is_column_top` 플래그처럼 추가 가드.

수정은 가능한 한 좁은 분기 (해당 case 만 만족하는 조건절 안) 으로 한정하여 다른 샘플 회귀 차단.

산출물: 코드 변경 + `working/task_m100_523_stage2.md` (수정 요약).

### Stage 3 — 검증

게이트:

1. `cargo build --release && cargo test --release` 모두 통과
2. `cargo clippy --release -- -D warnings` 통과
3. `scripts/svg_regression_diff.sh build HEAD~1 HEAD` 회귀 diff:
   - exam_science 변경 페이지: p1, p2 (의도된 정정)
   - 그 외 6 샘플 0 회귀
4. 시각 검증:
   - `samples/exam_science.hwp` page 1 안에 `<보기>` + 답안 정상 수용
   - page 2 가 문제 7 부터 시작 (PDF 일치)
5. 회귀 의심 샘플 추가 확인: `tac-img-02.hwpx`, `2010-01-06.hwpx`, `KTX-*` (수식 다발 경향)

산출물: `working/task_m100_523_stage3.md`.

### Stage 4 — 마감

- `report/task_m100_523_report.md` 최종 보고서 (수정 요약 + 누적 drift 측정값 before/after)
- `orders/20260502.md` 추가 처리 행 갱신
- 진단 로깅 정리 (정식 가드 내부로 보존 또는 제거)

## 3. 영향 범위

- 수정 대상은 `paragraph_layout.rs` 의 1~2 분기.
- 영향 받는 케이스: 동일 분기에 진입하는 모든 문단 → 회귀 diff 로 광범위 검증.
- pagination 자체는 수정하지 않음 (drift 제거 시 자연 해소 가설).

## 4. 위험 / 회피

- `has_tac_shape` 분기 변경은 인라인 수식이 다수인 모든 시험지/문제집에 영향 → 회귀 diff 의 KTX/exam_math/synam-001 케이스 점검.
- spacing 가드 추가는 중첩 표/머리말/꼬리말의 spacing 동작에 영향 가능 → 회귀 diff + tac-img-02 등 점검.
- Stage 1 에서 발원지가 명확하지 않거나 다발적이면, task 를 분리하여 단일 발원지부터 처리 (본 task 는 단발 fix 한정).

## 5. 비-목표

- 다른 페이지/샘플의 drift 는 본 task 범위 외.
- pagination 알고리즘 자체 (column-fit 판정) 은 수정하지 않음.
- Layout Refactor Phase 3 종합 작업으로 확장하지 않음.
