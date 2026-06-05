# Task #1297: 미주 내 큰 수식 겹침 — 구현계획서

- 이슈: [#1297](https://github.com/edwardkim/rhwp/issues/1297) / 브랜치 `local/task1297`
- 수행계획서: `task_m100_1297.md`
- 작성일: 2026-06-05

## 코드 경로 (조사 결과)

- 인라인 수식 렌더: `src/renderer/layout/paragraph_layout.rs:1829-1885`
  - 수식 AST → `EqLayout`(equation/layout.rs)로 layout_box(높이/baseline) 산출.
  - `eq_h = hwp_eq_h`(HWP 저장 높이 우선), `eq_y = y + baseline - layout_box.baseline*scale`.
  - 수식을 `line_node`의 **자식**으로 push. **그러나 `line_node`의 `line_height`(815-835에서 stored LINE_SEG 기반 산정)는 수식 높이로 확장하지 않음.**
- 줄 높이 산정: 같은 파일 `:806-836` — `max_fs`(런 폰트 최대) + stored `raw_lh` → `corrected_line_height`. 인라인 글상자(Shape)에는 `has_tac_shape && raw_lh > max_fs*1.5` 특례(829)가 있으나 **수식에는 동등 보정 없음**.
- 다음 문단 vpos: stored LINE_SEG vpos 기반. 미주는 줄 step이 작아(≈1352 HU) 2단 분수 수식의 실제 descent를 수용 못함 → 겹침.

## 가설 (Stage 1에서 계측 확정)

본문은 한컴이 LINE_SEG에 수식 포함 line_height를 굽지만, **미주 문단은 수식 높이가 stored line_height/다음 vpos 간격에 미반영**되어, 렌더 수식 descent가 줄 바닥을 넘어 다음 미주 문단을 침범.

## 구현 단계

### 1단계: 근본 원인 계측 확정 (계측 전용)

- 17쪽 미주 수식 문단(s0 [다른 풀이] `13/162`)에서 다음을 계측(임시 eprintln, 커밋 제외):
  - stored `raw_lh`(line_height) px, `max_fs`, `baseline`
  - `EqLayout` layout_box.height/baseline, `hwp_eq_h`, 최종 `eq_y`/`eq_h`/수식 바닥(`eq_y+eq_h`)
  - 다음 문단 y(line top)와 수식 바닥의 침범량(px)
- **본문 동종 수식 문단**과 동일 계측 → 본문은 왜 안 겹치는지(stored lh가 큰지, trailing 흡수인지) 대조.
- trailing 가산 위치 확인(메모리 `tech_trailing_model_no_ssot.md` — 이중가산 회피 조건 파악).
- 산출물: `task_m100_1297_stage1.md` (계측 표 + 침범량 + 본문 대조 + 수정 지점 확정).

### 2단계: 수식 줄 높이 보정 구현

- 인라인 수식이 있는 줄에서, **수식 실제 점유 높이(eq_y+eq_h 기준 줄 바닥 초과분)** 만큼 해당 줄의 effective bottom / 다음 줄 시작을 밀어 겹침 제거.
- Shape 특례(829)와 동일 축의 보정으로 설계하되 **수식 전용 분기** 추가. trailing(line_spacing) 가산과 **독립** — 줄 콘텐츠 높이 축에서만 보정하여 이중가산 회피(#1258 교훈).
- 미주/본문 공통 적용 가능하나, 1단계 계측에서 본문 회귀 위험이 확인되면 **미주 한정 게이트**로 좁힌다.
- 산출물: `task_m100_1297_stage2.md` (수정 diff 요약 + 17쪽 before/after SVG).

### 3단계: 검증 및 무회귀 확인

- 시각: 17쪽 [다른 풀이] `13/162` 다음 줄 분리 (한글 2022 PDF 정합) — PNG band 측정.
- 회귀: `cargo test` 전체 — height_cursor 단위(26), issue_1082(4), issue_1139, 문22 핀.
- 무회귀: 동일 문서 타 페이지 미주 + 대표 본문 수식 문서 SVG diff(줄 위치 불변).
- 산출물: `task_m100_1297_stage3.md` + 최종 `report/task_m100_1297_report.md`.

## 회귀 가드 (필수)

- 메모리 `tech_trailing_model_no_ssot.md`: 미주 trailing은 SSOT 부재 → 줄 높이 보정이 trailing과 이중가산되면 문22(484→511px 류) 회귀. **콘텐츠 높이 축만 건드린다.**
- 메모리 `tech_lazy_base_trailing_ls_gate.md`: 무조건 적용/제거 양쪽 회귀 → **조건부 게이트**가 정답. 본 건도 "보정 후 값이 기존보다 클 때만" 적용하는 단조 게이트로.

## 검증 기준 (수행계획서와 동일)

1. 17쪽 수식 겹침 해소(시각).
2. `cargo test` 전체 통과.
3. 미주/본문 타 위치 무회귀(SVG diff).
