# Task #674 최종 결과 보고서

## 1. 요약

**Issue #674**: paragraph_layout 줄 위치 vs row_heights 정합 — line_segs 부재 paragraph 마지막 줄 시각 클립

**결과**: `calc_para_lines_height` 의 `corrected_line_height` 보정 누락이 본질 결함으로 식별. 시그니처에 `styles` 추가 + 보정 적용으로 **본질 정정 완료**. 회귀 0. 시각 판정 ★ 통과.

**Task #671 ~ #674 시리즈 완성** — `samples/계획서.hwp` 1페이지 표 시각 결함 완전 해소.

## 2. 본질 진단 (Stage 1)

### 2.1 25.88 px 오프셋 본질 식별

`samples/계획서.hwp` 셀 [21] (r=5, c=1) 의 paragraph layout 시작 위치 계산:

```
[T674] cell[5,1] cell_y=353.49 pad_top=1.88 pad_bottom=1.88
[T674]   has_nested=false first_line_vpos=None effective_valign=Center
[T674]   inner_height=64.00 total_content_height=16.00
[T674]   text_y_start=379.37 (cell_y+pad_top=355.37, diff=+24.00)
```

### 2.2 본질 결함 메커니즘

```
calc_composed_paras_content_height
  ↓
calc_para_lines_height (corrected_line_height 누락!)
  → line.line_height (raw 5.33 px) 그대로 사용
  → 3줄 × 5.33 = 16.00 (잘못된 측정)
  ↓
total_content_height = 16.00
  ↓
mechanical_offset = (inner_height 64 - total 16) / 2 = 24.00 (Center 정렬)
  ↓
text_y_start = cell_y + pad_top + 24.00 = 379.37 (24px 위로 밀림)
  ↓
paragraph_layout 줄 layout 시작 위치 잘못 → 마지막 줄 cell-clip 영역 초과
  ↓
SVG cell-clip-81 (y=353.49, h=67.76, end=421.25) 안에 줄 0,1 만 들어감
줄 2 (y=422.04) clip 영역 초과 → 시각 클립
```

## 3. 본질 정정 (Stage 2)

### 3.1 정정 위치

`src/renderer/layout/table_layout.rs:746-781` `calc_para_lines_height`:

```rust
fn calc_para_lines_height(
    &self,
    lines: &[crate::renderer::composer::ComposedLine],
    pidx: usize,
    total_para_count: usize,
    para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
    styles: &ResolvedStyleSet,  // [Task #674] 신규
) -> f64 {
    ...
    let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
    let cell_ls_type = para_style.map(|s| s.line_spacing_type)
        .unwrap_or(crate::model::style::LineSpacingType::Percent);
    let lines_total: f64 = lines.iter()
        .enumerate()
        .map(|(i, line)| {
            let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
            let max_fs = line.runs.iter()
                .map(|r| styles.char_styles.get(r.char_style_id as usize)
                    .map(|cs| cs.font_size).unwrap_or(0.0))
                .fold(0.0f64, f64::max);
            let h = crate::renderer::corrected_line_height(
                raw_lh, max_fs, cell_ls_type, cell_ls_val);
            ...
        })
        .sum();
    ...
}
```

`height_measurer.rs:570-587` 와 동일 로직 — 측정/layout 일관성 정합.

### 3.2 호출자 시그니처 정정

| 파일:줄 | 함수 | 변경 |
|---------|------|------|
| `table_layout.rs:710` | calc_cell_paragraphs_content_height | calc_para_lines_height 호출 시 styles 추가 |
| `table_layout.rs:728` | calc_composed_paras_content_height | calc_para_lines_height 호출 시 styles 추가 |

### 3.3 정정 효과

| 항목 | BEFORE | AFTER |
|------|--------|-------|
| line_height 보정 | 없음 (raw 5.33) | corrected (21.33) ✅ |
| total_content_height (3줄) | 16.00 | **64.00** ✅ |
| mechanical_offset (Center) | 24.00 | **0.00** ✅ |
| text_y_start | 379.37 | **355.37** ✅ |
| 줄 2 y | 422.04 (clip 초과) | **397.71 (clip 안)** ✅ |

