# Task #321 v5 최종 보고서 — pi=0 block-table drift 보정

## 배경

`samples/21_언어_기출_편집가능본.hwp` page 1 이 PDF 원본과 col 1 시작 문단이 다른 회귀 보고
("21_언어_기출_편집가능본.pdf 처럼 나와야 함"). 조사 결과 task321 v3-정밀화 회귀가 아닌
**devel branch 부터 존재하던 누적 drift** 임이 확인됨.

## 진단 (Stage 5)

- 21_언어 page 1 col 0 drift = **+85.8 px** (= 우리 typeset 이 HWP LINE_SEG 위치 보다 86 px 더 사용)
- pi=1..pi=9 본문은 HWP 와 정확히 일치 (per-paragraph 누적 오차 0)
- **drift 76.3 px 의 출처는 pi=0 단독** — 4×5 폼 표(wrap=TopAndBottom, vert_rel_to=Paper)
  를 포함한 첫 문단의 block-table layout 이 본문 LINE_SEG 좌표계와 mismatch
- 진단 로그 강화: `RHWP_TYPESET_DRIFT` env-gated `TYPESET_DRIFT_PI` 라인 추가
  (sb/sa/lines/lh_sum/ls_sum/trail_ls/diff)

## 수정 (Stage 6)

`typeset_block_table` 의 fits 분기 앞에, 다음 조건일 때 cur_h 를 HWP first_vpos 로 jump 하고
표 effective_height 를 cur_h advance 에서 제외하는 가드 추가:

- `!treat_as_char`
- `wrap = TopAndBottom`
- `vert_rel_to = Paper`
- `current_column == 0`

표는 layout 단의 Paper-anchored 절대 좌표 경로로 원래 위치에 그려지므로 시각 무변경.

## 결과

| 항목 | 수정 전 | 수정 후 | 변화 |
|------|---------|---------|------|
| 21_언어 p1 col 0 drift | +85.8 px | +9.5 px | **-76.3** |
| 21_언어 p1 col 0 마지막 item | PartialParagraph pi=9 lines=0..11 | FullParagraph pi=9 (전체) | ✓ |
| 21_언어 p1 col 1 첫 item | "도출될 경우..." (pi=9 cont.) | **"적합성 검증이란..." (pi=10)** | ✓ PDF 일치 |

## 검증

- `cargo test --lib`: 992 passed, 0 failed
- `cargo test --test svg_snapshot`: 6 passed (form-002, issue-147, issue-157, issue-267,
  table-text, render_is_deterministic_within_process)
- `cargo clippy --release`: clean
- 페이지 수: 21_언어 16, exam_math 20, exam_kor 24, exam_eng 10, exam_math_8 1,
  exam_science 5, exam_social 5 — 모두 유지
- 시각 확인: PDF 와 일치

## 변경 파일

- `src/renderer/typeset.rs`:
  - 진단 로그 강화 (Stage 5a)
  - block-table 호스트 문단 advance 보정 (Stage 6a)

## 산출 문서

- 수행 계획서: `mydocs/plans/task_m100_321_v4.md`, `task_m100_321_v5.md`
- 구현 계획서: `mydocs/plans/task_m100_321_v4_impl.md`, `task_m100_321_v5_impl.md`
- 진단 보고서: `mydocs/working/task_m100_321_stage5.md`
- 수정 보고서: `mydocs/working/task_m100_321_stage6.md`
- 최종 보고서: 본 문서

## 관련 이슈

- Task #321 (드리프트 정량화·완화) 의 v4/v5 stage
- v3-정밀화 (#326) 의 후속 — 본문 좌표계와 표 좌표계 mismatch 해소

## 잔여 사항

- col 0 drift +9.5 px (= 마지막 trailing_ls) 은 측정 노이즈, 페이지 분할에 영향 없음
- 다른 페이지에서 Paper-anchored TopAndBottom 표가 col 1+ 에 등장하는 케이스 발견 시
  가드 확장 검토 필요 (현 단계에선 해당 패턴 없음)
