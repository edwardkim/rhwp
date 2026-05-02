# Task #533 구현계획서 — Square wrap 호스트 텍스트 영역 max advance

**작성일**: 2026-05-02
**이슈**: [#533](https://github.com/edwardkim/rhwp/issues/533)
**브랜치**: `local/task533`
**선행**: Stage 1 완료 (root cause 확정)

## 1. 적용 범위 (단일 본질 정정)

`src/renderer/layout.rs::layout_table_item` 의 비-TAC Square wrap 표 분기에서 `y_offset` advance 시 호스트 문단 텍스트 영역으로 max 처리.

## 2. 변경 정확 위치

### 2-1. `src/renderer/layout.rs::layout_table_item` (line 2501-2513 근처)

**현재 코드**:

```rust
if !tac_seg_applied && !is_outside_body {
    let comp = composed.get(para_index);
    let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
    if let Some(para_style) = styles.para_styles.get(para_style_id) {
        if para_style.spacing_after > 0.0 {
            y_offset += para_style.spacing_after;
        }
    }
    if let Some(seg) = para.line_segs.last() {
        let gap = if seg.line_spacing > 0 { seg.line_spacing } else { seg.line_height };
        y_offset += hwpunit_to_px(gap, self.dpi);
    }
}
```

**수정 후**:

```rust
if !tac_seg_applied && !is_outside_body {
    let comp = composed.get(para_index);
    let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
    if let Some(para_style) = styles.para_styles.get(para_style_id) {
        if para_style.spacing_after > 0.0 {
            y_offset += para_style.spacing_after;
        }
    }
    // [Task #533] Square wrap 호스트 문단: 표는 floating, 호스트 텍스트가
    // 표 옆을 흐른다. 호스트 last LINE_SEG 의 vpos+lh 영역이 표 bottom 보다
    // 아래일 때 호스트 텍스트 영역까지 y_offset 을 advance.
    // (대형 표가 텍스트보다 큰 경우는 max() 로 표 영역 우선 유지)
    if !is_tac {
        if let Some(Control::Table(t)) = para.controls.get(control_index) {
            if matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square) {
                if let Some(seg) = para.line_segs.last() {
                    let host_text_bottom = para_y_for_table + hwpunit_to_px(
                        seg.vertical_pos + seg.line_height, self.dpi);
                    if host_text_bottom > y_offset {
                        y_offset = host_text_bottom;
                    }
                }
            }
        }
    }
    if let Some(seg) = para.line_segs.last() {
        let gap = if seg.line_spacing > 0 { seg.line_spacing } else { seg.line_height };
        y_offset += hwpunit_to_px(gap, self.dpi);
    }
}
```

**핵심**:

- spacing_after 적용 후 / line_spacing 적용 전에 호스트 텍스트 영역 max 처리
- 표 bottom + spacing_after 가 호스트 텍스트 영역보다 크면 표 영역 유지 (대형 표 회귀 차단)
- `seg.vertical_pos + seg.line_height` 만 사용 (`line_spacing` 은 후속 += 에서 추가되므로 중복 방지)

### 2-2. 변경량

- **추가**: 12 라인 (control 체크 + line_segs.last() + max 조건)
- **삭제**: 0 라인
- **수정**: 0 라인 (기존 동작 보존)

## 3. 검증 케이스 정의

### 3-1. 본 결함 시각 검증

| 케이스 | 측정값 | 기대값 |
|--------|--------|--------|
| exam_kor p14 우측 단 pi=51 SVG y | 285.44 (현재) | 298.26 (수정 후) |
| exam_kor p14 우측 단 pi=50 line2 → pi=51 line0 gap | 11.73 px (현재) | 24.51 px (수정 후) |
| exam_kor p14 좌측 단 pi=37/40/47 직후 동일 | 11.31~11.73 px | 24.51 px |

### 3-2. 회귀 테스트

| 게이트 | 기대 |
|--------|------|
| `cargo test --lib` | 1116+ 통과 |
| `cargo test --test svg_snapshot` | 6/6 통과 (golden 갱신 가능 — 의도된 정정) |
| `cargo test --test issue_418` / `issue_501` | 통과 |
| `cargo clippy --lib` | 신규 warning 0 |

### 3-3. 광범위 샘플 회귀 (시각 비교)

`scripts/svg_regression_diff.sh` 로 비교:

- exam_kor (다단 + Square wrap 다수 — 본 결함 fix 검증) — p13, p14 변경 기대
- exam_math_no, exam_kor_math (다단 수식 + 인라인 표)
- treatise sample.hwp (Issue #530 표 셀 텍스트 겹침과 영역 분리)
- 단순 단일단 샘플 (회귀 차단)

## 4. Stage 정의 (재확정)

| Stage | 산출물 |
|-------|-------|
| **Stage 1** | Root cause 위치 확정 (완료, `task_m100_533_stage1.md`) |
| **Stage 2** | 구현계획서 (본 문서) — 작업지시자 승인 |
| **Stage 3** | 본 변경 적용 + cargo test --lib 통과 — `task_m100_533_stage3.md` |
| **Stage 4** | 광범위 회귀 검증 + golden SVG 갱신 — `task_m100_533_stage4.md` |
| **Stage 5** | 시각 판정 (작업지시자) + 최종 보고서 |

## 5. 회귀 위험 평가

### 5-1. 직접 영향 (의도)

- 비-TAC Square wrap 표 + 호스트 텍스트 영역 > 표 영역 케이스의 다음 문단 위치 정정 (~13 px 시프트 해소)

### 5-2. 잠재 회귀 영역

| 케이스 | 영향 | 대응 |
|--------|------|------|
| 대형 Square wrap 표 (표 > 텍스트) | y_offset.max() 로 변경 없음 | ✓ 회귀 차단 |
| TAC 인라인 표 | `!is_tac` 가드 → 변경 없음 | ✓ 회귀 차단 |
| TopAndBottom wrap 표 | wrap=Square 가드 → 변경 없음 | ✓ 회귀 차단 |
| 표 옆 텍스트가 짧은 경우 | host_text_bottom < y_offset → max 효과 없음 | ✓ 동작 동일 |
| 다단 (Multi-column) | 별도 column 의 paragraph layout 은 영향 없음 | ✓ 영역 분리 |

### 5-3. 메모리 정합

- `feedback_essential_fix_regression_risk` — Square wrap host advance 정정은 다단 + 표분할 상호작용 영향 가능 → 광범위 샘플 회귀 검증 필수
- `feedback_rule_not_heuristic` — HWP LINE_SEG vpos/lh 인코딩은 단일 룰. max() 패턴 으로 대형 표 케이스 자동 fallback (분기 없음)

## 6. 실패 시 롤백

본 변경은 단일 if 블록 추가이므로 코드 회복 trivial. Git revert 1 commit.

## 7. 산출물 (본 단계 — 구현계획서)

| 산출물 | 위치 |
|--------|------|
| 본 구현계획서 | `mydocs/plans/task_m100_533_impl.md` |
| 코드 변경 | **0** (계획만) |

## 8. 다음 단계

작업지시자 승인 후 Stage 3 (본 변경 적용 + 단위 테스트) 진행.

## 9. 승인 게이트

- [x] 변경 위치 명확 (`src/renderer/layout.rs:2501` 근처, +12 라인)
- [x] 본 결함 시각 검증 케이스 명시
- [x] 회귀 테스트 게이트 명시 (lib + svg_snapshot + issue_*)
- [x] 광범위 샘플 회귀 시나리오 명시
- [x] 회귀 위험 영역별 평가 + 가드 확인
