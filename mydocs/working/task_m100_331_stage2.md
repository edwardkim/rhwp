# Task #331 Stage 2 완료 보고서 — PartialParagraph + Layout 정합

- **이슈**: [#331](https://github.com/edwardkim/rhwp/issues/331)
- **브랜치**: `local/task331`

## 변경 내역

### 1. `src/renderer/typeset.rs` — PartialParagraph 마지막 partial 보정

```diff
             let part_line_height = fmt.line_advances_sum(cursor_line..end_line);
             let part_sp_after = if end_line >= line_count { fmt.spacing_after } else { 0.0 };
-            let part_height = sp_b + part_line_height + part_sp_after;
+            // Task #331: 마지막 partial 의 trailing line_spacing 은 advance 에서 제외
+            let trailing_ls_correction = if end_line >= line_count {
+                fmt.line_spacings.get(end_line - 1).copied().unwrap_or(0.0)
+            } else {
+                0.0
+            };
+            let part_height = sp_b + part_line_height + part_sp_after - trailing_ls_correction;
```

### 2. `src/renderer/layout/paragraph_layout.rs` — Layout y_advance 정합

```diff
             col_node.children.push(line_node);
-            // 줄간격 적용: 셀 내 마지막 문단의 마지막 줄에서만 trailing spacing 제외
+            // 줄간격 적용:
+            //  - 셀 내 마지막 문단의 마지막 줄: trailing spacing 제외 (기존 동작)
+            //  - Task #331: 본문 partial 의 마지막 visible 줄: trailing spacing 제외
+            //    (마지막 partial 이면 HWP vpos_h 와 일치, 중간 partial 이면 페이지 break 가 ls 흡수)
             let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
-            if !is_cell_last_line || cell_ctx.is_none() {
+            let is_partial_last_line = line_idx + 1 >= end;
+            let skip_trailing_ls = is_cell_last_line || (is_partial_last_line && cell_ctx.is_none());
+            if !skip_trailing_ls {
                 let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
                 y += line_height + line_spacing_px;
             } else {
                 y += line_height;
             }
```

**중요 발견**: typeset 만 수정하면 layout 의 y advance 가 9.5px 더 진행해 LAYOUT_OVERFLOW 가 발생. layout 의 모든 partial(중간 partial 포함) 마지막 visible 줄에서 trailing_ls 를 제외해야 typeset 과 정합. 중간 partial 의 trailing_ls 는 페이지 break 가 흡수하므로 의미상 올바름.

## 검증

```bash
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp
```

- LAYOUT_OVERFLOW 모두 사라짐
- 페이지 수 16 → 15
- pi=26 + 보기 ①②③ (pi=27,28,29) 모두 page 0 col 1 에 fit (cur_h pi=29 = 1124.5 < avail 1226.4) — PDF 일치
