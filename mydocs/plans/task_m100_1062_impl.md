# 구현계획서 — Task #1062: 다단 빈 문단 연속 trailing-ls drift

- 이슈: edwardkim/rhwp#1062
- 브랜치: `local/task1062`
- 수행계획서: `task_m100_1062.md` (승인 완료)

## 관련 코드 (조사 기반)

- 렌더러 bridge: `src/renderer/height_cursor.rs:114-161`
  - lazy_base 확립 시 `trailing_ls_hu` 결정 (138-146행).
  - #1049 가드: `vpos_continuous && prev_has_text` → bridge off(0), 아니면 trailing_ls 적용.
  - `vpos_lazy_base` 는 height_cursor 내에서 **설정만**(158행).
- 페이지네이터: `src/renderer/typeset.rs`
  - 다단 누적 `height_for_fit`(trailing_ls 제외): 1834-1841행 (Task #391).
  - `vpos_lazy_base` 리셋/동기화: 244 / 1089 / 1515 / 1523 / 1529행.
- 진단: `RHWP_TYPESET_DRIFT`(typeset.rs:1692~), `RHWP_VPOS_DEBUG`(height_cursor.rs:198~).

## 단계 (4단계)

### Stage 1 — 원인 정밀화 + 구분 신호 측정 (소스 변경 없음)

- `RHWP_VPOS_DEBUG` / `RHWP_TYPESET_DRIFT` 로 다음을 측정·대조표 작성:
  - 시험지 runaway 컬럼(예: 3-09 2022 page 18, pi 994~1081): lazy_base 값·리셋 빈도,
    `trailing_ls_hu` 적용 여부, `vpos_continuous`/`prev_has_text`, vpos_end 추이, end_y vs col_bottom.
  - 비회귀 케이스: 복학원서 page1, footnote-01 p1, exam_kor p5, exam_eng 8p, k-water-rfp p3.
- **확정 목표**: 문단당 +6px 누적이 (a) lazy_base 반복 리셋 때문인지 (b) vpos_end 흐름 때문인지
  단정. 시험지 빈 문단 연속과 복학원서 빈 문단 케이스를 **가르는 신호** 후보 식별.
- 산출물: `mydocs/working/task_m100_1062_stage1.md` (대조표 + baseline 수치).
- 커밋: 보고서만 (소스 무변경).

### Stage 2 — 구분 조건 설계 (페이퍼 검증)

- Stage 1 신호로 "trailing-ls bridge off 대상(시험지)" vs "bridge 유지(복학원서 등)" 분리 조건 1~2개 도출.
  후보 예: 빈 문단 run 길이·후행 항목 유형(표/폼 vs 본문 문단)·vpos 연속+빈문단 동시 신호 등.
- 5개 비회귀 케이스 + 대상 4종에 대한 **예상 동작 표**로 페이퍼 검증(코드 변경 전 모순 점검).
- 산출물: `mydocs/working/task_m100_1062_stage2.md`.
- 커밋: 보고서만.

### Stage 3 — 구현

- `height_cursor.rs:138-146` 의 `trailing_ls_hu` 결정에 Stage 2 조건 반영 (변경 최소화).
  필요 시 `typeset.rs` 동기화 지점 정합 — HWP3 전용 분기 추가 금지.
- 단위 TDD: 빈 문단 연속 drift 0 검증 테스트 추가, 복학원서/exam_kor 보존 테스트 유지.
- 산출물: 소스 + `mydocs/working/task_m100_1062_stage3.md`.
- 커밋: 소스 + stage3 보고서 (cargo fmt 변경 파일만).

### Stage 4 — 회귀 검증 + 정합

- 대상 4종(×2포맷): LAYOUT_OVERFLOW 대폭 감소 + `pdf/3-09월_교육_통합_2022.pdf` 등 쪽수·배치 정합.
- 비회귀 5케이스 전수: 복학원서·footnote-01·exam_kor·exam_eng·k-water-rfp.
- `cargo test --release` 0 failed, 골든 SVG 회귀 0, `cargo fmt --check`(변경 파일).
- 전 251 샘플 LAYOUT_OVERFLOW 합계 회귀 점검(devel 1624 대비 악화 없음).
- 산출물: `mydocs/working/task_m100_1062_stage4.md` → 이후 최종보고서 `report/task_m100_1062_report.md`.

## 완료 기준

- 대상 4종 본문 하단 overflow 해소(쪽수 PDF 정합) + 비회귀 0 + 골든 SVG 회귀 0.

## 리스크

- 복학원서(빈 문단 뒤 bridge 유지)와 시험지(제거)가 충돌 — Stage 2 페이퍼 검증에서
  분리 불가로 판명 시, 조건 재설계 또는 범위 재협의(작업지시자 승인).