## 4. 결정적 검증

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 5. 광범위 페이지네이션 회귀 sweep (Stage 3)

| 영역 | 결과 |
|------|------|
| BEFORE (task672) | 187 fixtures / 2013 pages |
| AFTER (task674) | 187 fixtures / **2013 pages** |
| **차이** | **0** ✅ |

회귀 위험 영역 완전 좁힘.

## 6. 시각 판정 게이트웨이 ★ 통과

`samples/계획서.hwp` 1페이지 표:

| 셀 | BEFORE (task672) | AFTER (task674) |
|----|-----------------|-----------------|
| [13] "탈레스 HSM 관리 시스템 및 REST API" | 2줄 정상 | 2줄 정상 (회귀 0) ✅ |
| [21] "목적" | 2줄 (마지막 줄 클립) | **3줄 모두 표시** ✅ |
| [52] "특허 취득" | 2 paragraph (◦특허의뢰 누락) | **3 paragraph 모두 표시** ✅ |
| 다른 셀 | — | 회귀 0 ✅ |

## 7. Task #671 ~ #674 시리즈 본질 영역 정합 정리

| Task | 본질 영역 | 정정 위치 |
|------|----------|-----------|
| #671 | 셀 paragraph line_segs 부재 → compose_lines 단일 ComposedLine 압축 | composer.rs (recompose_for_cell_width) + 6개 호출 위치 |
| #672 | TAC 표 비례 축소 (작은 차이도 발동) | height_measurer.rs:822 임계값 가드 |
| #674 | calc_para_lines_height corrected_line_height 누락 | table_layout.rs:746 시그니처 + 보정 |

3 task 시리즈 정합 — `samples/계획서.hwp` 시각 결함 완전 해소.

## 8. 회귀 위험 영역 좁힘 원칙

- **수정 영역 명시**: `calc_para_lines_height` 시그니처 + 보정 적용 (단일 함수)
- **본질 룰**: height_measurer 와 동일 로직 (측정/layout 일관성)
- **광범위 sweep 검증**: 187 fixture 페이지 수 차이 0
- **다른 영역 영향**: 정상 line_segs 인코딩된 paragraph 영향 미미 (보정 결과가 raw 값과 비슷)

## 9. 권위 자료

- `samples/계획서.hwp` — Task #671/#672/#674 권위 재현 영역 (이미 git tracked)

## 10. 최종 산출물

| 영역 | 파일 |
|------|------|
| 코드 정정 | `src/renderer/layout/table_layout.rs` (calc_para_lines_height + 호출자) |
| 수행계획서 | `mydocs/plans/task_m100_674.md` |
| 구현계획서 | `mydocs/plans/task_m100_674_impl.md` |
| 단계별 보고서 | `mydocs/working/task_m100_674_stage1.md`, `_stage2.md`, `_stage3.md` |
| 최종 보고서 | `mydocs/report/task_m100_674_report.md` (본 문서) |

## 11. 의존성

- **선행 의존**: Task #671 (PR #673), Task #672 (PR #675) — `local/task674` ← `local/task672` 분기

## 12. 정합 패턴 정리

- `feedback_rule_not_heuristic`: 본질 정정 (height_measurer 와 동일 로직)
- `feedback_hancom_compat_specific_over_general`: 측정/layout 일관성 정합
- `feedback_visual_judgment_authority`: 시각 판정 ★ 통과
- `feedback_close_issue_verify_merged`: Issue close 시 정정 코드 devel 머지 검증
- `project_dtp_identity`: 조판 엔진 정합성 강화

## 13. 후속 영역

Task #671 ~ #674 시리즈 완성. `samples/계획서.hwp` 의 본질적 시각 결함 모두 해소.
