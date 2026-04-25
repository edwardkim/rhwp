# Task #331 Stage 1 완료 보고서 — FullParagraph advance trailing_ls 보정

- **이슈**: [#331](https://github.com/edwardkim/rhwp/issues/331)
- **브랜치**: `local/task331`

## 변경 내역

`src/renderer/typeset.rs`:

```diff
         if st.current_height + fmt.height_for_fit <= available {
             st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
-            st.current_height += fmt.total_height;
+            // Task #331: trailing line_spacing 은 advance 에 포함하지 않음 (HWP vpos_h 일치)
+            st.current_height += fmt.height_for_fit;
             return;
         }
 ...
         if line_count == 0 {
             st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
-            st.current_height += fmt.total_height;
+            st.current_height += fmt.height_for_fit;
             return;
         }
```

## 검증

```bash
cargo build --release  # OK
RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0
```

- 페이지 수 16 → 15 (drift 보정 효과 확인)
- LAYOUT_OVERFLOW para=9 (FullParagraph) 잔존 → Stage 2 의 layout 정합 작업에서 해결 예정
- 새로 발견된 LAYOUT_OVERFLOW para=10 (PartialParagraph) → Stage 2 분할 경로 보정 대상

## 다음 단계

Stage 2: PartialParagraph 분할 경로의 마지막 partial trailing_ls 보정 + layout 측 정합.
