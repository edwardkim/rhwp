# 구현 계획서 — Task #1377 v2: 빈 TAC-수식 spacer phantom trailing 정합

- **이슈**: #1377 (M100) / 브랜치 `local/task1377` (Stage5 `70e2a118` 위)
- **수행계획서**: `mydocs/plans/task_m100_1377_v2.md` (승인됨)
- **단계**: 4단계. 각 단계 완료 후 승인 요청.

## 대상 코드

`src/renderer/layout/paragraph_layout.rs:4724~4748` — 미주 흐름(`endnote_line_vpos_base.is_some()`)
para 마지막 줄 trailing 계산. 현재:
```rust
let trailing = if line_idx + 1 < end
    || self.endnote_para_has_same_endnote_successor(para_index) {
    line_spacing_px            // ← pi=1128: 1984HU=26.45px (phantom between-notes 마커)
} else { 0.0 };
y + line_flow_height + trailing + tac_picture_label_extra
```

## Stage 1 — 판별 게이트 + 치환값 구현

paragraph_layout 미주 trailing 분기에 **phantom between-notes 마커 치환** 추가:

- **게이트 3조건 (AND)**:
  1. `para_is_treat_as_char_equation_only(para)` (빈 텍스트 + TAC 수식 only)
     — 헬퍼는 height_cursor.rs 에 기존 정의, paragraph_layout 에 동등 로컬 추가 or 공용화.
  2. `(line_spacing_hu − self.endnote_between_notes_hu.get()).abs() ≤ tol` (예 tol=120HU)
     — 마지막 줄 line_spacing 이 between-notes 마커와 근사(현 line_spacing_px 역산 HU).
  3. `self.endnote_para_has_same_endnote_successor(para_index)` (미주 내부 = 마커가 phantom).
- **치환값 (A)**: phantom 마커 대신 base-font 정상 line_spacing 산출.
  `corrected_line_metrics(raw_lh, base_ls_px, max_fs, ls_type, ls_val)` 또는 동등 계산으로
  150%-Percent base 줄간격(이웃 텍스트줄 = 452HU 목표) 도출 후 trailing 으로 사용.
  - 산출 곤란 시 1차 폴백: 직전/직후 미주 텍스트 줄의 line_spacing(=452) 참조.
- **격리 검증 (코드 무수정 도구)**: `RHWP_EN_SSOT_DEBUG=1 export-svg` 로 sep2020
  - pi=1128 advance 54.1 → **33.6±0.5**
  - pi=1129 시작 72.1 → **51.7±0.5**, pi=1131 bottom 445.3 → **418.9±0.5**
- 본 단계는 **fidelity 달성만** 확인. 전체 회귀는 Stage 2.
- 산출물: 코드 + `task_m100_1377_v2_stage1.md`(검증 수치) → 커밋.

## Stage 2 — 전체 회귀 가드

- `cargo test` **전체** 실행(--lib 아님). 핵심 가드:
  - `issue_1139_inline_picture_duplicate`(미주 PDF-frame 19건)
  - `height_cursor` 단위 26건, `issue_1082_endnote_multicolumn_drift` 4건
  - golden SVG / svg_snapshot
- 결과 분류: 무회귀 / 회귀 목록·원인(어떤 미주 para 가 게이트에 잘못 걸렸는지 EN_RENDER 대조).
- 산출물: `task_m100_1377_v2_stage2.md`(전체 결과 + 회귀 분석) → 커밋.

## Stage 3 — 판정 (무회귀 확정 / 회귀 미세조정 1회 / revert)

- **무회귀**: Stage1 코드 확정. 다음 단계.
- **회귀**: 게이트를 1회 미세조정(예 tol 축소, 치환값 (A)→폴백, 추가 조건) 후 Stage2 가드 재실행.
  - 재가드도 회귀 시 **revert**(Stage5 와 동일, "좁힌 접근도 불가" 정직 기록).
- 산출물: `task_m100_1377_v2_stage3.md`(판정 근거) → 커밋.

## Stage 4 — 최종 보고서

- `mydocs/report/task_m100_1377_v2_report.md`: v2 결과(성공/불가) + 수치 + 교훈.
- 메모리 `project_endnote_render_fidelity_plumbing` 갱신. orders/ 는 미변경(`feedback_orders_no_update`).
- 산출물 커밋 + `git status` 확인.

## 가드 원칙

- 회귀 0 이 절대 조건(Stage5 교훈). fidelity 이득만으로 1건이라도 PDF-frame/golden 회귀 시 채택 불가.
- 각 단계 후 승인 없이 다음 단계 진행 금지.
