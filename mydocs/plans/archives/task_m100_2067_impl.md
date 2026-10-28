# 구현계획서 — Task M100 #2067: layout_composed_paragraph 잔여 분해 (라운드 8)

- 수행계획서: `task_m100_2067.md` / 작성일: 2026-07-08 / 1단계 정밀 분석 반영

## 1. 정밀 분석 결과 (사각지대 체크리스트 6종 적용)

### 후보 1 — justify 여분 간격 (실범위 **2515~2659, 145줄**, 분기 ~30)

`let (extra_word_sp, extra_char_sp, extra_dash_sp) = if …;` 단일 표현식.
- 변이 0 — L2535 `ts.default_tab_width =`는 블록 내부 `let mut ts`(지역) 변이. ①~⑥ 해당 없음.
- L2652 `break`는 지역 수렴 루프(`for _ in 0..3`) 소속 — 외부 탈출 아님.
- 읽기 12개(comp_line/styles/cell_ctx + Copy 스칼라 9) — 임계 경계값이나 순수 계산이라
  직접 파라미터로 진행 (§6: struct 선행은 "읽기 12↑" — 정확히 12, mut 0이므로 면제 판단).
- 승격: `fn compute_line_extra_spacing(…) -> (f64, f64, f64)` (free fn 또는 &self 불요).

### 후보 2 — shape_markers (2815~2877, 63줄, 분기 ~24)

- 대입처럼 보인 8건 전부 match arm 오탐(체크리스트 ③). 실변이 0, 탈출 0.
- 읽기 2개: `show_ctrl: bool`, `para: &Option<&Paragraph>`.
- 승격: `fn collect_shape_marker_labels(show_ctrl, para) -> Vec<(usize, String)>`.
- 루프 불변 값이지만 **호이스팅하지 않고 호출 위치 유지** (동작 불변 원칙).

### 후보 3 — TAC 배치 블록군 (3개 블록, `continue`는 전부 블록-지역 루프 소속)

| 블록 | 범위 | 규모 | 변이 표면 |
|---|---|---|---|
| 3a TAC 그림 | 2991~3058 | 68줄·분기 17 | `line_node.children.push` / `x +=` / `current_line_reserved_tac_picture_height = Some(…)` |
| 3b 빈 문단 TAC 양식 | 3061~3099 | 39줄·분기 5 | `line_node.children.push` / `x +=` |
| 3c 빈 runs TAC 수식 인라인 | 3145~3385 | 241줄·분기 46 | `line_node.children.push` / `tree.set_inline_shape_position` / 지역 `row_inline_x` |

- **3c 소스분기 1건** (L3206 `self.is_hwp3_variant.get()` — indent scale 배율):
  §1 준수 위해 **caller에서 배율 계산 후 값 전달** (`hwp3_indent_scale: f64`) — 분기는
  caller 유지. 그 외 self 의존은 미주 프로파일 조회 4건(&self 메서드 호출, 분기 아님).
- 3a/3b: `x`는 값 왕복(파라미터+반환), reserved height는 `&mut Option<f64>`
  (compute_endnote_metrics 전례). 읽기 각 14/9개 — 3a는 Copy vars struct.
- 3c: 읽기 25↑ → §6 의무로 **`EquationTacLineVars` struct** (Copy 스칼라 ~18필드)
  설계 선행. `EmptyRunsLineVars`(동일 함수 내 전례, L3117) 패턴 준용.
- 3c 신규 함수는 CC 25 초과 예상(분기 46) → §5 예외 심사 기재. 내부 소분할(행 배치
  준비/수식 방출/마커)은 위험 대비 이득 낮아 1차에서는 통이동.

## 2. 단계 구성 (5단계 — 수행계획서 4단계에서 3단계를 2개로 세분)

1. ~~정밀 분석 + 구현계획서~~ (본 문서)
2. **추출 1+2** (justify 145줄 + shape_markers 63줄, 순수 함수 2건) → 게이트 → stage2 보고
3. **추출 3a+3b** (TAC 그림/양식 107줄, x 값 왕복 + &mut 1) → 게이트 → stage3 보고
4. **추출 3c** (수식 인라인 241줄, EquationTacLineVars + hwp3 배율 caller 유지) → 게이트 → stage4 보고
5. **재평가**: `--snapshot r8 --no-coverage` + 공식 CC 대비(146→100 내외 목표) + 최종 보고 + devel 반영

## 3. 게이트 (매 단계 동일)

fmt --check / clippy(release-test, all-targets) 0 / `--tests` FAILED 0 /
OVR 5샘플 회귀 0 / issue_1116 13/13 / rowbreak 20/20.

## 4. 위험 및 완화

- 3c의 클로저 3개(`tac_on_line`/`tac_row_for`/`row_base_x`)는 추출 함수 내부로 통이동
  (캡처 변이 없음 — 체크리스트 ⑥ 확인). fn 승격으로 컴파일러 검증 확보.
- `x`/`row_inline_x` 누적은 값 왕복·지역 유지로 컴파일러가 누락 검출.
- 원격 경합: PR #2065는 `paragraph_layout.rs` 미접촉 — devel 반영 시 재확인.
