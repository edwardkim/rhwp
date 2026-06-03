# Task #321 v6 최종 보고서 — 문단 border 시각 병합 + inset 정식 반영

## 배경

v5 (`task_m100_321_v5_report.md`) 적용 후 21_언어 page 1 col 0 의 pi=7 본문 테두리
영역에서 시각 회귀 보고:
- pi=6 (빈 문단, bf_id=7) 의 별도 작은 테두리 사각형이 pi=7 main 위에 그려져 "두 가로 띠"로 보임
- pi=7 본문이 border 와 너무 붙어 시각적 경계가 모호 (border_spacing=0, 시각 inset 없음)

devel 기준에서도 동일 → v5 회귀 아닌 **devel 기반 누적 이슈**.

## 진단

- HWP/PDF 는 인접 문단의 stroke (line_type/width/color) 가 동일하면 bf_id 가 달라도
  하나의 사각형으로 시각 병합. 우리 코드는 **bf_id 동일성** 만 검사 → 분리 렌더
- `ParaShape::border_spacing` 필드는 parsing 만 되고 렌더링에 미반영. border_spacing=0
  인 샘플에 default minimum inset 도 없어 text 가 border 에 붙음

## 수정 (Stage 7)

`src/renderer/layout.rs`, `src/renderer/layout/paragraph_layout.rs`:

1. **시각 stroke signature 병합**: 인접 ranges 의 visible stroke (line_type/width/color)
   가 동일하면 bf_id 가 달라도 하나의 group 으로 병합. invisible (`any_w=false`) 끼리는
   병합하지 않아 form-002 등 골든 무회귀.
2. **border_spacing inset 정식 반영**: `para_border_ranges` 튜플에 top/bottom inset
   전달. 그룹 첫 range 의 top, 마지막 range 의 bottom inset 적용. stroke 있을 때
   default 최소 2 px. 인접 다른 border group 과의 충돌 회피를 위해 prev/next touches
   검사 시 inset 0.

## 결과

| 항목 | 수정 전 (v5) | 수정 후 (v6) | 변화 |
|------|--------------|--------------|------|
| 21_언어 p1 col 0 stroke rect 수 | 2 (pi=6 별도 + pi=7-9 main) | **1 (병합)** | ✓ PDF 일치 |
| pi=7 border 안쪽 top inset | 0 px (touch) | **2 px** | ✓ 시각 분리 |
| col 0 / col 1 main rect 높이 | 871.7 / 460.0 | 887.6 / 462.0 | pi=6 흡수 + inset |
| 페이지 수 (7 sample) | 16/20/24/10/1/5/5 | 16/20/24/10/1/5/5 | 모두 유지 |

## 검증

- `cargo test --lib`: **992 passed**, 0 failed
- `cargo test --test svg_snapshot`: **6 passed** (form-002, issue-147, issue-157, issue-267, table-text, deterministic)
- `cargo clippy --release`: **clean**
- 시각 비교 (PDF vs SVG): col 0 단일 border + 적절한 inset, "[1~3]-..." 외부 평문 — PDF 와 일치

## 변경 파일

- `src/renderer/layout.rs`: 시각 stroke signature merge + 인접 touches 검사 + inset 적용
- `src/renderer/layout/paragraph_layout.rs`: push tuple 에 border_spacing[2]/[3] 전달
- `src/renderer/typeset.rs`: v5 (drift 보정) — 변경 없음, v5 commit 과 함께 동반

## 산출 문서

- 수행 계획서: `mydocs/plans/task_m100_321_v6.md`
- 구현 계획서: `mydocs/plans/task_m100_321_v6_impl.md`
- 단계 보고서: `mydocs/working/task_m100_321_stage7.md`
- 최종 보고서: 본 문서

## 관련 이슈

- Task #321 (드리프트 정량화·완화) 의 v5 후속
- v3 정밀화 (#326) 와 무관 (다른 코드 path)
- 외부 기여자 v5 와 동일 사이클로 묶어 한 PR/commit 으로 처리

## 잔여 사항

- `ResolvedBorderStyle::fill_declared` 추가 검토하다가 이번 사이클에서는 미반영 — 동일
  paragraph rendering 에서 stroke + fill 분리 처리가 필요한 시점에 재논의
- table/cell 의 border 렌더링은 본 수정과 무관 (별도 path)
- 표/셀 단위 border merge 일관화는 별도 task 권장
